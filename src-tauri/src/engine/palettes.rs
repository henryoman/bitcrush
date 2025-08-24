use crate::engine::color::hex_to_rgb;
use std::fs;
use tauri::path::BaseDirectory;
use tauri::Manager;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Palette {
    pub name: &'static str,
    pub colors: Vec<[u8; 3]>,
}

fn build_palette(name: &'static str, hexes: &[&'static str]) -> Palette {
    let colors = hexes.iter().filter_map(|h| hex_to_rgb(h)).collect();
    Palette { name, colors }
}

pub fn built_in_palettes() -> Vec<Palette> {
    vec![
        build_palette("Flying Tiger", &["#000000","#ffffff","#ff0000","#00ff00","#0000ff","#ffff00","#ffa500","#800080","#ff69b4","#00ffff"]),
        build_palette("Black & White", &["#000000","#ffffff"]),
        build_palette("Cozy 8", &["#2e294e","#541388","#f1e9da","#ffd400","#d90368","#0081a7","#00afb9","#fed9b7"]),
        build_palette("Retro Gaming", &["#0f0f23","#262b44","#3e4a5c","#5a6988","#738699","#8ea3b0","#a4c0c7","#c0dddd"]),
        build_palette("Sunset Vibes", &["#2d1b69","#11296b","#0f4c75","#3282b8","#bbe1fa","#ff6b6b","#ffa726","#ffcc02"]),
        build_palette("Forest Dreams", &["#1a3a2e","#16423c","#0f3460","#533a71","#6a994e","#a7c957","#f2e8cf","#bc4749"]),
    ]
}

pub fn get_palette_by_name(name: &str) -> Palette {
    let mut it = built_in_palettes().into_iter();
    if let Some(p) = it.clone().find(|p| p.name == name) { return p; }
    it.next().expect("at least one built-in palette")
}

fn parse_gpl(contents: &str, fallback_name: &str) -> Option<Palette> {
    let mut name: Option<String> = None;
    let mut colors: Vec<[u8;3]> = Vec::new();
    for line in contents.lines() {
        let line = line.trim_start_matches('\u{FEFF}').trim();
        if line.is_empty() { continue; }
        // allow both "#Palette Name:" and "Name:" (with/without leading '#')
        if line.starts_with('#') {
            let l = line[1..].trim();
            if l.starts_with("Palette Name:") {
                name = Some(l[13..].trim().to_string());
                continue;
            }
            if l.starts_with("Name:") {
                name = Some(l[5..].trim().to_string());
                continue;
            }
        } else if line.starts_with("Palette Name:") {
            name = Some(line[13..].trim().to_string());
            continue;
        } else if line.starts_with("Name:") {
            name = Some(line[5..].trim().to_string());
            continue;
        }
        if line.starts_with('#') || line.starts_with("GIMP Palette") { continue; }
        // Expect lines like: R\tG\tB\t(optional name)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            colors.push([r,g,b]);
        }
    }
    if colors.is_empty() { return None; }
    let nm = name.unwrap_or_else(|| fallback_name.to_string());
    // Leak name string to static for struct; acceptable since lifetime is app-long
    let nm_static: &'static str = Box::leak(nm.into_boxed_str());
    Some(Palette { name: nm_static, colors })
}

#[derive(serde::Deserialize)]
struct TomlPalette { name: String, colors: Vec<String> }

#[derive(serde::Deserialize)]
struct TomlPalettes { palette: Option<Vec<TomlPalette>> }

pub fn load_palettes(app: &tauri::AppHandle) -> Vec<Palette> {
    let mut out = built_in_palettes();
    // Resolve Resource/palettes, then target/.../resources/palettes, then compile-time src-tauri/resources/palettes
    let base = app
        .path()
        .resolve("palettes", BaseDirectory::Resource)
        .ok()
        .filter(|p| p.exists())
        .or_else(|| {
            app.path()
                .resource_dir()
                .ok()
                .map(|p| p.join("palettes"))
                .filter(|p| p.exists())
        })
        .or_else(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let dev = root.join("resources").join("palettes");
            if dev.exists() { Some(dev) } else { None }
        });
    if let Some(base_dir) = base {
        // Load TOML
        let toml_path = base_dir.join("palettes.toml");
        if toml_path.exists() {
            if let Ok(s) = fs::read_to_string(&toml_path) {
                if let Ok(doc) = toml::from_str::<TomlPalettes>(&s) {
                    if let Some(list) = doc.palette {
                        for p in list {
                            let mut cols: Vec<[u8;3]> = Vec::new();
                            for hx in p.colors.iter() {
                                if let Some(rgb) = hex_to_rgb(hx) { cols.push(rgb); }
                            }
                            if !cols.is_empty() {
                                let nm_static: &'static str = Box::leak(p.name.into_boxed_str());
                                out.push(Palette { name: nm_static, colors: cols });
                            }
                        }
                    }
                }
            }
        }
        // Load GPL directory
        let gpl_dir = base_dir.join("gpl");
        if gpl_dir.exists() && gpl_dir.is_dir() {
            if let Ok(rd) = fs::read_dir(&gpl_dir) {
                for ent in rd.flatten() {
                    let path = ent.path();
                    if path.extension().and_then(|e| e.to_str()).unwrap_or("") == "gpl" {
                        if let Ok(s) = fs::read_to_string(&path) {
                            let fallback = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Palette");
                            if let Some(p) = parse_gpl(&s, fallback) {
                                out.push(p);
                            }
                        }
                    }
                }
            }
        }
    }
    // De-duplicate by name (last wins)
    out.reverse();
    let mut seen: std::collections::HashSet<&'static str> = std::collections::HashSet::new();
    out.retain(|p| seen.insert(p.name));
    out.reverse();
    out
}

pub fn resolve_palette(app: &tauri::AppHandle, name: &str) -> Palette {
    let all = load_palettes(app);
    if let Some(p) = all.iter().find(|p| p.name == name) {
        return Palette { name: p.name, colors: p.colors.clone() };
    }
    // fallback built-ins
    get_palette_by_name(name)
}



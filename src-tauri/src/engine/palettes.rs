use crate::engine::color::hex_to_rgb;

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



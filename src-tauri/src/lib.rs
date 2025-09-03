mod engine;
mod types;

use engine::pipeline::{render_base_png, render_preview_png, render_base_png_with_palette, render_preview_png_with_palette};
use engine::filters::render_filters_preview_png;
use engine::palettes::{load_palettes, resolve_palette};
use types::RenderRequest;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn render_preview(app: tauri::AppHandle, req: RenderRequest) -> Result<String, String> {
    if let Some(name) = req.palette_name.clone() {
        let p = resolve_palette(&app, &name);
        if !p.colors.is_empty() {
            return render_preview_png_with_palette(req, p.colors).map_err(|e| e.to_string());
        }
    }
    render_preview_png(req).map_err(|e| e.to_string())
}

#[tauri::command]
fn render_filters_preview(req: RenderRequest) -> Result<String, String> {
    render_filters_preview_png(req).map_err(|e| e.to_string())
}

#[tauri::command]
fn render_base(app: tauri::AppHandle, req: RenderRequest) -> Result<String, String> {
    if let Some(name) = req.palette_name.clone() {
        let p = resolve_palette(&app, &name);
        if !p.colors.is_empty() {
            return render_base_png_with_palette(req, p.colors).map_err(|e| e.to_string());
        }
    }
    render_base_png(req).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_palettes(app: tauri::AppHandle) -> Vec<(String, Vec<[u8;3]>)> {
    load_palettes(&app)
        .into_iter()
        .map(|p| (p.name.to_string(), p.colors))
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![render_preview, render_base, list_palettes, render_filters_preview])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

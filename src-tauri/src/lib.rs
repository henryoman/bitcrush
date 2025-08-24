mod engine;
mod types;

use engine::pipeline::{render_base_png, render_preview_png};
use types::{AlgorithmName, RenderRequest};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn render_preview(req: RenderRequest) -> Result<String, String> {
    render_preview_png(req).map_err(|e| e.to_string())
}

#[tauri::command]
fn render_base(req: RenderRequest) -> Result<String, String> {
    render_base_png(req).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_palettes() -> Vec<&'static str> {
    // Placeholder: palettes will be loaded from resources in a later step
    vec![
        "Flying Tiger",
        "Black & White",
        "Cozy 8",
        "Retro Gaming",
        "Sunset Vibes",
        "Forest Dreams",
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![render_preview, render_base, list_palettes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

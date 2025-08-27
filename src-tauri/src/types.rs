use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RenderRequest {
    /// Data URL string (e.g. "data:image/png;base64,<...>") or absolute path in future
    pub image_data_url: String,
    /// Grid size (e.g. 32, 64)
    pub grid_size: u32,
    /// Algorithm name (e.g. "Standard", "Floyd–Steinberg", etc.)
    pub algorithm: String,
    /// Palette name to use (matches built-ins for now)
    pub palette_name: Option<String>,
    /// Desired preview size; upscaled image width/height (nearest multiple of grid)
    pub display_size: Option<u32>,
    /// Optional tone curve gamma (1.0 = no change). Typical 0.5..2.0
    #[serde(default)]
    pub tone_gamma: Option<f32>,
    /// Optional denoise sigma for Gaussian blur in source domain. Typical 0.0..2.5
    #[serde(default)]
    pub denoise_sigma: Option<f32>,
}



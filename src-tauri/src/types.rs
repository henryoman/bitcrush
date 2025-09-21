use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RenderRequest {
    /// Data URL string (e.g. "data:image/png;base64,<...>") or absolute path in future
    pub image_data_url: String,
    /// Grid width and height (e.g. 32x32, 384x192)
    pub grid_width: u32,
    pub grid_height: u32,
    /// Optional grid string like "32" or "384x192". If present, Rust parses it.
    #[serde(default)]
    pub grid_value: Option<String>,
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
    /// Optional pre-contrast applied to source before grid/pixelize (1.0 = no change)
    #[serde(default)]
    pub pre_contrast: Option<f32>,
    /// Optional pre-saturation applied to source before grid/pixelize (1.0 = no change)
    #[serde(default)]
    pub pre_saturation: Option<f32>,
    /// Optional pre-hue shift in degrees applied to source before grid/pixelize (0.0 = no change)
    #[serde(default)]
    pub pre_hue_degrees: Option<f32>,
    /// Optional invert and night vision prefilter
    #[serde(default)]
    pub invert_colors: Option<bool>,
    #[serde(default)]
    pub night_vision_prefilter: Option<bool>,
    /// Palette augmentation flags (affect render only; not persisted)
    #[serde(default)]
    pub add_black_to_palette: Option<bool>,
    #[serde(default)]
    pub add_white_to_palette: Option<bool>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterStep {
    /// Unique filter name/id (e.g., "Identity", "Brightness")
    pub name: String,
    /// Normalized strength in [0.0, 1.0]; filter-specific interpretation
    #[serde(default = "default_amount")]
    pub amount: f32,
    /// Whether this step should be applied
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_amount() -> f32 { 1.0 }
fn default_enabled() -> bool { true }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterChainRequest {
    /// Data URL string (e.g. "data:image/png;base64,<...>")
    pub image_data_url: String,
    /// Desired preview size; upscaled image width/height (integer scaling)
    #[serde(default)]
    pub display_size: Option<u32>,
    /// Ordered list of filter steps to apply
    #[serde(default)]
    pub steps: Vec<FilterStep>,
}


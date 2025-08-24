use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RenderRequest {
    /// Data URL string (e.g. "data:image/png;base64,<...>") or absolute path in future
    pub image_data_url: String,
    /// Grid size (e.g. 32, 64)
    pub grid_size: u32,
    /// Algorithm name (e.g. "Standard", "Floyd–Steinberg", etc.)
    pub algorithm: String,
}



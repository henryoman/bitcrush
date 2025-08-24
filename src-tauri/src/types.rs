use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RenderRequest {
    /// Data URL string (e.g. "data:image/png;base64,<...>") or absolute path in future
    pub image_data_url: String,
    /// Grid size (e.g. 32, 64)
    pub grid_size: u32,
    /// Algorithm name (defaults to Standard if unknown)
    pub algorithm: AlgorithmName,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum AlgorithmName {
    Standard,
    #[serde(other)]
    Other,
}

impl Default for AlgorithmName {
    fn default() -> Self { AlgorithmName::Standard }
}



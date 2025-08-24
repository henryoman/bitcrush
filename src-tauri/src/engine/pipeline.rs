use crate::types::{AlgorithmName, RenderRequest};
use image::{imageops::FilterType, DynamicImage, ImageBuffer, ImageOutputFormat, Rgba, RgbaImage};
use std::io::Cursor;
use thiserror::Error;

use super::algorithms::{get_algorithm_by_name, Algorithm};

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("unsupported image data url")]
    UnsupportedDataUrl,
    #[error(transparent)]
    Image(#[from] image::ImageError),
}

fn decode_data_url_to_image(data_url: &str) -> Result<DynamicImage, EngineError> {
    // Expect format: data:image/<type>;base64,<base64>
    let (header, b64) = data_url
        .split_once(",")
        .ok_or(EngineError::UnsupportedDataUrl)?;
    if !header.contains("base64") {
        return Err(EngineError::UnsupportedDataUrl);
    }
    let bytes = base64::decode(b64).map_err(|_| EngineError::UnsupportedDataUrl)?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

fn resize_to_grid(img: &DynamicImage, grid_size: u32) -> RgbaImage {
    img.resize_exact(grid_size, grid_size, FilterType::Nearest).to_rgba8()
}

fn upscale_center_to(img: &RgbaImage, display_size: u32) -> RgbaImage {
    let factor = (display_size / img.width()).max(1);
    let scaled = image::imageops::resize(img, img.width() * factor, img.height() * factor, FilterType::Nearest);
    let mut canvas: RgbaImage = ImageBuffer::from_pixel(
        display_size,
        display_size,
        Rgba([0, 0, 0, 0]),
    );
    let off_x = (display_size - scaled.width()) / 2;
    let off_y = (display_size - scaled.height()) / 2;
    image::imageops::overlay(&mut canvas, &scaled, off_x.into(), off_y.into());
    canvas
}

fn encode_png_base64(img: &RgbaImage) -> Result<String, EngineError> {
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img.clone()).write_to(&mut buf, ImageOutputFormat::Png)?;
    let b64 = base64::encode(buf.into_inner());
    Ok(format!("data:image/png;base64,{}", b64))
}

pub fn render_preview_png(req: RenderRequest) -> Result<String, EngineError> {
    let img = decode_data_url_to_image(&req.image_data_url)?;
    let mut grid = resize_to_grid(&img, req.grid_size);
    let algo = match req.algorithm {
        AlgorithmName::Standard => get_algorithm_by_name("Standard"),
        AlgorithmName::Other => get_algorithm_by_name("Standard"),
    };
    algo.process(&mut grid);
    let up = upscale_center_to(&grid, 640);
    encode_png_base64(&up)
}

pub fn render_base_png(req: RenderRequest) -> Result<String, EngineError> {
    let img = decode_data_url_to_image(&req.image_data_url)?;
    let mut grid = resize_to_grid(&img, req.grid_size);
    let algo = match req.algorithm {
        AlgorithmName::Standard => get_algorithm_by_name("Standard"),
        AlgorithmName::Other => get_algorithm_by_name("Standard"),
    };
    algo.process(&mut grid);
    encode_png_base64(&grid)
}



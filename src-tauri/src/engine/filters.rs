use crate::types::RenderRequest;
use image::{imageops::FilterType, DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use thiserror::Error;

// Filters pipeline is intentionally isolated from pixelizer-specific algorithms/dithers.

#[derive(Debug, Error)]
pub enum FilterError {
    #[error("unsupported image data url")]
    UnsupportedDataUrl,
    #[error(transparent)]
    Image(#[from] image::ImageError),
}

fn decode_data_url_to_image(data_url: &str) -> Result<DynamicImage, FilterError> {
    let (header, b64) = data_url
        .split_once(",")
        .ok_or(FilterError::UnsupportedDataUrl)?;
    if !header.contains("base64") {
        return Err(FilterError::UnsupportedDataUrl);
    }
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine;
    let bytes = B64
        .decode(b64)
        .map_err(|_| FilterError::UnsupportedDataUrl)?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

fn encode_png_base64(img: &RgbaImage) -> Result<String, FilterError> {
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img.clone()).write_to(&mut buf, ImageFormat::Png)?;
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine;
    let b64 = B64.encode(buf.into_inner());
    Ok(format!("data:image/png;base64,{}", b64))
}

// Keep handy for future filter passes that need explicit resizing.
#[allow(dead_code)]
fn resize_exact_rgba(img: &DynamicImage, w: u32, h: u32) -> RgbaImage {
    img.resize_exact(w, h, FilterType::Nearest).to_rgba8()
}

fn upscale_center_to(img: &RgbaImage, display_size: u32) -> RgbaImage {
    let max_dim = display_size.max(1);
    let factor_w = (max_dim / img.width()).max(1);
    let factor_h = (max_dim / img.height()).max(1);
    let factor = factor_w.min(factor_h);
    let scaled = image::imageops::resize(
        img,
        img.width() * factor,
        img.height() * factor,
        FilterType::Nearest,
    );
    let mut canvas: RgbaImage = image::ImageBuffer::from_pixel(max_dim, max_dim, Rgba([0, 0, 0, 0]));
    let off_x = (max_dim - scaled.width()) / 2;
    let off_y = (max_dim - scaled.height()) / 2;
    image::imageops::overlay(&mut canvas, &scaled, off_x.into(), off_y.into());
    canvas
}

/// Placeholder for the Filters pipeline: start by mirroring Pixelizer's resize/tone/denoise flow
/// but keep it isolated so future filter-specific steps live here.
pub fn render_filters_preview_png(req: RenderRequest) -> Result<String, FilterError> {
    let img = decode_data_url_to_image(&req.image_data_url)?;
    // For filters, start by not altering resolution; just convert to RGBA and upscale for preview.
    let grid = img.to_rgba8();
    let target = req.display_size.unwrap_or(560);
    let up = upscale_center_to(&grid, target);
    Ok(encode_png_base64(&up)?)
}



use crate::types::FilterChainRequest;
use image::{imageops::FilterType, DynamicImage, ImageFormat, RgbaImage};
use std::io::Cursor;
use thiserror::Error;

mod vhs;

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

fn upscale_center_to(img: &RgbaImage, display_size: u32) -> RgbaImage {
    // Whole-integer up/down scale, return just scaled image (UI aligns top-left)
    let max_dim = display_size.max(1);
    let (w, h) = (img.width(), img.height());
    let (target_w, target_h) = if w <= max_dim && h <= max_dim {
        let factor_w = (max_dim / w).max(1);
        let factor_h = (max_dim / h).max(1);
        let factor = factor_w.min(factor_h).max(1);
        (w * factor, h * factor)
    } else {
        let scale = (max_dim as f32 / w as f32).min(max_dim as f32 / h as f32);
        let tw = (w as f32 * scale).floor().max(1.0) as u32;
        let th = (h as f32 * scale).floor().max(1.0) as u32;
        (tw, th)
    };
    image::imageops::resize(img, target_w, target_h, FilterType::Nearest)
}

pub fn render_filters_preview_png(req: FilterChainRequest) -> Result<String, FilterError> {
    let img = decode_data_url_to_image(&req.image_data_url)?;
    // Performance: operate at a capped working size then upscale to display for speed
    let base0 = img.to_rgba8();
    // Prefilter sharp digital inputs once to avoid unrealistic high-frequency noise in VHS
    let base = vhs::prefilter_input_for_vhs(&base0);
    let (bw, bh) = (base.width(), base.height());
    let max_work = 640u32;
    let work = if bw.max(bh) > max_work {
        let scale = (max_work as f32 / bw as f32).min(max_work as f32 / bh as f32);
        let tw = (bw as f32 * scale).floor().max(1.0) as u32;
        let th = (bh as f32 * scale).floor().max(1.0) as u32;
        image::imageops::resize(&base, tw, th, FilterType::Triangle)
    } else {
        base
    };
    let mut frame = work;
    // Router: VHS variants (VHS 1..7)
    for step in &req.steps {
        if !step.enabled { continue; }
        let name = step.name.to_ascii_uppercase();
        match name.as_str() {
            "VHS 1" => { frame = vhs::apply_vhs1(&frame); }
            "VHS 2" => { frame = vhs::apply_vhs2(&frame); }
            "VHS 3" => { frame = vhs::apply_vhs3(&frame); }
            "VHS 4" => { frame = vhs::apply_vhs4(&frame); }
            "VHS 5" => { frame = vhs::apply_vhs5(&frame); }
            "VHS 6" => { frame = vhs::apply_vhs6(&frame); }
            "VHS 7" => { frame = vhs::apply_vhs7(&frame); }
            _ => {}
        }
    }
    let target = req.display_size.unwrap_or(560);
    let up = upscale_center_to(&frame, target);
    Ok(encode_png_base64(&up)?)
}



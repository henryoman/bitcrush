use crate::types::{FilterChainRequest, FilterStep};
use image::{imageops::FilterType, DynamicImage, ImageFormat, Rgba, RgbaImage};
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

pub trait Filter {
    fn name(&self) -> &'static str;
    fn apply(&self, img: &mut RgbaImage, amount: f32);
}

struct IdentityFilter;
impl Filter for IdentityFilter {
    fn name(&self) -> &'static str { "Identity" }
    fn apply(&self, _img: &mut RgbaImage, _amount: f32) {}
}

struct BrightnessFilter;
impl Filter for BrightnessFilter {
    fn name(&self) -> &'static str { "Brightness" }
    fn apply(&self, img: &mut RgbaImage, amount: f32) {
        let centered = (amount.clamp(0.0, 1.0) - 0.5) * 2.0; // -1..1
        let delta = (centered * 255.0) as i32;
        for p in img.pixels_mut() {
            let [r, g, b, a] = p.0;
            let nr = (r as i32 + delta).clamp(0, 255) as u8;
            let ng = (g as i32 + delta).clamp(0, 255) as u8;
            let nb = (b as i32 + delta).clamp(0, 255) as u8;
            *p = Rgba([nr, ng, nb, a]);
        }
    }
}

struct ContrastFilter;
impl Filter for ContrastFilter {
    fn name(&self) -> &'static str { "Contrast" }
    fn apply(&self, img: &mut RgbaImage, amount: f32) {
        let scale = 2.0_f32.powf((amount.clamp(0.0, 1.0) - 0.5) * 2.0);
        for p in img.pixels_mut() {
            let [r, g, b, a] = p.0;
            let fr = ((r as f32 / 255.0 - 0.5) * scale + 0.5) * 255.0;
            let fg = ((g as f32 / 255.0 - 0.5) * scale + 0.5) * 255.0;
            let fb = ((b as f32 / 255.0 - 0.5) * scale + 0.5) * 255.0;
            *p = Rgba([
                fr.clamp(0.0, 255.0) as u8,
                fg.clamp(0.0, 255.0) as u8,
                fb.clamp(0.0, 255.0) as u8,
                a,
            ]);
        }
    }
}

fn get_filter_by_name(name: &str) -> Option<Box<dyn Filter + Send + Sync>> {
    match name {
        "Identity" => Some(Box::new(IdentityFilter)),
        "Brightness" => Some(Box::new(BrightnessFilter)),
        "Contrast" => Some(Box::new(ContrastFilter)),
        "VHS" => Some(Box::new(vhs::VhsFilter)),
        _ => None,
    }
}

fn apply_filter_chain(mut img: RgbaImage, steps: &[FilterStep]) -> RgbaImage {
    for step in steps.iter() {
        if !step.enabled { continue; }
        if let Some(f) = get_filter_by_name(step.name.as_str()) {
            let amt = step.amount.clamp(0.0, 1.0);
            let mut_ref: &mut RgbaImage = &mut img;
            f.apply(mut_ref, amt);
        }
    }
    img
}

pub fn render_filters_preview_png(req: FilterChainRequest) -> Result<String, FilterError> {
    let img = decode_data_url_to_image(&req.image_data_url)?;
    let mut frame = img.to_rgba8();
    frame = apply_filter_chain(frame, &req.steps);
    let target = req.display_size.unwrap_or(560);
    let up = upscale_center_to(&frame, target);
    Ok(encode_png_base64(&up)?)
}



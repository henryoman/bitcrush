use crate::types::RenderRequest;
use image::{imageops::FilterType, DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use thiserror::Error;

use super::algorithms::get_algorithm_by_name;
use super::dither::{
    atkinson::apply_atkinson,
    bayer::{Bayer, Bayer2, Bayer8},
    burkes::apply_burkes,
    dual_color::apply_dual_color,
    edge::apply_edge_dithering,
    floyd_steinberg::FloydSteinberg,
    jarvis_judice_ninke::apply_jjn,
    ordered_selective::apply_ordered_selective,
    randomized_selective::apply_randomized_selective,
    selective::apply_selective,
    sierra::{apply_sierra, apply_sierra_lite, apply_two_row_sierra},
    stucki::apply_stucki,
};
use super::palettes::get_palette_by_name;
use crate::engine::algorithms::Algorithm;

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
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine;
    let bytes = B64
        .decode(b64)
        .map_err(|_| EngineError::UnsupportedDataUrl)?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

fn resize_to_grid(img: &DynamicImage, grid_w: u32, grid_h: u32) -> RgbaImage {
    img.resize_exact(grid_w, grid_h, FilterType::Nearest)
        .to_rgba8()
}
fn apply_pre_contrast_saturation(img: &DynamicImage, pre_contrast: Option<f32>, pre_saturation: Option<f32>) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let contrast = pre_contrast.unwrap_or(1.0);
    let saturation = pre_saturation.unwrap_or(1.0);
    if (contrast - 1.0).abs() < 0.001 && (saturation - 1.0).abs() < 0.001 {
        return DynamicImage::ImageRgba8(rgba);
    }
    let c = (contrast).max(0.01);
    let s = (saturation).max(0.0);
    for p in rgba.pixels_mut() {
        let [r, g, b, a] = p.0;
        // Contrast around mid-point 128
        let mut rf = (r as f32 - 128.0) * c + 128.0;
        let mut gf = (g as f32 - 128.0) * c + 128.0;
        let mut bf = (b as f32 - 128.0) * c + 128.0;
        // Saturation in HSL-ish via luma blend
        let y = 0.2126 * rf + 0.7152 * gf + 0.0722 * bf;
        rf = y + (rf - y) * s;
        gf = y + (gf - y) * s;
        bf = y + (bf - y) * s;
        *p = Rgba([
            rf.clamp(0.0, 255.0) as u8,
            gf.clamp(0.0, 255.0) as u8,
            bf.clamp(0.0, 255.0) as u8,
            a,
        ]);
    }
    DynamicImage::ImageRgba8(rgba)
}

// (removed deprecated preprocess; denoise is now applied after grid resize)

fn apply_tone_gamma(img: &mut RgbaImage, tone_gamma: Option<f32>) {
    if let Some(g) = tone_gamma {
        if (g - 1.0).abs() > 0.001 {
            let inv = 1.0 / g.max(0.05);
            for p in img.pixels_mut() {
                let [r, gc, b, a] = p.0;
                let cr = ((r as f32 / 255.0).powf(inv) * 255.0).clamp(0.0, 255.0) as u8;
                let cg = ((gc as f32 / 255.0).powf(inv) * 255.0).clamp(0.0, 255.0) as u8;
                let cb = ((b as f32 / 255.0).powf(inv) * 255.0).clamp(0.0, 255.0) as u8;
                *p = Rgba([cr, cg, cb, a]);
            }
        }
    }
}

fn apply_denoise_rgba(img: RgbaImage, denoise_sigma: Option<f32>) -> RgbaImage {
    if let Some(sigma) = denoise_sigma {
        if sigma > 0.01 {
            return image::imageops::blur(&img, sigma);
        }
    }
    img
}

fn parse_grid_value(value: &str) -> Option<(u32, u32)> {
    let s = value.trim().to_lowercase();
    if let Some((a, b)) = s.split_once('x') {
        let w = a.trim().parse::<u32>().ok()?;
        let h = b.trim().parse::<u32>().ok()?;
        return Some((w.max(1), h.max(1)));
    }
    if let Ok(n) = s.parse::<u32>() {
        let n = n.max(1);
        return Some((n, n));
    }
    None
}

fn resolve_grid(req: &RenderRequest) -> (u32, u32) {
    if let Some(ref gv) = req.grid_value {
        if let Some((w, h)) = parse_grid_value(gv) {
            return (w, h);
        }
    }
    let w = req.grid_width.max(1);
    let h = req.grid_height.max(1);
    (w, h)
}

fn upscale_center_to(img: &RgbaImage, display_size: u32) -> RgbaImage {
    // Maintain whole-integer scaling; return just the scaled image (top-left alignment in UI)
    let max_dim = display_size.max(1);
    let factor_w = (max_dim / img.width()).max(1);
    let factor_h = (max_dim / img.height()).max(1);
    let factor = factor_w.min(factor_h).max(1);
    image::imageops::resize(
        img,
        img.width() * factor,
        img.height() * factor,
        FilterType::Nearest,
    )
}

fn encode_png_base64(img: &RgbaImage) -> Result<String, EngineError> {
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img.clone()).write_to(&mut buf, ImageFormat::Png)?;
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine;
    let b64 = B64.encode(buf.into_inner());
    Ok(format!("data:image/png;base64,{}", b64))
}

pub fn render_preview_png(req: RenderRequest) -> Result<String, EngineError> {
    let img0 = decode_data_url_to_image(&req.image_data_url)?;
    // Preprocess in source domain
    let img = apply_pre_contrast_saturation(&img0, req.pre_contrast, req.pre_saturation);
    let (gw, gh) = resolve_grid(&req);
    let mut grid = resize_to_grid(&img, gw, gh);
    grid = apply_denoise_rgba(grid, req.denoise_sigma);
    apply_tone_gamma(&mut grid, req.tone_gamma);
    let algo = get_algorithm_by_name(req.algorithm.as_str());
    let palette_name = req.palette_name.as_deref().unwrap_or("Flying Tiger");
    let palette = get_palette_by_name(palette_name);
    let pal_slice: Vec<[u8; 3]> = palette.colors.clone();
    match req.algorithm.as_str() {
        "Floyd-Steinberg" | "Floyd–Steinberg" => FloydSteinberg.process(&mut grid, &pal_slice),
        "Bayer" => Bayer.process(&mut grid, &pal_slice),
        "Bayer 2x2" => Bayer2.process(&mut grid, &pal_slice),
        "Bayer 8x8" => Bayer8.process(&mut grid, &pal_slice),
        "Selective" => apply_selective(&mut grid, &pal_slice, 25.0),
        "Ordered Selective" => apply_ordered_selective(&mut grid, &pal_slice, 25.0),
        "Dual Color Dithering" => apply_dual_color(&mut grid, &pal_slice),
        "Edge Dithering" => apply_edge_dithering(&mut grid, &pal_slice),
        "Randomized Selective" => apply_randomized_selective(&mut grid, &pal_slice, 30.0),
        "Stucki" => apply_stucki(&mut grid, &pal_slice),
        "Atkinson" => apply_atkinson(&mut grid, &pal_slice),
        "Jarvis-Judice-Ninke" | "Jarvis, Judice, and Ninke" => apply_jjn(&mut grid, &pal_slice),
        "Burkes" => apply_burkes(&mut grid, &pal_slice),
        "Sierra" => apply_sierra(&mut grid, &pal_slice),
        "Two-Row Sierra" => apply_two_row_sierra(&mut grid, &pal_slice),
        "Sierra Lite" => apply_sierra_lite(&mut grid, &pal_slice),
        _ => algo.process(&mut grid, &pal_slice),
    }
    let target = req.display_size.unwrap_or(560);
    let up = upscale_center_to(&grid, target);
    encode_png_base64(&up)
}

pub fn render_base_png(req: RenderRequest) -> Result<String, EngineError> {
    let img = decode_data_url_to_image(&req.image_data_url)?;
    let (gw, gh) = resolve_grid(&req);
    let mut grid = resize_to_grid(&img, gw, gh);
    grid = apply_denoise_rgba(grid, req.denoise_sigma);
    apply_tone_gamma(&mut grid, req.tone_gamma);
    let algo = get_algorithm_by_name(req.algorithm.as_str());
    let palette_name = req.palette_name.as_deref().unwrap_or("Flying Tiger");
    let palette = get_palette_by_name(palette_name);
    let pal_slice: Vec<[u8; 3]> = palette.colors.clone();
    match req.algorithm.as_str() {
        "Floyd-Steinberg" | "Floyd–Steinberg" => FloydSteinberg.process(&mut grid, &pal_slice),
        "Bayer" => Bayer.process(&mut grid, &pal_slice),
        "Bayer 2x2" => Bayer2.process(&mut grid, &pal_slice),
        "Bayer 8x8" => Bayer8.process(&mut grid, &pal_slice),
        "Selective" => apply_selective(&mut grid, &pal_slice, 25.0),
        "Ordered Selective" => apply_ordered_selective(&mut grid, &pal_slice, 25.0),
        "Dual Color Dithering" => apply_dual_color(&mut grid, &pal_slice),
        "Edge Dithering" => apply_edge_dithering(&mut grid, &pal_slice),
        "Randomized Selective" => apply_randomized_selective(&mut grid, &pal_slice, 30.0),
        "Stucki" => apply_stucki(&mut grid, &pal_slice),
        "Atkinson" => apply_atkinson(&mut grid, &pal_slice),
        "Jarvis-Judice-Ninke" | "Jarvis, Judice, and Ninke" => apply_jjn(&mut grid, &pal_slice),
        "Burkes" => apply_burkes(&mut grid, &pal_slice),
        "Sierra" => apply_sierra(&mut grid, &pal_slice),
        "Two-Row Sierra" => apply_two_row_sierra(&mut grid, &pal_slice),
        "Sierra Lite" => apply_sierra_lite(&mut grid, &pal_slice),
        _ => algo.process(&mut grid, &pal_slice),
    }
    encode_png_base64(&grid)
}

// Versions that accept explicit palette colors (e.g., from GPL) to avoid relying on built-ins
pub fn render_preview_png_with_palette(
    req: RenderRequest,
    palette_colors: Vec<[u8; 3]>,
) -> Result<String, EngineError> {
    let img = decode_data_url_to_image(&req.image_data_url)?;
    let (gw, gh) = resolve_grid(&req);
    let mut grid = resize_to_grid(&img, gw, gh);
    grid = apply_denoise_rgba(grid, req.denoise_sigma);
    apply_tone_gamma(&mut grid, req.tone_gamma);
    let algo = get_algorithm_by_name(req.algorithm.as_str());
    let pal_slice: Vec<[u8; 3]> = palette_colors;
    match req.algorithm.as_str() {
        "Floyd-Steinberg" | "Floyd–Steinberg" => FloydSteinberg.process(&mut grid, &pal_slice),
        "Bayer" => Bayer.process(&mut grid, &pal_slice),
        "Bayer 2x2" => Bayer2.process(&mut grid, &pal_slice),
        "Bayer 8x8" => Bayer8.process(&mut grid, &pal_slice),
        "Selective" => apply_selective(&mut grid, &pal_slice, 25.0),
        "Ordered Selective" => apply_ordered_selective(&mut grid, &pal_slice, 25.0),
        "Dual Color Dithering" => apply_dual_color(&mut grid, &pal_slice),
        "Edge Dithering" => apply_edge_dithering(&mut grid, &pal_slice),
        "Randomized Selective" => apply_randomized_selective(&mut grid, &pal_slice, 30.0),
        "Stucki" => apply_stucki(&mut grid, &pal_slice),
        "Atkinson" => apply_atkinson(&mut grid, &pal_slice),
        "Jarvis-Judice-Ninke" | "Jarvis, Judice, and Ninke" => apply_jjn(&mut grid, &pal_slice),
        "Burkes" => apply_burkes(&mut grid, &pal_slice),
        "Sierra" => apply_sierra(&mut grid, &pal_slice),
        "Two-Row Sierra" => apply_two_row_sierra(&mut grid, &pal_slice),
        "Sierra Lite" => apply_sierra_lite(&mut grid, &pal_slice),
        _ => algo.process(&mut grid, &pal_slice),
    }
    let target = req.display_size.unwrap_or(560);
    let up = upscale_center_to(&grid, target);
    encode_png_base64(&up)
}

pub fn render_base_png_with_palette(
    req: RenderRequest,
    palette_colors: Vec<[u8; 3]>,
) -> Result<String, EngineError> {
    let img = decode_data_url_to_image(&req.image_data_url)?;
    let (gw, gh) = resolve_grid(&req);
    let mut grid = resize_to_grid(&img, gw, gh);
    grid = apply_denoise_rgba(grid, req.denoise_sigma);
    apply_tone_gamma(&mut grid, req.tone_gamma);
    let algo = get_algorithm_by_name(req.algorithm.as_str());
    let pal_slice: Vec<[u8; 3]> = palette_colors;
    match req.algorithm.as_str() {
        "Floyd-Steinberg" | "Floyd–Steinberg" => FloydSteinberg.process(&mut grid, &pal_slice),
        "Bayer" => Bayer.process(&mut grid, &pal_slice),
        "Bayer 2x2" => Bayer2.process(&mut grid, &pal_slice),
        "Bayer 8x8" => Bayer8.process(&mut grid, &pal_slice),
        "Selective" => apply_selective(&mut grid, &pal_slice, 25.0),
        "Ordered Selective" => apply_ordered_selective(&mut grid, &pal_slice, 25.0),
        "Dual Color Dithering" => apply_dual_color(&mut grid, &pal_slice),
        "Edge Dithering" => apply_edge_dithering(&mut grid, &pal_slice),
        "Randomized Selective" => apply_randomized_selective(&mut grid, &pal_slice, 30.0),
        "Stucki" => apply_stucki(&mut grid, &pal_slice),
        "Atkinson" => apply_atkinson(&mut grid, &pal_slice),
        "Jarvis-Judice-Ninke" | "Jarvis, Judice, and Ninke" => apply_jjn(&mut grid, &pal_slice),
        "Burkes" => apply_burkes(&mut grid, &pal_slice),
        "Sierra" => apply_sierra(&mut grid, &pal_slice),
        "Two-Row Sierra" => apply_two_row_sierra(&mut grid, &pal_slice),
        "Sierra Lite" => apply_sierra_lite(&mut grid, &pal_slice),
        _ => algo.process(&mut grid, &pal_slice),
    }
    encode_png_base64(&grid)
}

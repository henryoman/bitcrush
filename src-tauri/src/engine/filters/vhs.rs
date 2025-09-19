use image::{imageops::FilterType, GenericImageView, ImageBuffer, Pixel, Rgba, RgbaImage};
use rand::{rngs::StdRng, Rng, SeedableRng};

use super::Filter;

fn clamp_u8(x: f32) -> u8 { x.max(0.0).min(255.0) as u8 }

fn luma(r: f32, g: f32, b: f32) -> f32 { 0.2126 * r + 0.7152 * g + 0.0722 * b }

fn grade(img: &mut RgbaImage, amount: f32) {
    let amt = amount.clamp(0.0, 1.0);
    let contrast = 1.0 + 0.06 * amt; // up to ~+6%
    let gamma = 1.0 - 0.08 * amt; // lift mids slightly
    let warm_r = 1.0 + 0.05 * amt; // up to +5%
    let warm_g = 1.0;
    let warm_b = 1.0 - 0.05 * amt; // down to -5%
    for p in img.pixels_mut() {
        let [r, g, b, a] = p.0;
        let (mut r, mut g, mut b) = (r as f32, g as f32, b as f32);
        r = (r - 128.0) * contrast + 128.0;
        g = (g - 128.0) * contrast + 128.0;
        b = (b - 128.0) * contrast + 128.0;
        r *= warm_r; g *= warm_g; b *= warm_b;
        r = 255.0 * ((r / 255.0).powf(1.0 / gamma));
        g = 255.0 * ((g / 255.0).powf(1.0 / gamma));
        b = 255.0 * ((b / 255.0).powf(1.0 / gamma));
        *p = Rgba([clamp_u8(r), clamp_u8(g), clamp_u8(b), a]);
    }
}

fn soft_pixelate(img: &RgbaImage, amount: f32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let scale = 1.0 - 0.18 * amount.clamp(0.0, 1.0); // up to ~18% downscale
    let tw = (w as f32 * scale).max(1.0) as u32;
    let th = (h as f32 * scale).max(1.0) as u32;
    let small = image::imageops::resize(img, tw, th, FilterType::Triangle);
    image::imageops::resize(&small, w, h, FilterType::Nearest)
}

fn scanlines(img: &mut RgbaImage, amount: f32) {
    let darken = 1.0 - 0.12 * amount.clamp(0.0, 1.0); // up to 12%
    let (w, h) = img.dimensions();
    for y in 0..h {
        if y % 2 == 1 {
            for x in 0..w {
                let p = img.get_pixel_mut(x, y);
                let [r, g, b, a] = p.0;
                *p = Rgba([
                    clamp_u8((r as f32) * darken),
                    clamp_u8((g as f32) * darken),
                    clamp_u8((b as f32) * darken),
                    a,
                ]);
            }
        }
    }
}

fn chromatic_aberration(img: &RgbaImage, amount: f32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut out = img.clone();
    let shift = if amount < 0.5 { 1 } else { 2 }; // mild to moderate
    for y in 0..h {
        for x in 0..w {
            let xr = x.saturating_sub(shift as u32);
            let xb = (x + shift as u32).min(w - 1);
            let src_r = img.get_pixel(xr, y).0;
            let src_g = img.get_pixel(x, y).0;
            let src_b = img.get_pixel(xb, y).0;
            let r = src_r[0];
            let g = src_g[1];
            let b = src_b[2];
            let a = src_g[3];
            *out.get_pixel_mut(x, y) = Rgba([r, g, b, a]);
        }
    }
    out
}

fn box_blur(img: &RgbaImage, radius: u32) -> RgbaImage {
    if radius == 0 { return img.clone(); }
    let (w, h) = img.dimensions();
    let mut tmp = img.clone();
    let mut out = img.clone();
    let r = radius as i32;
    let norm = 1.0 / (2 * r + 1) as f32;
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for dx in -r..=r {
                let xx = (x as i32 + dx).clamp(0, (w - 1) as i32) as u32;
                let p = img.get_pixel(xx, y).0;
                acc[0] += p[0] as f32; acc[1] += p[1] as f32; acc[2] += p[2] as f32; acc[3] += p[3] as f32;
            }
            tmp.put_pixel(x, y, Rgba([
                clamp_u8(acc[0] * norm),
                clamp_u8(acc[1] * norm),
                clamp_u8(acc[2] * norm),
                clamp_u8(acc[3] * norm),
            ]));
        }
    }
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for dy in -r..=r {
                let yy = (y as i32 + dy).clamp(0, (h - 1) as i32) as u32;
                let p = tmp.get_pixel(x, yy).0;
                acc[0] += p[0] as f32; acc[1] += p[1] as f32; acc[2] += p[2] as f32; acc[3] += p[3] as f32;
            }
            out.put_pixel(x, y, Rgba([
                clamp_u8(acc[0] * norm),
                clamp_u8(acc[1] * norm),
                clamp_u8(acc[2] * norm),
                clamp_u8(acc[3] * norm),
            ]));
        }
    }
    out
}

fn bloom(base: &RgbaImage, amount: f32) -> RgbaImage {
    let (w, h) = base.dimensions();
    let mut mask = RgbaImage::new(w, h);
    let thresh = 200.0 - 60.0 * amount.clamp(0.0, 1.0); // lower threshold with amount
    for (x, y, p) in base.enumerate_pixels() {
        let [r, g, b, a] = p.0;
        let br = luma(r as f32, g as f32, b as f32);
        if br > thresh { mask.put_pixel(x, y, Rgba([r, g, b, a])); }
        else { mask.put_pixel(x, y, Rgba([0, 0, 0, 0])); }
    }
    let radius = if amount < 0.5 { 2 } else { 3 };
    let blurred = box_blur(&mask, radius);
    let mut out = base.clone();
    let strength = 0.18 + 0.20 * amount; // 18%..38%
    for (x, y, p) in out.enumerate_pixels_mut() {
        let [r, g, b, a] = p.0;
        let bb = blurred.get_pixel(x, y).0;
        let nr = clamp_u8(r as f32 + strength * bb[0] as f32);
        let ng = clamp_u8(g as f32 + strength * bb[1] as f32);
        let nb = clamp_u8(b as f32 + strength * bb[2] as f32);
        *p = Rgba([nr, ng, nb, a]);
    }
    out
}

fn grain(img: &mut RgbaImage, amount: f32) {
    let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF);
    let max_amt = 6.0 * amount.clamp(0.0, 1.0); // up to +/-6
    for p in img.pixels_mut() {
        let [r, g, b, a] = p.0;
        let n = |v: u8| -> u8 {
            let delta: f32 = rng.gen_range(-max_amt..max_amt);
            clamp_u8(v as f32 + delta)
        };
        *p = Rgba([n(r), n(g), n(b), a]);
    }
}

fn vignette(img: &mut RgbaImage, amount: f32) {
    let (w, h) = img.dimensions();
    let cx = (w as f32 - 1.0) * 0.5;
    let cy = (h as f32 - 1.0) * 0.5;
    let max_r = (cx.powi(2) + cy.powi(2)).sqrt();
    let strength = 0.08 + 0.12 * amount.clamp(0.0, 1.0); // 8%..20%
    for (x, y, p) in img.enumerate_pixels_mut() {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        let r = (dx * dx + dy * dy).sqrt() / max_r;
        let v = 1.0 - strength * r.powf(2.0);
        let [rr, gg, bb, a] = p.0;
        *p = Rgba([
            clamp_u8((rr as f32) * v),
            clamp_u8((gg as f32) * v),
            clamp_u8((bb as f32) * v),
            a,
        ]);
    }
}

pub struct VhsFilter;
impl Filter for VhsFilter {
    fn name(&self) -> &'static str { "VHS" }
    fn apply(&self, img: &mut RgbaImage, amount: f32) {
        let amt = amount.clamp(0.0, 1.0);
        let mut work = soft_pixelate(img, amt);
        grade(&mut work, amt);
        work = chromatic_aberration(&work, amt);
        scanlines(&mut work, amt);
        work = bloom(&work, amt);
        grain(&mut work, 0.6 + 0.4 * amt);
        vignette(&mut work, amt);
        *img = work;
    }
}



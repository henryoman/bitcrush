use image::{imageops::FilterType, Rgba, RgbaImage};
use rand::{rngs::StdRng, Rng, SeedableRng};

fn clamp_u8(x: f32) -> u8 { x.max(0.0).min(255.0) as u8 }

fn luma(r: f32, g: f32, b: f32) -> f32 { 0.2126 * r + 0.7152 * g + 0.0722 * b }

fn grade(img: &mut RgbaImage) {
    const CONTRAST: f32 = 1.04;
    const GAMMA: f32 = 0.94;
    const WARM_R: f32 = 1.03;
    const WARM_G: f32 = 1.00;
    const WARM_B: f32 = 0.97;
    for p in img.pixels_mut() {
        let [r, g, b, a] = p.0;
        let (mut r, mut g, mut b) = (r as f32, g as f32, b as f32);
        r = (r - 128.0) * CONTRAST + 128.0;
        g = (g - 128.0) * CONTRAST + 128.0;
        b = (b - 128.0) * CONTRAST + 128.0;
        r *= WARM_R; g *= WARM_G; b *= WARM_B;
        r = 255.0 * ((r / 255.0).powf(1.0 / GAMMA));
        g = 255.0 * ((g / 255.0).powf(1.0 / GAMMA));
        b = 255.0 * ((b / 255.0).powf(1.0 / GAMMA));
        *p = Rgba([clamp_u8(r), clamp_u8(g), clamp_u8(b), a]);
    }
}

fn soft_pixelate(img: &RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    let tw = (w as f32 * 0.87) as u32;
    let th = (h as f32 * 0.87) as u32;
    let small = image::imageops::resize(img, tw.max(1), th.max(1), FilterType::Triangle);
    image::imageops::resize(&small, w, h, FilterType::Nearest)
}

fn scanlines(img: &mut RgbaImage) {
    const DARKEN: f32 = 0.92;
    let (w, h) = img.dimensions();
    for y in 0..h {
        if y % 2 == 1 {
            for x in 0..w {
                let p = img.get_pixel_mut(x, y);
                let [r, g, b, a] = p.0;
                *p = Rgba([
                    clamp_u8((r as f32) * DARKEN),
                    clamp_u8((g as f32) * DARKEN),
                    clamp_u8((b as f32) * DARKEN),
                    a,
                ]);
            }
        }
    }
}

fn chromatic_aberration(img: &RgbaImage) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut out = img.clone();
    for y in 0..h {
        for x in 0..w {
            let src_r = img.get_pixel(x.saturating_sub(1), y).0;
            let src_g = img.get_pixel(x, y).0;
            let src_b = img.get_pixel((x + 1).min(w.saturating_sub(1)), y).0;
            *out.get_pixel_mut(x, y) = Rgba([src_r[0], src_g[1], src_b[2], src_g[3]]);
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
                let xx = (x as i32 + dx).clamp(0, (w.saturating_sub(1)) as i32) as u32;
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
                let yy = (y as i32 + dy).clamp(0, (h.saturating_sub(1)) as i32) as u32;
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

fn bloom(base: &RgbaImage) -> RgbaImage {
    let (w, h) = base.dimensions();
    let mut mask = RgbaImage::new(w, h);
    const THRESH: f32 = 200.0;
    for (x, y, p) in base.enumerate_pixels() {
        let [r, g, b, a] = p.0;
        let br = luma(r as f32, g as f32, b as f32);
        if br > THRESH { mask.put_pixel(x, y, Rgba([r, g, b, a])); }
        else { mask.put_pixel(x, y, Rgba([0, 0, 0, 0])); }
    }
    let blurred = box_blur(&mask, 2);
    let mut out = base.clone();
    const STRENGTH: f32 = 0.25;
    for (x, y, p) in out.enumerate_pixels_mut() {
        let [r, g, b, a] = p.0;
        let bb = blurred.get_pixel(x, y).0;
        let nr = clamp_u8(r as f32 + STRENGTH * bb[0] as f32);
        let ng = clamp_u8(g as f32 + STRENGTH * bb[1] as f32);
        let nb = clamp_u8(b as f32 + STRENGTH * bb[2] as f32);
        *p = Rgba([nr, ng, nb, a]);
    }
    out
}

fn grain(img: &mut RgbaImage, amount: f32) {
    let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF);
    for p in img.pixels_mut() {
        let [r, g, b, a] = p.0;
        let mut n = |v: u8| -> u8 {
            let delta: f32 = rng.gen_range(-amount..amount);
            clamp_u8(v as f32 + delta)
        };
        *p = Rgba([n(r), n(g), n(b), a]);
    }
}

fn vignette(img: &mut RgbaImage) {
    let (w, h) = img.dimensions();
    let cx = (w as f32 - 1.0) * 0.5;
    let cy = (h as f32 - 1.0) * 0.5;
    let max_r = (cx.powi(2) + cy.powi(2)).sqrt();
    for (x, y, p) in img.enumerate_pixels_mut() {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        let r = (dx * dx + dy * dy).sqrt() / max_r;
        let v = 1.0 - 0.12 * r.powf(2.0);
        let [rr, gg, bb, a] = p.0;
        *p = Rgba([
            clamp_u8((rr as f32) * v),
            clamp_u8((gg as f32) * v),
            clamp_u8((bb as f32) * v),
            a,
        ]);
    }
}

pub fn apply_vhs(src: &RgbaImage) -> RgbaImage {
    let mut work = soft_pixelate(src);
    grade(&mut work);
    work = chromatic_aberration(&work);
    scanlines(&mut work);
    work = bloom(&work);
    grain(&mut work, 3.5);
    vignette(&mut work);
    work
}



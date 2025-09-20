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

// removed soft_pixelate (superseded by prefilter_input_for_vhs)

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

// removed unused chromatic_aberration (superseded by chromatic_aberration_shift)

fn chromatic_aberration_shift(img: &RgbaImage, shift: i32) -> RgbaImage {
    // shift > 0 moves red left and blue right by `shift` pixels
    let (w, h) = img.dimensions();
    let mut out = img.clone();
    for y in 0..h {
        for x in 0..w {
            let xr = (x as i32 - shift).clamp(0, (w.saturating_sub(1)) as i32) as u32;
            let xb = (x as i32 + shift).clamp(0, (w.saturating_sub(1)) as i32) as u32;
            let pr = img.get_pixel(xr, y).0;
            let pg = img.get_pixel(x, y).0;
            let pb = img.get_pixel(xb, y).0;
            *out.get_pixel_mut(x, y) = Rgba([pr[0], pg[1], pb[2], pg[3]]);
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
    // Scanline-correlated and blocky grain to avoid crisp high-frequency noise
    // Typical VHS noise is horizontally smeared and not pixel-perfect.
    let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF);
    let (w, h) = img.dimensions();
    let block: u32 = (w / 160).max(2); // coarser blocks on larger images
    for y in 0..h {
        let row_bias: f32 = rng.gen_range(-amount..amount) * 0.4; // shared per-row component
        let mut x: u32 = 0;
        while x < w {
            let bump: f32 = rng.gen_range(-amount..amount) + row_bias;
            let x_end = (x + block).min(w);
            for xx in x..x_end {
                let p = img.get_pixel_mut(xx, y);
                let [r, g, b, a] = p.0;
                let nr = clamp_u8(r as f32 + bump);
                let ng = clamp_u8(g as f32 + bump);
                let nb = clamp_u8(b as f32 + bump);
                *p = Rgba([nr, ng, nb, a]);
            }
            x = x_end;
        }
    }
}

pub fn prefilter_input_for_vhs(src: &RgbaImage) -> RgbaImage {
    // Designed for sharp digital inputs: gently low-pass and lightly rasterize
    // 1) Mild gaussian blur to remove high-frequency crispness
    let blurred = image::imageops::blur(src, 0.6);
    // 2) Slight down-up sampling to introduce analog softness
    let (w, h) = blurred.dimensions();
    let dw = ((w as f32) * 0.92).round().max(1.0) as u32;
    let dh = ((h as f32) * 0.92).round().max(1.0) as u32;
    let small = image::imageops::resize(&blurred, dw, dh, FilterType::Triangle);
    image::imageops::resize(&small, w, h, FilterType::Nearest)
}

fn rasterize_lines(src: &RgbaImage, target_lines: u32) -> RgbaImage {
    // Downsample vertically to a fixed number of lines (e.g., ~240 NTSC)
    // then upscale using nearest to create authentic line structure.
    let (w, h) = src.dimensions();
    if target_lines == 0 || h <= target_lines { return src.clone(); }
    let small = image::imageops::resize(src, w, target_lines, FilterType::Triangle);
    image::imageops::resize(&small, w, h, FilterType::Nearest)
}

fn luma_dither(img: &mut RgbaImage, amplitude: f32) {
    // Tiny correlated luma dither to break banding without looking like grain
    let (w, h) = img.dimensions();
    let mut rng = StdRng::seed_from_u64(0xA11A_D17Eu64);
    for y in 0..h {
        let row_n = rng.gen_range(-amplitude..amplitude) * 0.5; // row component
        for x in 0..w {
            // mild blue-noise-ish with blocky step every 3 px
            let col_n = if x % 3 == 0 { rng.gen_range(-amplitude..amplitude) * 0.5 } else { 0.0 };
            let p = img.get_pixel_mut(x, y);
            let [r, g, b, a] = p.0;
            let yv = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
            let n = row_n + col_n;
            let nr = clamp_u8(r as f32 + n);
            let ng = clamp_u8(g as f32 + n);
            let nb = clamp_u8(b as f32 + n);
            let _ = yv; // keep calc to show intent; currently unused directly
            *p = Rgba([nr, ng, nb, a]);
        }
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

fn raster_jitter(src: &RgbaImage, period: u32, magnitude: i32) -> RgbaImage {
    // Every `period` rows, shift row horizontally by [-magnitude, magnitude]
    if period == 0 { return src.clone(); }
    let (w, h) = src.dimensions();
    let mut out = src.clone();
    let mut rng = StdRng::seed_from_u64(0xBADC_AB1E);
    for y in (0..h).step_by(period as usize) {
        let dx = rng.gen_range(-magnitude..=magnitude);
        for x in 0..w {
            let sx = (x as i32 + dx).clamp(0, (w.saturating_sub(1)) as i32) as u32;
            out.put_pixel(x, y, *src.get_pixel(sx, y));
        }
    }
    out
}

// (removed legacy alias apply_vhs)

pub fn apply_vhs1(src: &RgbaImage) -> RgbaImage {
    // Authentic baseline: slight soften + chroma bleed + scanlines + subtle grain
    let mut work = prefilter_input_for_vhs(src);
    // Introduce mild color bleed
    work = color_bleed_simple(&work, 1.2, 0.28);
    grade(&mut work);
    work = chromatic_aberration_shift(&work, 1);
    // NTSC-like line rasterization (~240 lines for typical VHS)
    work = rasterize_lines(&work, work.height().min(240).max(160));
    scanlines(&mut work);
    work = bloom(&work);
    // Very low, coarse grain
    grain(&mut work, 1.1);
    luma_dither(&mut work, 0.8);
    vignette(&mut work);
    work
}

pub fn apply_vhs2(src: &RgbaImage) -> RgbaImage {
    // Stronger CA and bloom, mild raster jitter, with authentic prefilter + raster
    let mut work = prefilter_input_for_vhs(src);
    work = color_bleed_simple(&work, 1.3, 0.32);
    grade(&mut work);
    work = chromatic_aberration_shift(&work, 2);
    work = rasterize_lines(&work, 240);
    scanlines(&mut work);
    work = bloom(&work);
    work = box_blur(&work, 1);
    grain(&mut work, 1.6);
    luma_dither(&mut work, 0.9);
    work = raster_jitter(&work, 6, 2);
    vignette(&mut work);
    work
}

pub fn apply_vhs3(src: &RgbaImage) -> RgbaImage {
    // Heavier CA, darker scanlines, mild jitter; keep noise low and coarse
    let mut work = prefilter_input_for_vhs(src);
    work = color_bleed_simple(&work, 1.4, 0.36);
    grade(&mut work);
    work = chromatic_aberration_shift(&work, 3);
    work = rasterize_lines(&work, 240);
    const DARKEN: f32 = 0.86;
    let (w, h) = work.dimensions();
    for y in 0..h { if y % 2 == 1 { for x in 0..w {
        let p = work.get_pixel_mut(x, y); let [r,g,b,a]=p.0;
        *p = Rgba([clamp_u8((r as f32)*DARKEN),clamp_u8((g as f32)*DARKEN),clamp_u8((b as f32)*DARKEN),a]);
    }}}
    work = bloom(&work);
    grain(&mut work, 1.9);
    luma_dither(&mut work, 1.0);
    work = raster_jitter(&work, 5, 3);
    vignette(&mut work);
    work
}

pub fn apply_vhs_realistic(src: &RgbaImage) -> RgbaImage {
    // Based on NTSC YUV characteristics with added authentic rasterization
    let (w, h) = src.dimensions();
    let wu = w as usize;
    let hu = h as usize;
    let mut y_buf = vec![0.0f32; wu * hu];
    let mut u_buf = vec![0.0f32; wu * hu];
    let mut v_buf = vec![0.0f32; wu * hu];
    let idx = |x: usize, y: usize| -> usize { y * wu + x };
    // RGB -> YUV (approx BT.601)
    for yy in 0..hu {
        for xx in 0..wu {
            let p = src.get_pixel(xx as u32, yy as u32).0;
            let r = p[0] as f32;
            let g = p[1] as f32;
            let b = p[2] as f32;
            let y = 0.299 * r + 0.587 * g + 0.114 * b;
            // Analog YUV: scale factors for U and V
            let u = (b - y) * 0.492;
            let v = (r - y) * 0.877;
            let i = idx(xx, yy);
            y_buf[i] = y;
            u_buf[i] = u;
            v_buf[i] = v;
        }
    }
    // Horizontal chroma smear (simulate low chroma bandwidth)
    let radius: i32 = 4; // ~9px window for stronger chroma smear
    let mut u_s = vec![0.0f32; wu * hu];
    let mut v_s = vec![0.0f32; wu * hu];
    let norm = 1.0 / (2 * radius + 1) as f32;
    for yy in 0..hu {
        for xx in 0..wu {
            let mut su = 0.0f32;
            let mut sv = 0.0f32;
            for dx in -radius..=radius {
                let x = (xx as i32 + dx).clamp(0, (wu - 1) as i32) as usize;
                let i = idx(x, yy);
                su += u_buf[i];
                sv += v_buf[i];
            }
            let i = idx(xx, yy);
            u_s[i] = su * norm;
            v_s[i] = sv * norm;
        }
    }
    // Desaturate and slight tint shift toward green
    let sat = 0.60f32;
    for i in 0..u_s.len() {
        u_s[i] *= sat;
        v_s[i] *= sat * 0.96; // minute tint
    }
    // Reconstruct RGB
    let mut out = src.clone();
    for yy in 0..hu {
        for xx in 0..wu {
            let i = idx(xx, yy);
            let y = y_buf[i];
            let u = u_s[i];
            let v = v_s[i];
            let r = y + v / 0.877;
            let b = y + u / 0.492;
            let g = (y - 0.299 * r - 0.114 * b) / 0.587;
            let r8 = clamp_u8(r);
            let g8 = clamp_u8(g);
            let b8 = clamp_u8(b);
            let a = src.get_pixel(xx as u32, yy as u32).0[3];
            *out.get_pixel_mut(xx as u32, yy as u32) = Rgba([r8, g8, b8, a]);
        }
    }
    // Gentle CRT cues
    let mut out2 = out;
    // Line rasterization first to avoid color banding appearing too crisp
    out2 = rasterize_lines(&out2, 240);
    scanlines(&mut out2);
    out2 = box_blur(&out2, 1);
    grain(&mut out2, 1.4);
    luma_dither(&mut out2, 0.8);
    vignette(&mut out2);
    out2
}

fn scanlines_mul(img: &mut RgbaImage, factor: f32) {
    // Slight additional darkening of every other row by `factor`
    let (w, h) = img.dimensions();
    for y in 0..h {
        if y % 2 == 1 {
            for x in 0..w {
                let p = img.get_pixel_mut(x, y);
                let [r, g, b, a] = p.0;
                *p = Rgba([
                    clamp_u8((r as f32) * factor),
                    clamp_u8((g as f32) * factor),
                    clamp_u8((b as f32) * factor),
                    a,
                ]);
            }
        }
    }
}

pub fn apply_vhs_realistic2(src: &RgbaImage) -> RgbaImage {
    // Start from the realistic baseline, then add a very subtle taste of VHS3
    let mut out = apply_vhs_realistic(src);
    // Slightly stronger scanline effect
    scanlines_mul(&mut out, 0.98);
    // Very light horizontal raster jitter, infrequent and tiny
    out = raster_jitter(&out, 12, 1);
    // Tiny increase in grain
    grain(&mut out, 3.0);
    out
}

fn adjust_saturation(img: &mut RgbaImage, factor: f32) {
    if (factor - 1.0).abs() < 0.001 { return; }
    for p in img.pixels_mut() {
        let [r, g, b, a] = p.0;
        let rf = r as f32; let gf = g as f32; let bf = b as f32;
        let y = 0.2126 * rf + 0.7152 * gf + 0.0722 * bf;
        let nr = y + (rf - y) * factor;
        let ng = y + (gf - y) * factor;
        let nb = y + (bf - y) * factor;
        *p = Rgba([
            clamp_u8(nr),
            clamp_u8(ng),
            clamp_u8(nb),
            a,
        ]);
    }
}

pub fn apply_vhs_realistic3(src: &RgbaImage) -> RgbaImage {
    // Build on Realistic 2 with a touch more of VHS3 character and a tiny re-saturation
    let mut out = apply_vhs_realistic(src);
    scanlines_mul(&mut out, 0.96);
    out = raster_jitter(&out, 10, 1);
    grain(&mut out, 3.5);
    // Very slight re-saturation so it's a hair richer than Realistic 2
    adjust_saturation(&mut out, 1.12);
    out
}

pub fn apply_vhs_realistic3_mix2(src: &RgbaImage) -> RgbaImage {
    // Start from Realistic baseline, then blend in Mix 2 characteristics and R3 tweaks
    let base = apply_vhs_realistic(src);
    let mut work = color_bleed_simple(&base, 1.2, 0.28);
    work = chromatic_aberration_shift(&work, 1);
    work = stripe_noise(&work, 0.02);
    scanlines_mul(&mut work, 0.96);
    work = raster_jitter(&work, 10, 1);
    grain(&mut work, 3.5);
    adjust_saturation(&mut work, 1.12);
    work
}

// Map older names to a clean set VHS 1..7 presets
pub fn apply_vhs4(src: &RgbaImage) -> RgbaImage { apply_vhs_realistic(src) }
pub fn apply_vhs5(src: &RgbaImage) -> RgbaImage { apply_vhs_realistic2(src) }
pub fn apply_vhs6(src: &RgbaImage) -> RgbaImage { apply_vhs_realistic3(src) }
pub fn apply_vhs7(src: &RgbaImage) -> RgbaImage { apply_vhs_realistic3_mix2(src) }


// --- Extra helpers for mixes ---
fn color_bleed_simple(src: &RgbaImage, sigma: f32, mix: f32) -> RgbaImage {
    // Horizontally soften color and mix back in; preserves alpha
    if sigma <= 0.0 || mix <= 0.0 { return src.clone(); }
    let blurred = image::imageops::blur(src, sigma);
    let mut out = src.clone();
    let t = mix.clamp(0.0, 1.0);
    for (x, y, p) in out.enumerate_pixels_mut() {
        let a = p.0[3];
        let b = blurred.get_pixel(x, y).0;
        let o = src.get_pixel(x, y).0;
        let nr = ((o[0] as f32) * (1.0 - t) + (b[0] as f32) * t).clamp(0.0, 255.0) as u8;
        let ng = ((o[1] as f32) * (1.0 - t) + (b[1] as f32) * t).clamp(0.0, 255.0) as u8;
        let nb = ((o[2] as f32) * (1.0 - t) + (b[2] as f32) * t).clamp(0.0, 255.0) as u8;
        *p = Rgba([nr, ng, nb, a]);
    }
    out
}

// removed unsharp_halos (no longer used)

fn stripe_noise(src: &RgbaImage, density: f32) -> RgbaImage {
    let (w, h) = src.dimensions();
    let mut out = src.clone();
    if density <= 0.0 { return out; }
    let mut rng = StdRng::seed_from_u64(0xFEED_FACE);
    let hits = ((h as f32) * density).round().max(1.0) as u32;
    for _ in 0..hits {
        let y = rng.gen_range(0..h);
        let x0 = rng.gen_range(0..w);
        let len = rng.gen_range(8..24);
        let mut intensity = 0.7f32;
        for i in 0..len {
            let x = (x0 + i).min(w - 1);
            let p = out.get_pixel(x, y).0;
            let add = (255.0 * 0.45 * intensity).clamp(0.0, 255.0);
            let nr = (p[0] as f32 + add).clamp(0.0, 255.0) as u8;
            let ng = (p[1] as f32 + add).clamp(0.0, 255.0) as u8;
            let nb = (p[2] as f32 + add).clamp(0.0, 255.0) as u8;
            out.put_pixel(x, y, Rgba([nr, ng, nb, p[3]]));
            intensity *= 0.86;
        }
    }
    out
}

// --- Mixes ---
// removed MIX1/2/3 in favor of VHS 4..7 presets


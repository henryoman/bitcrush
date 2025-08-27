use image::Rgba;

use crate::engine::color::{ciede2000, rgb_to_lab};
use crate::engine::algorithms::RgbaImage;

fn closest(r: u8, g: u8, b: u8, palette: &[[u8;3]]) -> [u8;3] {
    if palette.is_empty() { return [r,g,b]; }
    let lab = rgb_to_lab(r,g,b);
    let mut best = palette[0];
    let mut best_d = f32::INFINITY;
    for [pr,pg,pb] in palette.iter().copied() {
        let d = ciede2000(lab, rgb_to_lab(pr,pg,pb));
        if d < best_d { best_d = d; best = [pr,pg,pb]; }
    }
    best
}

// Atkinson diffusion, denominator 8, distributes to 6 neighbors
pub fn apply_atkinson(img: &mut RgbaImage, palette: &[[u8;3]]) {
    if palette.is_empty() { return; }
    let w = img.width() as i32;
    let h = img.height() as i32;
    let mut buf = img.clone();
    for y in 0..h {
        for x in 0..w {
            let p = buf.get_pixel(x as u32, y as u32).0;
            let (r,g,b,a) = (p[0], p[1], p[2], p[3]);
            let chosen = closest(r,g,b,palette);
            img.put_pixel(x as u32, y as u32, Rgba([chosen[0],chosen[1],chosen[2],a]));
            let er = r as i16 - chosen[0] as i16;
            let eg = g as i16 - chosen[1] as i16;
            let eb = b as i16 - chosen[2] as i16;

            let spread = |dx: i32, dy: i32, buf: &mut RgbaImage| {
                let nx = x + dx; let ny = y + dy;
                if nx >= 0 && nx < w && ny >= 0 && ny < h {
                    let mut q = buf.get_pixel(nx as u32, ny as u32).0;
                    let add = |c: u8, e: i16| -> u8 { (c as i16 + e / 8).clamp(0,255) as u8 };
                    q[0] = add(q[0], er); q[1] = add(q[1], eg); q[2] = add(q[2], eb);
                    buf.put_pixel(nx as u32, ny as u32, Rgba(q));
                }
            };

            spread(1, 0, &mut buf); spread(2, 0, &mut buf);
            spread(-1, 1, &mut buf); spread(0, 1, &mut buf); spread(1, 1, &mut buf);
            spread(0, 2, &mut buf);
        }
    }
}



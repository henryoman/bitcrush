use image::Rgba;

use crate::engine::color::{lab_distance, rgb_to_lab};
use crate::engine::algorithms::RgbaImage;

fn blue_noise(x: u32, y: u32, seed: u32) -> f32 {
    let mut n = x.wrapping_mul(73).wrapping_add(y.wrapping_mul(37)).wrapping_add(seed);
    n ^= n << 13; n = n.wrapping_sub(n.wrapping_mul(n.wrapping_mul(15731).wrapping_add(789221)).wrapping_add(1376312589));
    let v = (n & 0x7fffffff) as f32 / 0x7fffffff as f32;
    v
}

pub fn apply_randomized_selective(img: &mut RgbaImage, palette: &[[u8;3]], threshold: f32) {
    if palette.is_empty() { return; }
    let w = img.width();
    let h = img.height();
    let seed = 12345u32;
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x,y).0;
            let (r,g,b,a) = (p[0], p[1], p[2], p[3]);
            let lab = rgb_to_lab(r,g,b);
            let mut best = palette[0];
            let mut best_d = f32::INFINITY;
            for c in palette.iter().copied() {
                let d = lab_distance(lab, rgb_to_lab(c[0], c[1], c[2]));
                if d < best_d { best_d = d; best = c; }
            }
            if best_d > threshold {
                let noise = blue_noise(x,y,seed) - 0.5;
                let chosen = if noise > 0.0 { best } else { best };
                img.put_pixel(x,y,Rgba([chosen[0],chosen[1],chosen[2],a]));
            } else {
                img.put_pixel(x,y,Rgba([best[0],best[1],best[2],a]));
            }
        }
    }
}



use image::Rgba;

use crate::engine::color::{lab_distance, rgb_to_lab};
use crate::engine::algorithms::RgbaImage;

fn blue_noise(x: u32, y: u32, seed: u32) -> f32 {
    let mut n = x.wrapping_mul(73).wrapping_add(y.wrapping_mul(37)).wrapping_add(seed);
    n ^= n << 13; n = n.wrapping_sub(n.wrapping_mul(n.wrapping_mul(15731).wrapping_add(789221)).wrapping_add(1376312589));
    let v = (n & 0x7fffffff) as f32 / 0x7fffffff as f32;
    v
}

fn two_closest_with_dist(r: u8, g: u8, b: u8, palette: &[[u8;3]]) -> ([u8;3], f32, [u8;3], f32) {
    if palette.is_empty() { return ([r,g,b], 0.0, [r,g,b], 0.0); }
    let lab = rgb_to_lab(r,g,b);
    let mut best1 = palette[0];
    let mut best2 = palette[0];
    let mut d1 = f32::INFINITY;
    let mut d2 = f32::INFINITY;
    for c in palette.iter().copied() {
        let d = lab_distance(lab, rgb_to_lab(c[0], c[1], c[2]));
        if d < d1 { d2 = d1; best2 = best1; d1 = d; best1 = c; }
        else if d < d2 { d2 = d; best2 = c; }
    }
    (best1, d1, best2, d2)
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
            let (c1, d1, c2, d2) = two_closest_with_dist(r,g,b,palette);
            if d1 > threshold {
                // Randomized thresholding between two closest using blue-noise
                let noise = blue_noise(x,y,seed);
                let ratio = if d1 + d2 > 0.0 { d1 / (d1 + d2) } else { 0.5 };
                let chosen = if noise > ratio { c2 } else { c1 };
                let dst = img.get_pixel_mut(x,y);
                *dst = Rgba([chosen[0],chosen[1],chosen[2],a]);
            } else {
                let dst = img.get_pixel_mut(x,y);
                *dst = Rgba([c1[0],c1[1],c1[2],a]);
            }
        }
    }
}



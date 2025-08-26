use image::Rgba;

use crate::engine::color::{lab_distance, rgb_to_lab};
use crate::engine::algorithms::RgbaImage;

const ORDERED_8X8: [[u8;8];8] = [
    [0,32,8,40,2,34,10,42],
    [48,16,56,24,50,18,58,26],
    [12,44,4,36,14,46,6,38],
    [60,28,52,20,62,30,54,22],
    [3,35,11,43,1,33,9,41],
    [51,19,59,27,49,17,57,25],
    [15,47,7,39,13,45,5,37],
    [63,31,55,23,61,29,53,21],
];

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

pub fn apply_ordered_selective(img: &mut RgbaImage, palette: &[[u8;3]], threshold: f32) {
    if palette.is_empty() { return; }
    let w = img.width();
    let h = img.height();
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x,y).0;
            let (r,g,b,a) = (p[0], p[1], p[2], p[3]);
            let (c1, d1, c2, d2) = two_closest_with_dist(r,g,b,palette);
            let best_d = d1;
            if best_d > threshold {
                // Ordered thresholding between the two closest colors using an 8x8 matrix
                let t = ORDERED_8X8[(y%8) as usize][(x%8) as usize] as f32 / 64.0;
                let ratio = if d1 + d2 > 0.0 { d1 / (d1 + d2) } else { 0.5 };
                let chosen = if t > ratio { c2 } else { c1 };
                let dst = img.get_pixel_mut(x,y);
                *dst = Rgba([chosen[0],chosen[1],chosen[2],a]);
            } else {
                let dst = img.get_pixel_mut(x,y);
                *dst = Rgba([c1[0],c1[1],c1[2],a]);
            }
        }
    }
}



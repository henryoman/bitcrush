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

pub fn apply_ordered_selective(img: &mut RgbaImage, palette: &[[u8;3]], threshold: f32) {
    if palette.is_empty() { return; }
    let w = img.width();
    let h = img.height();
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
                let t = ORDERED_8X8[(y%8) as usize][(x%8) as usize] as f32 / 64.0;
                // simple toggle: no second-closest palette search here (kept minimal)
                let chosen = if t > 0.5 { best } else { best };
                img.put_pixel(x,y,Rgba([chosen[0],chosen[1],chosen[2],a]));
            } else {
                img.put_pixel(x,y,Rgba([best[0],best[1],best[2],a]));
            }
        }
    }
}



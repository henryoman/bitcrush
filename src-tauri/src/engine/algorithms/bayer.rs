use image::Rgba;
use rayon::prelude::*;

use crate::engine::color::{brightness, lab_distance, rgb_to_lab};

use super::{Algorithm, RgbaImage};

const BAYER_4X4: [[u8;4];4] = [
    [0, 8, 2,10],
    [12,4,14,6],
    [3,11,1, 9],
    [15,7,13,5],
];

fn find_two_closest(r: u8, g: u8, b: u8, palette: &[[u8;3]]) -> ([u8;3],[u8;3]) {
    if palette.is_empty() { return ([r,g,b],[r,g,b]); }
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
    (best1, best2)
}

#[derive(Debug, Clone, Copy)]
pub struct Bayer;

impl Algorithm for Bayer {
    fn process(&self, img: &mut RgbaImage, palette: &[[u8;3]]) {
        if palette.is_empty() { return; }
        let w = img.width();
        let h = img.height();
        let pixels = img.as_mut();
        
        // Process rows in parallel
        pixels.par_chunks_mut((w * 4) as usize)
            .enumerate()
            .for_each(|(y_idx, row)| {
                let y = y_idx as u32;
                for x in 0..w {
                    let idx = (x * 4) as usize;
                    let r = row[idx];
                    let g = row[idx + 1];
                    let b = row[idx + 2];
                    let a = row[idx + 3];
                    
                    let (c1, c2) = find_two_closest(r, g, b, palette);
                    let br = brightness(r, g, b);
                    let br1 = brightness(c1[0], c1[1], c1[2]);
                    let br2 = brightness(c2[0], c2[1], c2[2]);
                    let bayer_val = (BAYER_4X4[(y%4) as usize][(x%4) as usize] as f32) / 16.0;
                    let diff1 = (br - br1).abs();
                    let diff2 = (br - br2).abs();
                    let chosen = if bayer_val < diff1 && diff2 < diff1 * 1.5 { c2 } else { c1 };
                    row[idx] = chosen[0];
                    row[idx + 1] = chosen[1];
                    row[idx + 2] = chosen[2];
                    // a stays the same
                }
            });
    }
}



use image::Rgba;
use rayon::prelude::*;

use crate::engine::color::rgb_to_lab;

use super::{Algorithm, RgbaImage};

fn find_closest_artistic(r: u8, g: u8, b: u8, palette: &[[u8;3]], x: u32, y: u32) -> [u8;3] {
    if palette.is_empty() { return [r,g,b]; }
    let [l,a,b_] = rgb_to_lab(r,g,b);
    let mut best = palette[0];
    let mut best_d = f32::INFINITY;
    for [pr,pg,pb] in palette.iter().copied() {
        let [l2,a2,b2] = rgb_to_lab(pr,pg,pb);
        let dl = l - l2; let da = a - a2; let db = b_ - b2;
        let spatial = ((x as f32 * 0.7).sin() + (y as f32 * 0.5).cos()) * 2.0;
        let d = (dl*dl + da*da + db*db).sqrt() + spatial;
        if d < best_d { best_d = d; best = [pr,pg,pb]; }
    }
    best
}

#[derive(Debug, Clone, Copy)]
pub struct Artistic;

impl Algorithm for Artistic {
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
                    
                    let er = (((r as i16 - 128) as f32) * 1.2 + 128.0).clamp(0.0, 255.0) as u8;
                    let eg = (((g as i16 - 128) as f32) * 1.2 + 128.0).clamp(0.0, 255.0) as u8;
                    let eb = (((b as i16 - 128) as f32) * 1.2 + 128.0).clamp(0.0, 255.0) as u8;
                    let c = find_closest_artistic(er, eg, eb, palette, x, y);
                    row[idx] = c[0];
                    row[idx + 1] = c[1];
                    row[idx + 2] = c[2];
                    // a stays the same
                }
            });
    }
}



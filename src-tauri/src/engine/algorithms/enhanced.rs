use image::Rgba;
use rayon::prelude::*;

use crate::engine::color::{ciede2000, rgb_to_lab};

use super::{Algorithm, RgbaImage};

#[derive(Debug, Clone, Copy)]
pub struct Enhanced;

impl Algorithm for Enhanced {
    fn process(&self, img: &mut RgbaImage, palette: &[[u8;3]]) {
        if palette.is_empty() { return; }
        // Precompute LAB for palette
        let pal_lab: Vec<[f32;3]> = palette.iter()
            .map(|[r,g,b]| rgb_to_lab(*r, *g, *b))
            .collect();
        
        // Process rows in parallel
        let width = img.width();
        let pixels = img.as_mut();
        
        pixels.par_chunks_mut((width * 4) as usize)
            .for_each(|row| {
                for chunk in row.chunks_exact_mut(4) {
                    let r = chunk[0];
                    let g = chunk[1];
                    let b = chunk[2];
                    
                    let lab = rgb_to_lab(r, g, b);
                    let mut best = 0usize;
                    let mut best_d = f32::INFINITY;
                    for (i, pl) in pal_lab.iter().enumerate() {
                        let d = ciede2000(lab, *pl);
                        if d < best_d { best_d = d; best = i; }
                    }
                    let c = palette[best];
                    chunk[0] = c[0];
                    chunk[1] = c[1];
                    chunk[2] = c[2];
                    // chunk[3] (alpha) stays the same
                }
            });
    }
}



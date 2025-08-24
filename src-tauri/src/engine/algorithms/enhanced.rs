use image::Rgba;

use crate::engine::color::{lab_distance_enhanced, rgb_to_lab};

use super::{Algorithm, RgbaImage};

#[derive(Debug, Clone, Copy)]
pub struct Enhanced;

impl Algorithm for Enhanced {
    fn name(&self) -> &'static str { "Enhanced" }

    fn process(&self, img: &mut RgbaImage, palette: &[[u8;3]]) {
        if palette.is_empty() { return; }
        // Precompute LAB for palette
        let mut pal_lab: Vec<[f32;3]> = Vec::with_capacity(palette.len());
        for [r,g,b] in palette {
            pal_lab.push(rgb_to_lab(*r, *g, *b));
        }
        for pixel in img.pixels_mut() {
            let [r,g,b,a] = pixel.0;
            let lab = rgb_to_lab(r,g,b);
            let mut best = 0usize;
            let mut best_d = f32::INFINITY;
            for (i, pl) in pal_lab.iter().enumerate() {
                let d = lab_distance_enhanced(lab, *pl);
                if d < best_d { best_d = d; best = i; }
            }
            let c = palette[best];
            *pixel = Rgba([c[0], c[1], c[2], a]);
        }
    }
}



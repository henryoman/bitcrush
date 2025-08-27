use image::Rgba;

use crate::engine::color::{ciede2000, rgb_to_lab};

use super::{Algorithm, RgbaImage};

#[derive(Debug, Clone, Copy)]
pub struct Standard;

impl Algorithm for Standard {
    fn process(&self, img: &mut RgbaImage, palette: &[[u8;3]]) {
        if palette.is_empty() { return; }
        // Precompute LAB for palette
        let mut pal_lab: Vec<[f32;3]> = Vec::with_capacity(palette.len());
        for [pr,pg,pb] in palette.iter().copied() {
            pal_lab.push(rgb_to_lab(pr,pg,pb));
        }
        for pixel in img.pixels_mut() {
            let [r,g,b,a] = pixel.0;
            let lab = rgb_to_lab(r,g,b);
            let mut best_idx = 0usize;
            let mut best_d = f32::INFINITY;
            for (i, pl) in pal_lab.iter().enumerate() {
                let d = ciede2000(lab, *pl);
                if d < best_d { best_d = d; best_idx = i; }
            }
            let c = palette[best_idx];
            *pixel = Rgba([c[0], c[1], c[2], a]);
        }
    }
}



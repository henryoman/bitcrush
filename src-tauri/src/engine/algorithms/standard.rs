use image::Rgba;

use crate::engine::color::{lab_distance, rgb_to_lab};

use super::{Algorithm, RgbaImage};

#[derive(Debug, Clone, Copy)]
pub struct Standard;

impl Algorithm for Standard {
    fn process(&self, img: &mut RgbaImage, palette: &[[u8;3]]) {
        if palette.is_empty() { return; }
        for pixel in img.pixels_mut() {
            let [r,g,b,a] = pixel.0;
            let lab = rgb_to_lab(r,g,b);
            let mut best_idx = 0usize;
            let mut best_d = f32::INFINITY;
            for (i, [pr,pg,pb]) in palette.iter().copied().enumerate() {
                let d = lab_distance(lab, rgb_to_lab(pr,pg,pb));
                if d < best_d { best_d = d; best_idx = i; }
            }
            let c = palette[best_idx];
            *pixel = Rgba([c[0], c[1], c[2], a]);
        }
    }
}



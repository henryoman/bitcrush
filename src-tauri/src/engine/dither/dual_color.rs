use image::Rgba;

use crate::engine::color::{lab_distance, rgb_to_lab};
use crate::engine::algorithms::RgbaImage;

fn two_closest(r: u8, g: u8, b: u8, palette: &[[u8;3]]) -> ([u8;3],[u8;3]) {
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

pub fn apply_dual_color(img: &mut RgbaImage, palette: &[[u8;3]]) {
    if palette.is_empty() { return; }
    let w = img.width();
    let h = img.height();
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x,y).0;
            let (r,g,b,a) = (p[0], p[1], p[2], p[3]);
            let (c1,c2) = two_closest(r,g,b,palette);
            let brightness = (0.299*r as f32 + 0.587*g as f32 + 0.114*b as f32) / 255.0;
            let chosen = if brightness > 0.5 { c1 } else { c2 };
            img.put_pixel(x,y,Rgba([chosen[0], chosen[1], chosen[2], a]));
        }
    }
}



use image::Rgba;

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
        for y in 0..h {
            for x in 0..w {
                let p = img.get_pixel(x,y).0;
                let (r,g,b,a) = (p[0], p[1], p[2], p[3]);
                let (c1, c2) = find_two_closest(r,g,b,palette);
                let br = brightness(r,g,b);
                let br1 = brightness(c1[0],c1[1],c1[2]);
                let br2 = brightness(c2[0],c2[1],c2[2]);
                let bayer_val = (BAYER_4X4[(y%4) as usize][(x%4) as usize] as f32) / 16.0;
                let diff1 = (br - br1).abs();
                let diff2 = (br - br2).abs();
                let chosen = if bayer_val < diff1 && diff2 < diff1 * 1.5 { c2 } else { c1 };
                img.put_pixel(x,y,Rgba([chosen[0], chosen[1], chosen[2], a]));
            }
        }
    }
}



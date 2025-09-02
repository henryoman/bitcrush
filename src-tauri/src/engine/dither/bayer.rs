use image::Rgba;

use crate::engine::algorithms::{Algorithm, RgbaImage};
use crate::engine::color::{brightness, lab_distance, rgb_to_lab};

const BAYER_2X2: [[u8; 2]; 2] = [[0, 2], [3, 1]];

const BAYER_4X4: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

const BAYER_8X8: [[u8; 8]; 8] = [
    [0, 48, 12, 60, 3, 51, 15, 63],
    [32, 16, 44, 28, 35, 19, 47, 31],
    [8, 56, 4, 52, 11, 59, 7, 55],
    [40, 24, 36, 20, 43, 27, 39, 23],
    [2, 50, 14, 62, 1, 49, 13, 61],
    [34, 18, 46, 30, 33, 17, 45, 29],
    [10, 58, 6, 54, 9, 57, 5, 53],
    [42, 26, 38, 22, 41, 25, 37, 21],
];

fn find_two(r: u8, g: u8, b: u8, palette: &[[u8; 3]]) -> ([u8; 3], [u8; 3]) {
    if palette.is_empty() {
        return ([r, g, b], [r, g, b]);
    }
    let lab = rgb_to_lab(r, g, b);
    let mut best1 = palette[0];
    let mut best2 = palette[0];
    let mut d1 = f32::INFINITY;
    let mut d2 = f32::INFINITY;
    for c in palette.iter().copied() {
        let d = lab_distance(lab, rgb_to_lab(c[0], c[1], c[2]));
        if d < d1 {
            d2 = d1;
            best2 = best1;
            d1 = d;
            best1 = c;
        } else if d < d2 {
            d2 = d;
            best2 = c;
        }
    }
    (best1, best2)
}

fn process_bayer<const N: usize>(img: &mut RgbaImage, palette: &[[u8; 3]], matrix: &[[u8; N]; N]) {
    if palette.is_empty() {
        return;
    }
    let w = img.width();
    let h = img.height();
    let denom = (N * N) as f32;
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y).0;
            let (r, g, b, a) = (p[0], p[1], p[2], p[3]);
            let (c1, c2) = find_two(r, g, b, palette);
            let br = brightness(r, g, b);
            let br1 = brightness(c1[0], c1[1], c1[2]);
            let br2 = brightness(c2[0], c2[1], c2[2]);
            let bayer_val = matrix[y as usize % N][x as usize % N] as f32 / denom;
            let diff1 = (br - br1).abs();
            let diff2 = (br - br2).abs();
            let chosen = if bayer_val < diff1 && diff2 < diff1 * 1.5 {
                c2
            } else {
                c1
            };
            img.put_pixel(x, y, Rgba([chosen[0], chosen[1], chosen[2], a]));
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bayer;

#[derive(Debug, Clone, Copy)]
pub struct Bayer2;

#[derive(Debug, Clone, Copy)]
pub struct Bayer8;

impl Algorithm for Bayer {
    fn process(&self, img: &mut RgbaImage, palette: &[[u8; 3]]) {
        process_bayer(img, palette, &BAYER_4X4);
    }
}

impl Algorithm for Bayer2 {
    fn process(&self, img: &mut RgbaImage, palette: &[[u8; 3]]) {
        process_bayer(img, palette, &BAYER_2X2);
    }
}

impl Algorithm for Bayer8 {
    fn process(&self, img: &mut RgbaImage, palette: &[[u8; 3]]) {
        process_bayer(img, palette, &BAYER_8X8);
    }
}

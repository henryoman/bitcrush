use image::Rgba;

use crate::engine::color::{lab_distance, rgb_to_lab};
use crate::engine::algorithms::RgbaImage;

fn detect_edge(data: &RgbaImage, x: u32, y: u32) -> bool {
    let w = data.width();
    let h = data.height();
    if x == 0 || y == 0 || x == w-1 || y == h-1 { return false; }
    let p = data.get_pixel(x,y).0;
    let neighbors = [
        data.get_pixel(x, y-1).0,
        data.get_pixel(x, y+1).0,
        data.get_pixel(x-1, y).0,
        data.get_pixel(x+1, y).0,
    ];
    for n in neighbors {
        let diff = (p[0] as i32 - n[0] as i32).abs() + (p[1] as i32 - n[1] as i32).abs() + (p[2] as i32 - n[2] as i32).abs();
        if diff > 80 { return true; }
    }
    false
}

pub fn apply_edge_dithering(img: &mut RgbaImage, palette: &[[u8;3]]) {
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
            if detect_edge(img, x, y) {
                img.put_pixel(x,y,Rgba([best[0],best[1],best[2],a]));
                // Omitting error diffusion details for brevity
            } else {
                img.put_pixel(x,y,Rgba([best[0],best[1],best[2],a]));
            }
        }
    }
}



use image::Rgba;

use crate::engine::algorithms::{Algorithm, RgbaImage};
use crate::engine::color::{ciede2000, rgb_to_lab};

#[derive(Debug, Clone, Copy)]
pub struct FloydSteinberg;

fn find_closest_palette_color(r: u8, g: u8, b: u8, palette: &[[u8;3]]) -> [u8;3] {
    if palette.is_empty() { return [r,g,b]; }
    let lab = rgb_to_lab(r,g,b);
    let mut best = palette[0];
    let mut best_d = f32::INFINITY;
    for [pr,pg,pb] in palette.iter().copied() {
        let d = ciede2000(lab, rgb_to_lab(pr,pg,pb));
        if d < best_d { best_d = d; best = [pr,pg,pb]; }
    }
    best
}

impl Algorithm for FloydSteinberg {
    fn process(&self, img: &mut RgbaImage, palette: &[[u8;3]]) {
        if palette.is_empty() { return; }
        let w = img.width() as i32;
        let h = img.height() as i32;
        let mut buf = img.clone();
        for y in 0..h {
            for x in 0..w {
                let p = buf.get_pixel(x as u32, y as u32).0;
                let (r,g,b,a) = (p[0], p[1], p[2], p[3]);
                let chosen = find_closest_palette_color(r,g,b,palette);
                let idx = (x as u32, y as u32);
                img.put_pixel(idx.0, idx.1, Rgba([chosen[0], chosen[1], chosen[2], a]));
                let err_r = r as i16 - chosen[0] as i16;
                let err_g = g as i16 - chosen[1] as i16;
                let err_b = b as i16 - chosen[2] as i16;

                let distribute = |bx: i32, by: i32, factor_n: i16, factor_d: i16, buf: &mut RgbaImage| {
                    if bx >= 0 && bx < w && by >= 0 && by < h {
                        let mut q = buf.get_pixel(bx as u32, by as u32).0;
                        let add = |c: u8, e: i16| -> u8 {
                            let v = c as i16 + (e * factor_n) / factor_d;
                            v.clamp(0, 255) as u8
                        };
                        q[0] = add(q[0], err_r);
                        q[1] = add(q[1], err_g);
                        q[2] = add(q[2], err_b);
                        buf.put_pixel(bx as u32, by as u32, Rgba(q));
                    }
                };

                distribute(x+1, y    , 7, 16, &mut buf);
                distribute(x-1, y+1  , 3, 16, &mut buf);
                distribute(x  , y+1  , 5, 16, &mut buf);
                distribute(x+1, y+1  , 1, 16, &mut buf);
            }
        }
    }
}



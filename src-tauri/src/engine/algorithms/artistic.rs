use image::Rgba;

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
        for y in 0..h {
            for x in 0..w {
                let p = img.get_pixel(x,y).0;
                let (r,g,b,a) = (p[0], p[1], p[2], p[3]);
                let er = (((r as i16 - 128) as f32) * 1.2 + 128.0).clamp(0.0, 255.0) as u8;
                let eg = (((g as i16 - 128) as f32) * 1.2 + 128.0).clamp(0.0, 255.0) as u8;
                let eb = (((b as i16 - 128) as f32) * 1.2 + 128.0).clamp(0.0, 255.0) as u8;
                let c = find_closest_artistic(er,eg,eb,palette,x,y);
                img.put_pixel(x,y,Rgba([c[0],c[1],c[2],a]));
            }
        }
    }
}



use image::Rgba;

use crate::engine::color::{ciede2000, rgb_to_lab};
use crate::engine::algorithms::RgbaImage;

fn find_closest(r: u8, g: u8, b: u8, palette: &[[u8;3]]) -> [u8;3] {
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

// Stucki kernel weights (normalized denominator 42)
// Row 0:      0   0   X  8   4
// Row 1:      2   4   8  4   2
// Row 2:      1   2   4  2   1
pub fn apply_stucki(img: &mut RgbaImage, palette: &[[u8;3]]) {
    if palette.is_empty() { return; }
    let w = img.width() as i32;
    let h = img.height() as i32;
    let mut buf = img.clone();
    for y in 0..h {
        let left_to_right = (y % 2) == 0;
        let xr: Box<dyn Iterator<Item=i32>> = if left_to_right { Box::new(0..w) } else { Box::new((0..w).rev()) };
        for x in xr {
            let p = buf.get_pixel(x as u32, y as u32).0;
            let (r,g,b,a) = (p[0], p[1], p[2], p[3]);
            let chosen = find_closest(r,g,b,palette);
            img.put_pixel(x as u32, y as u32, Rgba([chosen[0],chosen[1],chosen[2],a]));
            let err_r = r as i16 - chosen[0] as i16;
            let err_g = g as i16 - chosen[1] as i16;
            let err_b = b as i16 - chosen[2] as i16;

            let scatter = |dx: i32, dy: i32, num: i16, den: i16, buf: &mut RgbaImage| {
                let nx = x + dx; let ny = y + dy;
                if nx >= 0 && nx < w && ny >= 0 && ny < h {
                    let mut q = buf.get_pixel(nx as u32, ny as u32).0;
                    let add = |c: u8, e: i16| -> u8 { (c as i16 + (e * num) / den).clamp(0,255) as u8 };
                    q[0] = add(q[0], err_r); q[1] = add(q[1], err_g); q[2] = add(q[2], err_b);
                    buf.put_pixel(nx as u32, ny as u32, Rgba(q));
                }
            };

            // Normalize by 42
            if left_to_right {
                scatter( 1, 0, 8, 42, &mut buf); scatter( 2, 0, 4, 42, &mut buf);
                scatter(-2, 1, 2, 42, &mut buf); scatter(-1, 1, 4, 42, &mut buf); scatter(0, 1, 8, 42, &mut buf); scatter(1, 1, 4, 42, &mut buf); scatter(2, 1, 2, 42, &mut buf);
                scatter(-2, 2, 1, 42, &mut buf); scatter(-1, 2, 2, 42, &mut buf); scatter(0, 2, 4, 42, &mut buf); scatter(1, 2, 2, 42, &mut buf); scatter(2, 2, 1, 42, &mut buf);
            } else {
                scatter(-1, 0, 8, 42, &mut buf); scatter(-2, 0, 4, 42, &mut buf);
                scatter( 2, 1, 2, 42, &mut buf); scatter( 1, 1, 4, 42, &mut buf); scatter(0, 1, 8, 42, &mut buf); scatter(-1, 1, 4, 42, &mut buf); scatter(-2, 1, 2, 42, &mut buf);
                scatter( 2, 2, 1, 42, &mut buf); scatter( 1, 2, 2, 42, &mut buf); scatter(0, 2, 4, 42, &mut buf); scatter(-1, 2, 2, 42, &mut buf); scatter(-2, 2, 1, 42, &mut buf);
            }
        }
    }
}



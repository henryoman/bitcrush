use image::Rgba;

use crate::engine::algorithms::RgbaImage;
use crate::engine::color::{ciede2000, rgb_to_lab};

fn luminance(p: [u8;4]) -> f32 { 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32 }

fn closest_palette_color(r: u8, g: u8, b: u8, palette: &[[u8;3]]) -> [u8;3] {
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

// Sobel gradient magnitude on luma; returns (gx, gy)
fn sobel_at(buf: &RgbaImage, x: i32, y: i32) -> (f32, f32) {
    let w = buf.width() as i32;
    let h = buf.height() as i32;
    let get = |ix: i32, iy: i32| -> f32 {
        if ix < 0 || iy < 0 || ix >= w || iy >= h { return 0.0; }
        luminance(buf.get_pixel(ix as u32, iy as u32).0) / 255.0
    };
    let tl = get(x-1,y-1); let tc = get(x,y-1); let tr = get(x+1,y-1);
    let ml = get(x-1,y  ); let _mc = get(x,y  ); let mr = get(x+1,y  );
    let bl = get(x-1,y+1); let bc = get(x,y+1); let br = get(x+1,y+1);
    let gx = (-1.0*tl) + (1.0*tr) + (-2.0*ml) + (2.0*mr) + (-1.0*bl) + (1.0*br);
    let gy = (-1.0*tl) + (-2.0*tc) + (-1.0*tr) + (1.0*bl) + (2.0*bc) + (1.0*br);
    (gx, gy)
}

pub fn apply_edge_dithering(img: &mut RgbaImage, palette: &[[u8;3]]) {
    if palette.is_empty() { return; }
    let w = img.width() as i32;
    let h = img.height() as i32;
    let mut buf = img.clone();

    for y in 0..h {
        for x in 0..w {
            let p = buf.get_pixel(x as u32, y as u32).0;
            let (r,g,b,a) = (p[0], p[1], p[2], p[3]);
            let chosen = closest_palette_color(r,g,b,palette);
            img.put_pixel(x as u32, y as u32, Rgba([chosen[0], chosen[1], chosen[2], a]));

            // Edge-aware: steer diffusion mostly along the edge tangent to reduce ringing across edges
            let (gx, gy) = sobel_at(&buf, x, y);
            let mag = (gx*gx + gy*gy).sqrt();
            // Tangent vector (edge direction) is perpendicular to gradient
            let tx = -gy; let ty = gx;
            // Normalize weights for neighbors using projection of offsets onto tangent
            let neighbors: &[(i32,i32,f32)] = &[
                (1, 0, 7.0/16.0),
                (-1, 1, 3.0/16.0),
                (0, 1, 5.0/16.0),
                (1, 1, 1.0/16.0),
            ];

            let err_r = r as f32 - chosen[0] as f32;
            let err_g = g as f32 - chosen[1] as f32;
            let err_b = b as f32 - chosen[2] as f32;

            for (dx, dy, base_w) in neighbors.iter().copied() {
                let nx = x + dx; let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= w || ny >= h { continue; }
                // steering factor: favor along tangent when strong gradient
                let proj = (dx as f32 * tx + dy as f32 * ty).abs();
                let steer = if mag > 0.01 { (1.0 + proj).min(2.0) } else { 1.0 };
                let wgt = base_w * steer;
                let mut q = buf.get_pixel(nx as u32, ny as u32).0;
                let add = |c: u8, e: f32, wgt: f32| -> u8 {
                    let v = c as f32 + e * wgt;
                    v.clamp(0.0, 255.0) as u8
                };
                q[0] = add(q[0], err_r, wgt);
                q[1] = add(q[1], err_g, wgt);
                q[2] = add(q[2], err_b, wgt);
                buf.put_pixel(nx as u32, ny as u32, Rgba(q));
            }
        }
    }
}



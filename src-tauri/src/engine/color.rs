pub fn hex_to_rgb(hex: &str) -> Option<[u8; 3]> {
    let s = hex.trim();
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 { return None; }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some([r, g, b])
}

#[inline]
pub fn rgb_to_lab(r: u8, g: u8, b: u8) -> [f32; 3] {
    // Normalize to 0..1
    let mut rn = r as f32 / 255.0;
    let mut gn = g as f32 / 255.0;
    let mut bn = b as f32 / 255.0;

    // Inverse gamma
    rn = if rn > 0.04045 { ((rn + 0.055) / 1.055).powf(2.4) } else { rn / 12.92 };
    gn = if gn > 0.04045 { ((gn + 0.055) / 1.055).powf(2.4) } else { gn / 12.92 };
    bn = if bn > 0.04045 { ((bn + 0.055) / 1.055).powf(2.4) } else { bn / 12.92 };

    // XYZ
    let mut x = rn * 0.4124564 + gn * 0.3575761 + bn * 0.1804375;
    let mut y = rn * 0.2126729 + gn * 0.7151522 + bn * 0.0721750;
    let mut z = rn * 0.0193339 + gn * 0.1191920 + bn * 0.9503041;

    // D65 white
    x /= 0.95047;
    y /= 1.0;
    z /= 1.08883;

    let fx = if x > 0.008856 { x.powf(1.0 / 3.0) } else { (7.787 * x) + (16.0 / 116.0) };
    let fy = if y > 0.008856 { y.powf(1.0 / 3.0) } else { (7.787 * y) + (16.0 / 116.0) };
    let fz = if z > 0.008856 { z.powf(1.0 / 3.0) } else { (7.787 * z) + (16.0 / 116.0) };

    let l = (116.0 * fy) - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    [l, a, b]
}

#[inline]
pub fn lab_distance(lab1: [f32; 3], lab2: [f32; 3]) -> f32 {
    let dl = lab1[0] - lab2[0];
    let da = lab1[1] - lab2[1];
    let db = lab1[2] - lab2[2];
    (dl * dl + da * da + db * db).sqrt()
}

#[inline]
pub fn lab_distance_enhanced(lab1: [f32; 3], lab2: [f32; 3]) -> f32 {
    // Weighted distance used by Enhanced in TS: 2*L^2 + 4*A^2 + 1*B^2
    let dl = lab1[0] - lab2[0];
    let da = lab1[1] - lab2[1];
    let db = lab1[2] - lab2[2];
    (2.0 * dl * dl + 4.0 * da * da + db * db).sqrt()
}

#[inline]
pub fn brightness(r: u8, g: u8, b: u8) -> f32 {
    (0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32)) / 255.0
}



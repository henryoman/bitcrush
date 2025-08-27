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
pub fn brightness(r: u8, g: u8, b: u8) -> f32 {
    (0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32)) / 255.0
}

// CIEDE2000 color difference (perceptual). Returns ΔE00.
// Implementation adapted to f32 for performance; inputs are Lab in D65.
#[inline]
pub fn ciede2000(lab1: [f32; 3], lab2: [f32; 3]) -> f32 {
    let (l1, a1, b1) = (lab1[0], lab1[1], lab1[2]);
    let (l2, a2, b2) = (lab2[0], lab2[1], lab2[2]);

    let avg_lp = 0.5 * (l1 + l2);
    let c1 = (a1 * a1 + b1 * b1).sqrt();
    let c2 = (a2 * a2 + b2 * b2).sqrt();
    let avg_c = 0.5 * (c1 + c2);

    let g = 0.5 * (1.0 - (avg_c.powf(7.0) / (avg_c.powf(7.0) + 25.0_f32.powf(7.0))).sqrt());
    let a1_prime = (1.0 + g) * a1;
    let a2_prime = (1.0 + g) * a2;
    let c1_prime = (a1_prime * a1_prime + b1 * b1).sqrt();
    let c2_prime = (a2_prime * a2_prime + b2 * b2).sqrt();

    let h1_prime = b1.atan2(a1_prime).to_degrees().rem_euclid(360.0);
    let h2_prime = b2.atan2(a2_prime).to_degrees().rem_euclid(360.0);

    let delta_lp = l2 - l1;
    let delta_cp = c2_prime - c1_prime;

    let h_prime_diff = if c1_prime * c2_prime == 0.0 {
        0.0
    } else if (h2_prime - h1_prime).abs() <= 180.0 {
        h2_prime - h1_prime
    } else if h2_prime <= h1_prime {
        h2_prime - h1_prime + 360.0
    } else {
        h2_prime - h1_prime - 360.0
    };

    let delta_hp = if c1_prime * c2_prime == 0.0 {
        0.0
    } else {
        2.0 * (c1_prime * c2_prime).sqrt() * (0.5 * h_prime_diff.to_radians()).sin()
    };

    let avg_hp = if c1_prime * c2_prime == 0.0 {
        h1_prime + h2_prime
    } else if (h1_prime - h2_prime).abs() <= 180.0 {
        0.5 * (h1_prime + h2_prime)
    } else if (h1_prime + h2_prime) < 360.0 {
        0.5 * (h1_prime + h2_prime + 360.0)
    } else {
        0.5 * (h1_prime + h2_prime - 360.0)
    };

    let t = 1.0
        - 0.17 * (avg_hp - 30.0).to_radians().cos()
        + 0.24 * (2.0 * avg_hp).to_radians().cos()
        + 0.32 * (3.0 * avg_hp + 6.0).to_radians().cos()
        - 0.20 * (4.0 * avg_hp - 63.0).to_radians().cos();

    let delta_ro = 30.0 * (-(((avg_hp - 275.0) / 25.0).powi(2))).exp();
    let rc = 2.0 * (avg_c.powf(7.0) / (avg_c.powf(7.0) + 25.0_f32.powf(7.0))).sqrt();
    let sl = 1.0 + (0.015 * (avg_lp - 50.0).powi(2)) / (20.0 + (avg_lp - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * avg_c;
    let sh = 1.0 + 0.015 * avg_c * t;
    let rt = -rc * (2.0 * delta_ro.to_radians()).sin();

    let kl = 1.0; let kc = 1.0; let kh = 1.0;
    let de = ((delta_lp / (kl * sl)).powi(2)
        + (delta_cp / (kc * sc)).powi(2)
        + (delta_hp / (kh * sh)).powi(2)
        + rt * (delta_cp / (kc * sc)) * (delta_hp / (kh * sh)))
        .sqrt();
    de
}



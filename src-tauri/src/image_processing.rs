use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

// ── Adjustment Payload (mirrors frontend state) ──

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjustmentPayload {
    // Basic
    #[serde(default)]
    pub temperature: f64, // 2000-50000K, default 6500
    #[serde(default)]
    pub tint: f64, // -150 to 150
    #[serde(default)]
    pub exposure: f64, // -5.0 to 5.0 EV
    #[serde(default)]
    pub contrast: f64, // -100 to 100
    #[serde(default)]
    pub highlights: f64, // -100 to 100
    #[serde(default)]
    pub shadows: f64, // -100 to 100
    #[serde(default)]
    pub whites: f64, // -100 to 100
    #[serde(default)]
    pub blacks: f64, // -100 to 100
    #[serde(default)]
    pub texture: f64, // -100 to 100
    #[serde(default)]
    pub clarity: f64, // -100 to 100
    #[serde(default)]
    pub dehaze: f64, // -100 to 100
    #[serde(default)]
    pub vibrance: f64, // -100 to 100
    #[serde(default)]
    pub saturation: f64, // -100 to 100

    // Tone Curve
    #[serde(default)]
    pub tone_curve_rgb: Vec<[f64; 2]>,
    #[serde(default)]
    pub tone_curve_r: Vec<[f64; 2]>,
    #[serde(default)]
    pub tone_curve_g: Vec<[f64; 2]>,
    #[serde(default)]
    pub tone_curve_b: Vec<[f64; 2]>,

    // HSL (8 colors × 3 properties each — flat arrays)
    #[serde(default)]
    pub hsl_hue: Vec<f64>,    // 8 values, -180 to 180
    #[serde(default)]
    pub hsl_sat: Vec<f64>,    // 8 values, -100 to 100
    #[serde(default)]
    pub hsl_lum: Vec<f64>,    // 8 values, -100 to 100

    // Color Grading
    #[serde(default)]
    pub cg_shadows_hue: f64,
    #[serde(default)]
    pub cg_shadows_sat: f64,
    #[serde(default)]
    pub cg_shadows_lum: f64,
    #[serde(default)]
    pub cg_midtones_hue: f64,
    #[serde(default)]
    pub cg_midtones_sat: f64,
    #[serde(default)]
    pub cg_midtones_lum: f64,
    #[serde(default)]
    pub cg_highlights_hue: f64,
    #[serde(default)]
    pub cg_highlights_sat: f64,
    #[serde(default)]
    pub cg_highlights_lum: f64,
    #[serde(default)]
    pub cg_blending: f64, // 0-100
    #[serde(default)]
    pub cg_balance: f64,  // -100 to 100

    // Detail
    #[serde(default)]
    pub sharpen_amount: f64,  // 0-150
    #[serde(default)]
    pub sharpen_radius: f64,  // 0.5-3.0
    #[serde(default)]
    pub sharpen_detail: f64,  // 0-100
    #[serde(default)]
    pub sharpen_masking: f64, // 0-100
    #[serde(default)]
    pub nr_luminance: f64,    // 0-100
    #[serde(default)]
    pub nr_lum_detail: f64,   // 0-100
    #[serde(default)]
    pub nr_lum_contrast: f64, // 0-100
    #[serde(default)]
    pub nr_color: f64,        // 0-100
    #[serde(default)]
    pub nr_color_detail: f64, // 0-100
    #[serde(default)]
    pub nr_color_smooth: f64, // 0-100

    // Lens Corrections
    #[serde(default)]
    pub lens_distortion: f64,  // -100 to 100
    #[serde(default)]
    pub lens_vignetting: f64,  // -100 to 100
    #[serde(default)]
    pub lens_ca_red: f64,      // -100 to 100
    #[serde(default)]
    pub lens_ca_blue: f64,     // -100 to 100

    // Effects
    #[serde(default)]
    pub vignette_amount: f64,     // -100 to 100
    #[serde(default)]
    pub vignette_midpoint: f64,   // 0-100
    #[serde(default)]
    pub vignette_roundness: f64,  // -100 to 100
    #[serde(default)]
    pub vignette_feather: f64,    // 0-100
    #[serde(default)]
    pub vignette_highlights: f64, // 0-100
    #[serde(default)]
    pub grain_amount: f64,        // 0-100
    #[serde(default)]
    pub grain_size: f64,          // 1-50
    #[serde(default)]
    pub grain_roughness: f64,     // 0-100

    // Calibration
    #[serde(default)]
    pub cal_shadow_tint: f64,  // -100 to 100
    #[serde(default)]
    pub cal_red_hue: f64,      // -100 to 100
    #[serde(default)]
    pub cal_red_sat: f64,      // -100 to 100
    #[serde(default)]
    pub cal_green_hue: f64,    // -100 to 100
    #[serde(default)]
    pub cal_green_sat: f64,    // -100 to 100
    #[serde(default)]
    pub cal_blue_hue: f64,     // -100 to 100
    #[serde(default)]
    pub cal_blue_sat: f64,     // -100 to 100
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessResult {
    pub data: String,     // base64-encoded RGBA
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize)]
pub struct HistogramResult {
    pub r: Vec<u32>,
    pub g: Vec<u32>,
    pub b: Vec<u32>,
    pub l: Vec<u32>,
}

// ── sRGB ↔ Linear conversions ──

#[inline(always)]
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline(always)]
fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

// ── Soft clip (smooth knee) to prevent hard clipping ──

#[inline(always)]
fn soft_clip(x: f64, knee: f64) -> f64 {
    if x <= knee {
        x
    } else {
        let excess = x - knee;
        let range = 1.0 - knee;
        knee + range * (1.0 - (-excess / range).exp())
    }
}

#[inline(always)]
fn soft_clip_low(x: f64, knee: f64) -> f64 {
    if x >= knee {
        x
    } else {
        let deficit = knee - x;
        let range = knee;
        if range <= 0.0 { return 0.0; }
        knee - range * (1.0 - (-deficit / range).exp())
    }
}

#[inline(always)]
fn soft_clip_both(x: f64) -> f64 {
    let x = soft_clip_low(x, 0.08);
    soft_clip(x, 0.92)
}

#[inline(always)]
fn clamp01(x: f64) -> f64 {
    x.max(0.0).min(1.0)
}

// ── RGB ↔ HSL conversion ──

fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-10 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

    let h = if (max - r).abs() < 1e-10 {
        let mut hh = (g - b) / d;
        if g < b { hh += 6.0; }
        hh
    } else if (max - g).abs() < 1e-10 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    (h * 60.0, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    if s.abs() < 1e-10 {
        return (l, l, l);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let h_norm = h / 360.0;

    fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
        if t < 1.0 / 2.0 { return q; }
        if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
        p
    }

    (
        hue_to_rgb(p, q, h_norm + 1.0 / 3.0),
        hue_to_rgb(p, q, h_norm),
        hue_to_rgb(p, q, h_norm - 1.0 / 3.0),
    )
}

// ── Tone Curve LUT Builder ──

fn build_curve_lut(points: &[[f64; 2]]) -> [u8; 256] {
    let mut lut = [0u8; 256];

    if points.len() < 2 {
        // Identity curve
        for i in 0..256 {
            lut[i] = i as u8;
        }
        return lut;
    }

    // Sort points by x
    let mut sorted: Vec<[f64; 2]> = points.to_vec();
    sorted.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());

    // Catmull-Rom spline interpolation
    // Add virtual endpoints for boundary conditions
    let mut pts = vec![];
    if sorted[0][0] > 0.0 {
        pts.push([0.0, 0.0]);
    }
    pts.extend_from_slice(&sorted);
    if sorted.last().unwrap()[0] < 255.0 {
        pts.push([255.0, 255.0]);
    }

    for i in 0..256 {
        let x = i as f64;
        let y = catmull_rom_interp(&pts, x);
        lut[i] = clamp01(y / 255.0).mul_add(255.0, 0.5) as u8;
    }

    lut
}

fn catmull_rom_interp(points: &[[f64; 2]], x: f64) -> f64 {
    let n = points.len();
    if n < 2 { return x; }

    // Find the segment
    let mut seg = 0;
    for i in 0..n - 1 {
        if x <= points[i + 1][0] {
            seg = i;
            break;
        }
        seg = i;
    }

    let p0 = if seg > 0 { points[seg - 1] } else { points[seg] };
    let p1 = points[seg];
    let p2 = points[(seg + 1).min(n - 1)];
    let p3 = points[(seg + 2).min(n - 1)];

    let range = p2[0] - p1[0];
    if range.abs() < 1e-10 { return p1[1]; }
    let t = ((x - p1[0]) / range).max(0.0).min(1.0);
    let t2 = t * t;
    let t3 = t2 * t;

    0.5 * ((2.0 * p1[1])
        + (-p0[1] + p2[1]) * t
        + (2.0 * p0[1] - 5.0 * p1[1] + 4.0 * p2[1] - p3[1]) * t2
        + (-p0[1] + 3.0 * p1[1] - 3.0 * p2[1] + p3[1]) * t3)
}

// ── HSL Color Range Weights ──
// 8 ranges: Red(0), Orange(1), Yellow(2), Green(3), Aqua(4), Blue(5), Purple(6), Magenta(7)
// Center hues: 0, 30, 60, 120, 180, 240, 285, 330

const HSL_CENTERS: [f64; 8] = [0.0, 30.0, 60.0, 120.0, 180.0, 240.0, 285.0, 330.0];

fn hsl_range_weight(hue: f64, center: f64) -> f64 {
    let mut diff = (hue - center).abs();
    if diff > 180.0 { diff = 360.0 - diff; }
    let width = 30.0; // smooth overlap width
    if diff > width { 0.0 } else {
        let t = diff / width;
        // Smooth cosine falloff
        0.5 * (1.0 + (t * PI).cos())
    }
}

// ── White Balance (temperature/tint to RGB multipliers) ──

fn temperature_to_rgb_mults(temp_k: f64) -> (f64, f64, f64) {
    // Simplified Planckian locus approximation
    // Reference: 6500K = neutral
    let t = temp_k.max(2000.0).min(50000.0);
    let t_norm = (t - 6500.0) / 6500.0;

    // Warm → boost red, reduce blue; Cool → boost blue, reduce red
    let r_mult = 1.0 + t_norm * 0.15;
    let b_mult = 1.0 - t_norm * 0.15;
    (r_mult, 1.0, b_mult)
}

// ── Simple pseudo-random for grain ──

#[inline(always)]
fn hash_pixel(x: u32, y: u32, seed: u32) -> f64 {
    let mut h = x.wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263))
        .wrapping_add(seed.wrapping_mul(2654435761));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h = h ^ (h >> 16);
    (h & 0xFFFF) as f64 / 65535.0
}

// ── Main Processing Pipeline ──

fn process_pixels(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    adj: &AdjustmentPayload,
) {
    let w = width as usize;
    let h = height as usize;

    // Pre-compute LUTs
    let lut_rgb = build_curve_lut(&adj.tone_curve_rgb);
    let lut_r = build_curve_lut(&adj.tone_curve_r);
    let lut_g = build_curve_lut(&adj.tone_curve_g);
    let lut_b = build_curve_lut(&adj.tone_curve_b);

    // Pre-compute factors
    let exposure_mult = (2.0_f64).powf(adj.exposure);
    let contrast_amt = adj.contrast / 100.0;
    let highlights_amt = adj.highlights / 100.0;
    let shadows_amt = adj.shadows / 100.0;
    let whites_amt = adj.whites / 100.0;
    let blacks_amt = adj.blacks / 100.0;
    let vibrance_amt = adj.vibrance / 100.0;
    let saturation_mult = 1.0 + adj.saturation / 100.0;

    // White balance
    let temp_k = if adj.temperature == 0.0 { 6500.0 } else { adj.temperature };
    let (wb_r, wb_g, wb_b) = temperature_to_rgb_mults(temp_k);
    let tint_shift = adj.tint / 150.0 * 0.05; // subtle green-magenta

    // Calibration factors
    let cal_shadow_tint = adj.cal_shadow_tint / 100.0;
    let cal_r_hue = adj.cal_red_hue / 100.0 * 10.0;
    let cal_r_sat = 1.0 + adj.cal_red_sat / 100.0 * 0.3;
    let cal_g_hue = adj.cal_green_hue / 100.0 * 10.0;
    let cal_g_sat = 1.0 + adj.cal_green_sat / 100.0 * 0.3;
    let cal_b_hue = adj.cal_blue_hue / 100.0 * 10.0;
    let cal_b_sat = 1.0 + adj.cal_blue_sat / 100.0 * 0.3;

    // Color grading
    let cg_blend = (adj.cg_blending.max(0.0).min(100.0) / 100.0 * 0.5 + 0.25).min(0.75);
    let cg_balance = adj.cg_balance / 100.0;

    // Pre-build a combined basic LUT for speed
    // (exposure + contrast + highlights/shadows/whites/blacks + tone curve)
    let mut basic_lut_r = [0u8; 256];
    let mut basic_lut_g = [0u8; 256];
    let mut basic_lut_b = [0u8; 256];

    for i in 0..256 {
        let c = i as f64 / 255.0;
        let lin = srgb_to_linear(c);

        // Exposure
        let exposed = lin * exposure_mult;

        // Contrast (S-curve around 0.18 gray in linear)
        let contrasted = if contrast_amt.abs() > 0.001 {
            let pivot = 0.18;
            let ratio = if pivot > 0.0 { exposed / pivot } else { exposed };
            let sign = if contrast_amt > 0.0 { 1.0 } else { -1.0 };
            let strength = contrast_amt.abs();
            let curved = ratio.powf(1.0 + sign * strength * 1.5);
            curved * pivot
        } else {
            exposed
        };

        // Highlights / Shadows / Whites / Blacks with soft knee
        let lum = contrasted;
        let mut adjusted = contrasted;

        // Highlights: affect upper range
        if highlights_amt.abs() > 0.001 {
            let mask = smoothstep(0.3, 0.9, lum);
            adjusted += highlights_amt * mask * 0.3 * lum;
        }

        // Shadows: affect lower range
        if shadows_amt.abs() > 0.001 {
            let mask = 1.0 - smoothstep(0.1, 0.6, lum);
            adjusted += shadows_amt * mask * 0.3 * (1.0 - lum);
        }

        // Whites: set white point
        if whites_amt.abs() > 0.001 {
            let mask = smoothstep(0.7, 1.0, lum);
            adjusted += whites_amt * mask * 0.15;
        }

        // Blacks: set black point
        if blacks_amt.abs() > 0.001 {
            let mask = 1.0 - smoothstep(0.0, 0.3, lum);
            adjusted += blacks_amt * mask * 0.1;
        }

        // Soft clip in linear space
        let soft = soft_clip_both(clamp01(adjusted));

        // Back to sRGB for tone curve LUT application
        let srgb_val = linear_to_srgb(soft);
        let idx = (clamp01(srgb_val) * 255.0 + 0.5) as usize;
        let idx = idx.min(255);

        // Apply tone curve LUTs
        let cr = lut_rgb[lut_r[idx] as usize];
        let cg = lut_rgb[lut_g[idx] as usize];
        let cb = lut_rgb[lut_b[idx] as usize];

        basic_lut_r[i] = cr;
        basic_lut_g[i] = cg;
        basic_lut_b[i] = cb;
    }

    // Process pixels in parallel using rayon
    let chunk_size = 4; // RGBA
    pixels.par_chunks_mut(chunk_size).enumerate().for_each(|(pixel_idx, chunk)| {
        if chunk.len() < 4 { return; }

        let px = pixel_idx % w;
        let py = pixel_idx / w;

        let mut r = chunk[0] as f64 / 255.0;
        let mut g = chunk[1] as f64 / 255.0;
        let mut b = chunk[2] as f64 / 255.0;
        // Alpha stays unchanged

        // 1. sRGB → Linear
        r = srgb_to_linear(r);
        g = srgb_to_linear(g);
        b = srgb_to_linear(b);

        // 2. White Balance
        r *= wb_r;
        g *= wb_g;
        b *= wb_b;
        // Tint (green-magenta)
        g += tint_shift;
        r -= tint_shift * 0.5;
        b -= tint_shift * 0.5;

        // 3. Calibration (shadow tint + primary adjustments)
        let lum_cal = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        if cal_shadow_tint.abs() > 0.001 {
            let shadow_mask = 1.0 - smoothstep(0.0, 0.4, lum_cal);
            g += cal_shadow_tint * shadow_mask * 0.05;
        }

        // 4. Convert to sRGB for LUT application
        r = linear_to_srgb(clamp01(r));
        g = linear_to_srgb(clamp01(g));
        b = linear_to_srgb(clamp01(b));

        // 5. Apply basic LUT (exposure + contrast + HL/SH/WH/BK + tone curve)
        let ri = (r * 255.0 + 0.5) as usize;
        let gi = (g * 255.0 + 0.5) as usize;
        let bi = (b * 255.0 + 0.5) as usize;
        r = basic_lut_r[ri.min(255)] as f64 / 255.0;
        g = basic_lut_g[gi.min(255)] as f64 / 255.0;
        b = basic_lut_b[bi.min(255)] as f64 / 255.0;

        // 6. HSL adjustments (back to linear for accuracy, then HSL)
        let has_hsl = !adj.hsl_hue.is_empty() && adj.hsl_hue.len() == 8;
        if has_hsl {
            let (mut hue, mut sat, mut lum) = rgb_to_hsl(r, g, b);
            let mut hue_shift = 0.0_f64;
            let mut sat_shift = 0.0_f64;
            let mut lum_shift = 0.0_f64;

            for ci in 0..8 {
                let w = hsl_range_weight(hue, HSL_CENTERS[ci]);
                if w > 0.001 {
                    hue_shift += adj.hsl_hue[ci] * w;
                    sat_shift += adj.hsl_sat[ci] / 100.0 * w;
                    lum_shift += adj.hsl_lum[ci] / 100.0 * w;
                }
            }

            hue = (hue + hue_shift) % 360.0;
            if hue < 0.0 { hue += 360.0; }
            sat = (sat + sat_shift * sat).max(0.0).min(1.0);
            lum = (lum + lum_shift * 0.3).max(0.0).min(1.0);

            let (nr, ng, nb) = hsl_to_rgb(hue, sat, lum);
            r = nr; g = ng; b = nb;
        }

        // 7. Calibration: primary adjustments
        if cal_r_hue.abs() > 0.001 || (cal_r_sat - 1.0).abs() > 0.001
            || cal_g_hue.abs() > 0.001 || (cal_g_sat - 1.0).abs() > 0.001
            || cal_b_hue.abs() > 0.001 || (cal_b_sat - 1.0).abs() > 0.001
        {
            let (mut hue, mut sat, lum) = rgb_to_hsl(r, g, b);
            // Red
            let rw = hsl_range_weight(hue, 0.0);
            if rw > 0.001 { hue += cal_r_hue * rw; sat *= 1.0 + (cal_r_sat - 1.0) * rw; }
            // Green
            let gw = hsl_range_weight(hue, 120.0);
            if gw > 0.001 { hue += cal_g_hue * gw; sat *= 1.0 + (cal_g_sat - 1.0) * gw; }
            // Blue
            let bw = hsl_range_weight(hue, 240.0);
            if bw > 0.001 { hue += cal_b_hue * bw; sat *= 1.0 + (cal_b_sat - 1.0) * bw; }

            hue = hue % 360.0;
            if hue < 0.0 { hue += 360.0; }
            sat = sat.max(0.0).min(1.0);

            let (nr, ng, nb) = hsl_to_rgb(hue, sat, lum);
            r = nr; g = ng; b = nb;
        }

        // 8. Color Grading
        let has_cg = adj.cg_shadows_sat.abs() > 0.001
            || adj.cg_midtones_sat.abs() > 0.001
            || adj.cg_highlights_sat.abs() > 0.001;
        if has_cg {
            let lum_cg = 0.299 * r + 0.587 * g + 0.114 * b;
            let balance_shift = cg_balance * 0.15;

            // Shadow/midtone/highlight masks with blend
            let shadow_mask = 1.0 - smoothstep(0.0 + balance_shift, cg_blend + balance_shift, lum_cg);
            let highlight_mask = smoothstep(1.0 - cg_blend + balance_shift, 1.0 + balance_shift, lum_cg);
            let midtone_mask = (1.0 - shadow_mask - highlight_mask).max(0.0);

            // Apply each wheel
            fn apply_wheel(r: &mut f64, g: &mut f64, b: &mut f64, hue_deg: f64, sat: f64, lum_adj: f64, mask: f64) {
                if sat.abs() < 0.001 && lum_adj.abs() < 0.001 { return; }
                let hue_rad = hue_deg * PI / 180.0;
                let strength = sat / 100.0 * mask * 0.15;
                // Project hue+sat to RGB offset
                let cr = hue_rad.cos() * strength;
                let cg_v = (hue_rad - 2.0 * PI / 3.0).cos() * strength;
                let cb = (hue_rad + 2.0 * PI / 3.0).cos() * strength;
                *r += cr;
                *g += cg_v;
                *b += cb;
                // Luminance adjustment
                let lum_delta = lum_adj / 100.0 * mask * 0.1;
                *r += lum_delta;
                *g += lum_delta;
                *b += lum_delta;
            }

            apply_wheel(&mut r, &mut g, &mut b, adj.cg_shadows_hue, adj.cg_shadows_sat, adj.cg_shadows_lum, shadow_mask);
            apply_wheel(&mut r, &mut g, &mut b, adj.cg_midtones_hue, adj.cg_midtones_sat, adj.cg_midtones_lum, midtone_mask);
            apply_wheel(&mut r, &mut g, &mut b, adj.cg_highlights_hue, adj.cg_highlights_sat, adj.cg_highlights_lum, highlight_mask);
        }

        // 9. Vibrance + Saturation
        if vibrance_amt.abs() > 0.001 || (saturation_mult - 1.0).abs() > 0.001 {
            let gray = 0.299 * r + 0.587 * g + 0.114 * b;

            // Vibrance: boost less saturated colors more
            if vibrance_amt.abs() > 0.001 {
                let max_c = r.max(g).max(b);
                let min_c = r.min(g).min(b);
                let cur_sat = if max_c > 0.0 { (max_c - min_c) / max_c } else { 0.0 };
                let boost = 1.0 + (1.0 - cur_sat) * vibrance_amt * 0.6;
                r = gray + boost * (r - gray);
                g = gray + boost * (g - gray);
                b = gray + boost * (b - gray);
            }

            // Global saturation
            if (saturation_mult - 1.0).abs() > 0.001 {
                let gray2 = 0.299 * r + 0.587 * g + 0.114 * b;
                r = gray2 + saturation_mult * (r - gray2);
                g = gray2 + saturation_mult * (g - gray2);
                b = gray2 + saturation_mult * (b - gray2);
            }
        }

        // 10. Final soft clip + clamp
        r = clamp01(soft_clip_both(r));
        g = clamp01(soft_clip_both(g));
        b = clamp01(soft_clip_both(b));

        // 11. Grain (luminance-weighted, seeded)
        if adj.grain_amount > 0.001 {
            let lum_grain = 0.299 * r + 0.587 * g + 0.114 * b;
            // More grain in midtones, less in shadows / highlights
            let mid_weight = 1.0 - (2.0 * lum_grain - 1.0).abs();
            let grain_strength = adj.grain_amount / 100.0 * 0.15 * (0.3 + 0.7 * mid_weight);

            // Seeded random based on position
            let noise = hash_pixel(px as u32, py as u32, 42) * 2.0 - 1.0;
            let grain_val = noise * grain_strength;
            r += grain_val;
            g += grain_val;
            b += grain_val;
        }

        // Clamp final
        chunk[0] = (clamp01(r) * 255.0 + 0.5) as u8;
        chunk[1] = (clamp01(g) * 255.0 + 0.5) as u8;
        chunk[2] = (clamp01(b) * 255.0 + 0.5) as u8;
    });

    // ── Post-pixel pass: Vignette ──
    if adj.vignette_amount.abs() > 0.001 {
        apply_vignette(pixels, width, height, adj);
    }

    // ── Lens vignetting correction ──
    if adj.lens_vignetting.abs() > 0.001 {
        let strength = adj.lens_vignetting / 100.0 * 0.4;
        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let max_r = (cx * cx + cy * cy).sqrt();

        pixels.par_chunks_mut(4).enumerate().for_each(|(i, chunk)| {
            if chunk.len() < 4 { return; }
            let px = (i % w) as f64;
            let py = (i / w) as f64;
            let dx = px - cx;
            let dy = py - cy;
            let dist = (dx * dx + dy * dy).sqrt() / max_r;
            let factor = 1.0 + strength * dist * dist;
            chunk[0] = (clamp01(chunk[0] as f64 / 255.0 * factor) * 255.0 + 0.5) as u8;
            chunk[1] = (clamp01(chunk[1] as f64 / 255.0 * factor) * 255.0 + 0.5) as u8;
            chunk[2] = (clamp01(chunk[2] as f64 / 255.0 * factor) * 255.0 + 0.5) as u8;
        });
    }
}

fn apply_vignette(pixels: &mut [u8], width: u32, height: u32, adj: &AdjustmentPayload) {
    let w = width as f64;
    let h = height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let amount = adj.vignette_amount / 100.0; // negative = darken, positive = lighten
    let midpoint = (adj.vignette_midpoint.max(1.0).min(100.0)) / 100.0;
    let roundness = adj.vignette_roundness / 100.0;
    let feather = (adj.vignette_feather.max(1.0).min(100.0)) / 100.0;
    let hl_protect = adj.vignette_highlights / 100.0;

    let wu = width as usize;

    pixels.par_chunks_mut(4).enumerate().for_each(|(i, chunk)| {
        if chunk.len() < 4 { return; }
        let px = (i % wu) as f64;
        let py = (i / wu) as f64;

        let dx = (px - cx) / cx;
        let dy = (py - cy) / cy;

        // Roundness: interpolate between circle and rectangle
        let round_factor = 1.0 + roundness * 0.5;
        let dist = (dx * dx * round_factor + dy * dy / round_factor).sqrt();

        let inner = midpoint * 0.8;
        let outer = inner + (1.0 - inner) * feather;
        let mask = smoothstep(inner, outer, dist);

        let vig = -amount * mask * 0.7;

        // Highlight protection
        let lum = (chunk[0] as f64 * 0.299 + chunk[1] as f64 * 0.587 + chunk[2] as f64 * 0.114) / 255.0;
        let hl_mask = 1.0 - hl_protect * smoothstep(0.6, 1.0, lum);
        let final_vig = vig * hl_mask;

        let factor = 1.0 + final_vig;
        chunk[0] = (clamp01(chunk[0] as f64 / 255.0 * factor) * 255.0 + 0.5) as u8;
        chunk[1] = (clamp01(chunk[1] as f64 / 255.0 * factor) * 255.0 + 0.5) as u8;
        chunk[2] = (clamp01(chunk[2] as f64 / 255.0 * factor) * 255.0 + 0.5) as u8;
    });
}

#[inline(always)]
fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).max(0.0).min(1.0);
    t * t * (3.0 - 2.0 * t)
}

// ── Tauri Commands ──

#[tauri::command]
pub async fn process_image(
    image_path: String,
    adjustments: AdjustmentPayload,
    preview: bool,
) -> Result<ProcessResult, String> {
    // Load image
    let img = image::open(&image_path).map_err(|e| format!("Failed to open image: {}", e))?;

    // Optionally resize for preview
    let img = if preview {
        let (w, h) = (img.width(), img.height());
        let max_edge = 800u32;
        if w > max_edge || h > max_edge {
            let scale = max_edge as f64 / w.max(h) as f64;
            let nw = (w as f64 * scale) as u32;
            let nh = (h as f64 * scale) as u32;
            img.resize(nw, nh, image::imageops::FilterType::Triangle)
        } else {
            img
        }
    } else {
        img
    };

    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let mut pixels = rgba.into_raw();

    // Process in a blocking thread
    let adj = adjustments;
    tokio::task::spawn_blocking(move || {
        process_pixels(&mut pixels, width, height, &adj);

        // Encode as base64
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&pixels);

        Ok(ProcessResult {
            data: encoded,
            width,
            height,
        })
    })
    .await
    .map_err(|e| format!("Processing error: {}", e))?
}

#[tauri::command]
pub async fn compute_histogram(
    image_data: Vec<u8>,
    width: u32,
    _height: u32,
) -> Result<HistogramResult, String> {
    tokio::task::spawn_blocking(move || {
        let mut r_hist = vec![0u32; 256];
        let mut g_hist = vec![0u32; 256];
        let mut b_hist = vec![0u32; 256];
        let mut l_hist = vec![0u32; 256];

        for chunk in image_data.chunks(4) {
            if chunk.len() < 4 { continue; }
            let ri = chunk[0] as usize;
            let gi = chunk[1] as usize;
            let bi = chunk[2] as usize;
            r_hist[ri] += 1;
            g_hist[gi] += 1;
            b_hist[bi] += 1;
            // Luminance
            let l = (0.299 * chunk[0] as f64 + 0.587 * chunk[1] as f64 + 0.114 * chunk[2] as f64) as usize;
            l_hist[l.min(255)] += 1;
        }

        Ok(HistogramResult {
            r: r_hist,
            g: g_hist,
            b: b_hist,
            l: l_hist,
        })
    })
    .await
    .map_err(|e| format!("Histogram error: {}", e))?
}

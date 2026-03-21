use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::sync::Mutex;

// ── Editor State (source image cache) ──

pub struct EditorState {
    pub cached_source: Mutex<Option<CachedImage>>,
    pub preview_path: Mutex<String>,
}

pub struct CachedImage {
    pub path: String,
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Default for EditorState {
    fn default() -> Self {
        let preview_path = std::env::temp_dir()
            .join("galleria_preview.bmp")
            .to_string_lossy()
            .to_string();
        Self {
            cached_source: Mutex::new(None),
            preview_path: Mutex::new(preview_path),
        }
    }
}

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
    pub preview_path: String,  // temp file path for frontend to load
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
    // CIE D-illuminant based white balance
    // Reference: D65 (6500K) = neutral
    let t = temp_k.max(2000.0).min(25000.0);
    let t2 = t * t;
    let t3 = t2 * t;

    // CIE daylight illuminant xy chromaticity
    let x = if t <= 7000.0 {
        -4.607e9 / t3 + 2.9678e6 / t2 + 0.09911e3 / t + 0.244063
    } else {
        -2.0064e9 / t3 + 1.9018e6 / t2 + 0.24748e3 / t + 0.237040
    };
    let y = -3.0 * x * x + 2.87 * x - 0.275;

    // D65 reference chromaticity
    let ref_x = 0.31271;
    let ref_y = 0.32902;

    // Simplified Bradford-like adaptation
    let dx = x - ref_x;
    let dy = y - ref_y;
    let r_mult = (1.0 + dx * 4.5 - dy * 1.5).max(0.4).min(2.5);
    let b_mult = (1.0 - dx * 1.5 + dy * 4.5).max(0.4).min(2.5);
    (r_mult, 1.0, b_mult)
}

// ── Separable Gaussian Blur (foundation for spatial ops) ──

fn build_gaussian_kernel(sigma: f64) -> Vec<f64> {
    let radius = (sigma * 2.5).ceil() as usize;
    let size = radius * 2 + 1;
    let mut kernel = vec![0.0f64; size];
    let s2 = 2.0 * sigma * sigma;
    let mut sum = 0.0;
    for i in 0..size {
        let x = i as f64 - radius as f64;
        kernel[i] = (-x * x / s2).exp();
        sum += kernel[i];
    }
    for v in kernel.iter_mut() { *v /= sum; }
    kernel
}

/// Gaussian blur on a single-channel f64 buffer (w×h)
fn gaussian_blur_channel(src: &[f64], w: usize, h: usize, sigma: f64) -> Vec<f64> {
    if sigma < 0.3 { return src.to_vec(); }
    let kernel = build_gaussian_kernel(sigma);
    let radius = kernel.len() / 2;

    // Horizontal pass
    let mut temp = vec![0.0f64; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0;
            for k in 0..kernel.len() {
                let sx = (x as isize + k as isize - radius as isize)
                    .max(0).min(w as isize - 1) as usize;
                sum += src[y * w + sx] * kernel[k];
            }
            temp[y * w + x] = sum;
        }
    }

    // Vertical pass
    let mut out = vec![0.0f64; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0;
            for k in 0..kernel.len() {
                let sy = (y as isize + k as isize - radius as isize)
                    .max(0).min(h as isize - 1) as usize;
                sum += temp[sy * w + x] * kernel[k];
            }
            out[y * w + x] = sum;
        }
    }
    out
}

/// Apply clarity (local contrast at large radius) to RGBA pixel buffer
fn apply_clarity(pixels: &mut [u8], w: usize, h: usize, strength: f64) {
    if strength.abs() < 0.001 { return; }
    let amt = strength / 100.0 * 0.6;

    // Extract luminance channel
    let mut lum = vec![0.0f64; w * h];
    for i in 0..(w * h) {
        let r = pixels[i * 4] as f64 / 255.0;
        let g = pixels[i * 4 + 1] as f64 / 255.0;
        let b = pixels[i * 4 + 2] as f64 / 255.0;
        lum[i] = 0.299 * r + 0.587 * g + 0.114 * b;
    }

    // Blur at large radius for local contrast
    let sigma = (w.min(h) as f64 * 0.02).max(8.0).min(30.0);
    let blurred = gaussian_blur_channel(&lum, w, h, sigma);

    // Apply: boost difference between original and blurred luminance
    for i in 0..(w * h) {
        let diff = lum[i] - blurred[i];
        let boost = 1.0 + amt * diff.signum() * diff.abs().sqrt() * 2.0;
        for c in 0..3 {
            let v = pixels[i * 4 + c] as f64 / 255.0;
            let nv = (v * boost).max(0.0).min(1.0);
            pixels[i * 4 + c] = (nv * 255.0 + 0.5) as u8;
        }
    }
}

/// Apply texture (local contrast at small radius, luminance-preserving)
fn apply_texture(pixels: &mut [u8], w: usize, h: usize, strength: f64) {
    if strength.abs() < 0.001 { return; }
    let amt = strength / 100.0 * 0.4;

    let mut lum = vec![0.0f64; w * h];
    for i in 0..(w * h) {
        let r = pixels[i * 4] as f64 / 255.0;
        let g = pixels[i * 4 + 1] as f64 / 255.0;
        let b = pixels[i * 4 + 2] as f64 / 255.0;
        lum[i] = 0.299 * r + 0.587 * g + 0.114 * b;
    }

    let blurred = gaussian_blur_channel(&lum, w, h, 2.0);

    for i in 0..(w * h) {
        let detail = lum[i] - blurred[i];
        let boost = amt * detail * 3.0;
        for c in 0..3 {
            let v = pixels[i * 4 + c] as f64 / 255.0;
            pixels[i * 4 + c] = ((v + boost).max(0.0).min(1.0) * 255.0 + 0.5) as u8;
        }
    }
}

/// Apply dehaze using simplified dark channel prior
fn apply_dehaze(pixels: &mut [u8], w: usize, h: usize, strength: f64) {
    if strength.abs() < 0.001 { return; }
    let amt = strength / 100.0;

    // Compute per-pixel dark channel (min of RGB)
    let mut dark = vec![0.0f64; w * h];
    for i in 0..(w * h) {
        let r = pixels[i * 4] as f64 / 255.0;
        let g = pixels[i * 4 + 1] as f64 / 255.0;
        let b = pixels[i * 4 + 2] as f64 / 255.0;
        dark[i] = r.min(g).min(b);
    }

    // Blur the dark channel to get local atmospheric estimate
    let sigma = (w.min(h) as f64 * 0.01).max(5.0).min(15.0);
    let atm = gaussian_blur_channel(&dark, w, h, sigma);

    // Estimate atmospheric light (bright region average)
    let atm_light = 0.95_f64; // simplified: assume near-white atmosphere

    for i in 0..(w * h) {
        let transmission = (1.0 - amt * atm[i] / atm_light).max(0.1);
        for c in 0..3 {
            let v = pixels[i * 4 + c] as f64 / 255.0;
            let dehazed = (v - atm_light * (1.0 - transmission)) / transmission;
            pixels[i * 4 + c] = (dehazed.max(0.0).min(1.0) * 255.0 + 0.5) as u8;
        }
    }
}

/// Unsharp mask sharpening
fn apply_sharpening(pixels: &mut [u8], w: usize, h: usize, amount: f64, radius: f64, _detail: f64, masking: f64) {
    if amount < 0.001 { return; }
    let amt = amount / 150.0;

    let mut lum = vec![0.0f64; w * h];
    for i in 0..(w * h) {
        let r = pixels[i * 4] as f64 / 255.0;
        let g = pixels[i * 4 + 1] as f64 / 255.0;
        let b = pixels[i * 4 + 2] as f64 / 255.0;
        lum[i] = 0.299 * r + 0.587 * g + 0.114 * b;
    }

    let blurred = gaussian_blur_channel(&lum, w, h, radius.max(0.5).min(3.0));

    // Edge masking: only sharpen edges (high local contrast areas)
    let mask_threshold = masking / 100.0 * 0.1;

    for i in 0..(w * h) {
        let detail_val = lum[i] - blurred[i];
        // Masking: suppress sharpening in smooth areas
        let edge_strength = detail_val.abs();
        if edge_strength < mask_threshold { continue; }

        let sharpen = amt * detail_val * 2.0;
        for c in 0..3 {
            let v = pixels[i * 4 + c] as f64 / 255.0;
            pixels[i * 4 + c] = ((v + sharpen).max(0.0).min(1.0) * 255.0 + 0.5) as u8;
        }
    }
}

/// Simplified noise reduction via edge-preserving blur
fn apply_noise_reduction(pixels: &mut [u8], w: usize, h: usize, lum_strength: f64, color_strength: f64) {
    // Luminance NR: blur luminance channel, blend back
    if lum_strength > 0.5 {
        let amt = lum_strength / 100.0 * 0.7;
        let mut lum = vec![0.0f64; w * h];
        for i in 0..(w * h) {
            let r = pixels[i * 4] as f64 / 255.0;
            let g = pixels[i * 4 + 1] as f64 / 255.0;
            let b = pixels[i * 4 + 2] as f64 / 255.0;
            lum[i] = 0.299 * r + 0.587 * g + 0.114 * b;
        }
        let blurred = gaussian_blur_channel(&lum, w, h, 1.5 + lum_strength / 30.0);

        for i in 0..(w * h) {
            let diff = (lum[i] - blurred[i]).abs();
            // Only smooth areas with small differences (noise), preserve edges
            let blend = if diff < 0.05 { amt } else { amt * (0.05 / diff).min(1.0) };
            let new_lum = lum[i] * (1.0 - blend) + blurred[i] * blend;
            let ratio = if lum[i] > 0.001 { new_lum / lum[i] } else { 1.0 };
            for c in 0..3 {
                let v = pixels[i * 4 + c] as f64 / 255.0;
                pixels[i * 4 + c] = ((v * ratio).max(0.0).min(1.0) * 255.0 + 0.5) as u8;
            }
        }
    }

    // Color NR: blur chrominance
    if color_strength > 0.5 {
        let amt = color_strength / 100.0 * 0.8;
        let sigma = 2.0 + color_strength / 25.0;
        // Extract Cb/Cr-like channels
        let mut cb = vec![0.0f64; w * h];
        let mut cr = vec![0.0f64; w * h];
        for i in 0..(w * h) {
            let r = pixels[i * 4] as f64 / 255.0;
            let g = pixels[i * 4 + 1] as f64 / 255.0;
            let b = pixels[i * 4 + 2] as f64 / 255.0;
            let y = 0.299 * r + 0.587 * g + 0.114 * b;
            cb[i] = b - y;
            cr[i] = r - y;
        }
        let cb_blur = gaussian_blur_channel(&cb, w, h, sigma);
        let cr_blur = gaussian_blur_channel(&cr, w, h, sigma);

        for i in 0..(w * h) {
            let r = pixels[i * 4] as f64 / 255.0;
            let g = pixels[i * 4 + 1] as f64 / 255.0;
            let b = pixels[i * 4 + 2] as f64 / 255.0;
            let y = 0.299 * r + 0.587 * g + 0.114 * b;
            let new_cb = cb[i] * (1.0 - amt) + cb_blur[i] * amt;
            let new_cr = cr[i] * (1.0 - amt) + cr_blur[i] * amt;
            pixels[i * 4]     = ((y + new_cr).max(0.0).min(1.0) * 255.0 + 0.5) as u8;
            pixels[i * 4 + 1] = ((y - 0.344136 * new_cb - 0.714136 * new_cr).max(0.0).min(1.0) * 255.0 + 0.5) as u8;
            pixels[i * 4 + 2] = ((y + new_cb).max(0.0).min(1.0) * 255.0 + 0.5) as u8;
        }
    }
}

/// Barrel/pincushion lens distortion correction
fn apply_lens_distortion(pixels: &mut [u8], w: usize, h: usize, strength: f64) {
    if strength.abs() < 0.001 { return; }
    let k = strength / 100.0 * 0.3; // distortion coefficient
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let max_r = (cx * cx + cy * cy).sqrt();

    let src = pixels.to_vec();
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f64 - cx) / max_r;
            let dy = (y as f64 - cy) / max_r;
            let r2 = dx * dx + dy * dy;
            let factor = 1.0 + k * r2;
            let sx = cx + dx * factor * max_r;
            let sy = cy + dy * factor * max_r;

            let idx = (y * w + x) * 4;
            if sx >= 0.0 && sx < (w - 1) as f64 && sy >= 0.0 && sy < (h - 1) as f64 {
                // Bilinear interpolation
                let x0 = sx.floor() as usize;
                let y0 = sy.floor() as usize;
                let fx = sx - x0 as f64;
                let fy = sy - y0 as f64;
                for c in 0..3 {
                    let v00 = src[(y0 * w + x0) * 4 + c] as f64;
                    let v10 = src[(y0 * w + x0 + 1) * 4 + c] as f64;
                    let v01 = src[((y0 + 1) * w + x0) * 4 + c] as f64;
                    let v11 = src[((y0 + 1) * w + x0 + 1) * 4 + c] as f64;
                    let v = v00 * (1.0 - fx) * (1.0 - fy) + v10 * fx * (1.0 - fy)
                          + v01 * (1.0 - fx) * fy + v11 * fx * fy;
                    pixels[idx + c] = v.max(0.0).min(255.0) as u8;
                }
            } else {
                pixels[idx] = 0;
                pixels[idx + 1] = 0;
                pixels[idx + 2] = 0;
            }
        }
    }
}

/// Chromatic aberration correction (per-channel radial scaling)
fn apply_ca_correction(pixels: &mut [u8], w: usize, h: usize, ca_red: f64, ca_blue: f64) {
    if ca_red.abs() < 0.001 && ca_blue.abs() < 0.001 { return; }
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let r_scale = 1.0 + ca_red / 100.0 * 0.005;
    let b_scale = 1.0 + ca_blue / 100.0 * 0.005;

    let src = pixels.to_vec();
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            // Red channel: sample from scaled position
            let rx = cx + (x as f64 - cx) * r_scale;
            let ry = cy + (y as f64 - cy) * r_scale;
            if rx >= 0.0 && rx < (w - 1) as f64 && ry >= 0.0 && ry < (h - 1) as f64 {
                let x0 = rx.floor() as usize; let y0 = ry.floor() as usize;
                let fx = rx - x0 as f64; let fy = ry - y0 as f64;
                let v = src[(y0 * w + x0) * 4] as f64 * (1.0 - fx) * (1.0 - fy)
                      + src[(y0 * w + x0 + 1) * 4] as f64 * fx * (1.0 - fy)
                      + src[((y0 + 1) * w + x0) * 4] as f64 * (1.0 - fx) * fy
                      + src[((y0 + 1) * w + x0 + 1) * 4] as f64 * fx * fy;
                pixels[idx] = v.max(0.0).min(255.0) as u8;
            }
            // Blue channel
            let bx = cx + (x as f64 - cx) * b_scale;
            let by = cy + (y as f64 - cy) * b_scale;
            if bx >= 0.0 && bx < (w - 1) as f64 && by >= 0.0 && by < (h - 1) as f64 {
                let x0 = bx.floor() as usize; let y0 = by.floor() as usize;
                let fx = bx - x0 as f64; let fy = by - y0 as f64;
                let v = src[(y0 * w + x0) * 4 + 2] as f64 * (1.0 - fx) * (1.0 - fy)
                      + src[(y0 * w + x0 + 1) * 4 + 2] as f64 * fx * (1.0 - fy)
                      + src[((y0 + 1) * w + x0) * 4 + 2] as f64 * (1.0 - fx) * fy
                      + src[((y0 + 1) * w + x0 + 1) * 4 + 2] as f64 * fx * fy;
                pixels[idx + 2] = v.max(0.0).min(255.0) as u8;
            }
        }
    }
}


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

    // ── Post-pixel spatial operations (order matters!) ──

    // Texture (small-radius local contrast)
    if adj.texture.abs() > 0.001 {
        apply_texture(pixels, w, h, adj.texture);
    }

    // Clarity (large-radius local contrast)
    if adj.clarity.abs() > 0.001 {
        apply_clarity(pixels, w, h, adj.clarity);
    }

    // Dehaze (dark channel prior)
    if adj.dehaze.abs() > 0.001 {
        apply_dehaze(pixels, w, h, adj.dehaze);
    }

    // Sharpening (unsharp mask)
    if adj.sharpen_amount > 0.001 {
        apply_sharpening(pixels, w, h, adj.sharpen_amount, adj.sharpen_radius, adj.sharpen_detail, adj.sharpen_masking);
    }

    // Noise Reduction
    if adj.nr_luminance > 0.5 || adj.nr_color > 0.5 {
        apply_noise_reduction(pixels, w, h, adj.nr_luminance, adj.nr_color);
    }

    // Lens Distortion
    if adj.lens_distortion.abs() > 0.001 {
        apply_lens_distortion(pixels, w, h, adj.lens_distortion);
    }

    // Chromatic Aberration Correction
    if adj.lens_ca_red.abs() > 0.001 || adj.lens_ca_blue.abs() > 0.001 {
        apply_ca_correction(pixels, w, h, adj.lens_ca_red, adj.lens_ca_blue);
    }

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
pub async fn load_editor_source(
    state: tauri::State<'_, EditorState>,
    image_path: String,
) -> Result<(u32, u32), String> {
    let img = image::open(&image_path).map_err(|e| format!("Failed to open image: {}", e))?;
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    let pixels = rgba.into_raw();

    let mut cache = state.cached_source.lock().unwrap();
    *cache = Some(CachedImage {
        path: image_path,
        rgba: pixels,
        width,
        height,
    });

    Ok((width, height))
}

#[tauri::command]
pub async fn unload_editor_source(
    state: tauri::State<'_, EditorState>,
) -> Result<(), String> {
    let mut cache = state.cached_source.lock().unwrap();
    *cache = None;
    Ok(())
}

#[tauri::command]
pub async fn process_image(
    state: tauri::State<'_, EditorState>,
    image_path: String,
    adjustments: AdjustmentPayload,
    preview: bool,
) -> Result<ProcessResult, String> {
    // Try to use cached source, fallback to loading from disk
    let (source_rgba, src_w, src_h) = {
        let cache = state.cached_source.lock().unwrap();
        if let Some(ref cached) = *cache {
            if cached.path == image_path {
                (cached.rgba.clone(), cached.width, cached.height)
            } else {
                drop(cache);
                let img = image::open(&image_path).map_err(|e| format!("Failed to open image: {}", e))?;
                let rgba = img.to_rgba8();
                (rgba.clone().into_raw(), rgba.width(), rgba.height())
            }
        } else {
            drop(cache);
            let img = image::open(&image_path).map_err(|e| format!("Failed to open image: {}", e))?;
            let rgba = img.to_rgba8();
            (rgba.clone().into_raw(), rgba.width(), rgba.height())
        }
    };

    // Resize for preview if needed
    let (mut pixels, width, height) = if preview {
        let max_edge = 800u32;
        if src_w > max_edge || src_h > max_edge {
            let img_buf = image::RgbaImage::from_raw(src_w, src_h, source_rgba)
                .ok_or_else(|| "Failed to create image buffer".to_string())?;
            let dyn_img = image::DynamicImage::ImageRgba8(img_buf);
            let scale = max_edge as f64 / src_w.max(src_h) as f64;
            let nw = (src_w as f64 * scale) as u32;
            let nh = (src_h as f64 * scale) as u32;
            let resized = dyn_img.resize(nw, nh, image::imageops::FilterType::Triangle);
            let rgba = resized.to_rgba8();
            (rgba.clone().into_raw(), rgba.width(), rgba.height())
        } else {
            (source_rgba, src_w, src_h)
        }
    } else {
        (source_rgba, src_w, src_h)
    };

    let preview_path = state.preview_path.lock().unwrap().clone();
    let adj = adjustments;

    tokio::task::spawn_blocking(move || {
        process_pixels(&mut pixels, width, height, &adj);

        // Write to temp file as BMP (fast, lossless, no compression overhead)
        let img_buf = image::RgbaImage::from_raw(width, height, pixels)
            .ok_or_else(|| "Failed to create image buffer".to_string())?;
        img_buf.save_with_format(&preview_path, image::ImageFormat::Bmp)
            .map_err(|e| format!("Preview save error: {}", e))?;

        Ok(ProcessResult {
            preview_path,
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

// ── Auto-Enhance ──

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoEnhanceResult {
    pub exposure: f64,
    pub contrast: f64,
    pub highlights: f64,
    pub shadows: f64,
    pub vibrance: f64,
    pub saturation: f64,
}

#[tauri::command]
pub async fn auto_enhance(
    image_path: String,
) -> Result<AutoEnhanceResult, String> {
    let img = image::open(&image_path).map_err(|e| format!("Failed to open image: {}", e))?;

    // Use a small version for analysis
    let small = img.resize(400, 400, image::imageops::FilterType::Triangle);
    let pixels = small.to_rgba8().into_raw();

    tokio::task::spawn_blocking(move || {
        let n = pixels.len() / 4;
        if n == 0 {
            return Ok(AutoEnhanceResult {
                exposure: 0.0, contrast: 0.0, highlights: 0.0,
                shadows: 0.0, vibrance: 0.0, saturation: 0.0,
            });
        }

        // Compute luminance histogram
        let mut l_hist = [0u32; 256];
        let mut total_lum = 0.0_f64;
        let mut total_sat = 0.0_f64;

        for chunk in pixels.chunks(4) {
            if chunk.len() < 4 { continue; }
            let r = chunk[0] as f64 / 255.0;
            let g = chunk[1] as f64 / 255.0;
            let b = chunk[2] as f64 / 255.0;
            let lum = 0.299 * r + 0.587 * g + 0.114 * b;
            let idx = (lum * 255.0).round() as usize;
            l_hist[idx.min(255)] += 1;
            total_lum += lum;

            let max_c = r.max(g).max(b);
            let min_c = r.min(g).min(b);
            if max_c > 0.0 {
                total_sat += (max_c - min_c) / max_c;
            }
        }

        let avg_lum = total_lum / n as f64;
        let avg_sat = total_sat / n as f64;

        // Compute percentiles
        let total_px = n as f64;
        let mut cumulative = 0u32;
        let mut p2 = 0usize;
        let mut p98 = 255usize;
        for i in 0..256 {
            cumulative += l_hist[i];
            if cumulative as f64 / total_px < 0.02 { p2 = i; }
            if cumulative as f64 / total_px < 0.98 { p98 = i; }
        }

        // Suggest exposure: target avg_lum of ~0.45
        let exposure = if avg_lum < 0.35 {
            (0.45 - avg_lum) * 3.0  // up to ~0.3 EV
        } else if avg_lum > 0.55 {
            (0.45 - avg_lum) * 2.0  // reduce
        } else {
            0.0
        };

        // Suggest contrast: low-contrast images get a boost
        let range = (p98 - p2) as f64 / 255.0;
        let contrast = if range < 0.7 {
            (0.7 - range) * 40.0  // up to ~12
        } else {
            0.0
        };

        // Highlights: if bright pixels are clipped, pull down
        let highlights: f64 = if p98 > 250 { -15.0 } else { 0.0 };

        // Shadows: if darks are crushed, open up
        let shadows: f64 = if p2 < 5 { 15.0 } else { 0.0 };

        // Vibrance: if saturation is low, boost
        let vibrance = if avg_sat < 0.3 {
            (0.3 - avg_sat) * 80.0  // up to ~24
        } else {
            0.0
        };

        let saturation = 0.0; // leave global saturation to user

        Ok(AutoEnhanceResult {
            exposure: (exposure * 10.0).round() / 10.0,
            contrast: contrast.round(),
            highlights: highlights.round(),
            shadows: shadows.round(),
            vibrance: vibrance.round(),
            saturation,
        })
    })
    .await
    .map_err(|e| format!("Auto-enhance error: {}", e))?
}

// ── Export Image ──

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOptions {
    pub format: String,       // "jpeg", "png", "webp"
    pub quality: u8,          // 1-100
    pub max_long_edge: u32,   // 0 = full resolution
}

#[tauri::command]
pub async fn export_image(
    image_path: String,
    adjustments: AdjustmentPayload,
    options: ExportOptions,
    output_path: String,
) -> Result<(), String> {
    let img = image::open(&image_path).map_err(|e| format!("Failed to open: {}", e))?;

    // Optionally resize
    let img = if options.max_long_edge > 0 {
        let (w, h) = (img.width(), img.height());
        let max_e = options.max_long_edge;
        if w > max_e || h > max_e {
            let scale = max_e as f64 / w.max(h) as f64;
            let nw = (w as f64 * scale) as u32;
            let nh = (h as f64 * scale) as u32;
            img.resize(nw, nh, image::imageops::FilterType::Lanczos3)
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
    let adj = adjustments;
    let fmt = options.format.clone();
    let quality = options.quality;
    let out = output_path.clone();

    tokio::task::spawn_blocking(move || {
        process_pixels(&mut pixels, width, height, &adj);

        let img_buf = image::RgbaImage::from_raw(width, height, pixels)
            .ok_or_else(|| "Failed to create image buffer".to_string())?;

        match fmt.as_str() {
            "png" => {
                img_buf.save_with_format(&out, image::ImageFormat::Png)
                    .map_err(|e| format!("PNG save error: {}", e))?;
            }
            "webp" => {
                // Save as PNG fallback (image crate webp support varies)
                img_buf.save_with_format(&out, image::ImageFormat::Png)
                    .map_err(|e| format!("WebP save error: {}", e))?;
            }
            _ => {
                // JPEG: convert to RGB
                let rgb_img = image::DynamicImage::ImageRgba8(img_buf).to_rgb8();
                let mut file = std::fs::File::create(&out)
                    .map_err(|e| format!("File create error: {}", e))?;
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut file,
                    quality,
                );
                encoder.encode(
                    rgb_img.as_raw(),
                    width,
                    height,
                    image::ExtendedColorType::Rgb8,
                ).map_err(|e| format!("JPEG encode error: {}", e))?;
            }
        }

        Ok(())
    })
    .await
    .map_err(|e| format!("Export error: {}", e))?
}

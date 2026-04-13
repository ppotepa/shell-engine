//! Lens blur — separable Gaussian convolution that produces a smooth, optically correct blur.
//!
//! Unlike the box [`blur`](super::blur) effect (square kernel, hard edge), this effect computes
//! true Gaussian weights so bright areas spread naturally and high-frequency detail softens
//! gradually — closer to a real camera aperture blur (bokeh / depth-of-field feel).
//!
//! Two-pass separable implementation: horizontal → vertical, each O(w·h·kernel_size).
//! At the typical SDL2 pixel resolution (640×360) the total work per frame is fast
//! enough to keep the game at 60 fps in release mode even with `passes: 2`.
//!
//! ## Parameters
//! | Param | Type | Default | Notes |
//! |-------|------|---------|-------|
//! | `radius` | f32 | 1.5 | Gaussian σ (standard deviation) in cells. Kernel extends to `ceil(3σ)` each side. |
//! | `intensity` | f32 | 1.0 | Blend between original (0) and blurred (1). |
//! | `passes` | int/f32 | 1 | Number of Gaussian passes (each doubles effective spread). |

use engine_core::buffer::{Buffer, TRUE_BLACK};
use engine_core::color::Color;
use engine_core::effects::{Effect, EffectTargetMask, Region};
use engine_core::scene::EffectParams;

use crate::metadata::{slider, EffectMetadata, P_EASING};
use crate::utils::color::colour_to_rgb;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "lens-blur",
    display_name: "Lens Blur",
    summary: "Gaussian aperture blur — smooth depth-of-field / bokeh feel on any layer or sprite.",
    category: "colour",
    compatible_targets: EffectTargetMask::ANY,
    params: &[
        slider(
            "radius",
            "Radius (σ)",
            "Gaussian standard deviation in cells. Higher = wider, softer blur.",
            0.5,
            6.0,
            1.5,
            "",
        ),
        slider(
            "intensity",
            "Intensity",
            "Blend between original (0) and fully blurred (1).",
            0.0,
            1.0,
            1.0,
            "",
        ),
        slider(
            "passes",
            "Passes",
            "Convolution passes. Each pass doubles the effective spread. 1–3 recommended.",
            1.0,
            3.0,
            1.0,
            "",
        ),
        P_EASING,
    ],
    sample: "- name: lens-blur\n  duration: 0\n  looping: true\n  params:\n    radius: 1.5\n    intensity: 0.85",
};

pub struct LensBlurEffect;

impl Effect for LensBlurEffect {
    fn apply(&self, _progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let sigma = params.radius.unwrap_or(1.5).clamp(0.3, 8.0);
        let intensity = params.intensity.unwrap_or(1.0).clamp(0.0, 1.0);
        let passes = params.passes.unwrap_or(1.0).clamp(1.0, 4.0).round() as u32;

        let w = region.width as usize;
        let h = region.height as usize;

        // Extract fg/bg colours from the region into flat RGB arrays (avoids repeated buffer lookups).
        let mut fg = vec![[0u8; 3]; w * h];
        let mut bg = vec![[0u8; 3]; w * h];
        let mut skip = vec![false; w * h]; // true = fully transparent, do not touch

        for dy in 0..h {
            for dx in 0..w {
                let cell = buffer
                    .get(region.x + dx as u16, region.y + dy as u16)
                    .cloned()
                    .unwrap_or_default();
                let idx = dy * w + dx;
                if cell.symbol == ' ' && matches!(cell.bg, Color::Reset) {
                    skip[idx] = true;
                    continue;
                }
                let (fr, fg_g, fb) = colour_to_rgb(cell.fg);
                fg[idx] = [fr, fg_g, fb];
                let bg_col = if matches!(cell.bg, Color::Reset) {
                    TRUE_BLACK
                } else {
                    cell.bg
                };
                let (br, bgr, bb) = colour_to_rgb(bg_col);
                bg[idx] = [br, bgr, bb];
            }
        }

        // Keep the original for intensity blending.
        let orig_fg = fg.clone();
        let orig_bg = bg.clone();

        // Apply `passes` rounds of separable Gaussian convolution.
        for _ in 0..passes {
            let kernel = build_gaussian_kernel(sigma);
            let r = kernel.len() / 2;

            // --- Horizontal pass ---
            let mut h_fg = fg.clone();
            let mut h_bg = bg.clone();
            for dy in 0..h {
                for dx in 0..w {
                    let center_idx = dy * w + dx;
                    if skip[center_idx] {
                        continue;
                    }
                    let (mut sr, mut sg, mut sb) = (0.0f32, 0.0f32, 0.0f32);
                    let (mut br, mut bg_g, mut bb) = (0.0f32, 0.0f32, 0.0f32);
                    let mut wsum = 0.0f32;
                    for (ki, &w_val) in kernel.iter().enumerate() {
                        let kx = dx as i32 + ki as i32 - r as i32;
                        if kx < 0 || kx >= w as i32 {
                            continue;
                        }
                        let src = dy * w + kx as usize;
                        if skip[src] {
                            continue;
                        }
                        sr += fg[src][0] as f32 * w_val;
                        sg += fg[src][1] as f32 * w_val;
                        sb += fg[src][2] as f32 * w_val;
                        br += bg[src][0] as f32 * w_val;
                        bg_g += bg[src][1] as f32 * w_val;
                        bb += bg[src][2] as f32 * w_val;
                        wsum += w_val;
                    }
                    if wsum > 0.0 {
                        h_fg[center_idx] =
                            [(sr / wsum) as u8, (sg / wsum) as u8, (sb / wsum) as u8];
                        h_bg[center_idx] =
                            [(br / wsum) as u8, (bg_g / wsum) as u8, (bb / wsum) as u8];
                    }
                }
            }

            // --- Vertical pass ---
            let mut v_fg = h_fg.clone();
            let mut v_bg = h_bg.clone();
            for dy in 0..h {
                for dx in 0..w {
                    let center_idx = dy * w + dx;
                    if skip[center_idx] {
                        continue;
                    }
                    let (mut sr, mut sg, mut sb) = (0.0f32, 0.0f32, 0.0f32);
                    let (mut br2, mut bg2, mut bb2) = (0.0f32, 0.0f32, 0.0f32);
                    let mut wsum = 0.0f32;
                    for (ki, &w_val) in kernel.iter().enumerate() {
                        let ky = dy as i32 + ki as i32 - r as i32;
                        if ky < 0 || ky >= h as i32 {
                            continue;
                        }
                        let src = ky as usize * w + dx;
                        if skip[src] {
                            continue;
                        }
                        sr += h_fg[src][0] as f32 * w_val;
                        sg += h_fg[src][1] as f32 * w_val;
                        sb += h_fg[src][2] as f32 * w_val;
                        br2 += h_bg[src][0] as f32 * w_val;
                        bg2 += h_bg[src][1] as f32 * w_val;
                        bb2 += h_bg[src][2] as f32 * w_val;
                        wsum += w_val;
                    }
                    if wsum > 0.0 {
                        v_fg[center_idx] =
                            [(sr / wsum) as u8, (sg / wsum) as u8, (sb / wsum) as u8];
                        v_bg[center_idx] =
                            [(br2 / wsum) as u8, (bg2 / wsum) as u8, (bb2 / wsum) as u8];
                    }
                }
            }

            fg = v_fg;
            bg = v_bg;
        }

        // Write back — lerp between original and blurred by intensity, preserving symbols.
        for dy in 0..h {
            for dx in 0..w {
                let idx = dy * w + dx;
                if skip[idx] {
                    continue;
                }
                let cell = buffer
                    .get(region.x + dx as u16, region.y + dy as u16)
                    .cloned()
                    .unwrap_or_default();

                let lerp =
                    |a: u8, b: u8| -> u8 { (a as f32 + (b as f32 - a as f32) * intensity) as u8 };

                let out_fg = Color::Rgb {
                    r: lerp(orig_fg[idx][0], fg[idx][0]),
                    g: lerp(orig_fg[idx][1], fg[idx][1]),
                    b: lerp(orig_fg[idx][2], fg[idx][2]),
                };
                let out_bg = Color::Rgb {
                    r: lerp(orig_bg[idx][0], bg[idx][0]),
                    g: lerp(orig_bg[idx][1], bg[idx][1]),
                    b: lerp(orig_bg[idx][2], bg[idx][2]),
                };
                buffer.set(
                    region.x + dx as u16,
                    region.y + dy as u16,
                    cell.symbol,
                    out_fg,
                    out_bg,
                );
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}

/// Build a normalized Gaussian kernel with `σ = sigma`, extending `ceil(3σ)` cells each side.
fn build_gaussian_kernel(sigma: f32) -> Vec<f32> {
    let r = (sigma * 3.0).ceil().max(1.0) as usize;
    let size = 2 * r + 1;
    let denom = 2.0 * sigma * sigma;
    let mut kernel = vec![0.0f32; size];
    let mut sum = 0.0f32;
    for i in 0..size {
        let x = i as f32 - r as f32;
        kernel[i] = (-x * x / denom).exp();
        sum += kernel[i];
    }
    for w in &mut kernel {
        *w /= sum;
    }
    kernel
}

//! Unified CRT post-processing pass.
//!
//! Combines up to four sub-effects — **Underlay** (glow), **Distort** (barrel
//! warp), **ScanGlitch** (horizontal bands), and **Ruby** (tint + edge reveal)
//! — into a single pixel-loop pass.  When scenes stack multiple CRT effects
//! the registry auto-coalesces them into one `CrtComposite`, eliminating
//! buffer swaps and redundant full-buffer iterations.
//!
//! Visual order preserved inside the single loop:
//! 1. Glow pre-pass — half-res blur (same quality as standalone Underlay)
//! 2. Per-pixel: **distort** → **glow blend** → **scan-glitch** → **ruby tint**

use super::glow::{GlowScratch, GLOW_SCRATCH};
use super::registry::PostFxBuiltin;
use super::{
    lerp_colour_local, normalize_bg, normalized_coords, rand01, scale_colour, PostFxContext,
};
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::effects::utils::color::colour_to_rgb;
use engine_core::scene::Effect;

// ── Precomputed scan-glitch band ──────────────────────────────────────────

#[derive(Clone, Copy)]
struct GlitchBand {
    center: i32,
    half: i32,
    shift_max: f32,
    chroma: i32,
    blend_base: f32,
    brightness: f32,
}

// ── Entry point ───────────────────────────────────────────────────────────

pub(super) fn apply(
    ctx: &PostFxContext<'_>,
    src: &Buffer,
    dst: &mut Buffer,
    sub_passes: &[(PostFxBuiltin, Effect)],
) {
    if src.width <= 2 || src.height <= 2 {
        dst.copy_back_from(src);
        return;
    }

    let frame = ctx.frame_count as u32;
    let t = ctx.scene_elapsed_ms as f32 / 1000.0;
    let w = (src.width - 1) as f32;
    let h = (src.height - 1) as f32;

    // ── Collect sub-effect configs ────────────────────────────────────────

    let mut underlay_count = 0usize;
    let mut glow_intensity_max = 0.0_f32;
    let mut glow_alpha_sum = 0.0_f32;
    let mut glow_brightness_sum = 0.0_f32;
    let mut glow_speed_sum = 0.0_f32;
    let mut glow_spread_max = 0.0_f32;
    let mut distort_effect = None;
    let mut scan_effect = None;
    let mut ruby_effect = None;
    for (kind, effect) in sub_passes {
        match kind {
            PostFxBuiltin::Underlay => {
                underlay_count += 1;
                glow_intensity_max = glow_intensity_max.max(effect.params.intensity.unwrap_or(1.05));
                glow_alpha_sum += effect.params.alpha.unwrap_or(0.30);
                glow_brightness_sum += effect.params.brightness.unwrap_or(1.08);
                glow_speed_sum += effect.params.speed.unwrap_or(0.35);
                glow_spread_max = glow_spread_max.max(effect.params.transparency.unwrap_or(0.32));
            }
            PostFxBuiltin::Distort => distort_effect = Some(effect),
            PostFxBuiltin::ScanGlitch => scan_effect = Some(effect),
            PostFxBuiltin::Ruby => ruby_effect = Some(effect),
            _ => {}
        }
    }

    // ── 1. Glow pre-pass ─────────────────────────────────────────────────

    let has_glow = underlay_count != 0;

    let (glow_alpha, glow_brightness, glow_speed) = if has_glow {
        let n = underlay_count as f32;
        (
            glow_alpha_sum.clamp(0.0, 1.0),
            glow_brightness_sum / n,
            glow_speed_sum / n,
        )
    } else {
        (0.0, 1.0, 0.35)
    };

    if has_glow {
        GLOW_SCRATCH.with(|scratch| {
            let mut s = scratch.borrow_mut();
            let GlowScratch { a, b, out } = &mut *s;
            super::glow::build_glow_map_inplace(
                src,
                glow_intensity_max,
                glow_spread_max,
                frame,
                a,
                b,
                out,
            );
        });
    }

    // ── 2. Distort params ────────────────────────────────────────────────

    let distort_cfg = distort_effect.map(|pass| {
        let intensity = pass.params.intensity.unwrap_or(0.32).clamp(0.0, 2.0);
        let distortion = pass.params.distortion.unwrap_or(0.10).clamp(0.0, 1.0);
        let curvature = pass.params.sphericality.unwrap_or(0.26).clamp(0.0, 1.0);
        let margin_ctl = pass.params.transparency.unwrap_or(0.24).clamp(0.0, 1.0);
        let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.6, 1.4);
        let intensity01 = (intensity / 2.0).clamp(0.0, 1.0);
        let strength = (0.35 * curvature + 0.25 * intensity01 + 0.40 * distortion).clamp(0.0, 1.0);
        let inset_x = (0.001 + 0.008 * margin_ctl + 0.004 * strength).clamp(0.0, 0.02);
        let inset_y = (0.002 + 0.012 * margin_ctl + 0.006 * strength).clamp(0.0, 0.03);
        (strength, inset_x, inset_y, brightness)
    });

    // ── 3. Scan-glitch bands ─────────────────────────────────────────────

    let mut bands: [Option<GlitchBand>; 2] = [None, None];
    let mut band_count = 0usize;
    if let Some(pass) = scan_effect {
            let intensity = pass.params.intensity.unwrap_or(0.35).clamp(0.0, 2.0);
            let speed = pass.params.speed.unwrap_or(0.65).clamp(0.0, 2.0);
            let thickness = pass.params.transparency.unwrap_or(0.35).clamp(0.0, 1.0);
            let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.6, 1.5);

            let band_half = (1.0 + thickness * 3.0).round() as i32;
            let extra = if rand01(3, 11, frame.wrapping_add(97)) < (0.08 + speed * 0.16) {
                2
            } else {
                1
            };

            for idx in 0..extra {
                let roll = rand01(
                    17 + idx as u16 * 13,
                    41,
                    frame.wrapping_add(991 + idx as u32 * 211),
                );
                if roll > 0.07 + speed * 0.23 {
                    continue;
                }
                let center = (rand01(29 + idx as u16 * 7, 5, frame.wrapping_add(3331))
                    * src.height.max(1) as f32) as i32;
                bands[band_count] = Some(GlitchBand {
                    center,
                    half: band_half,
                    shift_max: ((1.0 + intensity * 3.5) / 3.0).clamp(0.0, 8.0),
                    chroma: ((1.0 + intensity * 1.8) / 3.0).round() as i32,
                    blend_base: 0.16 + 0.30 * intensity,
                    brightness,
                });
                band_count += 1;
            }
    }

    // ── 4. Ruby params ───────────────────────────────────────────────────

    struct RubyCfg {
        intensity: f32,
        brightness: f32,
        ruby: Color,
        ruby_bg: Color,
        tint: f32,
        front: f32,
        band: f32,
        shift: i32,
        chroma: i32,
    }

    let ruby_cfg = ruby_effect.map(|pass| {
        let intensity = pass.params.intensity.unwrap_or(0.28).clamp(0.0, 2.0);
        let speed = pass.params.speed.unwrap_or(0.55).clamp(0.0, 2.0);
        let thickness = pass.params.transparency.unwrap_or(0.24).clamp(0.0, 1.0);
        let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.6, 1.6);

        let tempo_jitter = 0.88 + 0.28 * rand01(71, 9, frame / 14);
        RubyCfg {
            intensity,
            brightness,
            ruby: Color::Rgb {
                r: 190,
                g: 58,
                b: 88,
            },
            ruby_bg: Color::Rgb {
                r: 92,
                g: 20,
                b: 36,
            },
            tint: (0.08 + 0.22 * intensity).clamp(0.0, 0.45),
            front: ((t * (0.20 + 0.35 * speed) * tempo_jitter) % 1.0) * 0.5,
            band: (0.018 + 0.050 * thickness).clamp(0.01, 0.09),
            shift: ((0.5 + 1.5 * intensity) / 2.0).round() as i32,
            chroma: ((1.0 + intensity * 1.6) / 2.5).round() as i32,
        }
    });

    // ── 5. Main pixel loop ───────────────────────────────────────────────

    GLOW_SCRATCH.with(|scratch| {
        let s = scratch.borrow();
        let glow_out = &s.out;
        let glow_width = src.width as usize;

        for y in 0..src.height {
            for x in 0..src.width {
                let orig = match src.get(x, y).cloned() {
                    Some(c) => c,
                    None => continue,
                };

                // ── DISTORT ──────────────────────────────────────────
                let (sample, shade, distort_brightness) =
                    if let Some((strength, inset_x, inset_y, d_bright)) = distort_cfg {
                        let ux = if w <= 0.0 {
                            0.0
                        } else {
                            (x as f32 / w) * 2.0 - 1.0
                        };
                        let uy = if h <= 0.0 {
                            0.0
                        } else {
                            (y as f32 / h) * 2.0 - 1.0
                        };
                        let curve_x = (1.0 - (0.06 + 0.18 * strength) * uy * uy).clamp(0.72, 1.0);
                        let curve_y = (1.0 - (0.04 + 0.14 * strength) * ux * ux).clamp(0.74, 1.0);
                        let su = (ux * curve_x).clamp(-1.0, 1.0);
                        let sv = (uy * curve_y).clamp(-1.0, 1.0);
                        let u = inset_x + ((su + 1.0) * 0.5) * (1.0 - 2.0 * inset_x);
                        let v = inset_y + ((sv + 1.0) * 0.5) * (1.0 - 2.0 * inset_y);
                        let sx = (u.clamp(0.0, 1.0) * w).round() as u16;
                        let sy = (v.clamp(0.0, 1.0) * h).round() as u16;

                        let sample = src.get(sx, sy).cloned().unwrap_or(orig.clone());
                        let edge = ux.abs().max(uy.abs()).clamp(0.0, 1.0);
                        let shade = (1.0 - edge * (0.05 + 0.06 * strength)).clamp(0.82, 1.0);
                        (sample, shade, d_bright)
                    } else {
                        (orig.clone(), 1.0, 1.0)
                    };

                // Blend fg: preserve glyph identity.
                let fg_source = if orig.symbol != ' ' {
                    if let Some((strength, _, _, _)) = distort_cfg {
                        let blend = if sample.symbol == ' ' {
                            0.05
                        } else {
                            (0.08 + 0.14 * strength).clamp(0.0, 0.24)
                        };
                        lerp_colour_local(orig.fg, sample.fg, blend)
                    } else {
                        orig.fg
                    }
                } else {
                    sample.fg
                };

                let mut fg = scale_colour(fg_source, distort_brightness * shade);
                let mut bg = scale_colour(normalize_bg(sample.bg), (0.94 * shade).clamp(0.70, 1.0));
                let mut symbol = orig.symbol;

                // ── GLOW (empty cells only) ──────────────────────────
                if has_glow && symbol == ' ' {
                    let idx = (y as usize) * glow_width + x as usize;
                    if idx < glow_out.len() {
                        let pix = glow_out[idx];
                        if pix.a >= 0.004 {
                            let pulse = 0.90
                                + 0.10
                                    * ((t * (0.95 + glow_speed * 1.9) + y as f32 * 0.07).sin()
                                        * 0.5
                                        + 0.5);
                            let shimmer = 0.92 + 0.16 * rand01(x, y, frame.wrapping_add(4901));
                            let aura = (pix.a * glow_brightness * pulse * shimmer * glow_alpha)
                                .clamp(0.0, 1.0);
                            let glow_colour = Color::Rgb {
                                r: (pix.r * 255.0).round().clamp(0.0, 255.0) as u8,
                                g: (pix.g * 255.0).round().clamp(0.0, 255.0) as u8,
                                b: (pix.b * 255.0).round().clamp(0.0, 255.0) as u8,
                            };
                            bg = lerp_colour_local(
                                bg,
                                glow_colour,
                                (aura * (0.60 + 0.65 * glow_intensity_max)).clamp(0.0, 0.35),
                            );
                        }
                    }
                }

                // ── SCAN GLITCH ──────────────────────────────────────
                for band in bands[..band_count].iter().flatten() {
                    let dy = (y as i32) - band.center;
                    if dy.abs() > band.half {
                        continue;
                    }
                    let local = 1.0 - (dy.abs() as f32 / (band.half + 1) as f32);
                    let shift = (band.shift_max * (0.5 + 0.5 * local)).round() as i32;
                    let blend = (band.blend_base * local).clamp(0.0, 0.55);
                    let scan_bright = 1.0 + 0.30 * local * band.brightness;

                    let xi = x as i32;
                    let sx_r = (xi - shift).clamp(0, src.width as i32 - 1) as u16;
                    let sx_g = (xi - shift + band.chroma / 2).clamp(0, src.width as i32 - 1) as u16;
                    let sx_b = (xi - shift + band.chroma).clamp(0, src.width as i32 - 1) as u16;

                    let base_cell = src.get(sx_r, y).cloned().unwrap_or(orig);
                    let (rr, _, _) =
                        colour_to_rgb(src.get(sx_r, y).map(|c| c.fg).unwrap_or(base_cell.fg));
                    let (_, gg, _) =
                        colour_to_rgb(src.get(sx_g, y).map(|c| c.fg).unwrap_or(base_cell.fg));
                    let (_, _, bb) =
                        colour_to_rgb(src.get(sx_b, y).map(|c| c.fg).unwrap_or(base_cell.fg));
                    let chroma_fg = Color::Rgb {
                        r: rr,
                        g: gg,
                        b: bb,
                    };
                    fg = lerp_colour_local(fg, scale_colour(chroma_fg, scan_bright), blend);
                    symbol = base_cell.symbol;
                    break; // one band per pixel
                }

                // ── RUBY ─────────────────────────────────────────────
                if let Some(r) = &ruby_cfg {
                    let (nx, ny) = normalized_coords(x, y, src.width, src.height);
                    let radius =
                        ((nx * nx + ny * ny).sqrt() / std::f32::consts::SQRT_2).clamp(0.0, 1.0);
                    let center_weight = (1.0 - radius).powf(1.35);
                    let center_dark =
                        (1.0 - center_weight * (0.05 + 0.11 * r.intensity)).clamp(0.78, 1.0);

                    fg = scale_colour(
                        lerp_colour_local(fg, r.ruby, r.tint),
                        center_dark * r.brightness,
                    );
                    bg = scale_colour(lerp_colour_local(bg, r.ruby_bg, r.tint * 0.55), center_dark);

                    // Edge-reveal band.
                    let xn = if src.width <= 1 {
                        0.0
                    } else {
                        x as f32 / (src.width - 1) as f32
                    };
                    let yn = if src.height <= 1 {
                        0.0
                    } else {
                        y as f32 / (src.height - 1) as f32
                    };
                    let edge_dist = xn.min(1.0 - xn).min(yn.min(1.0 - yn));
                    let band_dist = (edge_dist - r.front).abs();
                    if band_dist <= r.band {
                        let xi = x as i32;
                        let sx = (xi - r.shift).clamp(0, src.width as i32 - 1) as u16;
                        let sx_g =
                            (xi - r.shift + r.chroma / 2).clamp(0, src.width as i32 - 1) as u16;
                        let sx_b = (xi - r.shift + r.chroma).clamp(0, src.width as i32 - 1) as u16;

                        if let Some(rsample) = src.get(sx, y).cloned() {
                            if symbol == ' ' && rsample.symbol != ' ' {
                                symbol = rsample.symbol;
                            }
                            let (rr, _, _) =
                                colour_to_rgb(src.get(sx, y).map(|c| c.fg).unwrap_or(rsample.fg));
                            let (_, gg, _) =
                                colour_to_rgb(src.get(sx_g, y).map(|c| c.fg).unwrap_or(rsample.fg));
                            let (_, _, bb) =
                                colour_to_rgb(src.get(sx_b, y).map(|c| c.fg).unwrap_or(rsample.fg));
                            let chroma_fg = Color::Rgb {
                                r: rr,
                                g: gg,
                                b: bb,
                            };
                            let local_r = 1.0 - (band_dist / r.band).clamp(0.0, 1.0);
                            let reveal_blend =
                                (0.12 + 0.30 * local_r * r.intensity).clamp(0.0, 0.45);
                            fg = lerp_colour_local(
                                fg,
                                scale_colour(chroma_fg, 1.0 + 0.18 * local_r),
                                reveal_blend,
                            );
                            bg = lerp_colour_local(
                                bg,
                                normalize_bg(rsample.bg),
                                (reveal_blend * 0.40).clamp(0.0, 0.20),
                            );
                        }
                    }
                }

                dst.set(x, y, symbol, fg, bg);
            }
        }
    });
}

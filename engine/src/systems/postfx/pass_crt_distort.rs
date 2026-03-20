use super::{normalize_bg, scale_colour, PostFxContext};
use crate::buffer::Buffer;
use crate::scene::Effect;

pub(super) fn apply(_ctx: &PostFxContext<'_>, src: &Buffer, dst: &mut Buffer, pass: &Effect) {
    let intensity = pass.params.intensity.unwrap_or(0.32).clamp(0.0, 2.0);
    let distortion = pass.params.distortion.unwrap_or(0.10).clamp(0.0, 1.0);
    let curvature = pass.params.sphericality.unwrap_or(0.26).clamp(0.0, 1.0);
    let margin_ctl = pass.params.transparency.unwrap_or(0.24).clamp(0.0, 1.0);
    let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.6, 1.4);

    if src.width <= 2 || src.height <= 2 {
        dst.clone_from(src);
        return;
    }

    // Safe, non-cropping CRT remap for low-resolution terminal buffers.
    // We never blank out edge cells; instead we remap samples with a small inset.
    let intensity01 = (intensity / 2.0).clamp(0.0, 1.0);
    let strength = (0.35 * curvature + 0.25 * intensity01 + 0.40 * distortion).clamp(0.0, 1.0);
    let inset_x = (0.001 + 0.008 * margin_ctl + 0.004 * strength).clamp(0.0, 0.02);
    let inset_y = (0.002 + 0.012 * margin_ctl + 0.006 * strength).clamp(0.0, 0.03);
    let w = (src.width - 1) as f32;
    let h = (src.height - 1) as f32;

    for y in 0..src.height {
        for x in 0..src.width {
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

            let Some(sample) = src.get(sx, sy).cloned() else {
                continue;
            };

            let Some(orig) = src.get(x, y).cloned() else {
                continue;
            };

            let edge = ux.abs().max(uy.abs()).clamp(0.0, 1.0);
            let shade = (1.0 - edge * (0.05 + 0.06 * strength)).clamp(0.82, 1.0);
            let fg_source = if orig.symbol != ' ' {
                // Keep text readable: preserve glyph identity, only tint slightly from warped sample.
                let blend = if sample.symbol == ' ' {
                    0.05
                } else {
                    (0.08 + 0.14 * strength).clamp(0.0, 0.24)
                };
                super::lerp_colour_local(orig.fg, sample.fg, blend)
            } else {
                sample.fg
            };
            let fg = scale_colour(fg_source, brightness * shade);
            let bg = scale_colour(normalize_bg(sample.bg), (0.94 * shade).clamp(0.70, 1.0));
            // Preserve original glyph identity to avoid double-text artifacts.
            dst.set(x, y, orig.symbol, fg, bg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::apply;
    use crate::buffer::Buffer;
    use crate::scene::{Effect, EffectParams, EffectTargetKind};
    use crossterm::style::Color;

    fn pass_with_defaults() -> Effect {
        Effect {
            name: "crt-distort".to_string(),
            duration: 0,
            looping: true,
            target_kind: EffectTargetKind::Any,
            params: EffectParams {
                intensity: Some(0.30),
                distortion: Some(0.10),
                sphericality: Some(0.20),
                transparency: Some(0.20),
                brightness: Some(1.0),
                ..EffectParams::default()
            },
        }
    }

    #[test]
    fn does_not_blank_edge_glyphs() {
        let mut src = Buffer::new(24, 8);
        for y in 0..src.height {
            for x in 0..src.width {
                src.set(x, y, '#', Color::White, Color::Black);
            }
        }
        let mut dst = Buffer::new(src.width, src.height);
        let ctx = super::PostFxContext {
            frame_count: 0,
            scene_elapsed_ms: 0,
            _phantom: std::marker::PhantomData,
        };

        apply(&ctx, &src, &mut dst, &pass_with_defaults());

        for (x, y) in [
            (0, 0),
            (src.width - 1, 0),
            (0, src.height - 1),
            (src.width - 1, src.height - 1),
        ] {
            let cell = dst.get(x, y).expect("edge cell exists");
            assert_eq!(cell.symbol, '#', "edge glyph at ({x},{y}) should remain");
        }
    }
}

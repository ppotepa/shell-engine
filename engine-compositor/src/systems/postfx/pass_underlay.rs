use super::glow::{GlowScratch, GLOW_SCRATCH};
use super::{lerp_colour_local, normalize_bg, rand01, PostFxContext};
use engine_core::buffer::Buffer;
use engine_core::scene::Effect;
use crossterm::style::Color;

pub(super) fn apply(ctx: &PostFxContext<'_>, src: &Buffer, dst: &mut Buffer, pass: &Effect) {
    if src.width == 0 || src.height == 0 {
        dst.copy_back_from(src);
        return;
    }
    // Photoshop-style model:
    // 1) duplicate visible scene content,
    // 2) offset the duplicate,
    // 3) blur it,
    // 4) blend only into empty cells under the original content.
    let intensity = pass.params.intensity.unwrap_or(1.05).clamp(0.0, 2.0);
    let alpha = pass.params.alpha.unwrap_or(0.30).clamp(0.0, 1.0);
    let spread = pass.params.transparency.unwrap_or(0.32).clamp(0.0, 1.0);
    let brightness = pass.params.brightness.unwrap_or(1.08).clamp(0.6, 2.0);
    let speed = pass.params.speed.unwrap_or(0.35).clamp(0.0, 1.2);
    // Keep underlay spatially aligned with source by default (offset 0x0).
    let _legacy_offset = pass.params.sphericality.unwrap_or(0.18).clamp(0.0, 1.0);
    let frame = ctx.frame_count as u32;

    GLOW_SCRATCH.with(|scratch| {
        let mut s = scratch.borrow_mut();
        {
            let GlowScratch { a, b, out } = &mut *s;
            super::glow::build_glow_map_inplace(src, intensity, spread, frame, a, b, out);
        }

        let width = src.width as usize;
        let t = ctx.scene_elapsed_ms as f32 / 1000.0;

        for y in 0..src.height {
            for x in 0..src.width {
                let Some(current) = src.get(x, y).cloned() else {
                    continue;
                };
                if current.symbol != ' ' {
                    // Preserve content layer untouched.
                    dst.set(x, y, current.symbol, current.fg, current.bg);
                    continue;
                }
                let pix = s.out[(y as usize) * width + x as usize];
                if pix.a < 0.004 {
                    dst.set(x, y, current.symbol, current.fg, current.bg);
                    continue;
                }

                let pulse =
                    0.90 + 0.10 * ((t * (0.95 + speed * 1.9) + y as f32 * 0.07).sin() * 0.5 + 0.5);
                let shimmer = 0.92 + 0.16 * rand01(x, y, frame.wrapping_add(4901));
                let aura = (pix.a * brightness * pulse * shimmer * alpha).clamp(0.0, 1.0);
                let glow_colour = Color::Rgb {
                    r: (pix.r * 255.0).round().clamp(0.0, 255.0) as u8,
                    g: (pix.g * 255.0).round().clamp(0.0, 255.0) as u8,
                    b: (pix.b * 255.0).round().clamp(0.0, 255.0) as u8,
                };
                let bg = lerp_colour_local(
                    normalize_bg(current.bg),
                    glow_colour,
                    (aura * (0.60 + 0.65 * intensity)).clamp(0.0, 0.35),
                );
                dst.set(x, y, current.symbol, current.fg, bg);
            }
        }
    });
}

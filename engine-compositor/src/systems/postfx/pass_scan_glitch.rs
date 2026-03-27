use super::{lerp_colour_local, rand01, scale_colour, PostFxContext};
use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::effects::utils::color::colour_to_rgb;
use engine_core::scene::Effect;

pub(super) fn apply(ctx: &PostFxContext<'_>, src: &Buffer, dst: &mut Buffer, pass: &Effect) {
    let intensity = pass.params.intensity.unwrap_or(0.35).clamp(0.0, 2.0);
    let speed = pass.params.speed.unwrap_or(0.65).clamp(0.0, 2.0);
    let thickness = pass.params.transparency.unwrap_or(0.35).clamp(0.0, 1.0);
    let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.6, 1.5);
    let frame = ctx.frame_count as u32;

    // This pass modifies only selected scan-bands, so preserve the full source first.
    dst.copy_back_from(src);

    let band_half = (1.0 + thickness * 3.0).round() as i32; // 1..4 lines each side
    let extra_bands = if rand01(3, 11, frame.wrapping_add(97)) < (0.08 + speed * 0.16) {
        2
    } else {
        1
    };

    for band_idx in 0..extra_bands {
        let roll = rand01(
            17 + band_idx as u16 * 13,
            41,
            frame.wrapping_add(991 + band_idx as u32 * 211),
        );
        let trigger = 0.07 + speed * 0.23;
        if roll > trigger {
            continue;
        }

        let center = (rand01(29 + band_idx as u16 * 7, 5, frame.wrapping_add(3331))
            * src.height.max(1) as f32) as i32;
        // Horizontal displacement tuned to remain subtle.
        let shift_max = ((1.0 + intensity * 3.5) / 3.0).clamp(0.0, 8.0);

        for dy in -band_half..=band_half {
            let y = center + dy;
            if y < 0 || y >= src.height as i32 {
                continue;
            }
            let local = 1.0 - (dy.abs() as f32 / (band_half + 1) as f32);
            let shift = (shift_max * (0.5 + 0.5 * local)).round() as i32;
            let chroma = ((1.0 + intensity * 1.8) / 3.0).round() as i32;
            let blend = (0.16 + 0.30 * intensity * local).clamp(0.0, 0.55);
            let scan_bright = 1.0 + 0.30 * local * brightness;

            for x in 0..src.width {
                let xi = x as i32;
                let sx = (xi - shift).clamp(0, src.width as i32 - 1) as u16;
                let sx_r = (xi - shift).clamp(0, src.width as i32 - 1) as u16;
                let sx_g = (xi - shift + chroma / 2).clamp(0, src.width as i32 - 1) as u16;
                let sx_b = (xi - shift + chroma).clamp(0, src.width as i32 - 1) as u16;

                let row = y as u16;
                let Some(base) = src.get(sx, row).cloned() else {
                    continue;
                };
                let Some(orig) = src.get(x, row).cloned() else {
                    continue;
                };

                let fg_r = src.get(sx_r, row).map(|c| c.fg).unwrap_or(base.fg);
                let fg_g = src.get(sx_g, row).map(|c| c.fg).unwrap_or(base.fg);
                let fg_b = src.get(sx_b, row).map(|c| c.fg).unwrap_or(base.fg);
                let (rr, _, _) = colour_to_rgb(fg_r);
                let (_, gg, _) = colour_to_rgb(fg_g);
                let (_, _, bb) = colour_to_rgb(fg_b);
                let chroma_fg = Color::Rgb {
                    r: rr,
                    g: gg,
                    b: bb,
                };
                let boosted = scale_colour(chroma_fg, scan_bright);
                let out_fg = lerp_colour_local(orig.fg, boosted, blend);

                dst.set(x, row, base.symbol, out_fg, base.bg);
            }
        }
    }
}

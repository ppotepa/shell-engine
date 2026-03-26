use super::{
    lerp_colour_local, normalize_bg, normalized_coords, rand01, scale_colour, PostFxContext,
};
use engine_core::color::Color;
use engine_core::buffer::Buffer;
use engine_core::effects::utils::color::colour_to_rgb;
use engine_core::scene::Effect;

pub(super) fn apply(ctx: &PostFxContext<'_>, src: &Buffer, dst: &mut Buffer, pass: &Effect) {
    let intensity = pass.params.intensity.unwrap_or(0.28).clamp(0.0, 2.0);
    let speed = pass.params.speed.unwrap_or(0.55).clamp(0.0, 2.0);
    let thickness = pass.params.transparency.unwrap_or(0.24).clamp(0.0, 1.0);
    let brightness = pass.params.brightness.unwrap_or(1.0).clamp(0.6, 1.6);
    let frame = ctx.frame_count as u32;

    let ruby = Color::Rgb {
        r: 190,
        g: 58,
        b: 88,
    };
    let ruby_bg = Color::Rgb {
        r: 92,
        g: 20,
        b: 36,
    };
    let tint = (0.08 + 0.22 * intensity).clamp(0.0, 0.45);

    // Slow inward sweep from edges with small tempo drift.
    let t = ctx.scene_elapsed_ms as f32 / 1000.0;
    let tempo_jitter = 0.88 + 0.28 * rand01(71, 9, frame / 14);
    let front = ((t * (0.20 + 0.35 * speed) * tempo_jitter) % 1.0) * 0.5; // 0=edge -> 0.5=center
    let band = (0.018 + 0.050 * thickness).clamp(0.01, 0.09);
    let shift = ((0.5 + 1.5 * intensity) / 2.0).round() as i32; // tiny right shift
    let chroma = ((1.0 + intensity * 1.6) / 2.5).round() as i32;

    for y in 0..src.height {
        for x in 0..src.width {
            let Some(base) = src.get(x, y).cloned() else {
                continue;
            };

            // Ruby tint + subtle center darkening (classic tube non-uniformity).
            let (nx, ny) = normalized_coords(x, y, src.width, src.height);
            let radius = ((nx * nx + ny * ny).sqrt() / std::f32::consts::SQRT_2).clamp(0.0, 1.0);
            let center_weight = (1.0 - radius).powf(1.35);
            let center_dark = (1.0 - center_weight * (0.05 + 0.11 * intensity)).clamp(0.78, 1.0);

            let mut fg = scale_colour(
                lerp_colour_local(base.fg, ruby, tint),
                center_dark * brightness,
            );
            let mut bg = scale_colour(
                lerp_colour_local(normalize_bg(base.bg), ruby_bg, tint * 0.55),
                center_dark,
            );
            let mut symbol = base.symbol;

            // Edge-reveal band: content "arrives" from edges toward center with tiny right glitch.
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
            let edge_dist = xn.min(1.0 - xn).min(yn.min(1.0 - yn)); // 0..0.5
            let band_dist = (edge_dist - front).abs();
            if band_dist <= band {
                let xi = x as i32;
                let sx = (xi - shift).clamp(0, src.width as i32 - 1) as u16;
                let sx_r = (xi - shift).clamp(0, src.width as i32 - 1) as u16;
                let sx_g = (xi - shift + chroma / 2).clamp(0, src.width as i32 - 1) as u16;
                let sx_b = (xi - shift + chroma).clamp(0, src.width as i32 - 1) as u16;
                let row = y;

                let Some(sample) = src.get(sx, row).cloned() else {
                    dst.set(x, y, symbol, fg, bg);
                    continue;
                };

                // When band hits empty cells, pull-in nearby glyph to mimic edge reveal.
                if symbol == ' ' && sample.symbol != ' ' {
                    symbol = sample.symbol;
                }

                let (rr, _, _) =
                    colour_to_rgb(src.get(sx_r, row).map(|c| c.fg).unwrap_or(sample.fg));
                let (_, gg, _) =
                    colour_to_rgb(src.get(sx_g, row).map(|c| c.fg).unwrap_or(sample.fg));
                let (_, _, bb) =
                    colour_to_rgb(src.get(sx_b, row).map(|c| c.fg).unwrap_or(sample.fg));
                let chroma_fg = Color::Rgb {
                    r: rr,
                    g: gg,
                    b: bb,
                };

                let local = 1.0 - (band_dist / band).clamp(0.0, 1.0);
                let reveal_blend = (0.12 + 0.30 * local * intensity).clamp(0.0, 0.45);
                let reveal_bright = 1.0 + 0.18 * local;
                fg = lerp_colour_local(fg, scale_colour(chroma_fg, reveal_bright), reveal_blend);
                bg = lerp_colour_local(
                    bg,
                    normalize_bg(sample.bg),
                    (reveal_blend * 0.40).clamp(0.0, 0.20),
                );
            }

            dst.set(x, y, symbol, fg, bg);
        }
    }
}

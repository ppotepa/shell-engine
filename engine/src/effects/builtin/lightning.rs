use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::effect::{Effect, Region};
use crate::effects::utils::color::{colour_to_rgb, lerp_colour};
use crate::effects::utils::noise::crt_hash;
use crate::scene::EffectParams;
use crossterm::style::Color;
use std::f32::consts::TAU;

fn n01(x: u16, y: u16, frame: u32) -> f32 {
    crt_hash(x, y, frame) as f32 / u32::MAX as f32
}

fn parse_anchor(anchor: Option<&str>, w: u16, seed: u32) -> f32 {
    if w == 0 {
        return 0.0;
    }
    let max_x = (w - 1) as f32;
    match anchor.map(|s| s.trim().to_ascii_lowercase()) {
        Some(v) if v == "random" => (seed as f32 / u32::MAX as f32) * max_x,
        Some(v) => v
            .parse::<f32>()
            .map(|x| x.clamp(0.0, max_x))
            .unwrap_or(max_x * 0.5),
        None => max_x * 0.5,
    }
}

fn brighten_cell(buffer: &mut Buffer, x: u16, y: u16, amount: f32) {
    if let Some(cell) = buffer.get(x, y).cloned() {
        let a = amount.clamp(0.0, 1.0);
        let fg = lerp_colour(cell.fg, Color::White, a);
        let bg = lerp_colour(cell.bg, Color::White, (a * 0.85).clamp(0.0, 1.0));
        buffer.set(x, y, cell.symbol, fg, bg);
    }
}

fn bolt_symbol(intensity: f32) -> char {
    let v = intensity.clamp(0.0, 1.0);
    if v > 0.82 {
        '█'
    } else if v > 0.52 {
        '▓'
    } else if v > 0.25 {
        '▒'
    } else {
        '░'
    }
}

fn apply_glow(buffer: &mut Buffer, region: Region, x: u16, y: u16, decay: f32) {
    for gy in y.saturating_sub(1)..=y.saturating_add(1) {
        if gy < region.y || gy >= region.y + region.height {
            continue;
        }
        for gx in x.saturating_sub(1)..=x.saturating_add(1) {
            if gx < region.x || gx >= region.x + region.width {
                continue;
            }
            if let Some(cell) = buffer.get(gx, gy).cloned() {
                let fg = lerp_colour(cell.fg, Color::White, 0.28 * decay);
                let sym = if cell.symbol == ' ' {
                    '░'
                } else {
                    cell.symbol
                };
                buffer.set(gx, gy, sym, fg, cell.bg);
            }
        }
    }
}

/// Short burst flash with multi-pulse behavior (closer to natural lightning).
pub struct LightningFlashEffect;

impl Effect for LightningFlashEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let p = progress.clamp(0.0, 1.0);
        let intensity = params.intensity.unwrap_or(2.4).max(0.0);
        let frame = (p * 10_000.0) as u32;

        let pulse1 = (-((p - 0.16) * 14.0).powi(2)).exp();
        let pulse2 = (-((p - 0.44) * 17.0).powi(2)).exp() * 0.78;
        let pulse3 = (-((p - 0.79) * 22.0).powi(2)).exp() * 0.55;
        let envelope = (pulse1 + pulse2 + pulse3).clamp(0.0, 1.0);

        for dy in 0..region.height {
            let y = region.y + dy;
            for dx in 0..region.width {
                let x = region.x + dx;
                let grain = 0.82 + n01(x, y, frame) * 0.32;
                let row_bias = 0.88 + (1.0 - dy as f32 / region.height as f32) * 0.18;
                let amount = (envelope * intensity * grain * row_bias).clamp(0.0, 1.0);
                brighten_cell(buffer, x, y, amount);
            }
        }
    }
}

/// Draws jagged lightning bolts with small branches and optional glow.
pub struct LightningBranchEffect;

impl Effect for LightningBranchEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let p = progress.clamp(0.0, 1.0);
        let frame = (p * 10_000.0) as u32;
        let strikes = params.strikes.unwrap_or(2).clamp(1, 6) as usize;
        let thickness = params.thickness.unwrap_or(1.0).clamp(1.0, 3.0);
        let glow = params.glow.unwrap_or(true);
        let decay = (1.0 - p * 0.45).clamp(0.25, 1.0);

        let h = region.height.max(1);
        let w = region.width.max(1);
        for s in 0..strikes {
            let seed_base = frame.wrapping_add((s as u32).wrapping_mul(101_903));
            let start = parse_anchor(
                params.start_x.as_deref(),
                w,
                crt_hash(region.x.wrapping_add(s as u16), region.y, seed_base),
            );
            let end = parse_anchor(
                params.end_x.as_deref(),
                w,
                crt_hash(
                    region.x,
                    region.y.wrapping_add(s as u16),
                    seed_base.wrapping_add(17),
                ),
            );

            let mut x = start;
            for iy in 0..h {
                let y = region.y + iy;
                let t = iy as f32 / (h.saturating_sub(1).max(1) as f32);
                let target = start + (end - start) * t;
                let jitter = (n01(region.x + (s as u16), y, seed_base) - 0.5) * 2.0 * thickness;
                x += (target - x) * 0.38 + jitter * 0.85;
                x = x.clamp(0.0, (w - 1) as f32);

                let abs_x = region.x + x.round() as u16;
                let core_intensity =
                    (0.82 + 0.18 * n01(abs_x, y, seed_base.wrapping_add(333))) * decay;

                let mut bolt_fg = Color::White;
                if let Some(cell) = buffer.get(abs_x, y) {
                    let (r, g, b) = colour_to_rgb(cell.fg);
                    let r = r.saturating_add((110.0 * decay) as u8);
                    let g = g.saturating_add((110.0 * decay) as u8);
                    let b = b.saturating_add((120.0 * decay) as u8);
                    bolt_fg = Color::Rgb { r, g, b };
                }
                let radius = thickness.round() as i32 - 1;
                for tx in -radius..=radius {
                    let xx = abs_x as i32 + tx;
                    if xx < region.x as i32 || xx >= (region.x + region.width) as i32 {
                        continue;
                    }
                    let x_cell = xx as u16;
                    let local_intensity =
                        (core_intensity - (tx.abs() as f32 * 0.22)).clamp(0.08, 1.0);
                    buffer.set(x_cell, y, bolt_symbol(local_intensity), bolt_fg, TRUE_BLACK);
                }

                if glow {
                    apply_glow(buffer, region, abs_x, y, decay);
                }

                // small side branch
                let branch_n = n01(abs_x, y, seed_base.wrapping_add(911));
                if branch_n > 0.92 && iy > 2 && iy < h - 2 {
                    let dir: i32 = if branch_n > 0.96 { 1 } else { -1 };
                    let len = 2 + ((branch_n * 4.0) as i32);
                    for bi in 1..=len {
                        let bx_i = abs_x as i32 + dir * bi;
                        let by_i = y as i32 + (bi / 2);
                        if bx_i < region.x as i32
                            || bx_i >= (region.x + region.width) as i32
                            || by_i < region.y as i32
                            || by_i >= (region.y + region.height) as i32
                        {
                            break;
                        }
                        let bx = bx_i as u16;
                        let by = by_i as u16;
                        let bi_decay = (decay - (bi as f32 * 0.08)).clamp(0.15, 1.0);
                        buffer.set(bx, by, bolt_symbol(bi_decay), Color::White, TRUE_BLACK);
                    }
                }
            }
        }
    }
}

/// Renders a spherical "tesla coil" electrical burst around the center.
pub struct TeslaOrbEffect;

impl Effect for TeslaOrbEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }
        let p = progress.clamp(0.0, 1.0);
        let frame = (p * 12_000.0) as u32;
        let strikes = params.strikes.unwrap_or(12).clamp(4, 28) as usize;
        let thickness = params.thickness.unwrap_or(1.5).clamp(1.0, 3.0);
        let glow = params.glow.unwrap_or(true);
        let intensity = params.intensity.unwrap_or(1.0).clamp(0.2, 2.4);

        let cx = region.x as f32 + region.width as f32 * 0.5;
        let cy = region.y as f32 + region.height as f32 * 0.5;
        let min_dim = region.width.min(region.height).max(8) as f32;
        let orb_radius = (min_dim * 0.11 + thickness).clamp(2.0, 8.0);
        let arc_radius = (orb_radius * 2.8).clamp(8.0, min_dim * 0.42);

        let pulse_a = (-((p - 0.14) * 15.0).powi(2)).exp();
        let pulse_b = (-((p - 0.47) * 20.0).powi(2)).exp() * 0.85;
        let pulse_c = (-((p - 0.82) * 24.0).powi(2)).exp() * 0.62;
        let envelope =
            ((0.28 + pulse_a + pulse_b + pulse_c).clamp(0.0, 1.0) * intensity).clamp(0.0, 1.0);

        let orb_i32 = orb_radius.ceil() as i32;
        for oy in -orb_i32..=orb_i32 {
            for ox in -orb_i32..=orb_i32 {
                let x = cx + ox as f32;
                let y = cy + oy as f32;
                let dist = ((ox * ox + oy * oy) as f32).sqrt();
                if dist > orb_radius {
                    continue;
                }
                let xi = x.round() as i32;
                let yi = y.round() as i32;
                if xi < region.x as i32
                    || yi < region.y as i32
                    || xi >= (region.x + region.width) as i32
                    || yi >= (region.y + region.height) as i32
                {
                    continue;
                }
                let xx = xi as u16;
                let yy = yi as u16;
                let core = ((1.0 - dist / orb_radius).clamp(0.0, 1.0) * 0.85 + 0.15) * envelope;
                let sym = bolt_symbol(core);
                let mut fg = Color::White;
                if let Some(cell) = buffer.get(xx, yy) {
                    let (r, g, b) = colour_to_rgb(cell.fg);
                    fg = Color::Rgb {
                        r: r.saturating_add((120.0 * core) as u8),
                        g: g.saturating_add((120.0 * core) as u8),
                        b: b.saturating_add((140.0 * core) as u8),
                    };
                }
                buffer.set(xx, yy, sym, fg, TRUE_BLACK);
                if glow {
                    apply_glow(buffer, region, xx, yy, envelope);
                }
            }
        }

        for s in 0..strikes {
            let seed = frame.wrapping_add((s as u32).wrapping_mul(81_329));
            let angle = n01(region.x.wrapping_add(s as u16), region.y, seed) * TAU;
            let spread = (n01(
                region.x,
                region.y.wrapping_add(s as u16),
                seed.wrapping_add(17),
            ) - 0.5)
                * 1.5;
            let start_r =
                orb_radius * (0.75 + n01(region.x, region.y, seed.wrapping_add(33)) * 0.3);
            let end_r = arc_radius * (0.82 + n01(region.x, region.y, seed.wrapping_add(61)) * 0.38);
            let end_angle = angle + spread + (p * 2.6 * if s % 2 == 0 { 1.0 } else { -1.0 });

            let start_x = cx + angle.cos() * start_r;
            let start_y = cy + angle.sin() * start_r;
            let end_x = cx + end_angle.cos() * end_r;
            let end_y = cy + end_angle.sin() * end_r;

            let dx = end_x - start_x;
            let dy = end_y - start_y;
            let base_steps = dx.abs().max(dy.abs()).ceil() as i32;
            let steps = base_steps.max(8) as usize;

            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let jitter_seed = seed.wrapping_add((i as u32).wrapping_mul(977));
                let jitter = (n01(region.x, region.y, jitter_seed) - 0.5) * 2.0;
                let side = (1.0 - (t * 2.0 - 1.0).abs()).max(0.0);
                let jx = jitter * thickness * (0.25 + side);
                let jy = jitter * thickness * (0.18 + side * 0.8);

                let x = start_x + dx * t + jx;
                let y = start_y + dy * t + jy;
                let xi = x.round() as i32;
                let yi = y.round() as i32;
                if xi < region.x as i32
                    || yi < region.y as i32
                    || xi >= (region.x + region.width) as i32
                    || yi >= (region.y + region.height) as i32
                {
                    continue;
                }
                let xx = xi as u16;
                let yy = yi as u16;
                let local = (envelope * (1.0 - t * 0.4) * (0.86 + n01(xx, yy, seed) * 0.14))
                    .clamp(0.0, 1.0);
                let sym = bolt_symbol(local);
                let mut fg = Color::White;
                if let Some(cell) = buffer.get(xx, yy) {
                    let (r, g, b) = colour_to_rgb(cell.fg);
                    fg = Color::Rgb {
                        r: r.saturating_add((100.0 * local) as u8),
                        g: g.saturating_add((110.0 * local) as u8),
                        b: b.saturating_add((130.0 * local) as u8),
                    };
                }
                buffer.set(xx, yy, sym, fg, TRUE_BLACK);
                if glow {
                    apply_glow(buffer, region, xx, yy, local);
                }
            }
        }
    }
}

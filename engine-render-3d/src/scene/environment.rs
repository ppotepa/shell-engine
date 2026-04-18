use engine_core::buffer::{Buffer, PixelCanvas};
use engine_core::color::Color;
use engine_core::scene::ResolvedViewProfile;

pub fn render_space_environment(buffer: &mut Buffer, view: &ResolvedViewProfile) {
    render_starfield(buffer, view);
    render_primary_star_glare(buffer, view);
}

fn render_starfield(buffer: &mut Buffer, view: &ResolvedViewProfile) {
    let env = &view.environment;
    let density = env.starfield_density.unwrap_or(0.0).clamp(0.0, 1.0);
    let brightness = env.starfield_brightness.unwrap_or(0.0).clamp(0.0, 1.5);
    if density <= 0.0 || brightness <= 0.0 || buffer.width == 0 || buffer.height == 0 {
        return;
    }

    let area = buffer.width as usize * buffer.height as usize;
    let star_count = ((area as f32 / 180.0) * density).round() as usize;
    if star_count == 0 {
        return;
    }

    let size_min = env.starfield_size_min.unwrap_or(1.0).clamp(0.5, 3.0);
    let size_max = env
        .starfield_size_max
        .unwrap_or(size_min.max(1.0))
        .clamp(size_min, 4.0);
    let (r, g, b) = star_rgb(brightness);
    let star_color = Color::rgb(r, g, b);
    let mut seed = starfield_seed(buffer.width, buffer.height, density, brightness);

    if let Some(canvas) = &mut buffer.pixel_canvas {
        for _ in 0..star_count {
            let x = (next_u32(&mut seed) % canvas.width as u32) as u16;
            let y = (next_u32(&mut seed) % canvas.height as u32) as u16;
            let size = lerp_size(
                size_min,
                size_max,
                next_u32(&mut seed) as f32 / u32::MAX as f32,
            );
            draw_star_pixels(canvas, x, y, size, r, g, b);
        }
        return;
    }

    for _ in 0..star_count {
        let x = (next_u32(&mut seed) % buffer.width as u32) as u16;
        let y = (next_u32(&mut seed) % buffer.height as u32) as u16;
        let size = lerp_size(
            size_min,
            size_max,
            next_u32(&mut seed) as f32 / u32::MAX as f32,
        );
        let glyph = if size >= 1.6 { '*' } else { '.' };
        buffer.set(x, y, glyph, star_color, Color::BLACK);
    }
}

fn render_primary_star_glare(buffer: &mut Buffer, view: &ResolvedViewProfile) {
    let env = &view.environment;
    let strength = env
        .primary_star_glare_strength
        .unwrap_or(0.0)
        .clamp(0.0, 1.5);
    if strength <= 0.0 || buffer.width == 0 || buffer.height == 0 {
        return;
    }
    let width = env
        .primary_star_glare_width
        .unwrap_or(0.18)
        .clamp(0.02, 1.0);
    let (r, g, b) = parse_hex_rgb(env.primary_star_color.as_deref().unwrap_or("#fff4d6"))
        .unwrap_or((255, 244, 214));

    if let Some(canvas) = &mut buffer.pixel_canvas {
        render_primary_star_glare_pixels(canvas, strength, width, r, g, b);
        return;
    }
    render_primary_star_glare_cells(buffer, strength, width, r, g, b);
}

fn star_rgb(brightness: f32) -> (u8, u8, u8) {
    let value = (180.0 + 75.0 * brightness.clamp(0.0, 1.0)).round() as u8;
    (value, value, (value as f32 * 0.98).round() as u8)
}

fn starfield_seed(width: u16, height: u16, density: f32, brightness: f32) -> u64 {
    let mut seed = 0xcbf29ce484222325_u64;
    seed ^= width as u64;
    seed = seed.wrapping_mul(0x100000001b3);
    seed ^= height as u64;
    seed = seed.wrapping_mul(0x100000001b3);
    seed ^= density.to_bits() as u64;
    seed = seed.wrapping_mul(0x100000001b3);
    seed ^= brightness.to_bits() as u64;
    seed
}

fn next_u32(seed: &mut u64) -> u32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*seed >> 32) as u32
}

fn lerp_size(min: f32, max: f32, t: f32) -> f32 {
    min + (max - min) * t.clamp(0.0, 1.0)
}

fn parse_hex_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.trim().strip_prefix('#').unwrap_or(value.trim());
    if hex.len() != 6 {
        return None;
    }
    Some((
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}

fn glare_curve(t: f32, strength: f32) -> f32 {
    let core = t.clamp(0.0, 1.0).powf(2.2);
    (core * strength * 0.7).clamp(0.0, 1.0)
}

fn blend_channel(base: u8, tint: u8, amount: f32) -> u8 {
    (base as f32 + tint as f32 * amount.clamp(0.0, 1.0))
        .clamp(0.0, 255.0)
        .round() as u8
}

fn render_primary_star_glare_pixels(
    canvas: &mut PixelCanvas,
    strength: f32,
    width: f32,
    r: u8,
    g: u8,
    b: u8,
) {
    let cw = canvas.width as f32;
    let ch = canvas.height as f32;
    let cx = -cw * 0.18;
    let cy = ch * 0.16;
    let radius = cw.max(ch) * (0.22 + width * 0.48);
    let radius_sq = radius * radius;
    let max_x = (cx + radius).ceil().clamp(0.0, cw - 1.0) as u16;
    let min_y = (cy - radius).floor().clamp(0.0, ch - 1.0) as u16;
    let max_y = (cy + radius).ceil().clamp(0.0, ch - 1.0) as u16;

    for y in min_y..=max_y {
        let py = y as f32 + 0.5;
        for x in 0..=max_x {
            let px = x as f32 + 0.5;
            let dx = px - cx;
            let dy = py - cy;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq >= radius_sq {
                continue;
            }
            let glow = glare_curve(1.0 - (dist_sq / radius_sq).sqrt(), strength);
            if glow <= 0.001 {
                continue;
            }
            let idx = (y as usize * canvas.width as usize + x as usize) * 4;
            canvas.data[idx] = blend_channel(canvas.data[idx], r, glow);
            canvas.data[idx + 1] = blend_channel(canvas.data[idx + 1], g, glow);
            canvas.data[idx + 2] = blend_channel(canvas.data[idx + 2], b, glow);
            canvas.data[idx + 3] = 255;
        }
    }
    canvas.dirty = true;
}

fn render_primary_star_glare_cells(
    buffer: &mut Buffer,
    strength: f32,
    width: f32,
    r: u8,
    g: u8,
    b: u8,
) {
    let bw = buffer.width as f32;
    let bh = buffer.height as f32;
    let cx = -bw * 0.18;
    let cy = bh * 0.16;
    let radius = bw.max(bh) * (0.22 + width * 0.48);
    let radius_sq = radius * radius;
    let max_x = (cx + radius).ceil().clamp(0.0, bw - 1.0) as u16;
    let min_y = (cy - radius).floor().clamp(0.0, bh - 1.0) as u16;
    let max_y = (cy + radius).ceil().clamp(0.0, bh - 1.0) as u16;
    let stride = buffer.width as usize;
    let cells = buffer.back_cells_mut();

    for y in min_y..=max_y {
        let py = y as f32 + 0.5;
        for x in 0..=max_x {
            let px = x as f32 + 0.5;
            let dx = px - cx;
            let dy = py - cy;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq >= radius_sq {
                continue;
            }
            let glow = glare_curve(1.0 - (dist_sq / radius_sq).sqrt(), strength);
            if glow <= 0.001 {
                continue;
            }
            let cell = &mut cells[y as usize * stride + x as usize];
            let (bg_r, bg_g, bg_b) = cell.bg.to_rgb();
            cell.bg = Color::rgb(
                blend_channel(bg_r, r, glow),
                blend_channel(bg_g, g, glow),
                blend_channel(bg_b, b, glow),
            );
            if cell.symbol != ' ' {
                let (fg_r, fg_g, fg_b) = cell.fg.to_rgb();
                cell.fg = Color::rgb(
                    blend_channel(fg_r, r, glow * 0.7),
                    blend_channel(fg_g, g, glow * 0.7),
                    blend_channel(fg_b, b, glow * 0.7),
                );
            }
        }
    }
    buffer.mark_all_dirty();
}

fn draw_star_pixels(canvas: &mut PixelCanvas, x: u16, y: u16, size: f32, r: u8, g: u8, b: u8) {
    canvas.set_pixel(x, y, r, g, b);
    if size < 1.6 {
        return;
    }
    if x > 0 {
        canvas.set_pixel(x - 1, y, r, g, b);
    }
    if x + 1 < canvas.width {
        canvas.set_pixel(x + 1, y, r, g, b);
    }
    if y > 0 {
        canvas.set_pixel(x, y - 1, r, g, b);
    }
    if y + 1 < canvas.height {
        canvas.set_pixel(x, y + 1, r, g, b);
    }
}

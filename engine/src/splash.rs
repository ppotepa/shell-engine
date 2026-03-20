//! Engine splash screen — shown once before the mod game loop starts.
//!
//! Displays the embedded Cognitos logo centered on the terminal. The splash
//! always starts on a white background, then fades out to the entry scene
//! background colour.

use std::io::{self, Write};
use std::time::Duration;

use crossterm::{cursor, queue, style, terminal};
use image::load_from_memory;

const SPLASH_PNG: &[u8] = include_bytes!("../assets/cognitos_splash.png");
const ALPHA_THRESHOLD: u8 = 16;

const HOLD_MS: u64 = 900;
const FADE_MS: u64 = 600;
const FADE_STEPS: u16 = 12;

/// Display the Cognitos engine logo centered on the terminal.
///
/// Must be called after the alternate screen has been entered and the console
/// cleared. This blocks the calling thread for ~1.5s.
pub fn show_splash(target_bg: style::Color) {
    if let Err(e) = try_show_splash(target_bg) {
        // Non-fatal — if splash fails, skip silently.
        crate::logging::warn("engine.splash", format!("splash display skipped: {e}"));
    }
}

fn try_show_splash(target_bg: style::Color) -> io::Result<()> {
    let img = load_from_memory(SPLASH_PNG)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .to_rgba8();

    let (term_w, term_h) = terminal::size()?;
    let (render_cols, render_rows) = fit_logo(img.width(), img.height(), term_w, term_h);

    // Fill full screen white regardless of logo fit (requirement: white background).
    let white = style::Color::White;
    let mut stdout = io::stdout();
    fill_solid(&mut stdout, term_w, term_h, white)?;

    if render_cols == 0 || render_rows == 0 {
        stdout.flush()?;
        std::thread::sleep(Duration::from_millis(HOLD_MS));
        fade_to_bg(&mut stdout, term_w, term_h, white, target_bg)?;
        return Ok(());
    }

    let origin_x = term_w.saturating_sub(render_cols) / 2;
    let origin_y = term_h.saturating_sub(render_rows) / 2;

    draw_logo(
        &mut stdout,
        &img,
        origin_x,
        origin_y,
        render_cols,
        render_rows,
        white,
        1.0,
    )?;
    stdout.flush()?;

    std::thread::sleep(Duration::from_millis(HOLD_MS));

    // Fade out: both the background (white -> target) and the logo (alpha -> 0).
    for step in 0..=FADE_STEPS {
        let t = step as f32 / FADE_STEPS.max(1) as f32;
        let bg = lerp_colour(white, target_bg, t);
        let fade = (1.0 - t).clamp(0.0, 1.0);

        fill_solid(&mut stdout, term_w, term_h, bg)?;
        draw_logo(
            &mut stdout,
            &img,
            origin_x,
            origin_y,
            render_cols,
            render_rows,
            bg,
            fade,
        )?;
        stdout.flush()?;

        let frame_ms = (FADE_MS / FADE_STEPS.max(1) as u64).max(1);
        std::thread::sleep(Duration::from_millis(frame_ms));
    }

    Ok(())
}

fn fade_to_bg(
    stdout: &mut io::Stdout,
    term_w: u16,
    term_h: u16,
    from: style::Color,
    to: style::Color,
) -> io::Result<()> {
    for step in 0..=FADE_STEPS {
        let t = step as f32 / FADE_STEPS.max(1) as f32;
        let bg = lerp_colour(from, to, t);
        fill_solid(stdout, term_w, term_h, bg)?;
        stdout.flush()?;

        let frame_ms = (FADE_MS / FADE_STEPS.max(1) as u64).max(1);
        std::thread::sleep(Duration::from_millis(frame_ms));
    }
    Ok(())
}

fn fill_solid(stdout: &mut io::Stdout, w: u16, h: u16, bg: style::Color) -> io::Result<()> {
    queue!(
        stdout,
        style::SetForegroundColor(bg),
        style::SetBackgroundColor(bg)
    )?;
    for y in 0..h {
        queue!(stdout, cursor::MoveTo(0, y))?;
        for _ in 0..w {
            queue!(stdout, style::Print(' '))?;
        }
    }
    Ok(())
}

fn draw_logo(
    stdout: &mut io::Stdout,
    img: &image::RgbaImage,
    origin_x: u16,
    origin_y: u16,
    render_cols: u16,
    render_rows: u16,
    bg: style::Color,
    fade: f32,
) -> io::Result<()> {
    let virtual_h = render_rows as u32 * 2;
    let bg_rgb = to_rgb(bg);

    for row in 0..render_rows {
        for col in 0..render_cols {
            let top = sample(img, col as u32, row as u32 * 2, render_cols as u32, virtual_h);
            let bot = sample(
                img,
                col as u32,
                row as u32 * 2 + 1,
                render_cols as u32,
                virtual_h,
            );

            let (sym, fg, cell_bg) = match render_halfblock_cell(top, bot, bg_rgb, fade) {
                Some(v) => v,
                None => continue,
            };

            queue!(
                stdout,
                cursor::MoveTo(origin_x + col, origin_y + row),
                style::SetForegroundColor(fg),
                style::SetBackgroundColor(cell_bg),
                style::Print(sym),
            )?;
        }
    }

    Ok(())
}

fn render_halfblock_cell(
    top: [u8; 4],
    bot: [u8; 4],
    bg: (u8, u8, u8),
    fade: f32,
) -> Option<(char, style::Color, style::Color)> {
    let (top_on, top_rgb) = composite_pixel(top, bg, fade);
    let (bot_on, bot_rgb) = composite_pixel(bot, bg, fade);

    match (top_on, bot_on) {
        (false, false) => None,
        (true, false) => Some(('▀', rgb(top_rgb), rgb(bg))),
        (false, true) => Some(('▄', rgb(bot_rgb), rgb(bg))),
        (true, true) => Some(('▀', rgb(top_rgb), rgb(bot_rgb))),
    }
}

fn composite_pixel(pixel: [u8; 4], bg: (u8, u8, u8), fade: f32) -> (bool, (u8, u8, u8)) {
    let a = (pixel[3] as f32 * fade).clamp(0.0, 255.0);
    if a < ALPHA_THRESHOLD as f32 {
        return (false, bg);
    }
    let alpha = (a / 255.0).clamp(0.0, 1.0);
    let src = (pixel[0], pixel[1], pixel[2]);
    (true, blend(bg, src, alpha))
}

fn blend(bg: (u8, u8, u8), fg: (u8, u8, u8), alpha: f32) -> (u8, u8, u8) {
    let inv = 1.0 - alpha;
    (
        (bg.0 as f32 * inv + fg.0 as f32 * alpha).round().clamp(0.0, 255.0) as u8,
        (bg.1 as f32 * inv + fg.1 as f32 * alpha).round().clamp(0.0, 255.0) as u8,
        (bg.2 as f32 * inv + fg.2 as f32 * alpha).round().clamp(0.0, 255.0) as u8,
    )
}

fn lerp_colour(from: style::Color, to: style::Color, t: f32) -> style::Color {
    let t = t.clamp(0.0, 1.0);
    let a = to_rgb(from);
    let b = to_rgb(to);
    rgb(blend(a, b, t))
}

fn rgb((r, g, b): (u8, u8, u8)) -> style::Color {
    style::Color::Rgb { r, g, b }
}

fn to_rgb(c: style::Color) -> (u8, u8, u8) {
    use style::Color;
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::DarkGrey => (85, 85, 85),
        Color::Red => (255, 0, 0),
        Color::DarkRed => (128, 0, 0),
        Color::Green => (0, 255, 0),
        Color::DarkGreen => (0, 128, 0),
        Color::Yellow => (255, 255, 0),
        Color::DarkYellow => (128, 128, 0),
        Color::Blue => (0, 0, 255),
        Color::DarkBlue => (0, 0, 128),
        Color::Magenta => (255, 0, 255),
        Color::DarkMagenta => (128, 0, 128),
        Color::Cyan => (0, 255, 255),
        Color::DarkCyan => (0, 128, 128),
        Color::White => (255, 255, 255),
        Color::Grey => (192, 192, 192),
        Color::AnsiValue(_) | Color::Reset => (0, 0, 0),
    }
}

/// Compute (render_cols, render_rows) that fit the logo into the terminal,
/// preserving aspect ratio and leaving a 1-cell margin.
fn fit_logo(img_w: u32, img_h: u32, term_w: u16, term_h: u16) -> (u16, u16) {
    let max_cols = term_w.saturating_sub(2) as u32;
    let max_rows = term_h.saturating_sub(2) as u32;
    if max_cols == 0 || max_rows == 0 || img_w == 0 || img_h == 0 {
        return (0, 0);
    }
    // halfblock: each terminal row covers 2 pixel rows
    let pixel_rows = img_h;
    let cols_by_w = max_cols;
    let rows_by_w = (pixel_rows * cols_by_w / img_w / 2).max(1);
    let rows_by_h = max_rows;
    let cols_by_h = (img_w * rows_by_h * 2 / pixel_rows).max(1);

    let (cols, rows) = if rows_by_w <= max_rows {
        (cols_by_w, rows_by_w)
    } else {
        (cols_by_h.min(max_cols), rows_by_h)
    };
    (cols.min(u16::MAX as u32) as u16, rows.min(u16::MAX as u32) as u16)
}

fn sample(
    img: &image::RgbaImage,
    col: u32,
    pixel_row: u32,
    cols: u32,
    pixel_rows: u32,
) -> [u8; 4] {
    let px = (col * img.width()) / cols.max(1);
    let py = (pixel_row * img.height()) / pixel_rows.max(1);
    let px = px.min(img.width().saturating_sub(1));
    let py = py.min(img.height().saturating_sub(1));
    img.get_pixel(px, py).0
}

//! Engine splash screen — shown once before the mod game loop starts.
//!
//! Displays the embedded Cognitos logo centered on the terminal, using
//! halfblock pixel rendering for colour fidelity. Visible for ~1.5 s.

use std::io::{self, Write};
use std::time::Duration;

use crossterm::{cursor, execute, queue, style, terminal};
use image::load_from_memory;

const SPLASH_PNG: &[u8] = include_bytes!("../assets/cognitos_splash.png");
const ALPHA_THRESHOLD: u8 = 16;
const DISPLAY_MS: u64 = 1500;

/// Display the Cognitos engine logo centered on the terminal for a brief moment.
///
/// Must be called after the alternate screen has been entered and the console cleared.
/// Blocks the calling thread for [`DISPLAY_MS`] milliseconds.
pub fn show_splash() {
    if let Err(e) = try_show_splash() {
        // Non-fatal — if splash fails, skip silently.
        crate::logging::warn("engine.splash", format!("splash display skipped: {e}"));
    }
}

fn try_show_splash() -> io::Result<()> {
    let img = load_from_memory(SPLASH_PNG)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .to_rgba8();

    let (term_w, term_h) = terminal::size()?;
    let (render_cols, render_rows) = fit_logo(img.width(), img.height(), term_w, term_h);

    if render_cols == 0 || render_rows == 0 {
        return Ok(());
    }

    let origin_x = term_w.saturating_sub(render_cols) / 2;
    let origin_y = term_h.saturating_sub(render_rows) / 2;

    let mut stdout = io::stdout();
    let virtual_h = render_rows as u32 * 2;

    for row in 0..render_rows {
        for col in 0..render_cols {
            let top = sample(&img, col as u32, row as u32 * 2, render_cols as u32, virtual_h);
            let bot = sample(&img, col as u32, row as u32 * 2 + 1, render_cols as u32, virtual_h);

            let top_on = top[3] >= ALPHA_THRESHOLD;
            let bot_on = bot[3] >= ALPHA_THRESHOLD;

            if !top_on && !bot_on {
                continue;
            }

            let (sym, fg, bg) = match (top_on, bot_on) {
                (true, false) => ('▀', to_color(top), style::Color::Black),
                (false, true) => ('▄', to_color(bot), style::Color::Black),
                (true, true) => ('▀', to_color(top), to_color(bot)),
                _ => unreachable!(),
            };

            queue!(
                stdout,
                cursor::MoveTo(origin_x + col, origin_y + row),
                style::SetForegroundColor(fg),
                style::SetBackgroundColor(bg),
                style::Print(sym),
            )?;
        }
    }
    stdout.flush()?;

    std::thread::sleep(Duration::from_millis(DISPLAY_MS));

    // Clear splash: repaint black before handing off to the game loop.
    execute!(
        stdout,
        style::SetForegroundColor(style::Color::Black),
        style::SetBackgroundColor(style::Color::Black),
        terminal::Clear(terminal::ClearType::All),
    )
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

fn sample(img: &image::RgbaImage, col: u32, pixel_row: u32, cols: u32, pixel_rows: u32) -> [u8; 4] {
    let px = (col * img.width()) / cols.max(1);
    let py = (pixel_row * img.height()) / pixel_rows.max(1);
    let px = px.min(img.width().saturating_sub(1));
    let py = py.min(img.height().saturating_sub(1));
    img.get_pixel(px, py).0
}

fn to_color(pixel: [u8; 4]) -> style::Color {
    style::Color::Rgb {
        r: pixel[0],
        g: pixel[1],
        b: pixel[2],
    }
}

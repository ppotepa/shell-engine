use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use engine_core::buffer::Buffer;
use engine_events::{EngineEvent, KeyCode, KeyEvent, KeyModifiers};
use engine_render::OverlayData;
use engine_runtime::{PresentationPolicy, compute_presentation_layout};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::{Color as SdlColor, PixelFormatEnum};
use sdl2::rect::Rect;

use crate::renderer::{LOGICAL_CELL_HEIGHT, LOGICAL_CELL_WIDTH};

pub(crate) enum RuntimeCommand {
    Present(Buffer, Option<OverlayData>),
    PollInput,
    Clear,
    Shutdown,
}

pub(crate) enum RuntimeResponse {
    Ack,
    Input(Vec<EngineEvent>),
}

pub(crate) struct Sdl2RuntimeClient {
    command_tx: Sender<RuntimeCommand>,
    response_rx: Receiver<RuntimeResponse>,
}

impl Sdl2RuntimeClient {
    pub(crate) fn spawn(
        output_width: u16,
        output_height: u16,
        presentation_policy: PresentationPolicy,
        window_ratio: Option<(u32, u32)>,
        pixel_scale: u32,
        vsync: bool,
    ) -> Result<Self, String> {
        let (command_tx, command_rx) = channel();
        let (response_tx, response_rx) = channel();

        thread::Builder::new()
            .name("sdl2-output".to_string())
            .spawn(move || {
                runtime_thread(
                    output_width,
                    output_height,
                    presentation_policy,
                    window_ratio,
                    pixel_scale,
                    vsync,
                    command_rx,
                    response_tx,
                )
            })
            .map_err(|error| error.to_string())?;

        Ok(Self {
            command_tx,
            response_rx,
        })
    }

    #[cfg(test)]
    pub(crate) fn disconnected_for_tests() -> Self {
        let (command_tx, _command_rx) = channel();
        let (_response_tx, response_rx) = channel();
        Self {
            command_tx,
            response_rx,
        }
    }

    pub(crate) fn request(&mut self, command: RuntimeCommand) -> Result<RuntimeResponse, String> {
        self.command_tx
            .send(command)
            .map_err(|error| error.to_string())?;
        self.response_rx.recv().map_err(|error| error.to_string())
    }
}

fn runtime_thread(
    output_width: u16,
    output_height: u16,
    presentation_policy: PresentationPolicy,
    window_ratio: Option<(u32, u32)>,
    pixel_scale: u32,
    vsync: bool,
    command_rx: Receiver<RuntimeCommand>,
    response_tx: Sender<RuntimeResponse>,
) {
    let _ = sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "nearest");
    let Ok(sdl) = sdl2::init() else {
        return;
    };
    let Ok(video) = sdl.video() else {
        return;
    };
    let mut current_output_width = output_width.max(1);
    let mut current_output_height = output_height.max(1);
    let pixel_scale = pixel_scale.max(1);
    let mut content_pixel_size = logical_dimensions(current_output_width, current_output_height);
    let (window_width, window_height) =
        window_dimensions(output_width, output_height, pixel_scale, window_ratio);
    let Ok(window) = video
        .window("Shell Quest", window_width, window_height)
        .position_centered()
        .resizable()
        .build()
    else {
        return;
    };
    let mut canvas_builder = window.into_canvas();
    if vsync {
        canvas_builder = canvas_builder.present_vsync();
    }
    let Ok(mut canvas) = canvas_builder.build() else {
        return;
    };
    let texture_creator = canvas.texture_creator();
    let mut pixel_buffer: Vec<u8> =
        vec![0u8; (content_pixel_size.0 * content_pixel_size.1 * 4) as usize];
    let Ok(mut frame_texture) = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGBA32,
        content_pixel_size.0,
        content_pixel_size.1,
    ) else {
        return;
    };
    let Ok(mut event_pump) = sdl.event_pump() else {
        return;
    };
    let mut window_pixel_size = current_window_pixel_size(&canvas);
    let mut last_frame_hash: u64 = 0;

    if frame_texture
        .update(None, &pixel_buffer, content_pixel_size.0 as usize * 4)
        .is_err()
        || present_texture(
            &mut canvas,
            &frame_texture,
            content_pixel_size,
            presentation_policy,
        )
        .is_err()
    {
        return;
    }

    while let Ok(command) = command_rx.recv() {
        let response = match command {
            RuntimeCommand::Present(buffer, overlay) => {
                if buffer.width != current_output_width || buffer.height != current_output_height {
                    current_output_width = buffer.width.max(1);
                    current_output_height = buffer.height.max(1);
                    content_pixel_size =
                        logical_dimensions(current_output_width, current_output_height);
                    pixel_buffer
                        .resize((content_pixel_size.0 * content_pixel_size.1 * 4) as usize, 0);
                    let Ok(new_texture) = texture_creator.create_texture_streaming(
                        PixelFormatEnum::RGBA32,
                        content_pixel_size.0,
                        content_pixel_size.1,
                    ) else {
                        let _ = response_tx.send(RuntimeResponse::Ack);
                        break;
                    };
                    frame_texture = new_texture;
                    last_frame_hash = 0;
                }

                rasterize_to_pixels(&buffer, &mut pixel_buffer, content_pixel_size.0);

                let frame_hash = fnv1a_hash(&pixel_buffer);
                let is_static = frame_hash == last_frame_hash && overlay.is_none();
                if !is_static {
                    last_frame_hash = frame_hash;
                    if frame_texture
                        .update(None, &pixel_buffer, content_pixel_size.0 as usize * 4)
                        .is_err()
                    {
                        let _ = response_tx.send(RuntimeResponse::Ack);
                        break;
                    }

                    canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
                    canvas.clear();
                    let present_rect = presentation_rect(
                        current_window_pixel_size(&canvas),
                        content_pixel_size,
                        presentation_policy,
                    );
                    if canvas.copy(&frame_texture, None, Some(present_rect)).is_err() {
                        let _ = response_tx.send(RuntimeResponse::Ack);
                        break;
                    }

                    if let Some(ref overlay_data) = overlay {
                        if !overlay_data.is_empty() {
                            if overlay_data.dim_scene {
                                draw_scene_dim(&mut canvas);
                            }
                            draw_overlay(&mut canvas, overlay_data);
                        }
                    }

                    canvas.present();
                }

                RuntimeResponse::Ack
            }
            RuntimeCommand::PollInput => RuntimeResponse::Input(poll_input(
                &mut canvas,
                &frame_texture,
                &mut event_pump,
                current_output_width,
                current_output_height,
                content_pixel_size,
                presentation_policy,
                &mut window_pixel_size,
            )),
            RuntimeCommand::Clear => {
                pixel_buffer.fill(0);
                last_frame_hash = 0;
                if frame_texture
                    .update(None, &pixel_buffer, content_pixel_size.0 as usize * 4)
                    .is_err()
                    || present_texture(
                        &mut canvas,
                        &frame_texture,
                        content_pixel_size,
                        presentation_policy,
                    )
                    .is_err()
                {
                    let _ = response_tx.send(RuntimeResponse::Ack);
                    break;
                }
                RuntimeResponse::Ack
            }
            RuntimeCommand::Shutdown => {
                let _ = response_tx.send(RuntimeResponse::Ack);
                break;
            }
        };
        let _ = response_tx.send(response);
    }
}

/// Linearly blend `fg` over `bg` at `alpha` (0.0 = all bg, 1.0 = all fg).
/// NOTE: Inlined into rasterize_to_pixels for common alpha values (0.25, 0.5, 0.75).
#[allow(dead_code)]
#[inline(always)]
fn blend_rgb(fg: (u8, u8, u8), bg: (u8, u8, u8), alpha: f32) -> (u8, u8, u8) {
    let a = alpha.clamp(0.0, 1.0);
    let blend = |f: u8, b: u8| (b as f32 + (f as f32 - b as f32) * a).round() as u8;
    (blend(fg.0, bg.0), blend(fg.1, bg.1), blend(fg.2, bg.2))
}

fn rasterize_to_pixels(buffer: &Buffer, pixel_buffer: &mut Vec<u8>, logical_w: u32) {
    let lw = logical_w as usize;
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let Some(cell) = buffer.get(x, y) else {
                continue;
            };
            let fg = cell.fg.to_rgb();
            let bg = cell.bg.to_rgb();
            // Pre-select blend function to avoid repeated clamp in blend_rgb inner loop
            let (top, bot) = match cell.symbol {
                '▀' => (fg, bg),
                '▄' => (bg, fg),
                ' ' => (bg, bg),
                // Shade chars: blend fg over bg at 25/50/75/100%
                // Use hard-coded alpha values for better inlining
                '░' => {
                    let r = ((bg.0 as f32 * 0.75) + (fg.0 as f32 * 0.25)).round() as u8;
                    let g = ((bg.1 as f32 * 0.75) + (fg.1 as f32 * 0.25)).round() as u8;
                    let b = ((bg.2 as f32 * 0.75) + (fg.2 as f32 * 0.25)).round() as u8;
                    ((r, g, b), (r, g, b))
                }
                '▒' => {
                    let r = ((bg.0 as f32 * 0.5) + (fg.0 as f32 * 0.5)).round() as u8;
                    let g = ((bg.1 as f32 * 0.5) + (fg.1 as f32 * 0.5)).round() as u8;
                    let b = ((bg.2 as f32 * 0.5) + (fg.2 as f32 * 0.5)).round() as u8;
                    ((r, g, b), (r, g, b))
                }
                '▓' => {
                    let r = ((bg.0 as f32 * 0.25) + (fg.0 as f32 * 0.75)).round() as u8;
                    let g = ((bg.1 as f32 * 0.25) + (fg.1 as f32 * 0.75)).round() as u8;
                    let b = ((bg.2 as f32 * 0.25) + (fg.2 as f32 * 0.75)).round() as u8;
                    ((r, g, b), (r, g, b))
                }
                '█' => (fg, fg),
                _ => (fg, fg),
            };
            let px = x as usize;
            let py = y as usize * 2;
            let i0 = (py * lw + px) * 4;
            let i1 = ((py + 1) * lw + px) * 4;
            pixel_buffer[i0] = top.0;
            pixel_buffer[i0 + 1] = top.1;
            pixel_buffer[i0 + 2] = top.2;
            pixel_buffer[i0 + 3] = 255;
            pixel_buffer[i1] = bot.0;
            pixel_buffer[i1 + 1] = bot.1;
            pixel_buffer[i1 + 2] = bot.2;
            pixel_buffer[i1 + 3] = 255;
        }
    }
}

fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}
fn present_texture(
    canvas: &mut sdl2::render::WindowCanvas,
    texture: &sdl2::render::Texture<'_>,
    content_pixel_size: (u32, u32),
    presentation_policy: PresentationPolicy,
) -> Result<(), String> {
    canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
    canvas.clear();
    let present_rect = presentation_rect(
        current_window_pixel_size(canvas),
        content_pixel_size,
        presentation_policy,
    );
    canvas
        .copy(texture, None, Some(present_rect))
        .map_err(|error| error.to_string())?;
    canvas.present();
    Ok(())
}

fn poll_input(
    canvas: &mut sdl2::render::WindowCanvas,
    frame_texture: &sdl2::render::Texture<'_>,
    event_pump: &mut sdl2::EventPump,
    output_width: u16,
    output_height: u16,
    content_pixel_size: (u32, u32),
    presentation_policy: PresentationPolicy,
    window_pixel_size: &mut (u32, u32),
) -> Vec<EngineEvent> {
    let mut events = Vec::new();
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => events.push(EngineEvent::Quit),
            Event::KeyDown {
                keycode: Some(keycode),
                keymod,
                repeat,
                ..
            } => {
                if repeat {
                    continue;
                }
                let key = KeyEvent::new(map_keycode(keycode), map_modifiers(keymod));
                if is_quit_key(key.code, key.modifiers) {
                    events.push(EngineEvent::Quit);
                } else {
                    events.push(EngineEvent::KeyPressed(key));
                }
            }
            Event::MouseMotion { x, y, .. } => {
                let present_rect =
                    presentation_rect(*window_pixel_size, content_pixel_size, presentation_policy);
                let (column, row) =
                    map_mouse_to_output(x, y, output_width, output_height, present_rect);
                events.push(EngineEvent::MouseMoved { column, row });
            }
            Event::Window {
                win_event: WindowEvent::Resized(_, _) | WindowEvent::SizeChanged(_, _),
                ..
            } => {
                *window_pixel_size = current_window_pixel_size(canvas);
                let _ =
                    present_texture(canvas, frame_texture, content_pixel_size, presentation_policy);
            }
            _ => {}
        }
    }
    events
}

fn logical_dimensions(width: u16, height: u16) -> (u32, u32) {
    (
        (width.max(1) as u32) * LOGICAL_CELL_WIDTH,
        (height.max(1) as u32) * LOGICAL_CELL_HEIGHT,
    )
}

fn window_dimensions(
    width: u16,
    height: u16,
    pixel_scale: u32,
    window_ratio: Option<(u32, u32)>,
) -> (u32, u32) {
    let (logical_width, logical_height) = logical_dimensions(width, height);
    let base_w = logical_width.saturating_mul(pixel_scale.max(1));
    let base_h = logical_height.saturating_mul(pixel_scale.max(1));
    if let Some((ratio_w, ratio_h)) = window_ratio {
        if ratio_w == 0 || ratio_h == 0 {
            return (base_w, base_h);
        }
        let forced_h = (base_w.saturating_mul(ratio_h) / ratio_w.max(1)).max(1);
        (base_w.max(1), forced_h)
    } else {
        (base_w.max(1), base_h.max(1))
    }
}

fn map_mouse_to_output(
    x: i32,
    y: i32,
    output_width: u16,
    output_height: u16,
    present_rect: Rect,
) -> (u16, u16) {
    let width = output_width.max(1) as u32;
    let height = output_height.max(1) as u32;
    let rect_width = present_rect.width().max(1);
    let rect_height = present_rect.height().max(1);
    let relative_x = (x - present_rect.x())
        .clamp(0, rect_width.saturating_sub(1) as i32) as u32;
    let relative_y = (y - present_rect.y())
        .clamp(0, rect_height.saturating_sub(1) as i32) as u32;

    (
        (relative_x.saturating_mul(width) / rect_width).min(width.saturating_sub(1)) as u16,
        (relative_y.saturating_mul(height) / rect_height).min(height.saturating_sub(1)) as u16,
    )
}

fn current_window_pixel_size(canvas: &sdl2::render::WindowCanvas) -> (u32, u32) {
    canvas
        .output_size()
        .unwrap_or_else(|_| canvas.window().size())
}

fn presentation_rect(
    window_pixel_size: (u32, u32),
    content_pixel_size: (u32, u32),
    presentation_policy: PresentationPolicy,
) -> Rect {
    let layout = compute_presentation_layout(
        window_pixel_size.0,
        window_pixel_size.1,
        content_pixel_size.0,
        content_pixel_size.1,
        presentation_policy,
    );
    Rect::new(
        layout.dst_x as i32,
        layout.dst_y as i32,
        layout.dst_width,
        layout.dst_height,
    )
}

pub(crate) fn map_keycode(keycode: Keycode) -> KeyCode {
    match keycode {
        Keycode::Return => KeyCode::Enter,
        Keycode::Backspace => KeyCode::Backspace,
        Keycode::Tab => KeyCode::Tab,
        Keycode::Escape => KeyCode::Esc,
        Keycode::Up => KeyCode::Up,
        Keycode::Down => KeyCode::Down,
        Keycode::Left => KeyCode::Left,
        Keycode::Right => KeyCode::Right,
        Keycode::Home => KeyCode::Home,
        Keycode::End => KeyCode::End,
        Keycode::PageUp => KeyCode::PageUp,
        Keycode::PageDown => KeyCode::PageDown,
        Keycode::Delete => KeyCode::Delete,
        Keycode::Insert => KeyCode::Insert,
        Keycode::F1 => KeyCode::F(1),
        Keycode::F2 => KeyCode::F(2),
        Keycode::F3 => KeyCode::F(3),
        Keycode::F4 => KeyCode::F(4),
        Keycode::F5 => KeyCode::F(5),
        Keycode::F6 => KeyCode::F(6),
        Keycode::F7 => KeyCode::F(7),
        Keycode::F8 => KeyCode::F(8),
        Keycode::F9 => KeyCode::F(9),
        Keycode::F10 => KeyCode::F(10),
        Keycode::F11 => KeyCode::F(11),
        Keycode::F12 => KeyCode::F(12),
        Keycode::Backquote => KeyCode::Char('`'),
        Keycode::Space => KeyCode::Char(' '),
        _ => keycode
            .name()
            .chars()
            .next()
            .map(|ch| KeyCode::Char(ch.to_ascii_lowercase()))
            .unwrap_or(KeyCode::Null),
    }
}

pub(crate) fn map_modifiers(keymod: Mod) -> KeyModifiers {
    let mut modifiers = KeyModifiers::NONE;
    if keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) {
        modifiers = modifiers | KeyModifiers::CONTROL;
    }
    if keymod.intersects(Mod::LALTMOD | Mod::RALTMOD) {
        modifiers = modifiers | KeyModifiers::ALT;
    }
    if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
        modifiers = modifiers | KeyModifiers::SHIFT;
    }
    modifiers
}

fn is_quit_key(code: KeyCode, modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::CONTROL)
        && matches!(
            code,
            KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Char('q') | KeyCode::Char('Q')
        )
}

/// Draw a semi-transparent dark tint over the entire window to dim the scene
/// behind the debug overlay. Uses SDL2 alpha blending.
fn draw_scene_dim(canvas: &mut sdl2::render::WindowCanvas) {
    let (win_w, win_h) = canvas
        .output_size()
        .unwrap_or_else(|_| canvas.window().size());
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 140));
    let _ = canvas.fill_rect(Rect::new(0, 0, win_w, win_h));
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

/// Render overlay text directly onto the canvas using the embedded bitmap font.
///
/// Characters are drawn at native window-pixel resolution (not game-buffer
/// resolution) so the text is always crisp and readable regardless of game
/// scaling. Line backgrounds use alpha blending when `bg_alpha < 255` for
/// a semi-transparent console look.
fn draw_overlay(canvas: &mut sdl2::render::WindowCanvas, overlay: &OverlayData) {
    use crate::bitmap_font::{GLYPH_HEIGHT, GLYPH_WIDTH, glyph};

    const OVERLAY_SCALE: u32 = 1;
    let char_w = GLYPH_WIDTH * OVERLAY_SCALE;
    let char_h = GLYPH_HEIGHT * OVERLAY_SCALE;

    let (win_w, _win_h) = canvas
        .output_size()
        .unwrap_or_else(|_| canvas.window().size());
    let max_cols = (win_w / char_w) as usize;

    for (row_idx, line) in overlay.lines.iter().enumerate() {
        let y_origin = row_idx as i32 * char_h as i32;
        let (bg_r, bg_g, bg_b) = line.bg.to_rgb();
        let (fg_r, fg_g, fg_b) = line.fg.to_rgb();

        // Fill background with alpha blending for semi-transparency.
        if line.bg_alpha < 255 {
            canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            canvas.set_draw_color(SdlColor::RGBA(bg_r, bg_g, bg_b, line.bg_alpha));
        } else {
            canvas.set_blend_mode(sdl2::render::BlendMode::None);
            canvas.set_draw_color(SdlColor::RGB(bg_r, bg_g, bg_b));
        }
        let _ = canvas.fill_rect(Rect::new(0, y_origin, win_w, char_h));

        // Render each character glyph (always opaque).
        canvas.set_blend_mode(sdl2::render::BlendMode::None);
        canvas.set_draw_color(SdlColor::RGB(fg_r, fg_g, fg_b));
        for (col_idx, ch) in line.text.chars().enumerate() {
            if col_idx >= max_cols {
                break;
            }
            if ch == ' ' {
                continue;
            }
            let bitmap = glyph(ch);
            let gx = col_idx as i32 * char_w as i32;
            for (py, &row_bits) in bitmap.iter().enumerate() {
                if row_bits == 0 {
                    continue;
                }
                for px in 0..8u32 {
                    if row_bits & (0x80 >> px) != 0 {
                        let _ = canvas.fill_rect(Rect::new(
                            gx + (px * OVERLAY_SCALE) as i32,
                            y_origin + (py as u32 * OVERLAY_SCALE) as i32,
                            OVERLAY_SCALE,
                            OVERLAY_SCALE,
                        ));
                    }
                }
            }
        }
    }
    // Reset blend mode to default.
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

#[cfg(test)]
mod tests {
    use super::{logical_dimensions, map_mouse_to_output, presentation_rect, window_dimensions};
    use crate::renderer::DEFAULT_PIXEL_SCALE;
    use engine_runtime::PresentationPolicy;
    use sdl2::rect::Rect;

    #[test]
    fn logical_surface_uses_double_height_per_output_row() {
        assert_eq!(logical_dimensions(120, 30), (120, 60));
        assert_eq!(window_dimensions(120, 30, DEFAULT_PIXEL_SCALE, None), (960, 480));
    }

    #[test]
    fn window_dimensions_respects_16_9_ratio() {
        assert_eq!(
            window_dimensions(180, 30, DEFAULT_PIXEL_SCALE, Some((16, 9))),
            (1440, 810)
        );
    }

    #[test]
    fn mouse_mapping_stretches_across_full_window() {
        let mapped = map_mouse_to_output(480, 320, 120, 30, Rect::new(0, 0, 960, 640));
        assert_eq!(mapped, (60, 15));
    }

    #[test]
    fn presentation_rect_uses_policy() {
        assert_eq!(
            presentation_rect((960, 640), (960, 480), PresentationPolicy::Fit),
            Rect::new(0, 80, 960, 480)
        );
        assert_eq!(
            presentation_rect((1200, 800), (960, 480), PresentationPolicy::Strict),
            Rect::new(120, 160, 960, 480)
        );
        assert_eq!(
            presentation_rect((1200, 800), (960, 480), PresentationPolicy::Stretch),
            Rect::new(0, 0, 1200, 800)
        );
    }

    #[test]
    fn mouse_mapping_respects_letterboxed_fit_rect() {
        let mapped = map_mouse_to_output(480, 320, 120, 30, Rect::new(0, 80, 960, 480));
        assert_eq!(mapped, (60, 15));
    }
}

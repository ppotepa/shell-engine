use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

use engine_core::color::Color;
use engine_events::{EngineEvent, KeyCode, KeyEvent, KeyModifiers};
use engine_render::{OverlayData, VectorOverlay};
use engine_runtime::{compute_presentation_layout, PresentationPolicy};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton as SdlMouseButton;
use sdl2::pixels::{Color as SdlColor, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::video::FullscreenType;

pub(crate) type GlyphPatch = (u16, u16, char, Color, Color);

pub(crate) enum RuntimeCommand {
    Present {
        width: u16,
        height: u16,
        patches: Vec<GlyphPatch>,
        overlay: Option<OverlayData>,
        vectors: Option<VectorOverlay>,
        /// Direct pixel canvas data (RGBA, row-major) for SDL2 bypass.
        /// When Some, uploaded directly to the texture before cell patches.
        pixel_canvas: Option<PixelCanvasData>,
    },
    SetSplashMode(bool),
    PollInput,
    Clear,
    Shutdown,
}

/// Direct pixel canvas data for SDL2 bypass — avoids Cell→char→pixel encoding.
pub(crate) struct PixelCanvasData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub(crate) enum RuntimeResponse {
    Ack,
    Input(Vec<EngineEvent>),
}

pub(crate) struct Sdl2RuntimeClient {
    command_tx: Sender<RuntimeCommand>,
    response_rx: Receiver<RuntimeResponse>,
}

pub(crate) fn sdl_profile_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("SHELL_ENGINE_SDL_PROFILE")
            .ok()
            .map(|raw| {
                matches!(
                    raw.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
    })
}

struct RuntimeProfile {
    frames: u64,
    presented_frames: u64,
    total_patches: u64,
    max_patches: usize,
    total_apply: Duration,
    total_upload: Duration,
    total_present: Duration,
    total_cmd: Duration,
    last_emit: Instant,
}

impl RuntimeProfile {
    fn new() -> Self {
        Self {
            frames: 0,
            presented_frames: 0,
            total_patches: 0,
            max_patches: 0,
            total_apply: Duration::ZERO,
            total_upload: Duration::ZERO,
            total_present: Duration::ZERO,
            total_cmd: Duration::ZERO,
            last_emit: Instant::now(),
        }
    }

    fn record(
        &mut self,
        patch_count: usize,
        apply: Duration,
        upload: Duration,
        present: Duration,
        cmd: Duration,
        presented: bool,
    ) {
        self.frames = self.frames.saturating_add(1);
        self.total_patches = self.total_patches.saturating_add(patch_count as u64);
        self.max_patches = self.max_patches.max(patch_count);
        self.total_apply += apply;
        self.total_upload += upload;
        self.total_present += present;
        self.total_cmd += cmd;
        if presented {
            self.presented_frames = self.presented_frames.saturating_add(1);
        }

        if self.last_emit.elapsed() < Duration::from_secs(1) {
            return;
        }

        let frames = self.frames.max(1);
        let avg_patches = self.total_patches as f64 / frames as f64;
        let avg_apply_us = self.total_apply.as_micros() as f64 / frames as f64;
        let avg_upload_us = self.total_upload.as_micros() as f64 / frames as f64;
        let avg_present_us = self.total_present.as_micros() as f64 / frames as f64;
        let avg_cmd_us = self.total_cmd.as_micros() as f64 / frames as f64;
        let present_ratio = self.presented_frames as f64 * 100.0 / frames as f64;
        engine_core::logging::debug(
            "sdl2.runtime",
            format!(
                "fps_window={} presented={:.1}% avg_patches={:.1} max_patches={} avg_us(cmd/apply/upload/present)={:.0}/{:.0}/{:.0}/{:.0}",
                frames,
                present_ratio,
                avg_patches,
                self.max_patches,
                avg_cmd_us,
                avg_apply_us,
                avg_upload_us,
                avg_present_us
            ),
        );

        self.frames = 0;
        self.presented_frames = 0;
        self.total_patches = 0;
        self.max_patches = 0;
        self.total_apply = Duration::ZERO;
        self.total_upload = Duration::ZERO;
        self.total_present = Duration::ZERO;
        self.total_cmd = Duration::ZERO;
        self.last_emit = Instant::now();
    }
}

impl Sdl2RuntimeClient {
    pub(crate) fn spawn(
        output_width: u16,
        output_height: u16,
        presentation_policy: PresentationPolicy,
        window_ratio: Option<(u32, u32)>,
        pixel_scale: u32,
        vsync: bool,
        window_title: String,
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
                    window_title,
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

#[allow(clippy::too_many_arguments)]
fn runtime_thread(
    output_width: u16,
    output_height: u16,
    presentation_policy: PresentationPolicy,
    window_ratio: Option<(u32, u32)>,
    pixel_scale: u32,
    vsync: bool,
    window_title: String,
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
    let (requested_window_width, requested_window_height) =
        window_dimensions(output_width, output_height, pixel_scale, window_ratio);
    let (window_width, window_height, window_pos) = fit_window_to_primary_display(
        &video,
        requested_window_width,
        requested_window_height,
    );
    let mut window_builder = video.window(&window_title, window_width, window_height);
    if let Some((x, y)) = window_pos {
        window_builder.position(x, y);
    } else {
        window_builder.position_centered();
    }
    let Ok(window) = window_builder.resizable().build() else {
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
        vec![0u8; pixel_buffer_size(content_pixel_size.0, content_pixel_size.1)];
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
    let mut splash_mode = false;
    let mut row_ranges: Vec<Option<(u16, u16)>> = vec![None; current_output_height as usize];
    let mut profile = if sdl_profile_enabled() {
        Some(RuntimeProfile::new())
    } else {
        None
    };

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
            RuntimeCommand::Present {
                width,
                height,
                patches,
                overlay,
                vectors,
                pixel_canvas,
            } => {
                let t_cmd = Instant::now();

                // ── Pixel canvas path: direct pixel upload ───────────────
                if let Some(pc) = &pixel_canvas {
                    let pc_size = (pc.width, pc.height);
                    if pc_size != content_pixel_size {
                        content_pixel_size = pc_size;
                        current_output_width = width.max(1);
                        current_output_height = height.max(1);
                        pixel_buffer.resize(pixel_buffer_size(pc_size.0, pc_size.1), 0);
                        let Ok(new_texture) = texture_creator.create_texture_streaming(
                            PixelFormatEnum::RGBA32,
                            pc_size.0,
                            pc_size.1,
                        ) else {
                            let _ = response_tx.send(RuntimeResponse::Ack);
                            break;
                        };
                        frame_texture = new_texture;
                        row_ranges.resize(pc_size.1 as usize, None);
                    }
                    // Copy pixel canvas data into local pixel buffer.
                    let copy_len = pc.data.len().min(pixel_buffer.len());
                    pixel_buffer[..copy_len].copy_from_slice(&pc.data[..copy_len]);
                    // Upload the entire texture (pixel canvas is the full frame).
                    let t_apply = Instant::now();
                    let upload_ok = frame_texture
                        .update(None, &pixel_buffer, pc_size.0 as usize * 4)
                        .is_ok();
                    let apply_dur = t_apply.elapsed();
                    if !upload_ok {
                        let _ = response_tx.send(RuntimeResponse::Ack);
                        break;
                    }

                    // Apply cell patches ON TOP for text/UI sprites.
                    let mut dirty = DirtyPixelRect::empty();
                    row_ranges.fill(None);
                    let patch_count = patches.len();
                    if !patches.is_empty() {
                        for patch in &patches {
                            apply_patch_to_pixels(
                                patch,
                                current_output_width,
                                current_output_height,
                                &mut pixel_buffer,
                                &mut dirty,
                            );
                            update_row_range(&mut row_ranges, patch);
                        }
                        if let Some(rect) = dirty.to_rect() {
                            let _ = update_texture_row_ranges(
                                &mut frame_texture,
                                &pixel_buffer,
                                content_pixel_size.0,
                                &row_ranges,
                                rect,
                            );
                        }
                    }

                    let active_policy =
                        get_active_presentation_policy(splash_mode, presentation_policy);
                    let present_rect = presentation_rect(
                        current_window_pixel_size(&canvas),
                        content_pixel_size,
                        active_policy,
                    );
                    let t_present = Instant::now();
                    clear_canvas(&mut canvas, SdlColor::RGB(0, 0, 0));
                    if canvas
                        .copy(&frame_texture, None, Some(present_rect))
                        .is_err()
                    {
                        let _ = response_tx.send(RuntimeResponse::Ack);
                        break;
                    }
                    if let Some(ref vector_data) = vectors {
                        if !vector_data.is_empty() {
                            draw_vectors(
                                &mut canvas,
                                vector_data,
                                present_rect,
                                content_pixel_size,
                            );
                        }
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
                    let present_dur = t_present.elapsed();

                    if let Some(profile) = profile.as_mut() {
                        profile.record(
                            patch_count,
                            apply_dur,
                            Duration::ZERO,
                            present_dur,
                            t_cmd.elapsed(),
                            true,
                        );
                    }
                    RuntimeResponse::Ack
                } else {
                    // ── Cell patch path (terminal compatibility) ─────────────
                    if width != current_output_width || height != current_output_height {
                        current_output_width = width.max(1);
                        current_output_height = height.max(1);
                        content_pixel_size =
                            logical_dimensions(current_output_width, current_output_height);
                        pixel_buffer.resize(
                            pixel_buffer_size(content_pixel_size.0, content_pixel_size.1),
                            0,
                        );
                        let Ok(new_texture) = texture_creator.create_texture_streaming(
                            PixelFormatEnum::RGBA32,
                            content_pixel_size.0,
                            content_pixel_size.1,
                        ) else {
                            let _ = response_tx.send(RuntimeResponse::Ack);
                            break;
                        };
                        frame_texture = new_texture;
                        pixel_buffer.fill(0);
                        row_ranges.resize(current_output_height as usize, None);
                        if frame_texture
                            .update(None, &pixel_buffer, content_pixel_size.0 as usize * 4)
                            .is_err()
                        {
                            let _ = response_tx.send(RuntimeResponse::Ack);
                            break;
                        }
                    }

                    let mut dirty = DirtyPixelRect::empty();
                    row_ranges.fill(None);
                    let patch_count = patches.len();
                    let t_apply = Instant::now();
                    for patch in &patches {
                        apply_patch_to_pixels(
                            patch,
                            current_output_width,
                            current_output_height,
                            &mut pixel_buffer,
                            &mut dirty,
                        );
                        update_row_range(&mut row_ranges, patch);
                    }
                    let apply_dur = t_apply.elapsed();

                    let mut upload_dur = Duration::ZERO;
                    if let Some(rect) = dirty.to_rect() {
                        let t_upload = Instant::now();
                        if update_texture_row_ranges(
                            &mut frame_texture,
                            &pixel_buffer,
                            content_pixel_size.0,
                            &row_ranges,
                            rect,
                        )
                        .is_err()
                        {
                            let _ = response_tx.send(RuntimeResponse::Ack);
                            break;
                        }
                        upload_dur = t_upload.elapsed();
                    }

                    let should_present =
                        dirty.has_updates || overlay.is_some() || vectors.is_some();
                    let mut present_dur = Duration::ZERO;
                    if should_present {
                        let active_policy =
                            get_active_presentation_policy(splash_mode, presentation_policy);
                        let present_rect = presentation_rect(
                            current_window_pixel_size(&canvas),
                            content_pixel_size,
                            active_policy,
                        );
                        let t_present = Instant::now();
                        if splash_mode {
                            let (r, g, b) = splash_clear_rgb(&pixel_buffer);
                            clear_canvas(&mut canvas, SdlColor::RGB(r, g, b));
                        } else {
                            clear_canvas(&mut canvas, SdlColor::RGB(0, 0, 0));
                        }
                        if canvas
                            .copy(&frame_texture, None, Some(present_rect))
                            .is_err()
                        {
                            let _ = response_tx.send(RuntimeResponse::Ack);
                            break;
                        }

                        if let Some(ref vector_data) = vectors {
                            if !vector_data.is_empty() {
                                draw_vectors(
                                    &mut canvas,
                                    vector_data,
                                    present_rect,
                                    content_pixel_size,
                                );
                            }
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
                        present_dur = t_present.elapsed();
                    }
                    if let Some(profile) = profile.as_mut() {
                        profile.record(
                            patch_count,
                            apply_dur,
                            upload_dur,
                            present_dur,
                            t_cmd.elapsed(),
                            should_present,
                        );
                    }

                    RuntimeResponse::Ack
                } // else (cell-patch path)
            }
            RuntimeCommand::PollInput => RuntimeResponse::Input(poll_input(
                &mut canvas,
                &frame_texture,
                &mut event_pump,
                current_output_width,
                current_output_height,
                content_pixel_size,
                get_active_presentation_policy(splash_mode, presentation_policy),
                &mut window_pixel_size,
            )),
            RuntimeCommand::SetSplashMode(enabled) => {
                splash_mode = enabled;
                RuntimeResponse::Ack
            }
            RuntimeCommand::Clear => {
                pixel_buffer.fill(0);
                let active_policy =
                    get_active_presentation_policy(splash_mode, presentation_policy);
                if frame_texture
                    .update(None, &pixel_buffer, content_pixel_size.0 as usize * 4)
                    .is_err()
                    || present_texture(
                        &mut canvas,
                        &frame_texture,
                        content_pixel_size,
                        active_policy,
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

#[derive(Debug, Clone, Copy)]
struct DirtyPixelRect {
    x_min: u32,
    x_max: u32,
    y_min: u32,
    y_max: u32,
    has_updates: bool,
}

impl DirtyPixelRect {
    #[inline]
    fn empty() -> Self {
        Self {
            x_min: 0,
            x_max: 0,
            y_min: 0,
            y_max: 0,
            has_updates: false,
        }
    }

    #[inline]
    fn include_cell(&mut self, x: u16, y: u16) {
        let x = x as u32;
        let y = y as u32;
        if !self.has_updates {
            self.x_min = x;
            self.x_max = x;
            self.y_min = y;
            self.y_max = y;
            self.has_updates = true;
            return;
        }
        self.x_min = self.x_min.min(x);
        self.x_max = self.x_max.max(x);
        self.y_min = self.y_min.min(y);
        self.y_max = self.y_max.max(y);
    }

    #[inline]
    fn to_rect(self) -> Option<(u32, u32, u32, u32)> {
        if !self.has_updates {
            return None;
        }
        Some((
            self.x_min,
            self.y_min,
            self.x_max - self.x_min + 1,
            self.y_max - self.y_min + 1,
        ))
    }
}

/// Maps a buffer cell to a single RGBA pixel colour.
///
/// In 1:1 pixel mode each buffer cell is exactly one screen pixel.
/// `▀`, `▄`, `█` and all non-space characters resolve to `fg`.
/// `' '` resolves to `bg`.  Shade characters blend proportionally.
#[inline]
fn cell_pixel_color(symbol: char, fg: (u8, u8, u8), bg: (u8, u8, u8)) -> (u8, u8, u8) {
    match symbol {
        ' ' => bg,
        '░' => blend(fg, bg, 0.25),
        '▒' => blend(fg, bg, 0.50),
        '▓' => blend(fg, bg, 0.75),
        _ => fg,
    }
}

#[inline]
fn blend(fg: (u8, u8, u8), bg: (u8, u8, u8), fg_weight: f32) -> (u8, u8, u8) {
    let bw = 1.0 - fg_weight;
    (
        ((bg.0 as f32 * bw) + (fg.0 as f32 * fg_weight)).round() as u8,
        ((bg.1 as f32 * bw) + (fg.1 as f32 * fg_weight)).round() as u8,
        ((bg.2 as f32 * bw) + (fg.2 as f32 * fg_weight)).round() as u8,
    )
}

fn apply_patch_to_pixels(
    patch: &GlyphPatch,
    output_width: u16,
    output_height: u16,
    pixel_buffer: &mut [u8],
    dirty: &mut DirtyPixelRect,
) {
    let (x, y, symbol, fg, bg) = *patch;
    if x >= output_width || y >= output_height {
        return;
    }
    let logical_w = output_width as usize;
    let color = cell_pixel_color(symbol, fg.to_rgb(), bg.to_rgb());
    let idx = (y as usize * logical_w + x as usize) * 4;
    if idx + 3 >= pixel_buffer.len() {
        return;
    }
    write_pixel_rgba(pixel_buffer, idx, color);
    dirty.include_cell(x, y);
}

fn update_texture_rect(
    texture: &mut sdl2::render::Texture<'_>,
    pixel_buffer: &[u8],
    logical_w: u32,
    rect: (u32, u32, u32, u32),
) -> Result<(), String> {
    let (x, y, w, h) = rect;
    if w == 0 || h == 0 {
        return Ok(());
    }
    let pitch = logical_w as usize * 4;
    let start = y as usize * pitch + x as usize * 4;
    let len = (h as usize - 1) * pitch + (w as usize * 4);
    let end = start.saturating_add(len);
    if end > pixel_buffer.len() {
        return Err(String::from("texture update rect out of bounds"));
    }
    texture
        .update(
            Some(Rect::new(x as i32, y as i32, w, h)),
            &pixel_buffer[start..end],
            pitch,
        )
        .map_err(|error| error.to_string())
}

fn update_texture_row_ranges(
    texture: &mut sdl2::render::Texture<'_>,
    pixel_buffer: &[u8],
    logical_w: u32,
    row_ranges: &[Option<(u16, u16)>],
    fallback_rect: (u32, u32, u32, u32),
) -> Result<(), String> {
    let mut updated_any = false;
    for (row, range) in row_ranges.iter().enumerate() {
        let Some((x_min, x_max)) = range else {
            continue;
        };
        if x_max < x_min {
            continue;
        }
        let x = *x_min as u32;
        let y = row as u32;
        let w = (*x_max - *x_min + 1) as u32;
        let h = 1;
        update_texture_rect(texture, pixel_buffer, logical_w, (x, y, w, h))?;
        updated_any = true;
    }
    if !updated_any {
        update_texture_rect(texture, pixel_buffer, logical_w, fallback_rect)?;
    }
    Ok(())
}

#[inline]
fn update_row_range(row_ranges: &mut [Option<(u16, u16)>], patch: &GlyphPatch) {
    let (x, y, _, _, _) = *patch;
    let row = y as usize;
    if row >= row_ranges.len() {
        return;
    }
    match &mut row_ranges[row] {
        Some((x_min, x_max)) => {
            *x_min = (*x_min).min(x);
            *x_max = (*x_max).max(x);
        }
        None => row_ranges[row] = Some((x, x)),
    }
}
fn present_texture(
    canvas: &mut sdl2::render::WindowCanvas,
    texture: &sdl2::render::Texture<'_>,
    content_pixel_size: (u32, u32),
    presentation_policy: PresentationPolicy,
) -> Result<(), String> {
    clear_canvas(canvas, SdlColor::RGB(0, 0, 0));
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

#[inline]
fn splash_clear_rgb(pixel_buffer: &[u8]) -> (u8, u8, u8) {
    if pixel_buffer.len() >= 3 {
        (pixel_buffer[0], pixel_buffer[1], pixel_buffer[2])
    } else {
        (0, 0, 0)
    }
}

#[allow(clippy::too_many_arguments)]
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
                let key = KeyEvent::new(map_keycode(keycode), map_modifiers(keymod));
                if is_fullscreen_toggle_key(key.code, key.modifiers) {
                    if repeat {
                        continue;
                    }
                    let _ = toggle_fullscreen(canvas);
                    *window_pixel_size = current_window_pixel_size(canvas);
                    let _ = present_texture(
                        canvas,
                        frame_texture,
                        content_pixel_size,
                        presentation_policy,
                    );
                } else if is_quit_key(key.code, key.modifiers) {
                    if repeat {
                        continue;
                    }
                    events.push(EngineEvent::Quit);
                } else {
                    events.push(EngineEvent::KeyDown { key, repeat });
                }
            }
            Event::KeyUp {
                keycode: Some(keycode),
                keymod,
                repeat,
                ..
            } => {
                if repeat {
                    continue;
                }
                let key = KeyEvent::new(map_keycode(keycode), map_modifiers(keymod));
                events.push(EngineEvent::KeyUp { key });
            }
            Event::MouseMotion { x, y, .. } => {
                let present_rect =
                    presentation_rect(*window_pixel_size, content_pixel_size, presentation_policy);
                let (vx, vy) = map_mouse_to_output(x, y, output_width, output_height, present_rect);
                events.push(EngineEvent::MouseMoved { x: vx, y: vy });
            }
            Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                let present_rect =
                    presentation_rect(*window_pixel_size, content_pixel_size, presentation_policy);
                let (vx, vy) = map_mouse_to_output(x, y, output_width, output_height, present_rect);
                let button = map_mouse_button(mouse_btn);
                events.push(EngineEvent::MouseButtonDown {
                    button,
                    x: vx,
                    y: vy,
                });
            }
            Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => {
                let present_rect =
                    presentation_rect(*window_pixel_size, content_pixel_size, presentation_policy);
                let (vx, vy) = map_mouse_to_output(x, y, output_width, output_height, present_rect);
                let button = map_mouse_button(mouse_btn);
                events.push(EngineEvent::MouseButtonUp {
                    button,
                    x: vx,
                    y: vy,
                });
            }
            Event::MouseWheel { y, .. } => {
                events.push(EngineEvent::MouseWheel { delta_y: y as f32 });
            }
            Event::Window {
                win_event: WindowEvent::Resized(_, _) | WindowEvent::SizeChanged(_, _),
                ..
            } => {
                *window_pixel_size = current_window_pixel_size(canvas);
                let _ = present_texture(
                    canvas,
                    frame_texture,
                    content_pixel_size,
                    presentation_policy,
                );
            }
            Event::Window {
                win_event: WindowEvent::FocusLost,
                ..
            } => {
                events.push(EngineEvent::InputFocusLost);
            }
            _ => {}
        }
    }
    events
}

fn logical_dimensions(width: u16, height: u16) -> (u32, u32) {
    (width.max(1) as u32, height.max(1) as u32)
}

fn fit_window_to_primary_display(
    video: &sdl2::VideoSubsystem,
    requested_width: u32,
    requested_height: u32,
) -> (u32, u32, Option<(i32, i32)>) {
    let Ok(bounds) = video.display_usable_bounds(0) else {
        return (requested_width.max(1), requested_height.max(1), None);
    };
    let max_w = ((bounds.width() as f32) * 0.9).round() as u32;
    let max_h = ((bounds.height() as f32) * 0.9).round() as u32;
    let fitted_w = requested_width.max(1).min(max_w.max(1));
    let fitted_h = requested_height.max(1).min(max_h.max(1));
    let x = bounds.x() + ((bounds.width() as i32 - fitted_w as i32) / 2);
    let y = bounds.y() + ((bounds.height() as i32 - fitted_h as i32) / 2);
    (fitted_w, fitted_h, Some((x, y)))
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
) -> (f32, f32) {
    let width = output_width.max(1) as f32;
    let height = output_height.max(1) as f32;
    let rect_width = present_rect.width().max(1) as f32;
    let rect_height = present_rect.height().max(1) as f32;
    let rel_x = (x - present_rect.x()) as f32;
    let rel_y = (y - present_rect.y()) as f32;
    let vx = (rel_x / rect_width * width).clamp(0.0, width - 1.0);
    let vy = (rel_y / rect_height * height).clamp(0.0, height - 1.0);
    (vx, vy)
}

fn map_mouse_button(btn: SdlMouseButton) -> engine_events::MouseButton {
    use engine_events::MouseButton;
    match btn {
        SdlMouseButton::Left => MouseButton::Left,
        SdlMouseButton::Right => MouseButton::Right,
        SdlMouseButton::Middle => MouseButton::Middle,
        _ => MouseButton::Left,
    }
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
        Keycode::Return | Keycode::KpEnter => KeyCode::Enter,
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
        modifiers |= KeyModifiers::CONTROL;
    }
    if keymod.intersects(Mod::LALTMOD | Mod::RALTMOD) {
        modifiers |= KeyModifiers::ALT;
    }
    if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
        modifiers |= KeyModifiers::SHIFT;
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

fn is_fullscreen_toggle_key(code: KeyCode, modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::ALT) && matches!(code, KeyCode::Enter)
}

fn toggle_fullscreen(canvas: &mut sdl2::render::WindowCanvas) -> Result<(), String> {
    let target = if matches!(
        canvas.window().fullscreen_state(),
        FullscreenType::Desktop | FullscreenType::True
    ) {
        FullscreenType::Off
    } else {
        FullscreenType::Desktop
    };

    canvas
        .window_mut()
        .set_fullscreen(target)
        .map_err(|error| error.to_string())
}

/// Draw a semi-transparent dark tint over the entire window to dim the scene
/// behind the debug overlay. Uses SDL2 alpha blending.
fn draw_scene_dim(canvas: &mut sdl2::render::WindowCanvas) {
    let (win_w, win_h) = canvas
        .output_size()
        .unwrap_or_else(|_| canvas.window().size());
    set_canvas_color_blended(canvas, 0, 0, 0, 140);
    let _ = canvas.fill_rect(Rect::new(0, 0, win_w, win_h));
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

/// Render overlay text directly onto the canvas using the engine's tiny generic bitmap font.
///
/// Characters are drawn at native window-pixel resolution (not game-buffer
/// resolution) so the text is always crisp and readable regardless of game
/// scaling. Line backgrounds use alpha blending when `bg_alpha < 255` for
/// a semi-transparent console look.
fn draw_overlay(canvas: &mut sdl2::render::WindowCanvas, overlay: &OverlayData) {
    use engine_core::markup::parse_spans;
    use engine_render::generic::generic_glyph_rows;

    // Use the standard 5x7 generic glyphs for the developer console.
    // The tiny 4x5 set stays too ambiguous even when scaled up.
    const OVERLAY_SCALE: u32 = 3;
    const GLYPH_W: u32 = 5;
    const GLYPH_H: u32 = 7;
    const GLYPH_GAP: u32 = 1;
    let glyph_w = GLYPH_W;
    let glyph_h = GLYPH_H;
    let glyph_gap = GLYPH_GAP;
    let char_advance = (glyph_w + glyph_gap).max(1) * OVERLAY_SCALE;
    let char_h = glyph_h * OVERLAY_SCALE;

    let (win_w, _win_h) = canvas
        .output_size()
        .unwrap_or_else(|_| canvas.window().size());
    let max_cols = (win_w / char_advance.max(1)) as usize;

    for (row_idx, line) in overlay.lines.iter().enumerate() {
        let y_origin = row_idx as i32 * char_h as i32;
        let (bg_r, bg_g, bg_b) = line.bg.to_rgb();

        // Fill background with alpha blending for semi-transparency.
        set_canvas_color_blended(canvas, bg_r, bg_g, bg_b, line.bg_alpha);
        let _ = canvas.fill_rect(Rect::new(0, y_origin, win_w, char_h));

        // Render each character glyph (always opaque), supporting inline
        // [colour]...[/] markup spans for emphasizing important values.
        canvas.set_blend_mode(sdl2::render::BlendMode::None);
        let mut col_idx = 0usize;
        for span in parse_spans(&line.text) {
            if col_idx >= max_cols {
                break;
            }
            let span_fg = span
                .colour
                .as_ref()
                .map(engine_core::color::Color::from)
                .unwrap_or(line.fg);
            let (span_r, span_g, span_b) = span_fg.to_rgb();
            canvas.set_draw_color(SdlColor::RGB(span_r, span_g, span_b));

            for ch in span.text.chars() {
                if col_idx >= max_cols {
                    break;
                }
                if ch != ' ' {
                    let Some(bitmap) = generic_glyph_rows(ch) else {
                        col_idx += 1;
                        continue;
                    };
                    let gx = col_idx as i32 * char_advance as i32;
                    for (py, &row_bits) in bitmap.iter().enumerate() {
                        if row_bits == 0 {
                            continue;
                        }
                        for px in 0..glyph_w {
                            if row_bits & (1u8 << (glyph_w - 1 - px)) != 0 {
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
                col_idx += 1;
            }
        }
    }
    // Reset blend mode to default.
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

/// Draw vector primitives directly on the SDL2 canvas at native resolution.
///
/// Converts buffer cell coordinates to canvas pixel coordinates using the
/// presentation rect and content pixel size, then draws outlines and fills.
fn draw_vectors(
    canvas: &mut sdl2::render::WindowCanvas,
    vectors: &VectorOverlay,
    present_rect: Rect,
    content_pixel_size: (u32, u32),
) {
    let (cpw, cph) = content_pixel_size;
    if cpw == 0 || cph == 0 {
        return;
    }
    let pr_x = present_rect.x() as f32;
    let pr_y = present_rect.y() as f32;
    let pr_w = present_rect.width() as f32;
    let pr_h = present_rect.height() as f32;
    let cpw_f = cpw as f32;
    let cph_f = cph as f32;

    for prim in &vectors.primitives {
        if prim.points.is_empty() {
            continue;
        }

        // Map buffer pixel coords → canvas pixel via the presentation rect.
        let canvas_pts: Vec<(i32, i32)> = prim
            .points
            .iter()
            .map(|p| {
                let cx = pr_x + p[0] * pr_w / cpw_f;
                let cy = pr_y + p[1] * pr_h / cph_f;
                (cx as i32, cy as i32)
            })
            .collect();

        // Single-point shape: draw a small dot (2×2 rect).
        if canvas_pts.len() == 1 {
            let (r, g, b) = prim.fg;
            canvas.set_draw_color(SdlColor::RGB(r, g, b));
            let _ = canvas.fill_rect(Rect::new(canvas_pts[0].0 - 1, canvas_pts[0].1 - 1, 3, 3));
            continue;
        }

        // Fill polygon if bg is set.
        if let Some((r, g, b)) = prim.bg {
            canvas.set_draw_color(SdlColor::RGB(r, g, b));
            scanline_fill_polygon(canvas, &canvas_pts);
        }

        // Draw outline.
        let (r, g, b) = prim.fg;
        canvas.set_draw_color(SdlColor::RGB(r, g, b));
        for i in 0..canvas_pts.len() - 1 {
            let _ = canvas.draw_line(canvas_pts[i], canvas_pts[i + 1]);
        }
        if prim.closed && canvas_pts.len() >= 3 {
            let _ = canvas.draw_line(canvas_pts[canvas_pts.len() - 1], canvas_pts[0]);
        }
    }
}

fn pixel_buffer_size(content_width: u32, content_height: u32) -> usize {
    (content_width * content_height * 4) as usize
}

fn get_active_presentation_policy(
    splash_mode: bool,
    presentation_policy: PresentationPolicy,
) -> PresentationPolicy {
    if splash_mode {
        PresentationPolicy::Fit
    } else {
        presentation_policy
    }
}

fn write_pixel_rgba(buf: &mut [u8], idx: usize, rgb: (u8, u8, u8)) {
    buf[idx] = rgb.0;
    buf[idx + 1] = rgb.1;
    buf[idx + 2] = rgb.2;
    buf[idx + 3] = 255;
}

fn clear_canvas(canvas: &mut sdl2::render::WindowCanvas, color: SdlColor) {
    canvas.set_draw_color(color);
    canvas.clear();
}

fn set_canvas_color_blended(
    canvas: &mut sdl2::render::WindowCanvas,
    r: u8,
    g: u8,
    b: u8,
    alpha: u8,
) {
    if alpha < 255 {
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(SdlColor::RGBA(r, g, b, alpha));
    } else {
        canvas.set_blend_mode(sdl2::render::BlendMode::None);
        canvas.set_draw_color(SdlColor::RGB(r, g, b));
    }
}

/// Scanline polygon fill using even-odd rule.
fn scanline_fill_polygon(canvas: &mut sdl2::render::WindowCanvas, points: &[(i32, i32)]) {
    if points.len() < 3 {
        return;
    }
    let min_y = points.iter().map(|p| p.1).min().unwrap();
    let max_y = points.iter().map(|p| p.1).max().unwrap();
    let n = points.len();

    for y in min_y..=max_y {
        let mut intersections: Vec<i32> = Vec::with_capacity(8);
        for i in 0..n {
            let j = (i + 1) % n;
            let (x1, y1) = (points[i].0 as f32, points[i].1 as f32);
            let (x2, y2) = (points[j].0 as f32, points[j].1 as f32);
            let yf = y as f32;
            if (y1 <= yf && y2 > yf) || (y2 <= yf && y1 > yf) {
                let x = x1 + (yf - y1) / (y2 - y1) * (x2 - x1);
                intersections.push(x as i32);
            }
        }
        intersections.sort_unstable();
        for pair in intersections.chunks(2) {
            if pair.len() == 2 {
                let _ = canvas.draw_line((pair[0], y), (pair[1], y));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_fullscreen_toggle_key, logical_dimensions, map_mouse_to_output, presentation_rect,
        window_dimensions,
    };
    use engine_core::color::Color;
    use engine_events::{KeyCode, KeyModifiers};
    use engine_runtime::PresentationPolicy;
    use sdl2::rect::Rect;

    #[test]
    fn logical_surface_is_one_to_one() {
        // 1:1 pixel mode — logical dimensions equal buffer dimensions, no doubling.
        assert_eq!(logical_dimensions(120, 30), (120, 30));
        assert_eq!(window_dimensions(120, 30, 1, None), (120, 30));
    }

    #[test]
    fn window_dimensions_respects_16_9_ratio() {
        // Forced ratio with pixel_scale=1 — width 180 × 9/16 = 101.
        assert_eq!(window_dimensions(180, 30, 1, Some((16, 9))), (180, 101));
    }

    #[test]
    fn mouse_mapping_stretches_across_full_window() {
        let mapped = map_mouse_to_output(480, 320, 120, 30, Rect::new(0, 0, 960, 640));
        assert_eq!(mapped, (60.0, 15.0));
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
        assert_eq!(mapped, (60.0, 15.0));
    }

    #[test]
    fn dirty_rect_converts_cell_to_single_pixel() {
        let mut dirty = super::DirtyPixelRect::empty();
        dirty.include_cell(10, 3);
        // In 1:1 mode each cell is exactly 1 pixel row tall.
        assert_eq!(dirty.to_rect(), Some((10, 3, 1, 1)));
    }

    #[test]
    fn patch_raster_writes_correct_pixel() {
        // Buffer: 4 wide × 3 tall → 4 × 3 × 4 bytes.
        let mut pixels = vec![0u8; (4 * 3 * 4) as usize];
        let mut dirty = super::DirtyPixelRect::empty();
        let patch = (
            2,
            1,
            '▀',
            Color::Rgb {
                r: 10,
                g: 20,
                b: 30,
            },
            Color::Rgb { r: 1, g: 2, b: 3 },
        );
        super::apply_patch_to_pixels(&patch, 4, 3, &mut pixels, &mut dirty);
        // `▀` → fg color at (x=2, y=1).
        let pitch = 4 * 4;
        let idx = 1 * pitch + 2 * 4;
        assert_eq!(&pixels[idx..idx + 4], &[10, 20, 30, 255]);
    }

    #[test]
    fn fullscreen_toggle_uses_alt_enter() {
        assert!(is_fullscreen_toggle_key(KeyCode::Enter, KeyModifiers::ALT));
        assert!(!is_fullscreen_toggle_key(
            KeyCode::Enter,
            KeyModifiers::NONE
        ));
        assert!(!is_fullscreen_toggle_key(
            KeyCode::Char('f'),
            KeyModifiers::ALT
        ));
    }
}

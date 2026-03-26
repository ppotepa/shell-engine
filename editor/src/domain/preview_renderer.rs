//! Shared preview rendering service for editor UI components.

use std::path::Path;

use engine::animation::{Animator, SceneStage};
use engine::assets::AssetRoot;
use engine::audio::AudioRuntime;
use engine::buffer::Buffer;
use engine::runtime_settings::RuntimeSettings;
use engine::scene::Scene;
use engine::scene_runtime::SceneRuntime;
use engine::systems::compositor::compositor_system;
use engine::world::World;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

pub const DEFAULT_SCENE_PREVIEW_DURATION_MS: u64 = 3_000;

pub struct PreviewRenderRequest<'a> {
    pub scene: &'a Scene,
    pub width: u16,
    pub height: u16,
    pub asset_root: &'a Path,
    pub progress: f32,
    pub duration_ms: u64,
}

pub fn render_scene_buffer(req: PreviewRenderRequest<'_>) -> Result<Buffer, String> {
    if !req.asset_root.exists() {
        return Err(format!(
            "asset root not found: {}",
            req.asset_root.display()
        ));
    }
    let progress = req.progress.clamp(0.0, 1.0);
    let elapsed = (progress * req.duration_ms as f32) as u64;

    let mut world = World::new();
    world.register(Buffer::new(req.width, req.height));
    world.register(AudioRuntime::null());
    world.register(RuntimeSettings::default());
    world.register(AssetRoot::new(req.asset_root.to_path_buf()));
    world.register_scoped(SceneRuntime::new(req.scene.clone()));

    let mut animator = Animator::new();
    animator.stage = SceneStage::OnIdle;
    animator.elapsed_ms = elapsed;
    animator.stage_elapsed_ms = elapsed;
    animator.scene_elapsed_ms = elapsed;
    world.register_scoped(animator);

    compositor_system(&mut world);

    world
        .get::<Buffer>()
        .cloned()
        .ok_or_else(|| String::from("Preview render did not produce a buffer"))
}

pub fn buffer_to_lines(buffer: &Buffer) -> Vec<Line<'static>> {
    let mut out = Vec::with_capacity(buffer.height as usize);
    for y in 0..buffer.height {
        let mut spans = Vec::with_capacity(buffer.width as usize);
        for x in 0..buffer.width {
            if let Some(cell) = buffer.get(x, y) {
                let symbol = if cell.symbol == '\0' {
                    ' '
                } else {
                    cell.symbol
                };
                let style = Style::default()
                    .fg(to_ratatui_color(cell.fg))
                    .bg(to_ratatui_color(cell.bg));
                spans.push(Span::styled(symbol.to_string(), style));
            }
        }
        out.push(Line::from(spans));
    }
    out
}

fn to_ratatui_color(color: engine_core::color::Color) -> Color {
    match color {
        engine_core::color::Color::Reset => Color::Reset,
        engine_core::color::Color::Black => Color::Black,
        engine_core::color::Color::DarkGrey => Color::DarkGray,
        engine_core::color::Color::Red => Color::Red,
        engine_core::color::Color::DarkRed => Color::LightRed,
        engine_core::color::Color::Green => Color::Green,
        engine_core::color::Color::DarkGreen => Color::LightGreen,
        engine_core::color::Color::Yellow => Color::Yellow,
        engine_core::color::Color::DarkYellow => Color::LightYellow,
        engine_core::color::Color::Blue => Color::Blue,
        engine_core::color::Color::DarkBlue => Color::LightBlue,
        engine_core::color::Color::Magenta => Color::Magenta,
        engine_core::color::Color::DarkMagenta => Color::LightMagenta,
        engine_core::color::Color::Cyan => Color::Cyan,
        engine_core::color::Color::DarkCyan => Color::LightCyan,
        engine_core::color::Color::White => Color::White,
        engine_core::color::Color::Grey => Color::Gray,
        engine_core::color::Color::Rgb { r, g, b } => Color::Rgb(r, g, b),
        engine_core::color::Color::AnsiValue(v) => Color::Indexed(v),
    }
}

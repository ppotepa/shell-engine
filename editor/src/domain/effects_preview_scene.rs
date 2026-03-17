use std::path::PathBuf;

use engine_core::effects::shared_dispatcher;
use engine_core::scene::{Easing, EffectParams, EffectTargetKind};

const PREVIEW_TEMPLATE_SPRITE: &str = include_str!("../../assets/effects-preview.scene.yml");
const PREVIEW_TEMPLATE_SCENE: &str = include_str!("../../assets/effects-preview-scene.scene.yml");
pub const PREVIEW_DURATION_MS: u64 = 1_600;
const DEFAULT_VIEWPORT_W: u16 = 32;
const DEFAULT_VIEWPORT_H: u16 = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewPlacement {
    Scene,
    Layer,
    PenguinSprite,
    CaptionSprite,
}

impl PreviewPlacement {
    /// Ambient effects (scene/layer placement) don't need a subject sprite.
    pub fn is_ambient(self) -> bool {
        matches!(self, PreviewPlacement::Scene | PreviewPlacement::Layer)
    }
}

pub fn build_preview_scene_yaml(
    effect_name: &str,
    params: &EffectParams,
    viewport_w: u16,
    viewport_h: u16,
) -> String {
    let placement = choose_preview_placement(effect_name);
    let effect_yaml = render_effect_yaml(effect_name, params, placement.target_kind());

    if placement.is_ambient() {
        // Scene-level or layer-level effects: full-screen template, no sprite subject.
        PREVIEW_TEMPLATE_SCENE
            .replace(
                "__SCENE_STAGES__",
                &render_stages_block("  ", placement == PreviewPlacement::Scene, &effect_yaml),
            )
            .replace(
                "__LAYER_STAGES__",
                &render_stages_block("      ", placement == PreviewPlacement::Layer, &effect_yaml),
            )
    } else {
        // Sprite-level effects: show penguin as the subject.
        // The scene uses rendered-mode: halfblock. In halfblock mode, composite_scene_halfblock
        // renders into a scratch buffer of height*2 then packs 2 virtual rows into 1 terminal
        // cell. So YAML `height: H` results in H/2 terminal rows after packing, and `y: Y` is
        // in virtual pixel units (2× terminal cells). We multiply heights and y-offsets by 2
        // here to produce the correct terminal-cell coverage from the cell-unit layout values.
        let layout = PreviewLayout::for_viewport(viewport_w, viewport_h);
        PREVIEW_TEMPLATE_SPRITE
            .replace("__PENGUIN_WIDTH__", &layout.penguin_width.to_string())
            .replace(
                "__PENGUIN_HEIGHT__",
                &(layout.penguin_height * 2).to_string(),
            )
            .replace(
                "__PENGUIN_OFFSET_Y__",
                &(layout.penguin_offset_y * 2).to_string(),
            )
            .replace(
                "__CAPTION_OFFSET_Y__",
                &(layout.caption_offset_y * 2).to_string(),
            )
            .replace(
                "__SCENE_STAGES__",
                &render_stages_block("  ", false, &effect_yaml),
            )
            .replace(
                "__LAYER_STAGES__",
                &render_stages_block("      ", false, &effect_yaml),
            )
            .replace(
                "__PENGUIN_STAGES__",
                &render_stages_block(
                    "          ",
                    placement == PreviewPlacement::PenguinSprite,
                    &effect_yaml,
                ),
            )
            .replace(
                "__CAPTION_STAGES__",
                &render_stages_block(
                    "          ",
                    placement == PreviewPlacement::CaptionSprite,
                    &effect_yaml,
                ),
            )
    }
}

pub fn build_preview_scene_yaml_default(effect_name: &str, params: &EffectParams) -> String {
    build_preview_scene_yaml(effect_name, params, DEFAULT_VIEWPORT_W, DEFAULT_VIEWPORT_H)
}

pub fn preview_asset_root() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../mods/shell-quest"),
        PathBuf::from("mods/shell-quest"),
    ];

    candidates
        .into_iter()
        .find(|path| path.join("assets/images/tux.png").exists() && path.join("mod.yaml").exists())
}

pub fn choose_preview_placement(effect_name: &str) -> PreviewPlacement {
    let meta = shared_dispatcher().metadata(effect_name);
    let targets = meta.compatible_targets;
    if targets.supports(EffectTargetKind::SpriteBitmap) {
        PreviewPlacement::PenguinSprite
    } else if targets.supports(EffectTargetKind::SpriteText) {
        PreviewPlacement::CaptionSprite
    } else if targets.supports(EffectTargetKind::Sprite) {
        PreviewPlacement::PenguinSprite
    } else if targets.supports(EffectTargetKind::Layer) {
        PreviewPlacement::Layer
    } else {
        PreviewPlacement::Scene
    }
}

impl PreviewPlacement {
    const fn target_kind(self) -> EffectTargetKind {
        match self {
            PreviewPlacement::Scene => EffectTargetKind::Scene,
            PreviewPlacement::Layer => EffectTargetKind::Layer,
            PreviewPlacement::PenguinSprite => EffectTargetKind::SpriteBitmap,
            PreviewPlacement::CaptionSprite => EffectTargetKind::SpriteText,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PreviewLayout {
    penguin_width: u16,
    penguin_height: u16,
    penguin_offset_y: i16,
    caption_offset_y: i16,
}

impl PreviewLayout {
    fn for_viewport(viewport_w: u16, viewport_h: u16) -> Self {
        let viewport_w = viewport_w.max(12);
        let viewport_h = viewport_h.max(8);

        // Calculate dimensions in terminal CELL units.
        // Leave 3 rows for caption + top/bottom margin.
        let max_cell_h = viewport_h.saturating_sub(3).max(4);
        let max_cell_w = viewport_w.saturating_sub(4).max(8);

        // Tux penguin pixel aspect is roughly 2:3 (w:h). In halfblock mode each terminal cell
        // covers 1 column × 2 pixel rows, so natural cell ratio is w_cells / h_cells ≈ 1.4.
        let desired_cell_w = ((max_cell_h as f32) * 1.4).round() as u16;
        let cell_w = desired_cell_w.min(max_cell_w);
        let cell_h = ((cell_w as f32) / 1.4).round() as u16;
        let cell_h = cell_h.max(4).min(max_cell_h);

        let free_vertical = viewport_h.saturating_sub(cell_h);
        let desired_gap = (free_vertical / 3).max(1);
        let caption_offset_y = ((cell_h / 2) + desired_gap) as i16;
        let penguin_offset_y = -((desired_gap as i16) / 2);

        Self {
            penguin_width: cell_w,
            penguin_height: cell_h,
            penguin_offset_y,
            caption_offset_y,
        }
    }
}

fn render_stages_block(indent: &str, include_effect: bool, effect_yaml: &str) -> String {
    let next = format!("{indent}  ");
    if include_effect {
        let effect_indent = format!("{next}      ");
        let effect_block = indent_block(effect_yaml, &effect_indent);
        format!(
            "{indent}on_idle:\n{next}looping: true\n{next}steps:\n{next}  - duration: {PREVIEW_DURATION_MS}\n{next}    effects:\n{effect_block}"
        )
    } else {
        format!("{indent}on_idle:\n{next}looping: true\n{next}steps: []")
    }
}

fn render_effect_yaml(
    effect_name: &str,
    params: &EffectParams,
    target_kind: EffectTargetKind,
) -> String {
    let mut lines = vec![
        format!("- name: {effect_name}"),
        format!("  duration: {PREVIEW_DURATION_MS}"),
        format!("  target_kind: {}", target_kind_name(target_kind)),
    ];

    let param_lines = render_params_yaml(params);
    if !param_lines.is_empty() {
        lines.push("  params:".to_string());
        for line in param_lines {
            lines.push(format!("    {line}"));
        }
    }

    lines.join("\n")
}

fn render_params_yaml(params: &EffectParams) -> Vec<String> {
    let mut lines = Vec::new();

    push_number(&mut lines, "angle", params.angle);
    push_number(&mut lines, "width", params.width);
    push_number(&mut lines, "falloff", params.falloff);
    push_number(&mut lines, "intensity", params.intensity);
    push_number(&mut lines, "amplitude_x", params.amplitude_x);
    push_number(&mut lines, "amplitude_y", params.amplitude_y);
    push_number(&mut lines, "frequency", params.frequency);
    push_number(&mut lines, "thickness", params.thickness);
    push_number(&mut lines, "amp_start", params.amp_start);
    push_number(&mut lines, "amp_coeff", params.amp_coeff);
    push_number(&mut lines, "freq_coeff", params.freq_coeff);
    push_number(&mut lines, "speed", params.speed);
    push_u16(&mut lines, "strikes", params.strikes);
    push_u8(&mut lines, "octave_count", params.octave_count);
    push_bool(&mut lines, "glow", params.glow);
    push_string(&mut lines, "coverage", params.coverage.as_deref());
    push_string(&mut lines, "orientation", params.orientation.as_deref());
    push_string(&mut lines, "target", params.target.as_deref());
    push_string(&mut lines, "start_x", params.start_x.as_deref());
    push_string(&mut lines, "end_x", params.end_x.as_deref());

    if !matches!(params.easing, Easing::Linear) {
        lines.push(format!("easing: {}", easing_name(&params.easing)));
    }

    if let Some(colour) = params.colour.as_ref() {
        lines.push(format!("colour: {}", colour_name(colour)));
    }

    lines
}

fn push_number(lines: &mut Vec<String>, key: &str, value: Option<f32>) {
    if let Some(value) = value {
        lines.push(format!("{key}: {}", trim_float(value)));
    }
}

fn push_u16(lines: &mut Vec<String>, key: &str, value: Option<u16>) {
    if let Some(value) = value {
        lines.push(format!("{key}: {value}"));
    }
}

fn push_u8(lines: &mut Vec<String>, key: &str, value: Option<u8>) {
    if let Some(value) = value {
        lines.push(format!("{key}: {value}"));
    }
}

fn push_bool(lines: &mut Vec<String>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        lines.push(format!("{key}: {value}"));
    }
}

fn push_string(lines: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        lines.push(format!("{key}: {value:?}"));
    }
}

fn indent_block(block: &str, indent: &str) -> String {
    block
        .lines()
        .map(|line| format!("{indent}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn trim_float(value: f32) -> String {
    let mut rendered = format!("{value:.3}");
    while rendered.contains('.') && rendered.ends_with('0') {
        rendered.pop();
    }
    if rendered.ends_with('.') {
        rendered.pop();
    }
    rendered
}

fn target_kind_name(target_kind: EffectTargetKind) -> &'static str {
    match target_kind {
        EffectTargetKind::Any => "any",
        EffectTargetKind::Scene => "scene",
        EffectTargetKind::Layer => "layer",
        EffectTargetKind::Sprite => "sprite",
        EffectTargetKind::SpriteText => "sprite_text",
        EffectTargetKind::SpriteBitmap => "sprite_bitmap",
    }
}

fn easing_name(easing: &Easing) -> &'static str {
    match easing {
        Easing::Linear => "linear",
        Easing::EaseIn => "ease-in",
        Easing::EaseOut => "ease-out",
        Easing::EaseInOut => "ease-in-out",
    }
}

fn colour_name(colour: &engine_core::scene::TermColour) -> String {
    match colour {
        engine_core::scene::TermColour::Black => "black".to_string(),
        engine_core::scene::TermColour::White => "white".to_string(),
        engine_core::scene::TermColour::Silver => "silver".to_string(),
        engine_core::scene::TermColour::Gray => "gray".to_string(),
        engine_core::scene::TermColour::Red => "red".to_string(),
        engine_core::scene::TermColour::Green => "green".to_string(),
        engine_core::scene::TermColour::Blue => "blue".to_string(),
        engine_core::scene::TermColour::Yellow => "yellow".to_string(),
        engine_core::scene::TermColour::Cyan => "cyan".to_string(),
        engine_core::scene::TermColour::Magenta => "magenta".to_string(),
        engine_core::scene::TermColour::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_preview_scene_yaml, choose_preview_placement, PreviewLayout, PreviewPlacement,
    };
    use crate::domain::effect_params;

    #[test]
    fn generated_preview_yaml_parses_into_scene() {
        let params = effect_params::default_effect_params("shine");
        let yaml = build_preview_scene_yaml("shine", &params, 32, 18);
        let scene: engine::scene::Scene = serde_yaml::from_str(&yaml).expect("preview scene");
        assert_eq!(scene.id, "effect_preview");
        assert_eq!(scene.layers.len(), 1);
    }

    #[test]
    fn prefers_bitmap_sprite_for_bitmap_capable_effects() {
        assert_eq!(
            choose_preview_placement("shine"),
            PreviewPlacement::PenguinSprite
        );
    }

    #[test]
    fn layout_scales_down_for_small_viewports() {
        let small = PreviewLayout::for_viewport(12, 8);
        let large = PreviewLayout::for_viewport(40, 20);
        assert!(small.penguin_width < large.penguin_width);
        assert!(small.penguin_height < large.penguin_height);
    }
}

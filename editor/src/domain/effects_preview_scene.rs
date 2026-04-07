use std::path::PathBuf;

use serde::Serialize;

use engine_effects::shared_dispatcher;
use engine_core::scene::{Easing, EffectParams, EffectTargetKind};

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

#[derive(Debug, Clone, Serialize)]
struct PreviewSceneDoc {
    id: String,
    title: String,
    cutscene: bool,
    #[serde(rename = "rendered-mode")]
    rendered_mode: String,
    bg_colour: String,
    stages: PreviewStages,
    layers: Vec<PreviewLayer>,
    next: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PreviewStages {
    on_idle: PreviewStage,
}

#[derive(Debug, Clone, Serialize)]
struct PreviewStage {
    looping: bool,
    steps: Vec<PreviewStep>,
}

#[derive(Debug, Clone, Serialize)]
struct PreviewStep {
    duration: u64,
    effects: Vec<PreviewEffect>,
}

#[derive(Debug, Clone, Serialize)]
struct PreviewEffect {
    name: String,
    duration: u64,
    target_kind: PreviewEffectTargetKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<PreviewEffectParams>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum PreviewEffectTargetKind {
    Any,
    Scene,
    Layer,
    Sprite,
    SpriteText,
    SpriteBitmap,
}

#[derive(Debug, Clone, Serialize)]
struct PreviewEffectParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    angle: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    falloff: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    intensity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sphericality: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transparency: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    brightness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amplitude_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amplitude_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thickness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amp_start: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amp_coeff: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    freq_coeff: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strikes: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    octave_count: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    glow: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    orientation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    easing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PreviewLayer {
    name: String,
    z_index: i32,
    visible: bool,
    stages: PreviewStages,
    sprites: Vec<PreviewSprite>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum PreviewSprite {
    Image {
        id: String,
        source: String,
        width: u16,
        height: u16,
        align_x: String,
        align_y: String,
        y: i32,
        stages: PreviewStages,
    },
    Text {
        id: String,
        content: String,
        align_x: String,
        align_y: String,
        y: i32,
        fg_colour: String,
        stages: PreviewStages,
    },
}

pub fn build_preview_scene_yaml(
    effect_name: &str,
    params: &EffectParams,
    viewport_w: u16,
    viewport_h: u16,
) -> String {
    let doc = build_preview_scene_doc(effect_name, params, viewport_w, viewport_h);
    serde_yaml::to_string(&doc).expect("serializing generated effect preview scene should succeed")
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

fn build_preview_scene_doc(
    effect_name: &str,
    params: &EffectParams,
    viewport_w: u16,
    viewport_h: u16,
) -> PreviewSceneDoc {
    let placement = choose_preview_placement(effect_name);
    let effect = build_effect_value(effect_name, params, placement.target_kind());
    PreviewSceneDoc {
        id: String::from("effect_preview"),
        title: String::from("Effect Preview"),
        cutscene: false,
        rendered_mode: String::from("halfblock"),
        bg_colour: String::from("black"),
        stages: PreviewStages {
            on_idle: build_stage(placement == PreviewPlacement::Scene, &effect),
        },
        layers: vec![build_preview_layer(
            placement, viewport_w, viewport_h, effect,
        )],
        next: None,
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

fn build_preview_layer(
    placement: PreviewPlacement,
    viewport_w: u16,
    viewport_h: u16,
    effect: PreviewEffect,
) -> PreviewLayer {
    let sprites = if placement.is_ambient() {
        Vec::new()
    } else {
        // Sprite-level effects: show penguin as subject.
        // In halfblock mode height and y-offset are in virtual row units (2x terminal rows).
        let layout = PreviewLayout::for_viewport(viewport_w, viewport_h);
        vec![
            PreviewSprite::Image {
                id: String::from("penguin"),
                source: String::from("/assets/images/tux.png"),
                width: layout.penguin_width,
                height: layout.penguin_height * 2,
                align_x: String::from("center"),
                align_y: String::from("center"),
                y: (layout.penguin_offset_y * 2) as i32,
                stages: PreviewStages {
                    on_idle: build_stage(placement == PreviewPlacement::PenguinSprite, &effect),
                },
            },
            PreviewSprite::Text {
                id: String::from("caption"),
                content: String::from("SHELL QUEST"),
                align_x: String::from("center"),
                align_y: String::from("center"),
                y: (layout.caption_offset_y * 2) as i32,
                fg_colour: String::from("white"),
                stages: PreviewStages {
                    on_idle: build_stage(placement == PreviewPlacement::CaptionSprite, &effect),
                },
            },
        ]
    };

    PreviewLayer {
        name: String::from("preview"),
        z_index: 0,
        visible: true,
        stages: PreviewStages {
            on_idle: build_stage(placement == PreviewPlacement::Layer, &effect),
        },
        sprites,
    }
}

fn build_stage(include_effect: bool, effect: &PreviewEffect) -> PreviewStage {
    let steps = if include_effect {
        vec![PreviewStep {
            duration: PREVIEW_DURATION_MS,
            effects: vec![effect.clone()],
        }]
    } else {
        Vec::new()
    };

    PreviewStage {
        looping: true,
        steps,
    }
}

fn build_effect_value(
    effect_name: &str,
    params: &EffectParams,
    target_kind: EffectTargetKind,
) -> PreviewEffect {
    PreviewEffect {
        name: effect_name.to_string(),
        duration: PREVIEW_DURATION_MS,
        target_kind: preview_target_kind(target_kind),
        params: render_params_yaml(params),
    }
}

fn preview_target_kind(target_kind: EffectTargetKind) -> PreviewEffectTargetKind {
    match target_kind {
        EffectTargetKind::Any => PreviewEffectTargetKind::Any,
        EffectTargetKind::Scene => PreviewEffectTargetKind::Scene,
        EffectTargetKind::Layer => PreviewEffectTargetKind::Layer,
        EffectTargetKind::Sprite => PreviewEffectTargetKind::Sprite,
        EffectTargetKind::SpriteText => PreviewEffectTargetKind::SpriteText,
        EffectTargetKind::SpriteBitmap => PreviewEffectTargetKind::SpriteBitmap,
    }
}

fn render_params_yaml(params: &EffectParams) -> Option<PreviewEffectParams> {
    let rendered = PreviewEffectParams {
        angle: params.angle,
        width: params.width,
        falloff: params.falloff,
        intensity: params.intensity,
        sphericality: params.sphericality,
        transparency: params.transparency,
        brightness: params.brightness,
        amplitude_x: params.amplitude_x,
        amplitude_y: params.amplitude_y,
        frequency: params.frequency,
        thickness: params.thickness,
        amp_start: params.amp_start,
        amp_coeff: params.amp_coeff,
        freq_coeff: params.freq_coeff,
        speed: params.speed,
        strikes: params.strikes,
        octave_count: params.octave_count,
        glow: params.glow,
        coverage: params.coverage.clone(),
        orientation: params.orientation.clone(),
        target: params.target.clone(),
        start_x: params.start_x.clone(),
        end_x: params.end_x.clone(),
        easing: if matches!(params.easing, Easing::Linear) {
            None
        } else {
            Some(easing_name(&params.easing).to_string())
        },
        colour: params.colour.as_ref().map(colour_name),
    };

    if rendered.is_empty() {
        None
    } else {
        Some(rendered)
    }
}

impl PreviewEffectParams {
    fn is_empty(&self) -> bool {
        self.angle.is_none()
            && self.width.is_none()
            && self.falloff.is_none()
            && self.intensity.is_none()
            && self.sphericality.is_none()
            && self.transparency.is_none()
            && self.brightness.is_none()
            && self.amplitude_x.is_none()
            && self.amplitude_y.is_none()
            && self.frequency.is_none()
            && self.thickness.is_none()
            && self.amp_start.is_none()
            && self.amp_coeff.is_none()
            && self.freq_coeff.is_none()
            && self.speed.is_none()
            && self.strikes.is_none()
            && self.octave_count.is_none()
            && self.glow.is_none()
            && self.coverage.is_none()
            && self.orientation.is_none()
            && self.target.is_none()
            && self.start_x.is_none()
            && self.end_x.is_none()
            && self.easing.is_none()
            && self.colour.is_none()
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
        engine_core::scene::TermColour::Black => String::from("black"),
        engine_core::scene::TermColour::White => String::from("white"),
        engine_core::scene::TermColour::Silver => String::from("silver"),
        engine_core::scene::TermColour::Gray => String::from("gray"),
        engine_core::scene::TermColour::Red => String::from("red"),
        engine_core::scene::TermColour::Green => String::from("green"),
        engine_core::scene::TermColour::Blue => String::from("blue"),
        engine_core::scene::TermColour::Yellow => String::from("yellow"),
        engine_core::scene::TermColour::Cyan => String::from("cyan"),
        engine_core::scene::TermColour::Magenta => String::from("magenta"),
        engine_core::scene::TermColour::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_preview_scene_yaml, choose_preview_placement, PreviewLayout, PreviewPlacement,
        PREVIEW_DURATION_MS,
    };
    use crate::domain::effect_params;
    use engine::scene::Sprite;

    #[test]
    fn generated_preview_yaml_parses_into_scene() {
        let params = effect_params::default_effect_params("shine");
        let yaml = build_preview_scene_yaml("shine", &params, 32, 18);
        let scene: engine::scene::Scene = serde_yaml::from_str(&yaml).expect("preview scene");
        assert_eq!(scene.id, "effect_preview");
        assert_eq!(scene.layers.len(), 1);
        assert_eq!(scene.stages.on_idle.steps.len(), 0);

        let layer = &scene.layers[0];
        assert_eq!(layer.sprites.len(), 2);

        let effect = match &layer.sprites[0] {
            Sprite::Image { stages, .. } => {
                assert_eq!(stages.on_idle.steps.len(), 1);
                &stages.on_idle.steps[0].effects[0]
            }
            other => panic!("expected image sprite, got {other:?}"),
        };
        assert_eq!(effect.name, "shine");
        assert_eq!(effect.duration, PREVIEW_DURATION_MS);
        assert_eq!(
            effect.target_kind,
            engine::scene::EffectTargetKind::SpriteBitmap
        );
        assert_eq!(effect.params.angle, Some(22.0));
        assert_eq!(effect.params.width, Some(5.0));
        assert_eq!(effect.params.falloff, Some(1.2));
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

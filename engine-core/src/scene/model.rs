//! Typed runtime scene model produced by scene compilation.
//!
//! These types represent the post-normalization boundary consumed by the scene
//! runtime, compositor, and behavior systems.

use super::color::TermColour;
use super::easing::Easing;
use super::sprite::Sprite;
use crate::animations::AnimationParams;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
pub enum SceneSpace {
    #[default]
    #[serde(rename = "2d")]
    TwoD,
    #[serde(rename = "3d")]
    ThreeD,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LayerSpace {
    #[default]
    Inherit,
    #[serde(rename = "2d")]
    TwoD,
    #[serde(rename = "3d")]
    ThreeD,
    Screen,
}

/// Runtime animation attached to a sprite after authored scene parsing.
///
/// Animations modify sprite transform and presentation, not the pixel data
/// inside rendered assets.
#[derive(Debug, Clone, Deserialize)]
pub struct Animation {
    pub name: String,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub params: AnimationParams,
}

/// Named runtime behavior attached to a scene, layer, or sprite after authored
/// scene compilation.
#[derive(Debug, Clone, Deserialize)]
pub struct BehaviorSpec {
    pub name: String,
    #[serde(default)]
    pub params: BehaviorParams,
}

/// Shared parameter bag for authored behavior declarations that survive into
/// the runtime scene model.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BehaviorParams {
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub stages: Vec<String>,
    #[serde(default)]
    pub time_scope: Option<String>,
    #[serde(default)]
    pub start_ms: Option<u64>,
    #[serde(default)]
    pub end_ms: Option<u64>,
    #[serde(default)]
    pub visible_ms: Option<u64>,
    #[serde(default)]
    pub hidden_ms: Option<u64>,
    #[serde(default)]
    pub period_ms: Option<u64>,
    #[serde(default)]
    pub phase_ms: Option<u64>,
    #[serde(default)]
    pub amplitude_x: Option<i32>,
    #[serde(default)]
    pub amplitude_y: Option<i32>,
    #[serde(default)]
    pub index: Option<usize>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub padding: Option<i32>,
    #[serde(default)]
    pub autoscale_height: Option<bool>,
    #[serde(default)]
    pub count: Option<usize>,
    #[serde(default)]
    pub window: Option<usize>,
    #[serde(default)]
    pub step_y: Option<i32>,
    #[serde(default)]
    pub endless: Option<bool>,
    #[serde(default)]
    pub item_prefix: Option<String>,
    #[serde(default)]
    pub src: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    /// Generic duration in milliseconds — exposed to Rhai scripts as `params.dur`.
    #[serde(default)]
    pub dur: Option<u64>,
}

/// Named visual effect with duration, loop flag, and arbitrary params.
#[derive(Debug, Clone, Deserialize)]
pub struct Effect {
    pub name: String,
    #[serde(default)]
    pub duration: u64,
    #[serde(default, rename = "loop")]
    pub looping: bool,
    #[serde(default)]
    pub target_kind: EffectTargetKind,
    #[serde(default)]
    pub params: EffectParams,
}

/// Declares which kind of render target an effect is authored for.
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectTargetKind {
    #[default]
    Any,
    Scene,
    Layer,
    Sprite,
    SpriteText,
    SpriteBitmap,
}

impl EffectTargetKind {
    pub const fn matches_effective(self, effective: Self) -> bool {
        match self {
            Self::Any => true,
            Self::Scene => matches!(effective, Self::Scene),
            Self::Layer => matches!(effective, Self::Layer),
            Self::Sprite => matches!(
                effective,
                Self::Sprite | Self::SpriteText | Self::SpriteBitmap
            ),
            Self::SpriteText => matches!(effective, Self::SpriteText),
            Self::SpriteBitmap => matches!(effective, Self::SpriteBitmap),
        }
    }
}

/// Optional named parameters for effect configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EffectParams {
    pub colour: Option<TermColour>,
    #[serde(default)]
    pub easing: Easing,
    /// Shine: scan angle in degrees (0 = vertical left→right, 90 = horizontal top→down).
    #[serde(default)]
    pub angle: Option<f32>,
    /// Shine: beam width (gaussian sigma in character cells).
    #[serde(default)]
    pub width: Option<f32>,
    /// Shine: falloff exponent — higher = sharper beam edges (default 1.0).
    #[serde(default)]
    pub falloff: Option<f32>,
    /// Shine: peak brightness multiplier 0.0–1.0 (default 1.0 = full white).
    #[serde(default)]
    pub intensity: Option<f32>,
    /// CRT reflection: curved-glass warp strength.
    #[serde(default)]
    pub sphericality: Option<f32>,
    /// CRT reflection: how strongly the reflected image remains visible.
    #[serde(default)]
    pub transparency: Option<f32>,
    /// CRT reflection: brightness multiplier applied to the reflected sample.
    #[serde(default)]
    pub brightness: Option<f32>,
    /// Generic postfx alpha / blend amount (0..1).
    #[serde(default)]
    pub alpha: Option<f32>,
    /// Generic postfx distortion strength (0..1).
    #[serde(default)]
    pub distortion: Option<f32>,
    /// Screen shake horizontal amplitude in cells.
    #[serde(default)]
    pub amplitude_x: Option<f32>,
    /// Screen shake vertical amplitude in cells.
    #[serde(default)]
    pub amplitude_y: Option<f32>,
    /// Shake oscillation frequency (cycles during effect duration).
    #[serde(default)]
    pub frequency: Option<f32>,
    /// Optional hint for fullscreen scope.
    #[serde(default)]
    pub coverage: Option<String>,
    /// Optional orientation for directional effects such as lightning bands.
    #[serde(default)]
    pub orientation: Option<String>,
    /// Optional target id/name hint for future targeted effects.
    #[serde(default)]
    pub target: Option<String>,
    /// Lightning branches: number of primary strikes.
    #[serde(default)]
    pub strikes: Option<u16>,
    /// Lightning branch thickness multiplier.
    #[serde(default)]
    pub thickness: Option<f32>,
    /// Lightning branch glow halo toggle.
    #[serde(default)]
    pub glow: Option<bool>,
    /// Lightning branch start anchor; accepts numeric string or "random".
    #[serde(default)]
    pub start_x: Option<String>,
    /// Lightning branch end anchor; accepts numeric string or "random".
    #[serde(default)]
    pub end_x: Option<String>,
    /// Shader-like FBM octave count (for procedural lightning variants).
    #[serde(default)]
    pub octave_count: Option<u8>,
    /// FBM starting amplitude.
    #[serde(default)]
    pub amp_start: Option<f32>,
    /// FBM amplitude attenuation per octave.
    #[serde(default)]
    pub amp_coeff: Option<f32>,
    /// FBM frequency multiplier per octave.
    #[serde(default)]
    pub freq_coeff: Option<f32>,
    /// FBM animation speed multiplier.
    #[serde(default)]
    pub speed: Option<f32>,
    /// Blur kernel half-size in cells. Used by: blur.
    #[serde(default)]
    pub radius: Option<f32>,
    /// Posterize quantization level count per RGB channel. Used by: posterize.
    #[serde(default)]
    pub levels: Option<u8>,
    /// Cutout: pre-simplify radius (cells)
    #[serde(default)]
    pub simplify: Option<f32>,
    /// Cutout: edge detection strength (0..1)
    #[serde(default)]
    pub edge_strength: Option<f32>,
    /// Cutout: edge detection fidelity/threshold (0..1)
    #[serde(default)]
    pub edge_fidelity: Option<f32>,
    /// Cutout: outline width in cells
    #[serde(default)]
    pub edge_width: Option<u8>,
    /// Cutout: saturation multiplier
    #[serde(default)]
    pub saturation: Option<f32>,
    /// Cutout blend mode: replace or overlay. Used by: cutout.
    #[serde(default)]
    pub blend_mode: Option<String>,
    /// CRT burn-in: brightness pump multiplier on first frame of transition.
    #[serde(default)]
    pub pump: Option<f32>,
    /// CRT burn-in: phosphor colour decay tint (0 = uniform, 1 = full P31 green shift).
    #[serde(default)]
    pub decay_tint: Option<f32>,
    /// Lens blur: number of convolution passes (1–4). Each pass doubles effective spread.
    #[serde(default)]
    pub passes: Option<f32>,
}

/// A single step in a stage — a group of effects that play in parallel.
/// Steps within a stage execute sequentially.
#[derive(Debug, Clone, Deserialize)]
pub struct Step {
    #[serde(default)]
    pub effects: Vec<Effect>,
    /// Optional minimum duration for the step (ms), regardless of effects.
    #[serde(default)]
    pub duration: Option<u64>,
}

impl Step {
    /// Duration of this step = max of explicit duration and max effect duration.
    pub fn duration_ms(&self) -> u64 {
        let effect_dur = self.effects.iter().map(|e| e.duration).max().unwrap_or(0);
        self.duration.unwrap_or(0).max(effect_dur)
    }
}

/// What causes transition from on_idle to on_leave.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StageTrigger {
    AnyKey,
    Timeout,
    #[default]
    None,
}

/// A lifecycle stage containing sequential steps.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Stage {
    #[serde(default)]
    pub trigger: StageTrigger,
    #[serde(default)]
    pub steps: Vec<Step>,
    /// If true, stage loops back to step 0 after all steps complete.
    #[serde(default)]
    pub looping: bool,
}

impl Stage {
    /// Total duration of this stage = sum of all step durations.
    pub fn duration_ms(&self) -> u64 {
        self.steps.iter().map(|step| step.duration_ms()).sum()
    }
}

/// Scene-level lifecycle stages materialized from authored stage YAML.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SceneStages {
    #[serde(default)]
    pub on_enter: Stage,
    #[serde(default)]
    pub on_idle: Stage,
    #[serde(default)]
    pub on_leave: Stage,
}

/// Scene-level audio cues keyed by lifecycle stage in the runtime scene model.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SceneAudio {
    #[serde(default)]
    pub on_enter: Vec<AudioCue>,
    #[serde(default)]
    pub on_idle: Vec<AudioCue>,
    #[serde(default)]
    pub on_leave: Vec<AudioCue>,
}

/// Authored menu option materialized into a runtime scene transition target.
#[derive(Debug, Clone, Deserialize)]
pub struct MenuOption {
    pub key: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub scene: Option<String>,
    pub next: String,
}

/// Runtime input profiles declared on a scene for specialized interactive modes.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SceneInput {
    /// Optional OBJ viewer controls profile.
    #[serde(default, rename = "obj-viewer")]
    pub obj_viewer: Option<ObjViewerControls>,
    /// Optional free-look scene camera controls profile.
    #[serde(default, rename = "free-look-camera", alias = "free_look_camera")]
    pub free_look_camera: Option<FreeLookCameraControls>,
    /// Optional orbit camera controls — Ctrl+F orbits a target OBJ sprite.
    #[serde(default, rename = "orbit-camera", alias = "orbit_camera")]
    pub orbit_camera: Option<ObjOrbitCameraControls>,
}

impl SceneInput {
    /// Returns names of all built-in input profiles.
    pub fn builtin_profiles() -> Vec<&'static str> {
        vec!["obj-viewer", "free-look-camera", "orbit-camera"]
    }
}

fn default_orbit_pitch() -> f32 {
    -30.0
}
fn default_orbit_distance() -> f32 {
    3.0
}
fn default_orbit_pitch_min() -> f32 {
    -85.0
}
fn default_orbit_pitch_max() -> f32 {
    -5.0
}
fn default_orbit_distance_min() -> f32 {
    0.5
}
fn default_orbit_distance_max() -> f32 {
    8.0
}
fn default_orbit_distance_step() -> f32 {
    0.25
}
fn default_orbit_drag_sensitivity() -> f32 {
    0.5
}

/// Declarative orbit-camera controls for a single OBJ sprite target.
///
/// Ctrl+F toggles orbit mode. Left-drag rotates; `+`/`-` zoom.
/// Declared under `input.orbit-camera:` in scene YAML.
#[derive(Debug, Clone, Deserialize)]
pub struct ObjOrbitCameraControls {
    /// Sprite ID to orbit around.
    pub target: String,
    /// Initial yaw in degrees (default: 0.0).
    #[serde(default)]
    pub yaw: f32,
    /// Initial pitch in degrees (default: -30.0).
    #[serde(default = "default_orbit_pitch")]
    pub pitch: f32,
    /// Initial camera distance (default: 3.0).
    #[serde(default = "default_orbit_distance")]
    pub distance: f32,
    /// Minimum pitch clamp (default: -85.0).
    #[serde(default = "default_orbit_pitch_min", rename = "pitch-min")]
    pub pitch_min: f32,
    /// Maximum pitch clamp (default: -5.0).
    #[serde(default = "default_orbit_pitch_max", rename = "pitch-max")]
    pub pitch_max: f32,
    /// Minimum camera distance (default: 0.5).
    #[serde(default = "default_orbit_distance_min", rename = "distance-min")]
    pub distance_min: f32,
    /// Maximum camera distance (default: 8.0).
    #[serde(default = "default_orbit_distance_max", rename = "distance-max")]
    pub distance_max: f32,
    /// Distance change per `+`/`-` key press (default: 0.25).
    #[serde(default = "default_orbit_distance_step", rename = "distance-step")]
    pub distance_step: f32,
    /// Mouse drag sensitivity — degrees per pixel (default: 0.5).
    #[serde(
        default = "default_orbit_drag_sensitivity",
        rename = "drag-sensitivity"
    )]
    pub drag_sensitivity: f32,
}

/// Whether authored UI should reset with scene lifecycle or persist globally.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum UiPersistence {
    #[default]
    Scene,
    Global,
}

fn default_ui_enabled() -> bool {
    true
}

/// Scene-level UI runtime contract.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SceneUi {
    #[serde(default = "default_ui_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub persist: UiPersistence,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default, rename = "focus-order", alias = "focus_order")]
    pub focus_order: Vec<String>,
}

impl Default for SceneUi {
    fn default() -> Self {
        Self {
            enabled: true,
            persist: UiPersistence::Scene,
            theme: None,
            focus_order: Vec::new(),
        }
    }
}

/// Which slice of the celestial hierarchy this scene intends to resolve.
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CelestialScope {
    #[default]
    Local,
    System,
    Region,
}

/// Coordinate frame used to interpret celestial rendering and gameplay.
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CelestialFrame {
    #[default]
    FocusRelative,
    Barycentric,
    SurfaceLocal,
}

/// Time source used for celestial pose resolution.
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CelestialClockSource {
    #[default]
    Scene,
    Campaign,
    Fixed,
}

/// Scene-level binding to the persistent celestial domain.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SceneCelestial {
    #[serde(default)]
    pub scope: CelestialScope,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default, rename = "focus-body", alias = "focus_body")]
    pub focus_body: Option<String>,
    #[serde(default, rename = "focus-site", alias = "focus_site")]
    pub focus_site: Option<String>,
    #[serde(default)]
    pub frame: CelestialFrame,
    #[serde(default, rename = "clock-source", alias = "clock_source")]
    pub clock_source: CelestialClockSource,
}

impl Default for SceneCelestial {
    fn default() -> Self {
        Self {
            scope: CelestialScope::Local,
            region: None,
            system: None,
            focus_body: None,
            focus_site: None,
            frame: CelestialFrame::FocusRelative,
            clock_source: CelestialClockSource::Scene,
        }
    }
}

/// Declarative OBJ viewer controls target.
#[derive(Debug, Clone, Deserialize)]
pub struct ObjViewerControls {
    /// ID of target OBJ sprite receiving controls.
    pub sprite_id: String,
}

fn default_free_look_move_speed() -> f32 {
    1.5
}

fn default_free_look_mouse_sensitivity() -> f32 {
    1.0
}

/// Declarative free-look scene camera controls.
#[derive(Debug, Clone, Deserialize)]
pub struct FreeLookCameraControls {
    /// Camera translation speed in scene-world units per second.
    #[serde(
        default = "default_free_look_move_speed",
        rename = "move-speed",
        alias = "move_speed"
    )]
    pub move_speed: f32,
    /// Mouse look sensitivity multiplier.
    #[serde(
        default = "default_free_look_mouse_sensitivity",
        rename = "mouse-sensitivity",
        alias = "mouse_sensitivity"
    )]
    pub mouse_sensitivity: f32,
}

impl Default for FreeLookCameraControls {
    fn default() -> Self {
        Self {
            move_speed: default_free_look_move_speed(),
            mouse_sensitivity: default_free_look_mouse_sensitivity(),
        }
    }
}

/// Audio cue descriptor (design hook only; playback is external).
#[derive(Debug, Clone, Deserialize)]
pub struct AudioCue {
    #[serde(default)]
    pub at_ms: u64,
    pub cue: String,
    #[serde(default)]
    pub volume: Option<f32>,
}

/// Layer-level lifecycle stages materialized from authored layer YAML.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LayerStages {
    #[serde(default)]
    pub on_enter: Stage,
    #[serde(default)]
    pub on_idle: Stage,
    #[serde(default)]
    pub on_leave: Stage,
}

/// Runtime compositing layer assembled from authored layer YAML and any object
/// expansion output.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Layer {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub z_index: i32,
    #[serde(default = "default_visible")]
    pub visible: bool,
    #[serde(default)]
    pub ui: bool,
    #[serde(default)]
    pub space: LayerSpace,
    #[serde(default)]
    pub stages: LayerStages,
    #[serde(default)]
    pub behaviors: Vec<BehaviorSpec>,
    #[serde(default)]
    pub sprites: Vec<Sprite>,
}

fn default_visible() -> bool {
    true
}

/// Runtime scene assembled from authored YAML, package fragments, templates,
/// and object expansion.
///
/// # YAML
///
/// A scene may originate from a single `scenes/*.yml` file or from a packaged
/// directory rooted at `scene.yml` plus `layers/`, `templates/`, and `objects/`
/// fragments merged before deserialization.
#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub cutscene: bool,
    #[serde(default, rename = "target-fps", alias = "target_fps")]
    pub target_fps: Option<u16>,
    #[serde(default)]
    pub space: SceneSpace,
    #[serde(default)]
    pub celestial: SceneCelestial,
    #[serde(default, rename = "virtual-size-override")]
    pub virtual_size_override: Option<String>,
    pub bg_colour: Option<TermColour>,
    #[serde(default)]
    pub stages: SceneStages,
    #[serde(default)]
    pub behaviors: Vec<BehaviorSpec>,
    #[serde(default)]
    pub audio: SceneAudio,
    #[serde(default)]
    pub ui: SceneUi,
    #[serde(default)]
    pub layers: Vec<Layer>,
    #[serde(default, alias = "menu_options", rename = "menu-options")]
    pub menu_options: Vec<MenuOption>,
    #[serde(default)]
    pub input: SceneInput,
    /// Shader-like software post-process passes applied after compositing.
    #[serde(default)]
    pub postfx: Vec<Effect>,
    pub next: Option<String>,
    /// When `true`, all static OBJ sprites in this scene are pre-rendered into
    /// `ObjFrameCache` synchronously during scene load (before the scene is activated).
    /// The compositor then uses cached frames instead of live 3D rendering.
    #[serde(default)]
    pub prerender: bool,
    /// Palette color bindings extracted from `@palette.<key>` syntax in YAML.
    /// Applied at runtime whenever the active palette changes.
    #[serde(default, skip_serializing)]
    pub palette_bindings: Vec<PaletteBinding>,
    /// Game state text bindings extracted from `@game_state.<path>` syntax in sprite `content` fields.
    /// Applied at runtime each frame so sprites reflect live game state.
    #[serde(default, skip_serializing)]
    pub game_state_bindings: Vec<GameStateBinding>,
    /// GUI widget declarations — logical widgets bound to visual sprite ids.
    /// engine-scene-runtime converts these into engine-gui GuiWidgetDef at runtime.
    #[serde(default)]
    pub gui: SceneGui,
}

/// A color binding between a sprite and a palette key, extracted from YAML `@palette.<key>` syntax.
///
/// When the scene runtime detects a palette change it replays all bindings as
/// `SetProperty` commands so the sprites reflect the current palette colors.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PaletteBinding {
    /// Sprite object id (runtime target).
    pub target: String,
    /// Property path, e.g. `"style.fg"` or `"style.bg"`.
    pub prop: String,
    /// Palette color key, e.g. `"hud_value"`.
    pub key: String,
}

/// A text content binding between a sprite and a `game_state` path, extracted from YAML
/// `@game_state.<path>` syntax in `content` fields.
///
/// Applied by the scene runtime each frame so that sprites always reflect live game state.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GameStateBinding {
    /// Sprite object id (runtime target).
    pub target: String,
    /// JSON pointer path into game_state (e.g. `"/score"`).
    pub path: String,
}

/// GUI widget declarations for a scene — author-facing serde types.
///
/// Converted to `engine-gui::GuiWidgetDef` by engine-scene-runtime at construction time.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SceneGui {
    #[serde(default)]
    pub widgets: Vec<SceneGuiWidgetDef>,
}

/// Author-facing GUI widget definition.  Mirrors `engine-gui::GuiWidgetDef` without
/// pulling that crate into engine-core's dependency tree.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SceneGuiWidgetDef {
    Slider {
        id: String,
        #[serde(default)]
        sprite: String,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default = "default_slider_w")]
        w: i32,
        #[serde(default = "default_slider_h")]
        h: i32,
        #[serde(default)]
        min: f64,
        #[serde(default = "default_slider_max")]
        max: f64,
        #[serde(default)]
        value: f64,
        #[serde(default, rename = "hit-padding")]
        hit_padding: i32,
        /// Sprite id of the slider handle/thumb — engine auto-positions it.
        #[serde(default)]
        handle: String,
    },
    Button {
        id: String,
        #[serde(default)]
        sprite: String,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default = "default_slider_w")]
        w: i32,
        #[serde(default = "default_slider_h")]
        h: i32,
    },
    Toggle {
        id: String,
        #[serde(default)]
        sprite: String,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default = "default_slider_w")]
        w: i32,
        #[serde(default = "default_slider_h")]
        h: i32,
        #[serde(default)]
        on: bool,
    },
    Panel {
        id: String,
        #[serde(default)]
        sprite: String,
        #[serde(default)]
        visible: bool,
    },
}

fn default_slider_w() -> i32 {
    120
}
fn default_slider_h() -> i32 {
    12
}
fn default_slider_max() -> f64 {
    1.0
}

impl Scene {
    /// Total duration of the on_enter stage in milliseconds.
    /// This is the primary cutscene/intro duration for most scenes.
    pub fn on_enter_duration_ms(&self) -> u64 {
        self.stages.on_enter.duration_ms()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CelestialClockSource, CelestialFrame, CelestialScope, Scene, SceneSpace, UiPersistence,
    };
    use crate::scene::Stage;

    #[test]
    fn parses_scene_target_fps_kebab_case() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: scene-a
title: A
target-fps: 30
layers: []
"#,
        )
        .expect("scene should parse");

        assert_eq!(scene.target_fps, Some(30));
    }

    #[test]
    fn parses_scene_target_fps_snake_case_alias() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: scene-b
title: B
target_fps: 24
layers: []
"#,
        )
        .expect("scene should parse");

        assert_eq!(scene.target_fps, Some(24));
    }

    #[test]
    fn parses_scene_postfx_list() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: postfx-scene
title: PostFX
postfx:
  - name: crt-filter
    params:
      intensity: 0.6
layers: []
"#,
        )
        .expect("scene should parse");

        assert_eq!(scene.postfx.len(), 1);
        assert_eq!(scene.postfx[0].name, "crt-filter");
        assert_eq!(scene.postfx[0].params.intensity, Some(0.6));
    }

    #[test]
    fn parses_stage_step_without_effects_field() {
        let stage = serde_yaml::from_str::<Stage>(
            r#"
steps:
  - duration: 300
"#,
        )
        .expect("stage should parse");

        assert_eq!(stage.steps.len(), 1);
        assert!(stage.steps[0].effects.is_empty());
        assert_eq!(stage.steps[0].duration, Some(300));
    }

    #[test]
    fn scene_ui_defaults_to_enabled_scene_persistence() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: ui-default
title: UI Default
layers: []
"#,
        )
        .expect("scene should parse");

        assert!(scene.ui.enabled);
        assert_eq!(scene.ui.persist, UiPersistence::Scene);
        assert_eq!(scene.ui.theme, None);
    }

    #[test]
    fn parses_scene_and_layer_ui_flags() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: ui-explicit
title: UI Explicit
ui:
  enabled: false
  persist: global
layers:
  - name: hud
    ui: true
    sprites: []
"#,
        )
        .expect("scene should parse");

        assert!(!scene.ui.enabled);
        assert_eq!(scene.ui.persist, UiPersistence::Global);
        assert_eq!(scene.layers.len(), 1);
        assert!(scene.layers[0].ui);
    }

    #[test]
    fn parses_scene_ui_focus_order_with_aliases() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: ui-focus
title: UI Focus
ui:
  focus-order:
    - prompt-a
    - prompt-b
layers: []
"#,
        )
        .expect("scene should parse");
        assert_eq!(
            scene.ui.focus_order,
            vec!["prompt-a".to_string(), "prompt-b".to_string()]
        );

        let alias_scene = serde_yaml::from_str::<Scene>(
            r#"
id: ui-focus-alias
title: UI Focus Alias
ui:
  focus_order:
    - terminal-prompt
layers: []
"#,
        )
        .expect("scene should parse");
        assert_eq!(
            alias_scene.ui.focus_order,
            vec!["terminal-prompt".to_string()]
        );
    }

    #[test]
    fn parses_scene_ui_theme() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: ui-theme
title: UI Theme
ui:
  theme: win98
layers: []
"#,
        )
        .expect("scene should parse");
        assert_eq!(scene.ui.theme.as_deref(), Some("win98"));
    }

    #[test]
    fn parses_scene_celestial_binding() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: orbital-nav
title: Orbital Nav
space: 3d
celestial:
  scope: system
  region: local-cluster
  system: sol
  focus-body: earth
  focus_site: leo
  frame: barycentric
  clock-source: campaign
layers: []
"#,
        )
        .expect("scene should parse");

        assert_eq!(scene.space, SceneSpace::ThreeD);
        assert_eq!(scene.celestial.scope, CelestialScope::System);
        assert_eq!(scene.celestial.region.as_deref(), Some("local-cluster"));
        assert_eq!(scene.celestial.system.as_deref(), Some("sol"));
        assert_eq!(scene.celestial.focus_body.as_deref(), Some("earth"));
        assert_eq!(scene.celestial.focus_site.as_deref(), Some("leo"));
        assert_eq!(scene.celestial.frame, CelestialFrame::Barycentric);
        assert_eq!(scene.celestial.clock_source, CelestialClockSource::Campaign);
    }
}

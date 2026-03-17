use super::color::TermColour;
use super::easing::Easing;
use super::sprite::Sprite;
use crate::animations::AnimationParams;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
pub enum SceneRenderedMode {
    #[default]
    #[serde(rename = "cell")]
    Cell,
    #[serde(rename = "halfblock")]
    HalfBlock,
    #[serde(rename = "quadblock")]
    QuadBlock,
    #[serde(rename = "braille")]
    Braille,
}

/// A sprite position animation (tween). Modifies sprite transform, not pixel colors.
#[derive(Debug, Clone, Deserialize)]
pub struct Animation {
    pub name: String,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub params: AnimationParams,
}

/// Named runtime behavior attached to a game object.
#[derive(Debug, Clone, Deserialize)]
pub struct BehaviorSpec {
    pub name: String,
    #[serde(default)]
    pub params: BehaviorParams,
}

/// Optional named parameters for runtime behavior configuration.
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
}

/// A single step in a stage — a group of effects that play in parallel.
/// Steps within a stage execute sequentially.
#[derive(Debug, Clone, Deserialize)]
pub struct Step {
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

/// Lifecycle stages for a scene.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SceneStages {
    #[serde(default)]
    pub on_enter: Stage,
    #[serde(default)]
    pub on_idle: Stage,
    #[serde(default)]
    pub on_leave: Stage,
}

/// Declarative audio cue hooks for scene lifecycle stages.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SceneAudio {
    #[serde(default)]
    pub on_enter: Vec<AudioCue>,
    #[serde(default)]
    pub on_idle: Vec<AudioCue>,
    #[serde(default)]
    pub on_leave: Vec<AudioCue>,
}

/// Keyboard menu route available from an on_idle any-key scene.
#[derive(Debug, Clone, Deserialize)]
pub struct MenuOption {
    pub key: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default, rename = "selected-effect", alias = "selected_effect")]
    pub selected_effect: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub scene: Option<String>,
    pub next: String,
}

/// Scene-level interactive input profile configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SceneInput {
    /// Optional OBJ viewer controls profile.
    #[serde(default, rename = "obj-viewer")]
    pub obj_viewer: Option<ObjViewerControls>,
}

/// Declarative OBJ viewer controls target.
#[derive(Debug, Clone, Deserialize)]
pub struct ObjViewerControls {
    /// ID of target OBJ sprite receiving controls.
    pub sprite_id: String,
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

/// Lifecycle stages for a layer.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LayerStages {
    #[serde(default)]
    pub on_enter: Stage,
    #[serde(default)]
    pub on_idle: Stage,
    #[serde(default)]
    pub on_leave: Stage,
}

/// A compositing layer — a named group of sprites sharing z_index and lifecycle effects.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Layer {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub z_index: i32,
    #[serde(default = "default_visible")]
    pub visible: bool,
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

/// A parsed scene loaded from a `.yml` file.
#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub cutscene: bool,
    #[serde(default, rename = "rendered-mode")]
    pub rendered_mode: SceneRenderedMode,
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
    pub layers: Vec<Layer>,
    #[serde(default, alias = "menu_options", rename = "menu-options")]
    pub menu_options: Vec<MenuOption>,
    #[serde(default)]
    pub input: SceneInput,
    pub next: Option<String>,
}

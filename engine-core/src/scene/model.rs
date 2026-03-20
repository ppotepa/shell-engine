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
    /// Optional terminal size tester controls profile.
    #[serde(default, rename = "terminal-size-tester")]
    pub terminal_size_tester: Option<TerminalSizeTesterControls>,
    /// Optional interactive terminal shell profile.
    #[serde(default, rename = "terminal-shell", alias = "terminal_shell")]
    pub terminal_shell: Option<TerminalShellControls>,
}

impl SceneInput {
    /// Returns names of all built-in input profiles.
    pub fn builtin_profiles() -> Vec<&'static str> {
        vec!["obj-viewer", "terminal-size-tester", "terminal-shell"]
    }
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

/// Declarative OBJ viewer controls target.
#[derive(Debug, Clone, Deserialize)]
pub struct ObjViewerControls {
    /// ID of target OBJ sprite receiving controls.
    pub sprite_id: String,
}

/// Declarative terminal-size tester controls.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TerminalSizeTesterControls {
    /// Optional preset list in WIDTHxHEIGHT format, e.g. "120x36".
    #[serde(default)]
    pub presets: Vec<String>,
}

fn default_terminal_prompt_prefix() -> String {
    "> ".to_string()
}

fn default_terminal_max_lines() -> usize {
    120
}

fn default_terminal_prompt_wrap() -> bool {
    true
}

fn default_terminal_prompt_min_lines() -> u16 {
    1
}

fn default_terminal_prompt_max_lines() -> u16 {
    4
}

fn default_terminal_prompt_growth_ms() -> u64 {
    120
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TerminalShellOutput {
    Single(String),
    Multi(Vec<String>),
}

impl TerminalShellOutput {
    pub fn lines(&self) -> Vec<String> {
        match self {
            Self::Single(line) => vec![line.clone()],
            Self::Multi(lines) => lines.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TerminalShellCommand {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub output: Option<TerminalShellOutput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TerminalShellControls {
    /// Text sprite id used for the command prompt line.
    #[serde(rename = "prompt-sprite-id", alias = "prompt_sprite_id")]
    pub prompt_sprite_id: String,
    /// Text sprite id used for command output transcript.
    #[serde(rename = "output-sprite-id", alias = "output_sprite_id")]
    pub output_sprite_id: String,
    /// Optional panel widget id that hosts the prompt sprite.
    #[serde(default, rename = "prompt-panel-id", alias = "prompt_panel_id")]
    pub prompt_panel_id: Option<String>,
    /// Optional panel widget id used as visual shadow for prompt panel auto-grow sync.
    #[serde(
        default,
        rename = "prompt-shadow-panel-id",
        alias = "prompt_shadow_panel_id"
    )]
    pub prompt_shadow_panel_id: Option<String>,
    /// Prompt prefix rendered before the current command line.
    #[serde(
        default = "default_terminal_prompt_prefix",
        rename = "prompt-prefix",
        alias = "prompt_prefix"
    )]
    pub prompt_prefix: String,
    /// Enables prompt word wrapping inside the prompt panel.
    #[serde(
        default = "default_terminal_prompt_wrap",
        rename = "prompt-wrap",
        alias = "prompt_wrap"
    )]
    pub prompt_wrap: bool,
    /// Enables auto-growing prompt panel height based on wrapped line count.
    #[serde(
        default = "default_terminal_prompt_wrap",
        rename = "prompt-auto-grow",
        alias = "prompt_auto_grow"
    )]
    pub prompt_auto_grow: bool,
    /// Minimum number of input lines kept visible in prompt panel.
    #[serde(
        default = "default_terminal_prompt_min_lines",
        rename = "prompt-min-lines",
        alias = "prompt_min_lines"
    )]
    pub prompt_min_lines: u16,
    /// Maximum number of wrapped input lines shown before clipping.
    #[serde(
        default = "default_terminal_prompt_max_lines",
        rename = "prompt-max-lines",
        alias = "prompt_max_lines"
    )]
    pub prompt_max_lines: u16,
    /// Target animation time for panel auto-grow transitions.
    #[serde(
        default = "default_terminal_prompt_growth_ms",
        rename = "prompt-growth-ms",
        alias = "prompt_growth_ms"
    )]
    pub prompt_growth_ms: u64,
    /// Maximum number of output transcript lines preserved on screen.
    #[serde(
        default = "default_terminal_max_lines",
        rename = "max-lines",
        alias = "max_lines"
    )]
    pub max_lines: usize,
    /// Initial transcript lines shown when scene loads.
    #[serde(default)]
    pub banner: Vec<String>,
    /// Optional custom command table, in addition to built-ins.
    #[serde(default)]
    pub commands: Vec<TerminalShellCommand>,
    /// Optional message shown for unknown commands.
    #[serde(default, rename = "unknown-message", alias = "unknown_message")]
    pub unknown_message: Option<String>,
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
}

#[cfg(test)]
mod tests {
    use super::{Scene, TerminalShellOutput, UiPersistence};
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
  - name: terminal-crt
    params:
      intensity: 0.6
layers: []
"#,
        )
        .expect("scene should parse");

        assert_eq!(scene.postfx.len(), 1);
        assert_eq!(scene.postfx[0].name, "terminal-crt");
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
    fn parses_terminal_shell_input_profile() {
        let scene = serde_yaml::from_str::<Scene>(
            r#"
id: terminal-shell
title: Terminal
input:
  terminal-shell:
    prompt-sprite-id: terminal-prompt
    output-sprite-id: terminal-output
    prompt-panel-id: prompt-panel
    prompt-shadow-panel-id: prompt-panel-shadow
    prompt-prefix: "$ "
    prompt-wrap: true
    prompt-auto-grow: true
    prompt-min-lines: 1
    prompt-max-lines: 3
    prompt-growth-ms: 200
    max-lines: 80
    banner:
      - boot ok
    commands:
      - name: status
        output:
          - online
layers: []
"#,
        )
        .expect("scene should parse");

        let controls = scene
            .input
            .terminal_shell
            .expect("terminal-shell controls should parse");
        assert_eq!(controls.prompt_sprite_id, "terminal-prompt");
        assert_eq!(controls.output_sprite_id, "terminal-output");
        assert_eq!(controls.prompt_panel_id.as_deref(), Some("prompt-panel"));
        assert_eq!(
            controls.prompt_shadow_panel_id.as_deref(),
            Some("prompt-panel-shadow")
        );
        assert_eq!(controls.prompt_prefix, "$ ");
        assert!(controls.prompt_wrap);
        assert!(controls.prompt_auto_grow);
        assert_eq!(controls.prompt_min_lines, 1);
        assert_eq!(controls.prompt_max_lines, 3);
        assert_eq!(controls.prompt_growth_ms, 200);
        assert_eq!(controls.max_lines, 80);
        assert_eq!(controls.banner, vec!["boot ok".to_string()]);
        assert_eq!(controls.commands.len(), 1);
    }

    #[test]
    fn terminal_shell_output_supports_string_and_array() {
        let single: TerminalShellOutput =
            serde_yaml::from_str("online").expect("single output should parse");
        assert_eq!(single.lines(), vec!["online".to_string()]);

        let multiple: TerminalShellOutput =
            serde_yaml::from_str("[a, b]").expect("array output should parse");
        assert_eq!(multiple.lines(), vec!["a".to_string(), "b".to_string()]);
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
}

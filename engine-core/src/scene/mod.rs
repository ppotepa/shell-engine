//! Scene authoring and runtime model types shared across the scene pipeline.
//!
//! This module exposes the typed runtime scene model shared by the engine.
//! Authoring-side YAML documents live in `engine-authoring`.

/// Terminal colour type for scene backgrounds and sprites.
pub mod color;
/// Easing function variants for animations and effects.
pub mod easing;
/// Static field-metadata tables for the authoring editor.
pub mod metadata;
/// Runtime scene model: [`Scene`], [`Layer`], [`Stage`], and related types.
pub mod model;
/// Sprite template expansion types.
pub mod sprite;
/// Template substitution helpers.
pub mod template;

pub use crate::animations::AnimationParams;
pub use color::TermColour;
pub use easing::Easing;
pub use metadata::{LAYER_FIELDS, OBJECT_FIELDS, SCENE_FIELDS, SPRITE_FIELDS};
pub use model::{
    Animation, AudioCue, BehaviorParams, BehaviorSpec, Effect, EffectParams, EffectTargetKind,
    Layer, LayerStages, MenuOption, ObjViewerControls, Scene, SceneAudio, SceneInput,
    SceneRenderedMode, SceneStages, Stage, StageTrigger, Step, TerminalSizeTesterControls,
};
pub use sprite::{FlexDirection, HorizontalAlign, Sprite, SpriteSizePreset, VerticalAlign};

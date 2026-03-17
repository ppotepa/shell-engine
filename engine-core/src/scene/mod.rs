//! Scene authoring and runtime model types shared across the scene pipeline.
//!
//! This module exposes both the authored YAML compilation boundary
//! (`SceneDocument`, `ObjectDocument`) and the typed runtime scene model
//! (`Scene`, `Layer`, `Sprite`).

/// Terminal colour type for scene backgrounds and sprites.
pub mod color;
/// Scene and object document (YAML-deserialisable) types.
pub mod document;
/// Easing function variants for animations and effects.
pub mod easing;
/// Static field-metadata tables for the authoring editor.
pub mod metadata;
/// Runtime scene model: [`Scene`], [`Layer`], [`Stage`], and related types.
pub mod model;
/// Object/prefab document and logic specification types.
pub mod object;
/// Sprite template expansion types.
pub mod sprite;
/// Template substitution helpers.
pub mod template;
/// Typed scalar and colour value wrappers.
pub mod value;

pub use crate::animations::AnimationParams;
pub use color::TermColour;
pub use document::SceneDocument;
pub use easing::Easing;
pub use metadata::{LAYER_FIELDS, OBJECT_FIELDS, SCENE_FIELDS, SPRITE_FIELDS};
pub use model::{
    Animation, AudioCue, BehaviorParams, BehaviorSpec, Effect, EffectParams, EffectTargetKind,
    Layer, LayerStages, MenuOption, ObjViewerControls, Scene, SceneAudio, SceneInput,
    SceneRenderedMode, SceneStages, Stage, StageTrigger, Step, TerminalSizeTesterControls,
};
pub use object::{LogicKind, LogicSpec, ObjectDocument};
pub use sprite::{FlexDirection, HorizontalAlign, Sprite, SpriteSizePreset, VerticalAlign};
pub use value::{ColorValue, ScalarValue};

pub mod color;
pub mod document;
pub mod easing;
pub mod metadata;
pub mod model;
pub mod object;
pub mod sprite;
pub mod template;
pub mod value;

pub use crate::animations::AnimationParams;
pub use color::TermColour;
pub use document::SceneDocument;
pub use easing::Easing;
pub use model::{
    Animation, AudioCue, BehaviorParams, BehaviorSpec, Effect, EffectParams, EffectTargetKind,
    Layer, LayerStages, MenuOption, ObjViewerControls, Scene, SceneAudio, SceneInput,
    SceneRenderedMode, SceneStages, Stage, StageTrigger, Step, TerminalSizeTesterControls,
};
pub use object::{LogicKind, LogicSpec, ObjectDocument};
pub use sprite::{HorizontalAlign, Sprite, SpriteSizePreset, VerticalAlign};
pub use value::{ColorValue, ScalarValue};
pub use metadata::{LAYER_FIELDS, OBJECT_FIELDS, SCENE_FIELDS, SPRITE_FIELDS};

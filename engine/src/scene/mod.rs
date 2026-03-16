pub mod color;
pub mod easing;
pub mod model;
pub mod sprite;

pub use crate::animations::AnimationParams;
pub use color::TermColour;
pub use easing::Easing;
pub use model::{
    Animation, AudioCue, BehaviorParams, BehaviorSpec, Effect, EffectParams, Layer, LayerStages,
    Scene, SceneAudio, SceneRenderedMode, SceneStages, Stage, StageTrigger, Step,
};
pub use sprite::{HorizontalAlign, Sprite, VerticalAlign};

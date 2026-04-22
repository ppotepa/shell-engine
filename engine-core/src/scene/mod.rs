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
/// Scene-level planet generation/render contract.
pub mod planet_spec;
/// Sprite template expansion types.
pub mod sprite;
/// Template substitution helpers.
pub mod template;
/// Typed scene transition references used across behavior and runtime seams.
pub mod transition;
/// UI theme registry and preset styles.
pub mod ui_theme;
/// Reusable 3D scene view, lighting, and environment profile contracts.
pub mod view_profile;

pub use crate::animations::AnimationParams;
pub use color::TermColour;
pub use easing::Easing;
pub use metadata::{LAYER_FIELDS, OBJECT_FIELDS, SCENE_FIELDS, SPRITE_FIELDS};
pub use model::{
    Animation, AudioCue, BehaviorParams, BehaviorSpec, CelestialClockSource, CelestialFrame,
    CelestialScope, Effect, EffectParams, EffectTargetKind, FreeLookCameraControls,
    GameStateBinding, Layer, LayerSpace, LayerStages, MenuOption, ObjOrbitCameraControls,
    ObjViewerControls, PaletteBinding, Scene, SceneAudio, SceneCelestial, SceneInput,
    SceneLighting, SceneSpace, SceneStages, SceneUi, SceneView, Stage, StageTrigger, Step,
    UiPersistence,
};
pub use planet_spec::{
    PlanetSpec, PlanetSpecAtmosphere, PlanetSpecBody, PlanetSpecGenerator, PlanetSpecRender,
};
pub use sprite::{
    CameraSource, FlexDirection, HorizontalAlign, Sprite, SpriteSizePreset, TextOverflowMode,
    TextTransform, TextWrapMode, VerticalAlign,
};
pub use transition::SceneTransitionTarget;
pub use ui_theme::{
    normalize_theme_key, resolve_ui_theme, resolve_ui_theme_or_default, ScrollListThemeStyle,
    UiThemeStyle, WindowThemeStyle,
};
pub use view_profile::{
    builtin_lighting_profile, builtin_space_environment_profile, builtin_view_profile,
    merge_lighting_profile, merge_space_environment_profile, resolve_scene_view_profile,
    LightingProfile, ResolvedViewProfile, SpaceEnvironmentProfile, TonemapOperator, ViewProfile,
    ViewProfileOverrides,
};

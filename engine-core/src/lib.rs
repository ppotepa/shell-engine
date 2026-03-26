//! Engine-core: Pure data + algorithmic runtime primitives.
//! Used by both the game engine runtime and editor/tooling.

/// Animation types and built-in animation effects.
pub mod animations;
/// Authoring metadata and field catalogues for the editor.
pub mod authoring;
/// Terminal cell buffer for rendering.
pub mod buffer;
/// Visual effects system and built-in effect implementations.
pub mod effects;
/// Run-scoped file logging for launchers, runtime, and editor.
pub mod logging;
/// Markup parsing and rendering utilities.
pub mod markup;
/// Scene data model, authoring types, and runtime model.
pub mod scene;
/// Core scene-object types: [`GameObjectKind`] and [`GameObject`].
pub mod game_object;
/// Persistent game state singleton: generic JSON key-value store for cross-scene data.
pub mod game_state;
/// Pure data types shared between scene runtime and behavior system.
/// Includes TargetResolver, ObjectRuntimeState, RawKeyEvent, SidecarIoFrameState.
pub mod scene_runtime_types;
/// Render pipeline strategy traits and default implementations.
pub mod strategy;
/// Type-erased resource container (World) for engine ECS.
pub mod world;
/// Domain access traits for typed resource retrieval from World.
pub mod access;
/// Asset resolution helpers (AssetRoot).
pub mod assets;
/// Lazy cache for optional assets: `AssetCache<T>`.
pub mod asset_cache;
/// Terminal capability detection and mod-level requirements.
pub mod terminal_caps;

#[cfg(test)]
mod access_tests;
pub mod asset_source;

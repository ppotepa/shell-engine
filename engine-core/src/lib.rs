//! Engine-core: Pure data + algorithmic runtime primitives.
//! Used by both the game engine runtime and editor/tooling.

/// Domain access traits for typed resource retrieval from World.
pub mod access;
/// Animation types and built-in animation effects.
pub mod animations;
/// Lazy cache for optional assets: `AssetCache<T>`.
pub mod asset_cache;
/// Asset resolution helpers (AssetRoot).
pub mod assets;
/// Authoring metadata and field catalogues for the editor.
pub mod authoring;
/// Platform-agnostic color abstraction (RGB + named colors).
pub mod color;
/// Terminal cell buffer for rendering.
pub mod buffer;
/// Visual effects system and built-in effect implementations.
pub mod effects;
/// Core scene-object types: [`GameObjectKind`] and [`GameObject`].
pub mod game_object;
/// Persistent game state singleton: generic JSON key-value store for cross-scene data.
pub mod game_state;
/// Run-scoped file logging for launchers, runtime, and editor.
pub mod logging;
/// Markup parsing and rendering utilities.
pub mod markup;
/// Scene data model, authoring types, and runtime model.
pub mod scene;
/// Pure data types shared between scene runtime and behavior system.
/// Includes TargetResolver, ObjectRuntimeState, RawKeyEvent, SidecarIoFrameState.
pub mod scene_runtime_types;
/// Render pipeline strategy traits and default implementations.
pub mod strategy;
/// Type-erased resource container (World) for engine ECS.
pub mod world;

#[cfg(test)]
mod access_tests;
pub mod asset_source;

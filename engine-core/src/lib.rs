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
/// Render pipeline strategy traits and default implementations.
pub mod strategy;

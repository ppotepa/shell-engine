//! Built-in startup checks — re-exports all concrete [`StartupCheck`](super::check::StartupCheck) implementations.

mod effect_registry;
mod font_glyph_coverage;
mod font_manifest;
mod image_assets;
mod scene_graph;
mod terminal_requirements;

pub use effect_registry::EffectRegistryCheck;
pub use font_glyph_coverage::FontGlyphCoverageCheck;
pub use font_manifest::FontManifestCheck;
pub use image_assets::ImageAssetsCheck;
pub use scene_graph::SceneGraphCheck;
pub use terminal_requirements::TerminalRequirementsCheck;

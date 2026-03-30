//! Authored YAML document model.
//!
//! This module will own scene/object/value AST types that describe the
//! human-authored input format before compilation into runtime `Scene` data.

mod object;
mod scene;
mod scene_helpers;
mod value;

pub use object::{LogicKind, LogicSpec, ObjectDocument};
pub use scene::SceneDocument;
pub use value::{ColorValue, ScalarValue};

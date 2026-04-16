//! Authored YAML document model.
//!
//! This module will own scene/object/value AST types that describe the
//! human-authored input format before compilation into runtime `Scene` data.

mod atmosphere_profile;
mod camera_profile;
mod material;
mod object;
mod render_scene3d;
mod scene;
mod scene_helpers;
mod value;
mod viewport3d;
mod world_profile;

pub use atmosphere_profile::AtmosphereProfileDocument;
pub use camera_profile::CameraProfileDocument;
pub use material::{MaterialDocument, MaterialParamDocument, MaterialValueDocument};
pub use object::{LogicKind, LogicSpec, ObjectDocument};
pub use render_scene3d::RenderScene3dDocument;
pub use scene::SceneDocument;
pub use value::{ColorValue, ScalarValue};
pub use viewport3d::Viewport3dDocument;
pub use world_profile::WorldProfileDocument;

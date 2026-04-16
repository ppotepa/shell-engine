use super::camera::Camera3DInstance;
use super::dirty::DirtyState3D;
use super::lights::Light3DInstance;
use super::nodes::Node3DInstance;
use super::viewport::Viewport3DInstance;

#[derive(Debug, Clone, Default)]
pub struct Scene3DInstance {
    pub id: Option<String>,
    pub nodes: Vec<Node3DInstance>,
    pub cameras: Vec<Camera3DInstance>,
    pub lights: Vec<Light3DInstance>,
    pub viewports: Vec<Viewport3DInstance>,
    pub dirty: DirtyState3D,
}

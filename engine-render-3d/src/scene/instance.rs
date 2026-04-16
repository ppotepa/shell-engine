use super::camera::Camera3DInstance;
use super::dirty::DirtyState3D;
use super::lights::Light3DInstance;
use super::nodes::Node3DInstance;
use super::viewport::Viewport3DInstance;
use engine_core::render_types::DirtyMask3D;

#[derive(Debug, Clone, Default)]
pub struct Scene3DInstance {
    pub id: Option<String>,
    pub nodes: Vec<Node3DInstance>,
    pub cameras: Vec<Camera3DInstance>,
    pub lights: Vec<Light3DInstance>,
    pub viewports: Vec<Viewport3DInstance>,
    pub dirty: DirtyState3D,
}

impl Scene3DInstance {
    pub fn mark_dirty(&mut self, mask: DirtyMask3D) {
        self.dirty.mark(mask);
    }

    pub fn take_dirty_mask(&mut self) -> DirtyMask3D {
        self.dirty.take()
    }
}

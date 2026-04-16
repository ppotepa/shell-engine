use engine_core::render_types::DirtyMask3D;

#[derive(Debug, Clone, Copy, Default)]
pub struct DirtyState3D {
    pub mask: DirtyMask3D,
}

impl DirtyState3D {
    pub fn mark(&mut self, dirty: DirtyMask3D) {
        self.mask.insert(dirty);
    }

    pub fn clear(&mut self) {
        self.mask = DirtyMask3D::empty();
    }
}

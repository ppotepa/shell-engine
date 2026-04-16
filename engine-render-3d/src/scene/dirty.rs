use engine_core::render_types::DirtyMask3D;

#[derive(Debug, Clone, Copy, Default)]
pub struct DirtyState3D {
    pub mask: DirtyMask3D,
}

impl DirtyState3D {
    pub fn mark(&mut self, dirty: DirtyMask3D) {
        self.mask.insert(dirty);
    }

    pub fn take(&mut self) -> DirtyMask3D {
        let mask = self.mask;
        self.mask = DirtyMask3D::empty();
        mask
    }

    pub fn contains(&self, dirty: DirtyMask3D) -> bool {
        self.mask.contains(dirty)
    }

    pub fn clear(&mut self) {
        self.mask = DirtyMask3D::empty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_contains_and_take_cover_all_3d_categories() {
        let mut dirty = DirtyState3D::default();
        let expected = DirtyMask3D::TRANSFORM
            | DirtyMask3D::MATERIAL
            | DirtyMask3D::ATMOSPHERE
            | DirtyMask3D::LIGHTING
            | DirtyMask3D::CAMERA
            | DirtyMask3D::WORLDGEN
            | DirtyMask3D::VISIBILITY;
        dirty.mark(expected);

        assert!(dirty.contains(DirtyMask3D::TRANSFORM));
        assert!(dirty.contains(DirtyMask3D::MATERIAL));
        assert!(dirty.contains(DirtyMask3D::ATMOSPHERE));
        assert!(dirty.contains(DirtyMask3D::LIGHTING));
        assert!(dirty.contains(DirtyMask3D::CAMERA));
        assert!(dirty.contains(DirtyMask3D::WORLDGEN));
        assert!(dirty.contains(DirtyMask3D::VISIBILITY));
        assert_eq!(dirty.take(), expected);
        assert_eq!(dirty.mask, DirtyMask3D::empty());
    }
}

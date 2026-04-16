use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct DirtyMask3D: u32 {
        const TRANSFORM = 1 << 0;
        const MATERIAL = 1 << 1;
        const ATMOSPHERE = 1 << 2;
        const LIGHTING = 1 << 3;
        const CAMERA = 1 << 4;
        const MESH = 1 << 5;
        const WORLDGEN = 1 << 6;
        const VISIBILITY = 1 << 7;
    }
}

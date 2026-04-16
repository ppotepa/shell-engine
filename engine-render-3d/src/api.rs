use crate::scene::Scene3DInstance;
use engine_core::buffer::Buffer;
use engine_core::render_types::{Camera3DState, ViewportRect};

/// Canonical 3D render pipeline contract.
///
/// This is introduced as a stable seam for gradually moving 3D rendering
/// internals out of compositor into this domain crate.
pub trait Render3dPipeline<I, O> {
    fn render(&self, input: I) -> O;
}

#[derive(Debug)]
pub struct Render3dInput<'a> {
    pub viewport: ViewportRect,
    pub scene: &'a Scene3DInstance,
    pub camera: &'a Camera3DState,
    pub frame_time_ms: u64,
}

#[derive(Debug)]
pub struct Render3dOutput {
    pub color: Buffer,
}

impl Render3dOutput {
    pub fn new(color: Buffer) -> Self {
        Self { color }
    }
}

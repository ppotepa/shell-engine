/// Canonical 3D render pipeline contract.
///
/// This is introduced as a stable seam for gradually moving 3D rendering
/// internals out of compositor into this domain crate.
pub trait Render3dPipeline<I, O> {
    fn render(&self, input: I) -> O;
}

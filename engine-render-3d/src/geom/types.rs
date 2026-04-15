#[derive(Debug, Clone, Copy)]
pub struct ProjectedVertex {
    pub x: f32,
    pub y: f32,
    pub depth: f32,
    /// Rotated/translated world-space position.
    pub view: [f32; 3],
    /// Rotated smooth vertex normal in world space.
    pub normal: [f32; 3],
    /// Pre-rotation local-space position.
    pub local: [f32; 3],
    /// Pre-computed terrain noise value.
    pub terrain_noise: f32,
}

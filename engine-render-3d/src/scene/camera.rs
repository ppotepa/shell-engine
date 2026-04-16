use engine_core::render_types::Camera3DState;

#[derive(Debug, Clone)]
pub struct Camera3DInstance {
    pub id: String,
    pub state: Camera3DState,
    pub active: bool,
}

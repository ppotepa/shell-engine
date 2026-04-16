use engine_core::render_types::ViewportRect;

#[derive(Debug, Clone)]
pub struct Viewport3DInstance {
    pub id: Option<String>,
    pub scene_id: String,
    pub camera_id: Option<String>,
    pub rect: ViewportRect,
    pub visible: bool,
}

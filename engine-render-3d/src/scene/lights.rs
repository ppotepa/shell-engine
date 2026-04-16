use engine_core::render_types::Light3D;

#[derive(Debug, Clone)]
pub struct Light3DInstance {
    pub id: Option<String>,
    pub light: Light3D,
    pub enabled: bool,
}

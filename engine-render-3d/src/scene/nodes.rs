use super::materials::MaterialInstance;
use engine_core::render_types::Transform3D;

#[derive(Debug, Clone)]
pub struct MeshInstance {
    pub source: String,
    pub material: Option<MaterialInstance>,
}

#[derive(Debug, Clone)]
pub struct GeneratedWorldInstance {
    pub profile_id: Option<String>,
    pub params_uri: Option<String>,
    pub material: Option<MaterialInstance>,
}

#[derive(Debug, Clone)]
pub struct Billboard3DInstance {
    pub image_source: String,
    pub size: [f32; 2],
    pub material: Option<MaterialInstance>,
}

#[derive(Debug, Clone)]
pub enum Renderable3D {
    Mesh(MeshInstance),
    GeneratedWorld(GeneratedWorldInstance),
    Billboard(Billboard3DInstance),
}

#[derive(Debug, Clone)]
pub struct Node3DInstance {
    pub id: String,
    pub transform: Transform3D,
    pub visible: bool,
    pub renderable: Renderable3D,
}

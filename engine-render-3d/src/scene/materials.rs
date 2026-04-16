use engine_core::render_types::MaterialParam;

#[derive(Debug, Clone, Default)]
pub struct MaterialInstance {
    pub id: Option<String>,
    pub params: Vec<MaterialParam>,
}

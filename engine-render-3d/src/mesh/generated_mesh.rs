#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedMesh {
    pub mesh_source: Option<String>,
    pub params_uri: Option<String>,
}

impl GeneratedMesh {
    pub fn new(mesh_source: Option<String>, params_uri: Option<String>) -> Self {
        Self {
            mesh_source,
            params_uri,
        }
    }
}

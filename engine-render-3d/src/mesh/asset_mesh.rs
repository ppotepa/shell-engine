#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetMesh {
    pub source: String,
}

impl AssetMesh {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }

    pub fn into_source(self) -> String {
        self.source
    }
}

pub fn normalize_asset_mesh_source(source: &str) -> String {
    AssetMesh::new(source).into_source()
}

#[cfg(test)]
mod tests {
    use super::normalize_asset_mesh_source;

    #[test]
    fn keeps_mesh_source_string_stable() {
        let source = "/assets/3d/sample.obj";
        assert_eq!(normalize_asset_mesh_source(source), source);
    }
}

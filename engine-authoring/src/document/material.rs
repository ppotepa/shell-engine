use serde::{Deserialize, Serialize};

/// Authored material value shape for 3D viewport documents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialValueDocument {
    Scalar(f32),
    ColorRgb([u8; 3]),
    Bool(bool),
    Text(String),
}

/// Authored material parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialParamDocument {
    pub name: String,
    pub value: MaterialValueDocument,
}

/// Authored material entry.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MaterialDocument {
    pub id: String,
    #[serde(default)]
    pub surface_mode: Option<String>,
    #[serde(default)]
    pub params: Vec<MaterialParamDocument>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialValue {
    Scalar(f32),
    ColorRgb([u8; 3]),
    Bool(bool),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialParam {
    pub name: String,
    pub value: MaterialValue,
}

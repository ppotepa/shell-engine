use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ScalarValue {
    Number(f32),
    Expression(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ColorValue {
    Literal(crate::scene::TermColour),
    Expression(String),
}


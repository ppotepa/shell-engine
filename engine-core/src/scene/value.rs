//! Flexible authored scalar value wrappers used while YAML is still allowed
//! to contain literal values or expressions.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
/// Authored scalar field that may be either a concrete number or an expression
/// string awaiting later interpretation.
pub enum ScalarValue {
    Number(f32),
    Expression(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
/// Authored colour field that may be either a concrete terminal colour or an
/// expression string awaiting later interpretation.
pub enum ColorValue {
    Literal(crate::scene::TermColour),
    Expression(String),
}

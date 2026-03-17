//! Shared authored-field metadata used by engine, schema generation, and editor tooling.

/// Which authored entity a field belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Scene,
    Layer,
    Sprite,
    Object,
    Effect,
}

/// Supported value representation for a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Number,
    Integer,
    Boolean,
    Text,
    Colour,
    Select,
    SelectList,
}

/// Allowed authored source forms for a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    Literal,
    Expression,
    Binding,
    Animation,
}

/// Field requirement level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    Optional,
    Required,
    RequiredIf {
        field: &'static str,
        equals: &'static str,
    },
}

/// Generic metadata for one authored field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldMetadata {
    pub target: TargetKind,
    pub name: &'static str,
    pub value_kind: ValueKind,
    pub requirement: Requirement,
    pub description: &'static str,
    pub default_text: Option<&'static str>,
    pub default_number: Option<f32>,
    pub enum_options: Option<&'static [&'static str]>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub unit: Option<&'static str>,
    pub sources: &'static [ValueSource],
}

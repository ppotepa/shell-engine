use crate::authoring::metadata::{FieldMetadata, Requirement, TargetKind, ValueKind, ValueSource};

const LIT_ONLY: &[ValueSource] = &[ValueSource::Literal];
const LIT_EXPR_BIND_ANIM: &[ValueSource] = &[
    ValueSource::Literal,
    ValueSource::Expression,
    ValueSource::Binding,
    ValueSource::Animation,
];
const SPRITE_TYPE_OPTIONS: &[&str] = &["text", "image", "obj", "grid"];
const LOGIC_TYPE_OPTIONS: &[&str] = &["native", "graph", "script"];

pub static SCENE_FIELDS: &[FieldMetadata] = &[
    FieldMetadata {
        target: TargetKind::Scene,
        name: "id",
        value_kind: ValueKind::Text,
        requirement: Requirement::Required,
        description: "Unique scene identifier.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::Scene,
        name: "title",
        value_kind: ValueKind::Text,
        requirement: Requirement::Required,
        description: "Human-readable scene title.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::Scene,
        name: "bg",
        value_kind: ValueKind::Colour,
        requirement: Requirement::Optional,
        description: "Scene background colour.",
        default_text: Some("black"),
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: &[
            ValueSource::Literal,
            ValueSource::Expression,
            ValueSource::Binding,
        ],
    },
];

pub static LAYER_FIELDS: &[FieldMetadata] = &[
    FieldMetadata {
        target: TargetKind::Layer,
        name: "name",
        value_kind: ValueKind::Text,
        requirement: Requirement::Required,
        description: "Layer identifier.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::Layer,
        name: "z_index",
        value_kind: ValueKind::Integer,
        requirement: Requirement::Optional,
        description: "Layer render order.",
        default_text: None,
        default_number: Some(0.0),
        enum_options: None,
        min: None,
        max: None,
        step: Some(1.0),
        unit: None,
        sources: LIT_ONLY,
    },
];

pub static SPRITE_FIELDS: &[FieldMetadata] = &[
    FieldMetadata {
        target: TargetKind::Sprite,
        name: "type",
        value_kind: ValueKind::Select,
        requirement: Requirement::Required,
        description: "Sprite kind.",
        default_text: None,
        default_number: None,
        enum_options: Some(SPRITE_TYPE_OPTIONS),
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::Sprite,
        name: "content",
        value_kind: ValueKind::Text,
        requirement: Requirement::RequiredIf {
            field: "type",
            equals: "text",
        },
        description: "Text content for text sprites.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: &[
            ValueSource::Literal,
            ValueSource::Binding,
            ValueSource::Expression,
        ],
    },
    FieldMetadata {
        target: TargetKind::Sprite,
        name: "source",
        value_kind: ValueKind::Text,
        requirement: Requirement::RequiredIf {
            field: "type",
            equals: "image",
        },
        description: "Asset source path (image/obj).",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::Sprite,
        name: "size",
        value_kind: ValueKind::Integer,
        requirement: Requirement::Optional,
        description: "Size preset; optional and type-dependent.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: Some(1.0),
        max: Some(3.0),
        step: Some(1.0),
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::Sprite,
        name: "effects",
        value_kind: ValueKind::Text,
        requirement: Requirement::Optional,
        description: "Optional per-sprite staged effects.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
];

pub static OBJECT_FIELDS: &[FieldMetadata] = &[
    FieldMetadata {
        target: TargetKind::Object,
        name: "name",
        value_kind: ValueKind::Text,
        requirement: Requirement::Required,
        description: "Object/prefab identifier.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::Object,
        name: "exports",
        value_kind: ValueKind::Text,
        requirement: Requirement::Optional,
        description: "Optional default substitution values.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_EXPR_BIND_ANIM,
    },
    FieldMetadata {
        target: TargetKind::Object,
        name: "logic.type",
        value_kind: ValueKind::Select,
        requirement: Requirement::Optional,
        description: "Object logic backend type.",
        default_text: Some("native"),
        default_number: None,
        enum_options: Some(LOGIC_TYPE_OPTIONS),
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::Object,
        name: "effects",
        value_kind: ValueKind::Text,
        requirement: Requirement::Optional,
        description: "Optional object-level effects (if present in object authored shape).",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
];

#[cfg(test)]
mod tests {
    use super::{OBJECT_FIELDS, SPRITE_FIELDS};
    use crate::authoring::metadata::Requirement;

    #[test]
    fn sprite_content_is_required_for_text_type() {
        let field = SPRITE_FIELDS
            .iter()
            .find(|f| f.name == "content")
            .expect("content metadata");
        assert_eq!(
            field.requirement,
            Requirement::RequiredIf {
                field: "type",
                equals: "text"
            }
        );
    }

    #[test]
    fn object_effects_are_optional() {
        let field = OBJECT_FIELDS
            .iter()
            .find(|f| f.name == "effects")
            .expect("effects metadata");
        assert_eq!(field.requirement, Requirement::Optional);
    }
}

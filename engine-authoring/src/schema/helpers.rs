//! Schema primitive helpers (schema_ref, array_items_ref, etc.).

use engine_core::authoring::metadata::FieldMetadata;
use serde_yaml::{Mapping, Value};

pub(super) fn enum_schema(values: Vec<String>) -> Value {
    let mut m = Mapping::new();
    m.insert(
        Value::String("type".to_string()),
        Value::String("string".to_string()),
    );
    m.insert(
        Value::String("enum".to_string()),
        Value::Sequence(values.into_iter().map(Value::String).collect()),
    );
    Value::Mapping(m)
}

/// Converts FieldMetadata to JSON Schema property definition.
pub(super) fn field_metadata_to_schema(field: &FieldMetadata) -> Value {
    use engine_core::authoring::metadata::ValueKind;

    let mut prop = Mapping::new();

    match field.value_kind {
        ValueKind::Number => {
            prop.insert(
                Value::String("type".to_string()),
                Value::String("number".to_string()),
            );
        }
        ValueKind::Integer => {
            prop.insert(
                Value::String("type".to_string()),
                Value::String("integer".to_string()),
            );
        }
        ValueKind::Boolean => {
            prop.insert(
                Value::String("type".to_string()),
                Value::String("boolean".to_string()),
            );
        }
        ValueKind::Text | ValueKind::Colour | ValueKind::Select => {
            prop.insert(
                Value::String("type".to_string()),
                Value::String("string".to_string()),
            );
        }
        ValueKind::SelectList => {
            prop.insert(
                Value::String("type".to_string()),
                Value::String("array".to_string()),
            );
        }
    }

    if !field.description.is_empty() {
        prop.insert(
            Value::String("description".to_string()),
            Value::String(field.description.to_string()),
        );
    }

    if let Some(default) = field.default_text {
        prop.insert(
            Value::String("default".to_string()),
            Value::String(default.to_string()),
        );
    } else if let Some(default) = field.default_number {
        prop.insert(
            Value::String("default".to_string()),
            serde_yaml::to_value(default).unwrap(),
        );
    }

    if let Some(options) = field.enum_options {
        let values = Value::Sequence(
            options
                .iter()
                .map(|s| Value::String(s.to_string()))
                .collect(),
        );
        if matches!(field.value_kind, ValueKind::SelectList) {
            let mut items = Mapping::new();
            items.insert(
                Value::String("type".to_string()),
                Value::String("string".to_string()),
            );
            items.insert(Value::String("enum".to_string()), values);
            prop.insert(Value::String("items".to_string()), Value::Mapping(items));
        } else {
            prop.insert(Value::String("enum".to_string()), values);
        }
    }

    if let Some(min) = field.min {
        prop.insert(
            Value::String("minimum".to_string()),
            serde_yaml::to_value(min).unwrap(),
        );
    }
    if let Some(max) = field.max {
        prop.insert(
            Value::String("maximum".to_string()),
            serde_yaml::to_value(max).unwrap(),
        );
    }
    if let Some(step) = field.step {
        if step > 0.0 {
            prop.insert(
                Value::String("multipleOf".to_string()),
                serde_yaml::to_value(step).unwrap(),
            );
        }
    }

    Value::Mapping(prop)
}

pub(super) fn schema_ref(target: &str) -> Value {
    Value::Mapping(mapping_with("$ref", Value::String(target.to_string())))
}

pub(super) fn array_items_ref(target: &str) -> Value {
    let mut map = Mapping::new();
    map.insert(
        Value::String("type".to_string()),
        Value::String("array".to_string()),
    );
    map.insert(Value::String("items".to_string()), schema_ref(target));
    Value::Mapping(map)
}

pub(super) fn object_additional_properties_ref(target: &str) -> Value {
    let mut map = Mapping::new();
    map.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    map.insert(
        Value::String("additionalProperties".to_string()),
        schema_ref(target),
    );
    Value::Mapping(map)
}

pub(super) fn object_schema() -> Value {
    let mut map = Mapping::new();
    map.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    map.insert(
        Value::String("default".to_string()),
        Value::Mapping(Mapping::new()),
    );
    map.insert(
        Value::String("additionalProperties".to_string()),
        Value::Bool(true),
    );
    Value::Mapping(map)
}

pub(super) fn non_empty_string_schema() -> Value {
    let mut map = Mapping::new();
    map.insert(
        Value::String("type".to_string()),
        Value::String("string".to_string()),
    );
    map.insert(
        Value::String("minLength".to_string()),
        serde_yaml::to_value(1).expect("minLength"),
    );
    Value::Mapping(map)
}

pub(super) fn suggested_enum_strings(values: Vec<String>) -> Value {
    Value::Mapping(mapping_with(
        "anyOf",
        Value::Sequence(vec![enum_schema(values), non_empty_string_schema()]),
    ))
}

pub(super) fn null_schema() -> Value {
    Value::Mapping(mapping_with("type", Value::String("null".to_string())))
}

pub(super) fn suggested_string_refs(targets: &[&str]) -> Value {
    let mut variants: Vec<Value> = targets.iter().map(|target| schema_ref(target)).collect();
    variants.push(non_empty_string_schema());
    Value::Mapping(mapping_with("anyOf", Value::Sequence(variants)))
}

pub(super) fn nullable_suggested_string_refs(targets: &[&str]) -> Value {
    let mut variants: Vec<Value> = targets.iter().map(|target| schema_ref(target)).collect();
    variants.push(non_empty_string_schema());
    variants.push(null_schema());
    Value::Mapping(mapping_with("anyOf", Value::Sequence(variants)))
}

pub(super) fn mapping_with(key: &str, value: Value) -> Mapping {
    let mut map = Mapping::new();
    map.insert(Value::String(key.to_string()), value);
    map
}

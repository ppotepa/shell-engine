//! Schema source-of-truth for authoring files.
//!
//! This module owns generated authoring schema fragments so the generator CLI,
//! tests, and future editor integrations all consume the same descriptors.

use anyhow::{Context, Result};
use engine_core::authoring::catalog::{behavior_catalog, static_catalog};
use engine_core::effects::{shared_dispatcher, ParamControl};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::repository::is_discoverable_scene_path;

/// One generated schema file produced for a mod.
#[derive(Debug, Clone)]
pub struct GeneratedSchemaFile {
    /// Output file name relative to the generator output directory.
    pub file_name: String,
    /// YAML schema document content represented as structured data.
    pub value: Value,
}

/// Generates every schema fragment for one mod root.
pub fn generate_mod_schema_files(mod_root: &Path) -> Result<Vec<GeneratedSchemaFile>> {
    let mod_name = mod_root
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid mod path: {}", mod_root.display()))?;

    let scene_ids = collect_scene_ids(mod_root)?;
    let scene_paths = collect_scene_paths(mod_root)?;
    let scene_refs: BTreeSet<String> = scene_ids
        .iter()
        .cloned()
        .chain(scene_paths.iter().cloned())
        .collect();
    let object_names = collect_object_names(mod_root)?;
    let object_refs = collect_object_ref_values(mod_root, &object_names)?;
    let mut effect_names = collect_effect_names(mod_root)?;
    for name in static_catalog().effect_names {
        effect_names.insert((*name).to_string());
    }
    let layer_refs = collect_scene_partial_refs(mod_root, "layers")?;
    let sprite_refs = collect_scene_partial_refs(mod_root, "sprites")?;
    let template_refs = collect_scene_partial_refs(mod_root, "templates")?;
    let effect_refs = collect_scene_partial_refs(mod_root, "effects")?;

    // Asset catalogs for autocomplete
    let font_names = collect_font_names(mod_root)?;
    let font_specs = collect_font_specs(&font_names);
    let image_paths = collect_image_paths(mod_root)?;
    let model_paths = collect_model_paths(mod_root)?;
    let sprite_ids = collect_sprite_ids(mod_root)?;
    let template_names = collect_template_names(mod_root)?;

    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/mods/{mod_name}/schemas/catalog.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("schema {mod_name} catalog")),
    );

    let mut defs = Mapping::new();
    defs.insert(
        Value::String("scene_ids".to_string()),
        enum_schema(scene_ids.into_iter().collect()),
    );
    defs.insert(
        Value::String("scene_paths".to_string()),
        enum_schema(scene_paths.into_iter().collect()),
    );
    defs.insert(
        Value::String("scene_refs".to_string()),
        enum_schema(scene_refs.into_iter().collect()),
    );
    defs.insert(
        Value::String("object_names".to_string()),
        enum_schema(object_names.into_iter().collect()),
    );
    defs.insert(
        Value::String("effect_names".to_string()),
        enum_schema(effect_names.iter().cloned().collect()),
    );
    defs.insert(
        Value::String("layer_refs".to_string()),
        enum_schema(layer_refs.into_iter().collect()),
    );
    defs.insert(
        Value::String("sprite_refs".to_string()),
        enum_schema(sprite_refs.into_iter().collect()),
    );
    defs.insert(
        Value::String("template_refs".to_string()),
        enum_schema(template_refs.into_iter().collect()),
    );
    defs.insert(
        Value::String("object_refs".to_string()),
        enum_schema(object_refs.into_iter().collect()),
    );
    defs.insert(
        Value::String("effect_refs".to_string()),
        enum_schema(effect_refs.into_iter().collect()),
    );
    defs.insert(
        Value::String("font_names".to_string()),
        enum_schema(font_names.into_iter().collect()),
    );
    defs.insert(
        Value::String("font_specs".to_string()),
        enum_schema(font_specs.into_iter().collect()),
    );
    defs.insert(
        Value::String("image_paths".to_string()),
        enum_schema(image_paths.into_iter().collect()),
    );
    defs.insert(
        Value::String("model_paths".to_string()),
        enum_schema(model_paths.into_iter().collect()),
    );
    defs.insert(
        Value::String("sprite_ids".to_string()),
        enum_schema(sprite_ids.into_iter().collect()),
    );
    defs.insert(
        Value::String("template_names".to_string()),
        enum_schema(template_names.into_iter().collect()),
    );
    root.insert(Value::String("$defs".to_string()), Value::Mapping(defs));

    Ok(vec![
        output_file("schemas/catalog.yaml".to_string(), Value::Mapping(root)),
        output_file(
            "schemas/mod.yaml".to_string(),
            build_mod_overlay_schema(mod_name),
        ),
        output_file(
            "schemas/scenes.yaml".to_string(),
            build_scene_overlay_schema(mod_name),
        ),
        output_file(
            "schemas/object.yaml".to_string(),
            build_object_doc_overlay_schema(mod_name),
        ),
        output_file(
            "schemas/objects.yaml".to_string(),
            build_objects_file_overlay_schema(mod_name),
        ),
        output_file(
            "schemas/layers.yaml".to_string(),
            build_layers_file_overlay_schema(mod_name),
        ),
        output_file(
            "schemas/templates.yaml".to_string(),
            build_templates_file_overlay_schema(mod_name),
        ),
        output_file(
            "schemas/sprites.yaml".to_string(),
            build_sprites_file_overlay_schema(mod_name),
        ),
        output_file(
            "schemas/effects.yaml".to_string(),
            build_effect_file_overlay_schema(mod_name, &effect_names),
        ),
    ])
}

/// Renders one schema document as YAML with a trailing newline.
pub fn render_schema_file(value: &Value) -> Result<String> {
    let mut yaml = serde_yaml::to_string(value)?;
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }
    Ok(yaml)
}

fn output_file(file_name: String, value: Value) -> GeneratedSchemaFile {
    GeneratedSchemaFile { file_name, value }
}

fn enum_schema(values: Vec<String>) -> Value {
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

/// Builds schema for all built-in behaviors with their parameter schemas.
#[cfg(test)]
fn build_behavior_schema() -> Value {
    use engine_core::authoring::catalog::behavior_catalog;
    use engine_core::authoring::metadata::Requirement;

    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(
            "https://shell-quest.local/schemas/generated/behaviors.schema.yaml".to_string(),
        ),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String("Generated behavior schemas from metadata".to_string()),
    );

    // Build oneOf with all behavior variants
    let catalog = behavior_catalog();
    let mut one_of_variants = Vec::new();

    for (name, fields) in catalog {
        let mut variant = Mapping::new();
        variant.insert(
            Value::String("type".to_string()),
            Value::String("object".to_string()),
        );
        variant.insert(
            Value::String("additionalProperties".to_string()),
            Value::Bool(false),
        );

        // Required name field matching this behavior
        let required = vec![Value::String("name".to_string())];
        let mut properties = Mapping::new();

        // name property with const
        let mut name_prop = Mapping::new();
        name_prop.insert(
            Value::String("const".to_string()),
            Value::String(name.to_string()),
        );
        properties.insert(Value::String("name".to_string()), Value::Mapping(name_prop));

        // params property with nested fields
        if !fields.is_empty() {
            let mut params_schema = Mapping::new();
            params_schema.insert(
                Value::String("type".to_string()),
                Value::String("object".to_string()),
            );
            params_schema.insert(
                Value::String("additionalProperties".to_string()),
                Value::Bool(false),
            );

            let mut param_props = Mapping::new();
            let mut param_required = Vec::new();
            for field in &fields {
                param_props.insert(
                    Value::String(field.name.to_string()),
                    field_metadata_to_schema(field),
                );
                if matches!(field.requirement, Requirement::Required) {
                    param_required.push(Value::String(field.name.to_string()));
                }
            }

            params_schema.insert(
                Value::String("properties".to_string()),
                Value::Mapping(param_props),
            );
            if !param_required.is_empty() {
                params_schema.insert(
                    Value::String("required".to_string()),
                    Value::Sequence(param_required),
                );
            }

            properties.insert(
                Value::String("params".to_string()),
                Value::Mapping(params_schema),
            );
        }

        variant.insert(
            Value::String("properties".to_string()),
            Value::Mapping(properties),
        );
        variant.insert(
            Value::String("required".to_string()),
            Value::Sequence(required),
        );

        one_of_variants.push(Value::Mapping(variant));
    }

    let mut defs = Mapping::new();
    let mut behavior_def = Mapping::new();
    behavior_def.insert(
        Value::String("oneOf".to_string()),
        Value::Sequence(one_of_variants),
    );
    defs.insert(
        Value::String("behavior".to_string()),
        Value::Mapping(behavior_def),
    );

    root.insert(Value::String("$defs".to_string()), Value::Mapping(defs));
    Value::Mapping(root)
}

/// Converts FieldMetadata to JSON Schema property definition.
fn field_metadata_to_schema(field: &engine_core::authoring::metadata::FieldMetadata) -> Value {
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

    // Description
    if !field.description.is_empty() {
        prop.insert(
            Value::String("description".to_string()),
            Value::String(field.description.to_string()),
        );
    }

    // Default
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

    // Enum options
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

    // Number constraints
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
        prop.insert(
            Value::String("multipleOf".to_string()),
            serde_yaml::to_value(step).unwrap(),
        );
    }

    Value::Mapping(prop)
}

/// Builds schema for all built-in animations with their parameter schemas.
#[cfg(test)]
fn build_animation_schema() -> Value {
    use engine_core::authoring::catalog::animation_catalog;
    use engine_core::authoring::metadata::Requirement;

    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(
            "https://shell-quest.local/schemas/generated/animations.schema.yaml".to_string(),
        ),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String("Generated animation schemas from metadata".to_string()),
    );

    // Build oneOf with all animation variants
    let catalog = animation_catalog();
    let mut one_of_variants = Vec::new();

    for (name, fields) in catalog {
        let mut variant = Mapping::new();
        variant.insert(
            Value::String("type".to_string()),
            Value::String("object".to_string()),
        );
        variant.insert(
            Value::String("additionalProperties".to_string()),
            Value::Bool(false),
        );

        // Required name field matching this animation
        let required = vec![Value::String("name".to_string())];
        let mut properties = Mapping::new();

        // name property with const
        let mut name_prop = Mapping::new();
        name_prop.insert(
            Value::String("const".to_string()),
            Value::String(name.to_string()),
        );
        properties.insert(Value::String("name".to_string()), Value::Mapping(name_prop));

        // params property with nested fields
        if !fields.is_empty() {
            let mut params_schema = Mapping::new();
            params_schema.insert(
                Value::String("type".to_string()),
                Value::String("object".to_string()),
            );
            params_schema.insert(
                Value::String("additionalProperties".to_string()),
                Value::Bool(false),
            );

            let mut param_props = Mapping::new();
            let mut param_required = Vec::new();
            for field in &fields {
                param_props.insert(
                    Value::String(field.name.to_string()),
                    field_metadata_to_schema(field),
                );
                if matches!(field.requirement, Requirement::Required) {
                    param_required.push(Value::String(field.name.to_string()));
                }
            }

            params_schema.insert(
                Value::String("properties".to_string()),
                Value::Mapping(param_props),
            );
            if !param_required.is_empty() {
                params_schema.insert(
                    Value::String("required".to_string()),
                    Value::Sequence(param_required),
                );
            }

            properties.insert(
                Value::String("params".to_string()),
                Value::Mapping(params_schema),
            );
        }

        variant.insert(
            Value::String("properties".to_string()),
            Value::Mapping(properties),
        );
        variant.insert(
            Value::String("required".to_string()),
            Value::Sequence(required),
        );

        one_of_variants.push(Value::Mapping(variant));
    }

    let mut defs = Mapping::new();
    let mut animation_def = Mapping::new();
    animation_def.insert(
        Value::String("oneOf".to_string()),
        Value::Sequence(one_of_variants),
    );
    defs.insert(
        Value::String("animation".to_string()),
        Value::Mapping(animation_def),
    );

    root.insert(Value::String("$defs".to_string()), Value::Mapping(defs));
    Value::Mapping(root)
}

/// Builds schema for all built-in input profiles.
#[cfg(test)]
fn build_input_profile_schema() -> Value {
    use engine_core::authoring::catalog::input_profile_catalog;

    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(
            "https://shell-quest.local/schemas/generated/input-profiles.schema.yaml".to_string(),
        ),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String("Generated input profile schemas from metadata".to_string()),
    );

    let mut defs = Mapping::new();

    // Enum of all profile names
    let profiles = input_profile_catalog();
    let mut profile_enum = Mapping::new();
    profile_enum.insert(
        Value::String("type".to_string()),
        Value::String("string".to_string()),
    );
    profile_enum.insert(
        Value::String("enum".to_string()),
        Value::Sequence(
            profiles
                .into_iter()
                .map(|name| Value::String(name.to_string()))
                .collect(),
        ),
    );
    defs.insert(
        Value::String("input_profile".to_string()),
        Value::Mapping(profile_enum),
    );

    root.insert(Value::String("$defs".to_string()), Value::Mapping(defs));
    Value::Mapping(root)
}

/// Builds allOf conditional blocks from RequiredIf metadata for a set of fields.
/// Returns `if/then` JSON Schema objects — one per unique (field, equals) pair.
#[cfg(test)]
#[allow(dead_code)]
fn build_required_if_allof(
    fields: &[engine_core::authoring::metadata::FieldMetadata],
) -> Vec<Value> {
    use engine_core::authoring::metadata::Requirement;
    use std::collections::BTreeMap;

    let mut groups: BTreeMap<(&str, &str), Vec<&str>> = BTreeMap::new();
    for f in fields {
        if let Requirement::RequiredIf { field, equals } = f.requirement {
            groups.entry((field, equals)).or_default().push(f.name);
        }
    }

    groups
        .into_iter()
        .map(|((field, equals), required_fields)| {
            let mut if_props = Mapping::new();
            if_props.insert(
                Value::String(field.to_string()),
                Value::Mapping(mapping_with("const", Value::String(equals.to_string()))),
            );
            let if_block = Value::Mapping(mapping_with("properties", Value::Mapping(if_props)));
            let then_block = Value::Mapping(mapping_with(
                "required",
                Value::Sequence(
                    required_fields
                        .iter()
                        .map(|f| Value::String(f.to_string()))
                        .collect(),
                ),
            ));
            let mut block = Mapping::new();
            block.insert(Value::String("if".to_string()), if_block);
            block.insert(Value::String("then".to_string()), then_block);
            Value::Mapping(block)
        })
        .collect()
}

/// Builds a generated schema for scene/layer/sprite/object field constraints.
/// Emits `$defs` for `sprite_required_if` (if/then allOf blocks from metadata).
/// Referenced by scene.schema.yaml sprite def to keep RequiredIf auto-generated.
#[cfg(test)]
#[allow(dead_code)]
fn build_scene_fields_schema() -> Value {
    use engine_core::scene::{LAYER_FIELDS, OBJECT_FIELDS, SCENE_FIELDS, SPRITE_FIELDS};

    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(
            "https://shell-quest.local/schemas/generated/scene-fields.schema.yaml".to_string(),
        ),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String("Generated scene/layer/sprite/object field constraints".to_string()),
    );

    let mut defs = Mapping::new();

    // sprite_required_if: allOf blocks generated from RequiredIf metadata
    let sprite_if_blocks = build_required_if_allof(SPRITE_FIELDS);
    if !sprite_if_blocks.is_empty() {
        let mut sprite_constraints = Mapping::new();
        sprite_constraints.insert(
            Value::String("allOf".to_string()),
            Value::Sequence(sprite_if_blocks),
        );
        defs.insert(
            Value::String("sprite_required_if".to_string()),
            Value::Mapping(sprite_constraints),
        );
    }

    // scene_required: fields marked Required in SCENE_FIELDS
    let scene_required: Vec<Value> = SCENE_FIELDS
        .iter()
        .filter(|f| f.requirement == engine_core::authoring::metadata::Requirement::Required)
        .map(|f| Value::String(f.name.to_string()))
        .collect();
    if !scene_required.is_empty() {
        defs.insert(
            Value::String("scene_required".to_string()),
            Value::Mapping(mapping_with("required", Value::Sequence(scene_required))),
        );
    }

    // layer_required: fields marked Required in LAYER_FIELDS
    let layer_required: Vec<Value> = LAYER_FIELDS
        .iter()
        .filter(|f| f.requirement == engine_core::authoring::metadata::Requirement::Required)
        .map(|f| Value::String(f.name.to_string()))
        .collect();
    if !layer_required.is_empty() {
        defs.insert(
            Value::String("layer_required".to_string()),
            Value::Mapping(mapping_with("required", Value::Sequence(layer_required))),
        );
    }

    // sprite_required: fields marked Required in SPRITE_FIELDS (non-conditional)
    let sprite_required: Vec<Value> = SPRITE_FIELDS
        .iter()
        .filter(|f| f.requirement == engine_core::authoring::metadata::Requirement::Required)
        .map(|f| Value::String(f.name.to_string()))
        .collect();
    if !sprite_required.is_empty() {
        defs.insert(
            Value::String("sprite_required".to_string()),
            Value::Mapping(mapping_with("required", Value::Sequence(sprite_required))),
        );
    }

    // object_required: fields marked Required in OBJECT_FIELDS
    let object_required: Vec<Value> = OBJECT_FIELDS
        .iter()
        .filter(|f| f.requirement == engine_core::authoring::metadata::Requirement::Required)
        .map(|f| Value::String(f.name.to_string()))
        .collect();
    if !object_required.is_empty() {
        defs.insert(
            Value::String("object_required".to_string()),
            Value::Mapping(mapping_with("required", Value::Sequence(object_required))),
        );
    }

    root.insert(Value::String("$defs".to_string()), Value::Mapping(defs));
    Value::Mapping(root)
}

/// Builds documentation schema for all authoring sugar transformations.
#[cfg(test)]
fn build_sugar_schema() -> Value {
    use engine_core::authoring::catalog::sugar_catalog;

    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String("https://shell-quest.local/schemas/generated/sugar.schema.yaml".to_string()),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String("Authoring sugar transformations catalog".to_string()),
    );
    root.insert(
        Value::String("description".to_string()),
        Value::String("Documents all shorthand syntax, aliases, and normalizers applied during YAML authoring. These are automatically expanded by the compiler before deserialization.".to_string()),
    );

    let catalog = sugar_catalog();
    let mut defs = Mapping::new();

    // Aliases section
    let mut aliases_array = Vec::new();
    for (from, to) in catalog.aliases {
        let mut alias = Mapping::new();
        alias.insert(
            Value::String("from".to_string()),
            Value::String(from.to_string()),
        );
        alias.insert(
            Value::String("to".to_string()),
            Value::String(to.to_string()),
        );
        aliases_array.push(Value::Mapping(alias));
    }
    let mut aliases_def = Mapping::new();
    aliases_def.insert(
        Value::String("description".to_string()),
        Value::String("Field name aliases automatically renamed during compilation".to_string()),
    );
    aliases_def.insert(
        Value::String("type".to_string()),
        Value::String("array".to_string()),
    );
    aliases_def.insert(
        Value::String("items".to_string()),
        Value::Sequence(aliases_array),
    );
    defs.insert(
        Value::String("aliases".to_string()),
        Value::Mapping(aliases_def),
    );

    // Shorthands section
    let mut shorthands_array = Vec::new();
    for shorthand in catalog.shorthands {
        let mut sh = Mapping::new();
        sh.insert(
            Value::String("name".to_string()),
            Value::String(shorthand.name.to_string()),
        );
        sh.insert(
            Value::String("description".to_string()),
            Value::String(shorthand.description.to_string()),
        );
        sh.insert(
            Value::String("from_syntax".to_string()),
            Value::String(shorthand.from_syntax.to_string()),
        );
        sh.insert(
            Value::String("to_structure".to_string()),
            Value::String(shorthand.to_structure.to_string()),
        );
        shorthands_array.push(Value::Mapping(sh));
    }
    let mut shorthands_def = Mapping::new();
    shorthands_def.insert(
        Value::String("description".to_string()),
        Value::String("Shorthand syntax automatically expanded during compilation".to_string()),
    );
    shorthands_def.insert(
        Value::String("type".to_string()),
        Value::String("array".to_string()),
    );
    shorthands_def.insert(
        Value::String("items".to_string()),
        Value::Sequence(shorthands_array),
    );
    defs.insert(
        Value::String("shorthands".to_string()),
        Value::Mapping(shorthands_def),
    );

    // Normalizers section
    let normalizers_array: Vec<Value> = catalog
        .normalizers
        .iter()
        .map(|name| Value::String(name.to_string()))
        .collect();
    let mut normalizers_def = Mapping::new();
    normalizers_def.insert(
        Value::String("description".to_string()),
        Value::String("Normalizer functions applied during document processing (see engine-authoring/src/document/scene.rs)".to_string()),
    );
    normalizers_def.insert(
        Value::String("type".to_string()),
        Value::String("array".to_string()),
    );
    normalizers_def.insert(
        Value::String("items".to_string()),
        Value::Mapping(mapping_with("type", Value::String("string".to_string()))),
    );
    normalizers_def.insert(
        Value::String("enum".to_string()),
        Value::Sequence(normalizers_array),
    );
    defs.insert(
        Value::String("normalizers".to_string()),
        Value::Mapping(normalizers_def),
    );

    root.insert(Value::String("$defs".to_string()), Value::Mapping(defs));
    Value::Mapping(root)
}

fn build_mod_overlay_schema(mod_name: &str) -> Value {
    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/mods/{mod_name}/schemas/mod.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("schema {mod_name} mod")),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../../../schemas/mod.schema.yaml"),
            Value::Mapping(mod_overlay_patch()),
        ]),
    );
    Value::Mapping(root)
}

fn build_scene_overlay_schema(mod_name: &str) -> Value {
    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/mods/{mod_name}/schemas/scenes.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("schema {mod_name} scenes")),
    );
    root.insert(
        Value::String("$defs".to_string()),
        Value::Mapping(shared_overlay_defs()),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../../../schemas/scene.schema.yaml"),
            Value::Mapping(scene_overlay_patch()),
        ]),
    );
    Value::Mapping(root)
}

fn build_object_doc_overlay_schema(mod_name: &str) -> Value {
    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/mods/{mod_name}/schemas/object.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("schema {mod_name} object")),
    );
    root.insert(
        Value::String("$defs".to_string()),
        Value::Mapping(mapping_with(
            "object_logic_overlay",
            Value::Mapping(object_logic_overlay_def()),
        )),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../../../schemas/object.schema.yaml"),
            Value::Mapping(object_doc_overlay_patch()),
        ]),
    );
    Value::Mapping(root)
}

fn build_objects_file_overlay_schema(mod_name: &str) -> Value {
    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/mods/{mod_name}/schemas/objects.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("schema {mod_name} objects")),
    );
    root.insert(
        Value::String("type".to_string()),
        Value::String("array".to_string()),
    );
    root.insert(
        Value::String("items".to_string()),
        Value::Mapping(mapping_with(
            "allOf",
            Value::Sequence(vec![
                schema_ref("../../../schemas/objects-file.schema.yaml#/items"),
                Value::Mapping(object_instance_overlay_patch()),
            ]),
        )),
    );
    Value::Mapping(root)
}

fn build_layers_file_overlay_schema(mod_name: &str) -> Value {
    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/mods/{mod_name}/schemas/layers.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("schema {mod_name} layers")),
    );
    root.insert(
        Value::String("$defs".to_string()),
        Value::Mapping(shared_overlay_defs()),
    );
    root.insert(
        Value::String("type".to_string()),
        Value::String("array".to_string()),
    );
    root.insert(
        Value::String("items".to_string()),
        schema_ref("#/$defs/layer_overlay"),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref("../../../schemas/layers-file.schema.yaml")]),
    );
    Value::Mapping(root)
}

fn build_templates_file_overlay_schema(mod_name: &str) -> Value {
    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/mods/{mod_name}/schemas/templates.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("schema {mod_name} templates")),
    );
    root.insert(
        Value::String("$defs".to_string()),
        Value::Mapping(shared_overlay_defs()),
    );
    root.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    root.insert(
        Value::String("default".to_string()),
        Value::Mapping(Mapping::new()),
    );
    root.insert(
        Value::String("additionalProperties".to_string()),
        schema_ref("#/$defs/sprite_overlay"),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref(
            "../../../schemas/templates-file.schema.yaml",
        )]),
    );
    Value::Mapping(root)
}

fn build_sprites_file_overlay_schema(mod_name: &str) -> Value {
    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/mods/{mod_name}/schemas/sprites.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("schema {mod_name} sprites")),
    );
    root.insert(
        Value::String("$defs".to_string()),
        Value::Mapping(shared_overlay_defs()),
    );
    root.insert(
        Value::String("type".to_string()),
        Value::String("array".to_string()),
    );
    root.insert(
        Value::String("items".to_string()),
        schema_ref("#/$defs/sprite_overlay"),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref(
            "../../../schemas/sprites-file.schema.yaml",
        )]),
    );
    Value::Mapping(root)
}

fn build_effect_file_overlay_schema(mod_name: &str, effect_names: &BTreeSet<String>) -> Value {
    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/mods/{mod_name}/schemas/effects.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("schema {mod_name} effects")),
    );
    root.insert(
        Value::String("type".to_string()),
        Value::String("array".to_string()),
    );
    root.insert(
        Value::String("items".to_string()),
        Value::Mapping(mapping_with(
            "oneOf",
            Value::Sequence(effect_variant_schemas(mod_name, effect_names)),
        )),
    );
    Value::Mapping(root)
}

fn effect_variant_schemas(mod_name: &str, effect_names: &BTreeSet<String>) -> Vec<Value> {
    effect_names
        .iter()
        .map(|effect_name| {
            let meta = shared_dispatcher().metadata(effect_name);
            let mut name_props = Mapping::new();
            name_props.insert(
                Value::String("name".to_string()),
                Value::Mapping(mapping_with(
                    "const",
                    Value::String(effect_name.to_string()),
                )),
            );
            name_props.insert(
                Value::String("params".to_string()),
                effect_params_schema(meta.params),
            );

            let mut patch = Mapping::new();
            patch.insert(
                Value::String("properties".to_string()),
                Value::Mapping(name_props),
            );
            patch.insert(
                Value::String("title".to_string()),
                Value::String(format!("{effect_name} effect variant")),
            );

            Value::Mapping(mapping_with(
                "allOf",
                Value::Sequence(vec![
                    schema_ref("../../../schemas/effect-file.schema.yaml#/items"),
                    Value::Mapping(patch),
                    Value::Mapping(mapping_with(
                        "description",
                        Value::String(format!(
                            "{effect_name} overlay from {mod_name} generated metadata"
                        )),
                    )),
                ]),
            ))
        })
        .collect()
}

fn effect_params_schema(params: &'static [engine_core::effects::ParamMetadata]) -> Value {
    let mut properties = Mapping::new();
    for param in params {
        let mut schema = Mapping::new();
        for (k, v) in param_control_schema(&param.control) {
            schema.insert(k, v);
        }
        schema.insert(
            Value::String("description".to_string()),
            Value::String(param.description.to_string()),
        );
        properties.insert(
            Value::String(param.name.to_string()),
            Value::Mapping(schema),
        );
    }

    let mut map = Mapping::new();
    map.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    map.insert(
        Value::String("additionalProperties".to_string()),
        Value::Bool(false),
    );
    map.insert(
        Value::String("properties".to_string()),
        Value::Mapping(properties),
    );
    Value::Mapping(map)
}

fn param_control_schema(control: &ParamControl) -> Mapping {
    let mut map = Mapping::new();
    match control {
        ParamControl::Slider {
            min,
            max,
            step,
            unit: _,
        } => {
            map.insert(
                Value::String("type".to_string()),
                Value::String("number".to_string()),
            );
            map.insert(
                Value::String("minimum".to_string()),
                serde_yaml::to_value(*min).expect("min value"),
            );
            map.insert(
                Value::String("maximum".to_string()),
                serde_yaml::to_value(*max).expect("max value"),
            );
            map.insert(
                Value::String("multipleOf".to_string()),
                serde_yaml::to_value(*step).expect("step value"),
            );
        }
        ParamControl::Select { options, default } => {
            map.insert(
                Value::String("type".to_string()),
                Value::String("string".to_string()),
            );
            map.insert(
                Value::String("enum".to_string()),
                Value::Sequence(
                    options
                        .iter()
                        .map(|v| Value::String((*v).to_string()))
                        .collect(),
                ),
            );
            map.insert(
                Value::String("default".to_string()),
                Value::String((*default).to_string()),
            );
        }
        ParamControl::Toggle { default } => {
            map.insert(
                Value::String("type".to_string()),
                Value::String("boolean".to_string()),
            );
            map.insert(Value::String("default".to_string()), Value::Bool(*default));
        }
        ParamControl::Text { default } | ParamControl::Colour { default } => {
            map.insert(
                Value::String("type".to_string()),
                Value::String("string".to_string()),
            );
            map.insert(
                Value::String("default".to_string()),
                Value::String((*default).to_string()),
            );
        }
    }
    map
}

fn mod_overlay_patch() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("entrypoint".to_string()),
        Value::Mapping(mapping_with(
            "anyOf",
            Value::Sequence(vec![
                schema_ref("./catalog.yaml#/$defs/scene_paths"),
                schema_ref("../../../schemas/mod.schema.yaml#/properties/entrypoint"),
            ]),
        )),
    );
    let mut root = Mapping::new();
    root.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    root
}

fn scene_overlay_patch() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("next".to_string()),
        nullable_suggested_string_refs(&["./catalog.yaml#/$defs/scene_refs"]),
    );
    props.insert(
        Value::String("menu-options".to_string()),
        menu_options_overlay(),
    );
    props.insert(
        Value::String("menu_options".to_string()),
        menu_options_overlay(),
    );
    props.insert(Value::String("objects".to_string()), objects_overlay());
    props.insert(Value::String("input".to_string()), scene_input_overlay());
    props.insert(
        Value::String("behaviors".to_string()),
        array_items_ref("#/$defs/behavior_overlay"),
    );
    props.insert(
        Value::String("layers".to_string()),
        array_items_ref("#/$defs/layer_overlay"),
    );
    props.insert(
        Value::String("templates".to_string()),
        object_additional_properties_ref("#/$defs/sprite_overlay"),
    );
    props.insert(
        Value::String("stages".to_string()),
        schema_ref("#/$defs/scene_stages_overlay"),
    );

    let mut root = Mapping::new();
    root.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    root
}

fn menu_options_overlay() -> Value {
    let mut option_props = Mapping::new();
    option_props.insert(
        Value::String("next".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/scene_refs"]),
    );
    option_props.insert(
        Value::String("scene".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/scene_refs"]),
    );
    option_props.insert(
        Value::String("to".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/scene_refs"]),
    );
    let mut items = Mapping::new();
    items.insert(
        Value::String("properties".to_string()),
        Value::Mapping(option_props),
    );
    Value::Mapping(mapping_with("items", Value::Mapping(items)))
}

fn objects_overlay() -> Value {
    Value::Mapping(mapping_with(
        "items",
        Value::Mapping(object_instance_overlay_patch()),
    ))
}

fn scene_input_overlay() -> Value {
    let mut obj_viewer_props = Mapping::new();
    obj_viewer_props.insert(
        Value::String("sprite_id".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/sprite_ids"]),
    );
    let mut obj_viewer = Mapping::new();
    obj_viewer.insert(
        Value::String("properties".to_string()),
        Value::Mapping(obj_viewer_props),
    );

    let mut input_props = Mapping::new();
    input_props.insert(
        Value::String("obj-viewer".to_string()),
        Value::Mapping(obj_viewer),
    );

    let mut input = Mapping::new();
    input.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    input.insert(
        Value::String("properties".to_string()),
        Value::Mapping(input_props),
    );
    Value::Mapping(input)
}

fn object_instance_overlay_patch() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("use".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/object_refs"]),
    );
    props.insert(
        Value::String("ref".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/object_refs"]),
    );

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    patch
}

fn object_doc_overlay_patch() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("logic".to_string()),
        schema_ref("#/$defs/object_logic_overlay"),
    );

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    patch
}

fn object_logic_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("behavior".to_string()),
        suggested_enum_strings(
            behavior_catalog()
                .into_iter()
                .map(|(name, _)| name.to_string())
                .collect(),
        ),
    );

    let conditional_blocks: Vec<Value> = behavior_catalog()
        .into_iter()
        .map(|(behavior_name, fields)| {
            let mut if_props = Mapping::new();
            if_props.insert(
                Value::String("behavior".to_string()),
                Value::Mapping(mapping_with(
                    "const",
                    Value::String(behavior_name.to_string()),
                )),
            );
            let mut if_block = Mapping::new();
            if_block.insert(
                Value::String("properties".to_string()),
                Value::Mapping(if_props),
            );

            let mut then_props = Mapping::new();
            then_props.insert(
                Value::String("params".to_string()),
                behavior_params_schema(&fields),
            );
            let mut then_block = Mapping::new();
            then_block.insert(
                Value::String("properties".to_string()),
                Value::Mapping(then_props),
            );

            let mut block = Mapping::new();
            block.insert(Value::String("if".to_string()), Value::Mapping(if_block));
            block.insert(
                Value::String("then".to_string()),
                Value::Mapping(then_block),
            );
            Value::Mapping(block)
        })
        .collect();

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    if !conditional_blocks.is_empty() {
        patch.insert(
            Value::String("allOf".to_string()),
            Value::Sequence(conditional_blocks),
        );
    }

    let mut overlay = Mapping::new();
    overlay.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../../../schemas/object.schema.yaml#/properties/logic"),
            Value::Mapping(patch),
        ]),
    );
    overlay
}

fn behavior_params_schema(fields: &[engine_core::authoring::metadata::FieldMetadata]) -> Value {
    use engine_core::authoring::metadata::Requirement;

    let mut props = Mapping::new();
    let mut required = Vec::new();
    for field in fields {
        props.insert(
            Value::String(field.name.to_string()),
            field_metadata_to_schema(field),
        );
        if matches!(field.requirement, Requirement::Required) {
            required.push(Value::String(field.name.to_string()));
        }
    }

    let mut schema = Mapping::new();
    schema.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    schema.insert(
        Value::String("additionalProperties".to_string()),
        Value::Bool(false),
    );
    schema.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    if !required.is_empty() {
        schema.insert(
            Value::String("required".to_string()),
            Value::Sequence(required),
        );
    }
    Value::Mapping(schema)
}

fn shared_overlay_defs() -> Mapping {
    let mut defs = Mapping::new();
    defs.insert(
        Value::String("step_overlay".to_string()),
        Value::Mapping(step_overlay_def()),
    );
    defs.insert(
        Value::String("stage_overlay".to_string()),
        Value::Mapping(stage_overlay_def()),
    );
    defs.insert(
        Value::String("scene_stages_overlay".to_string()),
        Value::Mapping(scene_stages_overlay_def()),
    );
    defs.insert(
        Value::String("lifecycle_stages_overlay".to_string()),
        Value::Mapping(lifecycle_stages_overlay_def()),
    );
    defs.insert(
        Value::String("sprite_overlay".to_string()),
        Value::Mapping(sprite_overlay_def()),
    );
    defs.insert(
        Value::String("behavior_overlay".to_string()),
        Value::Mapping(behavior_overlay_def()),
    );
    defs.insert(
        Value::String("layer_overlay".to_string()),
        Value::Mapping(layer_overlay_def()),
    );
    defs
}

fn step_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("effects".to_string()),
        array_items_ref("./effects.yaml#/items"),
    );

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );

    let mut step = Mapping::new();
    step.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../../../schemas/scene.schema.yaml#/$defs/step"),
            Value::Mapping(patch),
        ]),
    );
    step
}

fn scene_stages_overlay_def() -> Mapping {
    lifecycle_stages_overlay_def_with_base("../../../schemas/scene.schema.yaml#/$defs/scene_stages")
}

fn lifecycle_stages_overlay_def() -> Mapping {
    lifecycle_stages_overlay_def_with_base("../../../schemas/scene.schema.yaml#/$defs/layer_stages")
}

fn lifecycle_stages_overlay_def_with_base(base_ref: &str) -> Mapping {
    let mut stage_props = Mapping::new();
    for stage in ["on_enter", "on_idle", "on_leave"] {
        stage_props.insert(
            Value::String(stage.to_string()),
            schema_ref("#/$defs/stage_overlay"),
        );
    }

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(stage_props),
    );

    let mut stages = Mapping::new();
    stages.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref(base_ref), Value::Mapping(patch)]),
    );
    stages
}

fn stage_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("steps".to_string()),
        array_items_ref("#/$defs/step_overlay"),
    );

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );

    let mut stage = Mapping::new();
    stage.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../../../schemas/scene.schema.yaml#/$defs/stage"),
            Value::Mapping(patch),
        ]),
    );
    stage
}

fn layer_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("stages".to_string()),
        schema_ref("#/$defs/lifecycle_stages_overlay"),
    );
    props.insert(
        Value::String("behaviors".to_string()),
        array_items_ref("#/$defs/behavior_overlay"),
    );
    props.insert(
        Value::String("sprites".to_string()),
        array_items_ref("#/$defs/sprite_overlay"),
    );

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );

    let mut layer = Mapping::new();
    layer.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../../../schemas/scene.schema.yaml#/$defs/layer"),
            Value::Mapping(patch),
        ]),
    );
    layer
}

fn sprite_overlay_def() -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("use".to_string()),
        suggested_string_refs(&["./catalog.yaml#/$defs/template_names"]),
    );
    props.insert(Value::String("args".to_string()), object_schema());
    props.insert(
        Value::String("source".to_string()),
        suggested_string_refs(&[
            "./catalog.yaml#/$defs/image_paths",
            "./catalog.yaml#/$defs/model_paths",
        ]),
    );
    props.insert(
        Value::String("font".to_string()),
        nullable_suggested_string_refs(&["./catalog.yaml#/$defs/font_specs"]),
    );
    props.insert(
        Value::String("stages".to_string()),
        schema_ref("#/$defs/lifecycle_stages_overlay"),
    );
    props.insert(
        Value::String("behaviors".to_string()),
        array_items_ref("#/$defs/behavior_overlay"),
    );
    props.insert(
        Value::String("children".to_string()),
        array_items_ref("#/$defs/sprite_overlay"),
    );

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );

    let mut sprite = Mapping::new();
    sprite.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../../../schemas/scene.schema.yaml#/$defs/sprite"),
            Value::Mapping(patch),
        ]),
    );
    sprite
}

fn behavior_overlay_def() -> Mapping {
    let variants = behavior_catalog()
        .into_iter()
        .map(|(behavior_name, fields)| behavior_variant_overlay(behavior_name, &fields))
        .collect();

    let mut behavior = Mapping::new();
    behavior.insert(
        Value::String("oneOf".to_string()),
        Value::Sequence(variants),
    );
    behavior
}

fn behavior_variant_overlay(
    behavior_name: &str,
    fields: &[engine_core::authoring::metadata::FieldMetadata],
) -> Value {
    let mut props = Mapping::new();
    props.insert(
        Value::String("name".to_string()),
        Value::Mapping(mapping_with(
            "const",
            Value::String(behavior_name.to_string()),
        )),
    );

    let mut params_props = Mapping::new();
    for field in fields {
        if matches!(field.name, "target" | "sprite_id") {
            params_props.insert(
                Value::String(field.name.to_string()),
                suggested_string_refs(&["./catalog.yaml#/$defs/sprite_ids"]),
            );
        }
    }

    if !params_props.is_empty() {
        let mut params = Mapping::new();
        params.insert(
            Value::String("type".to_string()),
            Value::String("object".to_string()),
        );
        params.insert(
            Value::String("properties".to_string()),
            Value::Mapping(params_props),
        );
        props.insert(Value::String("params".to_string()), Value::Mapping(params));
    }

    let mut patch = Mapping::new();
    patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    patch.insert(
        Value::String("title".to_string()),
        Value::String(format!("{behavior_name} behavior overlay")),
    );

    Value::Mapping(mapping_with(
        "allOf",
        Value::Sequence(vec![
            schema_ref("../../../schemas/generated/behaviors.schema.yaml#/$defs/behavior"),
            Value::Mapping(patch),
        ]),
    ))
}

fn schema_ref(target: &str) -> Value {
    Value::Mapping(mapping_with("$ref", Value::String(target.to_string())))
}

fn array_items_ref(target: &str) -> Value {
    let mut map = Mapping::new();
    map.insert(
        Value::String("type".to_string()),
        Value::String("array".to_string()),
    );
    map.insert(Value::String("items".to_string()), schema_ref(target));
    Value::Mapping(map)
}

fn object_additional_properties_ref(target: &str) -> Value {
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

fn object_schema() -> Value {
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

fn non_empty_string_schema() -> Value {
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

fn suggested_enum_strings(values: Vec<String>) -> Value {
    Value::Mapping(mapping_with(
        "anyOf",
        Value::Sequence(vec![enum_schema(values), non_empty_string_schema()]),
    ))
}

fn null_schema() -> Value {
    Value::Mapping(mapping_with("type", Value::String("null".to_string())))
}

fn suggested_string_refs(targets: &[&str]) -> Value {
    let mut variants: Vec<Value> = targets.iter().map(|target| schema_ref(target)).collect();
    variants.push(non_empty_string_schema());
    Value::Mapping(mapping_with("anyOf", Value::Sequence(variants)))
}

fn nullable_suggested_string_refs(targets: &[&str]) -> Value {
    let mut variants: Vec<Value> = targets.iter().map(|target| schema_ref(target)).collect();
    variants.push(non_empty_string_schema());
    variants.push(null_schema());
    Value::Mapping(mapping_with("anyOf", Value::Sequence(variants)))
}

fn mapping_with(key: &str, value: Value) -> Mapping {
    let mut map = Mapping::new();
    map.insert(Value::String(key.to_string()), value);
    map
}

fn collect_scene_ids(mod_root: &Path) -> Result<BTreeSet<String>> {
    let scenes_root = mod_root.join("scenes");
    let mut ids = BTreeSet::new();
    for file in yaml_files_under(&scenes_root)? {
        let rel = match file.strip_prefix(mod_root) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        if !is_discoverable_scene_path(&rel) {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&file) {
            if let Ok(v) = serde_yaml::from_str::<Value>(&raw) {
                if let Some(id) = v.get("id").and_then(Value::as_str) {
                    ids.insert(id.to_string());
                }
            }
        }
    }
    Ok(ids)
}

fn collect_scene_paths(mod_root: &Path) -> Result<BTreeSet<String>> {
    let scenes_root = mod_root.join("scenes");
    let mut paths = BTreeSet::new();
    for file in yaml_files_under(&scenes_root)? {
        let rel = match file.strip_prefix(mod_root) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        if !is_discoverable_scene_path(&rel) {
            continue;
        }
        paths.insert(format!("/{rel}"));
    }
    Ok(paths)
}

fn collect_object_names(mod_root: &Path) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for file in yaml_files_under(&mod_root.join("objects"))? {
        if let Ok(raw) = fs::read_to_string(&file) {
            if let Ok(v) = serde_yaml::from_str::<Value>(&raw) {
                if let Some(name) = v.get("name").and_then(Value::as_str) {
                    names.insert(name.to_string());
                }
            }
        }
    }
    Ok(names)
}

fn collect_object_ref_values(
    mod_root: &Path,
    object_names: &BTreeSet<String>,
) -> Result<BTreeSet<String>> {
    let mut refs = object_names.clone();

    for file in yaml_files_under(&mod_root.join("objects"))? {
        if let Ok(rel) = file.strip_prefix(mod_root) {
            refs.insert(format!("/{}", rel.to_string_lossy().replace('\\', "/")));
        }
    }

    for file in yaml_files_under(&mod_root.join("scenes/shared/objects"))? {
        if let Ok(rel) = file.strip_prefix(mod_root) {
            refs.insert(format!("/{}", rel.to_string_lossy().replace('\\', "/")));
        }
    }

    Ok(refs)
}

fn collect_effect_names(mod_root: &Path) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for file in yaml_files_under(&mod_root.join("scenes"))? {
        if let Ok(raw) = fs::read_to_string(&file) {
            if let Ok(v) = serde_yaml::from_str::<Value>(&raw) {
                collect_effect_names_from_value(&v, &mut names);
            }
        }
    }
    Ok(names)
}

fn collect_effect_names_from_value(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Mapping(map) => {
            if let Some(name) = map
                .get(Value::String("name".to_string()))
                .and_then(Value::as_str)
            {
                if map.contains_key(Value::String("duration".to_string())) {
                    out.insert(name.to_string());
                }
            }
            for v in map.values() {
                collect_effect_names_from_value(v, out);
            }
        }
        Value::Sequence(seq) => {
            for entry in seq {
                collect_effect_names_from_value(entry, out);
            }
        }
        _ => {}
    }
}

fn collect_scene_partial_refs(mod_root: &Path, part_dir: &str) -> Result<BTreeSet<String>> {
    let scenes_root = mod_root.join("scenes");
    if !scenes_root.exists() {
        return Ok(BTreeSet::new());
    }
    let mut refs = BTreeSet::new();
    for scene_dir in fs::read_dir(&scenes_root)
        .with_context(|| format!("failed to read {}", scenes_root.display()))?
    {
        let scene_dir = scene_dir?;
        let scene_path = scene_dir.path();
        if !scene_path.is_dir() {
            continue;
        }
        let scene_name = match scene_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };
        let part_root = scene_path.join(part_dir);
        if !part_root.exists() {
            continue;
        }
        for file in yaml_files_under(&part_root)? {
            if let Ok(rel) = file.strip_prefix(&part_root) {
                refs.insert(format!("{scene_name}/{part_dir}/{}", rel.to_string_lossy()));
            }
        }
    }
    Ok(refs)
}

/// Collects font names from assets/fonts/**/manifest.yaml files.
fn collect_font_names(mod_root: &Path) -> Result<BTreeSet<String>> {
    let fonts_root = mod_root.join("assets/fonts");
    let mut names = BTreeSet::new();
    if !fonts_root.exists() {
        return Ok(names);
    }
    for manifest_file in yaml_files_under(&fonts_root)? {
        if manifest_file.file_name().and_then(|n| n.to_str()) != Some("manifest.yaml") {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&manifest_file) {
            if let Ok(v) = serde_yaml::from_str::<Value>(&raw) {
                if let Some(name) = v.get("name").and_then(Value::as_str) {
                    names.insert(name.to_string());
                }
            }
        }
    }
    Ok(names)
}

fn collect_font_specs(font_names: &BTreeSet<String>) -> BTreeSet<String> {
    let mut specs = BTreeSet::from([
        "generic".to_string(),
        "generic:1".to_string(),
        "generic:tiny".to_string(),
        "generic:2".to_string(),
        "generic:standard".to_string(),
        "generic:3".to_string(),
        "generic:large".to_string(),
        "generic:half".to_string(),
        "generic:quad".to_string(),
        "generic:braille".to_string(),
    ]);

    for name in font_names {
        specs.insert(name.clone());
        for mode in ["ascii", "raster", "terminal-pixels"] {
            specs.insert(format!("{name}:{mode}"));
        }
    }

    specs
}

/// Collects image paths from assets/images/**/*.png files.
fn collect_image_paths(mod_root: &Path) -> Result<BTreeSet<String>> {
    let images_root = mod_root.join("assets/images");
    let mut paths = BTreeSet::new();
    if !images_root.exists() {
        return Ok(paths);
    }
    walk_images(&images_root, &images_root, &mut paths)?;
    Ok(paths)
}

fn walk_images(root: &Path, current: &Path, out: &mut BTreeSet<String>) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read {}", current.display()))?
    {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            walk_images(root, &p, out)?;
            continue;
        }
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or_default();
        if ext == "png" {
            if let Ok(rel) = p.strip_prefix(root) {
                out.insert(format!(
                    "/assets/images/{}",
                    rel.to_string_lossy().replace('\\', "/")
                ));
            }
        }
    }
    Ok(())
}

/// Collects OBJ model paths from scenes/**/*.obj and assets/models/**/*.obj files.
fn collect_model_paths(mod_root: &Path) -> Result<BTreeSet<String>> {
    let mut paths = BTreeSet::new();

    // Collect from scenes/**/*.obj
    let scenes_root = mod_root.join("scenes");
    if scenes_root.exists() {
        walk_models(&scenes_root, &scenes_root, &mut paths, "scenes")?;
    }

    // Collect from assets/models/**/*.obj
    let models_root = mod_root.join("assets/models");
    if models_root.exists() {
        walk_models(&models_root, &models_root, &mut paths, "assets/models")?;
    }

    Ok(paths)
}

fn walk_models(
    root: &Path,
    current: &Path,
    out: &mut BTreeSet<String>,
    prefix: &str,
) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read {}", current.display()))?
    {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            walk_models(root, &p, out, prefix)?;
            continue;
        }
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or_default();
        if ext == "obj" {
            if let Ok(rel) = p.strip_prefix(root) {
                out.insert(format!(
                    "/{}/{}",
                    prefix,
                    rel.to_string_lossy().replace('\\', "/")
                ));
            }
        }
    }
    Ok(())
}

/// Collects sprite IDs from all scene YAML files.
fn collect_sprite_ids(mod_root: &Path) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for file in yaml_files_under(&mod_root.join("scenes"))? {
        if let Ok(raw) = fs::read_to_string(&file) {
            if let Ok(v) = serde_yaml::from_str::<Value>(&raw) {
                collect_sprite_ids_from_value(&v, &mut ids);
            }
        }
    }
    Ok(ids)
}

fn collect_sprite_ids_from_value(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Mapping(map) => {
            // Check if this is a sprite with an id field
            if let Some(id) = map
                .get(Value::String("id".to_string()))
                .and_then(Value::as_str)
            {
                // Verify it's actually a sprite by checking for sprite-related fields
                if map.contains_key(Value::String("type".to_string()))
                    || map.contains_key(Value::String("content".to_string()))
                    || map.contains_key(Value::String("source".to_string()))
                {
                    out.insert(id.to_string());
                }
            }
            for v in map.values() {
                collect_sprite_ids_from_value(v, out);
            }
        }
        Value::Sequence(seq) => {
            for entry in seq {
                collect_sprite_ids_from_value(entry, out);
            }
        }
        _ => {}
    }
}

/// Collects template names from scenes/**/templates/*.yml files.
fn collect_template_names(mod_root: &Path) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    let scenes_root = mod_root.join("scenes");
    if !scenes_root.exists() {
        return Ok(names);
    }

    for file in yaml_files_under(&scenes_root)? {
        let rel = match file.strip_prefix(mod_root) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        let Ok(raw) = fs::read_to_string(&file) else {
            continue;
        };
        let Ok(v) = serde_yaml::from_str::<Value>(&raw) else {
            continue;
        };

        if is_discoverable_scene_path(&rel) {
            if let Some(templates) = v
                .as_mapping()
                .and_then(|map| map.get(Value::String("templates".to_string())))
                .and_then(Value::as_mapping)
            {
                collect_template_names_from_mapping(templates, &mut names);
            }
            continue;
        }

        if rel.contains("/templates/") {
            if let Some(map) = v.as_mapping() {
                collect_template_names_from_mapping(map, &mut names);
            }
        }
    }
    Ok(names)
}

fn collect_template_names_from_mapping(map: &Mapping, out: &mut BTreeSet<String>) {
    for key in map.keys() {
        if let Some(name) = key.as_str() {
            out.insert(name.to_string());
        }
    }
}

fn yaml_files_under(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    walk_yaml(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_yaml(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            walk_yaml(&p, out)?;
            continue;
        }
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or_default();
        if ext == "yml" || ext == "yaml" {
            out.push(p);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        build_animation_schema, build_behavior_schema, build_input_profile_schema,
        build_sugar_schema, generate_mod_schema_files, render_schema_file,
    };
    use serde_yaml::Value;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn generate_fragment_contains_dynamic_defs() {
        let temp_root = unique_temp_dir("authoring-schema-test");
        let mod_root = temp_root.join("playground");
        fs::create_dir_all(mod_root.join("scenes/intro/layers")).expect("create layers");
        fs::create_dir_all(mod_root.join("scenes/intro/sprites")).expect("create sprites");
        fs::create_dir_all(mod_root.join("scenes/intro/templates")).expect("create templates");
        fs::create_dir_all(mod_root.join("scenes/shared/objects")).expect("create shared objects");
        fs::create_dir_all(mod_root.join("objects")).expect("create objects");
        fs::create_dir_all(mod_root.join("assets/fonts/mono")).expect("create fonts");
        fs::create_dir_all(mod_root.join("assets/images")).expect("create images");
        fs::write(mod_root.join("mod.yaml"), "name: playground\n").expect("write mod");
        fs::write(
            mod_root.join("scenes/intro/scene.yml"),
            "id: intro\ntemplates:\n  title-card:\n    type: text\n    content: TEST\neffects:\n  - name: fade-in\n    duration: 1.0\n",
        )
        .expect("write scene");
        fs::write(
            mod_root.join("scenes/intro/layers/bg.yml"),
            "name: background\n",
        )
        .expect("write layer partial");
        fs::write(
            mod_root.join("scenes/intro/templates/common.yml"),
            "menu-item:\n  type: text\n  content: START\n",
        )
        .expect("write template partial");
        fs::write(mod_root.join("objects/npc.yml"), "name: npc\n").expect("write object");
        fs::write(
            mod_root.join("scenes/shared/objects/banner.yml"),
            "name: banner\nsprites:\n  - type: text\n    content: SHARED\n",
        )
        .expect("write shared object");
        fs::write(
            mod_root.join("assets/fonts/mono/manifest.yaml"),
            "name: Mono Display\n",
        )
        .expect("write font manifest");
        fs::write(mod_root.join("assets/images/logo.png"), b"").expect("write image");
        fs::write(mod_root.join("scenes/intro/cube.obj"), "").expect("write model");

        let files = generate_mod_schema_files(&mod_root).expect("generate schemas");
        let object_overlay = files
            .iter()
            .find(|file| file.file_name == "schemas/object.yaml")
            .expect("object overlay");
        let root = files
            .iter()
            .find(|file| file.file_name == "schemas/catalog.yaml")
            .expect("root schema");
        let yaml = root.value.as_mapping().expect("schema mapping");
        let defs = yaml
            .get(Value::String("$defs".to_string()))
            .and_then(Value::as_mapping)
            .expect("defs mapping");

        assert!(defs.contains_key(Value::String("scene_ids".to_string())));
        assert!(defs.contains_key(Value::String("scene_paths".to_string())));
        assert!(defs.contains_key(Value::String("scene_refs".to_string())));
        assert!(defs.contains_key(Value::String("object_names".to_string())));
        assert!(defs.contains_key(Value::String("object_refs".to_string())));
        assert!(defs.contains_key(Value::String("effect_names".to_string())));
        assert!(defs.contains_key(Value::String("layer_refs".to_string())));
        assert!(defs.contains_key(Value::String("font_names".to_string())));
        assert!(defs.contains_key(Value::String("font_specs".to_string())));
        assert!(defs.contains_key(Value::String("image_paths".to_string())));
        assert!(defs.contains_key(Value::String("model_paths".to_string())));
        assert!(defs.contains_key(Value::String("sprite_ids".to_string())));
        assert!(defs.contains_key(Value::String("template_names".to_string())));

        let scene_paths = defs
            .get(Value::String("scene_paths".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("enum".to_string())))
            .and_then(Value::as_sequence)
            .expect("scene_paths enum");
        assert!(scene_paths
            .iter()
            .any(|v| v.as_str() == Some("/scenes/intro/scene.yml")));

        let scene_refs = defs
            .get(Value::String("scene_refs".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("enum".to_string())))
            .and_then(Value::as_sequence)
            .expect("scene_refs enum");
        assert!(scene_refs.iter().any(|v| v.as_str() == Some("intro")));
        assert!(scene_refs
            .iter()
            .any(|v| v.as_str() == Some("/scenes/intro/scene.yml")));

        let object_refs = defs
            .get(Value::String("object_refs".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("enum".to_string())))
            .and_then(Value::as_sequence)
            .expect("object_refs enum");
        assert!(object_refs.iter().any(|v| v.as_str() == Some("npc")));
        assert!(object_refs
            .iter()
            .any(|v| v.as_str() == Some("/objects/npc.yml")));
        assert!(object_refs
            .iter()
            .any(|v| v.as_str() == Some("/scenes/shared/objects/banner.yml")));

        let font_specs = defs
            .get(Value::String("font_specs".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("enum".to_string())))
            .and_then(Value::as_sequence)
            .expect("font_specs enum");
        assert!(font_specs
            .iter()
            .any(|v| v.as_str() == Some("generic:quad")));
        assert!(font_specs
            .iter()
            .any(|v| v.as_str() == Some("Mono Display:raster")));

        let image_paths = defs
            .get(Value::String("image_paths".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("enum".to_string())))
            .and_then(Value::as_sequence)
            .expect("image_paths enum");
        assert!(image_paths
            .iter()
            .any(|v| v.as_str() == Some("/assets/images/logo.png")));

        let model_paths = defs
            .get(Value::String("model_paths".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("enum".to_string())))
            .and_then(Value::as_sequence)
            .expect("model_paths enum");
        assert!(model_paths
            .iter()
            .any(|v| v.as_str() == Some("/scenes/intro/cube.obj")));

        let template_names = defs
            .get(Value::String("template_names".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("enum".to_string())))
            .and_then(Value::as_sequence)
            .expect("template_names enum");
        assert!(template_names
            .iter()
            .any(|v| v.as_str() == Some("title-card")));
        assert!(template_names
            .iter()
            .any(|v| v.as_str() == Some("menu-item")));

        let object_overlay_yaml =
            render_schema_file(&object_overlay.value).expect("render object overlay");
        assert!(object_overlay_yaml.contains("../../../schemas/object.schema.yaml"));
        assert!(object_overlay_yaml.contains("const: blink"));
        assert!(object_overlay_yaml.contains("behavior:"));
        assert!(object_overlay_yaml.contains("visible_ms:"));
    }

    #[test]
    fn committed_generated_schemas_are_current() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .canonicalize()
            .expect("repo root");
        for mod_name in ["playground", "shell-quest"] {
            let mod_root = repo_root.join("mods").join(mod_name);
            let files = generate_mod_schema_files(&mod_root).expect("generate committed schemas");
            assert!(!files.is_empty(), "expected schema files for {mod_name}");
        }
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{}-{now}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn test_collect_font_names() {
        let temp = unique_temp_dir("font-test");
        fs::create_dir_all(temp.join("assets/fonts/mono")).unwrap();
        fs::write(
            temp.join("assets/fonts/mono/manifest.yaml"),
            "name: mono-bold\n",
        )
        .unwrap();

        let names = super::collect_font_names(&temp).unwrap();
        assert!(names.contains("mono-bold"));
    }

    #[test]
    fn test_collect_image_paths() {
        let temp = unique_temp_dir("image-test");
        fs::create_dir_all(temp.join("assets/images/ui")).unwrap();
        fs::write(temp.join("assets/images/logo.png"), b"").unwrap();
        fs::write(temp.join("assets/images/ui/button.png"), b"").unwrap();

        let paths = super::collect_image_paths(&temp).unwrap();
        assert!(paths.contains("/assets/images/logo.png"));
        assert!(paths.contains("/assets/images/ui/button.png"));
    }

    #[test]
    fn test_collect_model_paths() {
        let temp = unique_temp_dir("model-test");
        fs::create_dir_all(temp.join("scenes/intro")).unwrap();
        fs::create_dir_all(temp.join("assets/models")).unwrap();
        fs::write(temp.join("scenes/intro/cube.obj"), "").unwrap();
        fs::write(temp.join("assets/models/sphere.obj"), "").unwrap();

        let paths = super::collect_model_paths(&temp).unwrap();
        assert!(paths.contains("/scenes/intro/cube.obj"));
        assert!(paths.contains("/assets/models/sphere.obj"));
    }

    #[test]
    fn test_collect_sprite_ids() {
        let temp = unique_temp_dir("sprite-id-test");
        fs::create_dir_all(temp.join("scenes")).unwrap();
        fs::write(
            temp.join("scenes/test.yml"),
            "layers:\n  - sprites:\n      - id: logo\n        type: text\n        content: Test\n",
        )
        .unwrap();

        let ids = super::collect_sprite_ids(&temp).unwrap();
        assert!(ids.contains("logo"));
    }

    #[test]
    fn test_collect_template_names() {
        let temp = unique_temp_dir("template-test");
        fs::create_dir_all(temp.join("scenes/intro/templates")).unwrap();
        fs::write(
            temp.join("scenes/intro/templates/button.yml"),
            "menu-button:\n  type: text\n  content: START\n",
        )
        .unwrap();

        let names = super::collect_template_names(&temp).unwrap();
        assert!(names.contains("menu-button"));
    }

    #[test]
    fn test_behavior_schema_generation() {
        let behavior_schema = build_behavior_schema();

        let defs = behavior_schema
            .as_mapping()
            .and_then(|m| m.get(Value::String("$defs".to_string())))
            .and_then(Value::as_mapping)
            .expect("$defs in behaviors schema");

        let behavior_def = defs
            .get(Value::String("behavior".to_string()))
            .and_then(Value::as_mapping)
            .expect("behavior def");

        let one_of = behavior_def
            .get(Value::String("oneOf".to_string()))
            .and_then(Value::as_sequence)
            .expect("oneOf variants");

        assert!(!one_of.is_empty(), "should have behavior variants");

        // Check that at least one known behavior exists
        let has_blink = one_of.iter().any(|variant| {
            variant
                .as_mapping()
                .and_then(|m| m.get(Value::String("properties".to_string())))
                .and_then(Value::as_mapping)
                .and_then(|props| props.get(Value::String("name".to_string())))
                .and_then(Value::as_mapping)
                .and_then(|name_prop| name_prop.get(Value::String("const".to_string())))
                .and_then(Value::as_str)
                == Some("blink")
        });
        assert!(has_blink, "blink behavior should be in schema");
    }

    #[test]
    fn test_animation_schema_generation() {
        let animation_schema = build_animation_schema();

        let defs = animation_schema
            .as_mapping()
            .and_then(|m| m.get(Value::String("$defs".to_string())))
            .and_then(Value::as_mapping)
            .expect("$defs in animations schema");

        let animation_def = defs
            .get(Value::String("animation".to_string()))
            .and_then(Value::as_mapping)
            .expect("animation def");

        let one_of = animation_def
            .get(Value::String("oneOf".to_string()))
            .and_then(Value::as_sequence)
            .expect("oneOf variants");

        assert!(!one_of.is_empty(), "should have animation variants");

        // Check that float animation exists
        let has_float = one_of.iter().any(|variant| {
            variant
                .as_mapping()
                .and_then(|m| m.get(Value::String("properties".to_string())))
                .and_then(Value::as_mapping)
                .and_then(|props| props.get(Value::String("name".to_string())))
                .and_then(Value::as_mapping)
                .and_then(|name_prop| name_prop.get(Value::String("const".to_string())))
                .and_then(Value::as_str)
                == Some("float")
        });
        assert!(has_float, "float animation should be in schema");
    }

    #[test]
    fn test_input_profile_schema_generation() {
        let profile_schema = build_input_profile_schema();

        let defs = profile_schema
            .as_mapping()
            .and_then(|m| m.get(Value::String("$defs".to_string())))
            .and_then(Value::as_mapping)
            .expect("$defs in input-profiles schema");

        let profile_def = defs
            .get(Value::String("input_profile".to_string()))
            .and_then(Value::as_mapping)
            .expect("input_profile def");

        let enum_values = profile_def
            .get(Value::String("enum".to_string()))
            .and_then(Value::as_sequence)
            .expect("enum values");

        assert!(!enum_values.is_empty(), "should have profile values");

        // Check that known profiles exist
        let has_obj_viewer = enum_values.iter().any(|v| v.as_str() == Some("obj-viewer"));
        let has_terminal_tester = enum_values
            .iter()
            .any(|v| v.as_str() == Some("terminal-size-tester"));

        assert!(has_obj_viewer, "obj-viewer profile should be in schema");
        assert!(
            has_terminal_tester,
            "terminal-size-tester profile should be in schema"
        );
    }

    #[test]
    fn test_sugar_schema_generation() {
        let sugar_schema = build_sugar_schema();

        let defs = sugar_schema
            .as_mapping()
            .and_then(|m| m.get(Value::String("$defs".to_string())))
            .and_then(Value::as_mapping)
            .expect("$defs in sugar schema");

        // Check aliases
        let aliases = defs
            .get(Value::String("aliases".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("items".to_string())))
            .and_then(Value::as_sequence)
            .expect("aliases items");
        assert!(!aliases.is_empty(), "should have alias definitions");

        // Check shorthands
        let shorthands = defs
            .get(Value::String("shorthands".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("items".to_string())))
            .and_then(Value::as_sequence)
            .expect("shorthands items");
        assert!(!shorthands.is_empty(), "should have shorthand definitions");

        // Check that pause shorthand exists
        let has_pause = shorthands.iter().any(|sh| {
            sh.as_mapping()
                .and_then(|m| m.get(Value::String("name".to_string())))
                .and_then(Value::as_str)
                == Some("pause")
        });
        assert!(has_pause, "pause shorthand should be in schema");

        // Check normalizers
        let normalizers = defs
            .get(Value::String("normalizers".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(Value::String("enum".to_string())))
            .and_then(Value::as_sequence)
            .expect("normalizers enum");
        assert!(!normalizers.is_empty(), "should have normalizer names");
    }

    #[test]
    fn test_no_schema_drift() {
        // Verify that generated schemas include all runtime behaviors and animations
        use engine_core::authoring::catalog::{animation_catalog, behavior_catalog};
        use serde_yaml::Value;

        let behavior_schema = build_behavior_schema();
        let animation_schema = build_animation_schema();

        // Check behaviors
        let behavior_catalog = behavior_catalog();
        let defs = behavior_schema.get("$defs").expect("$defs");
        let defs_map = defs.as_mapping().expect("$defs as mapping");
        let behavior = defs_map
            .get(&Value::String("behavior".to_string()))
            .expect("behavior");
        let behavior_map = behavior.as_mapping().expect("behavior as mapping");
        let oneof = behavior_map
            .get(&Value::String("oneOf".to_string()))
            .expect("oneOf");
        let behavior_oneof = oneof.as_sequence().expect("oneOf as sequence");

        assert_eq!(
            behavior_oneof.len(),
            behavior_catalog.len(),
            "Generated schema should have oneOf entry for each behavior in catalog"
        );

        // Check animations
        let animation_catalog = animation_catalog();
        let defs = animation_schema.get("$defs").expect("$defs");
        let defs_map = defs.as_mapping().expect("$defs as mapping");
        let animation = defs_map
            .get(&Value::String("animation".to_string()))
            .expect("animation");
        let animation_map = animation.as_mapping().expect("animation as mapping");
        let oneof = animation_map
            .get(&Value::String("oneOf".to_string()))
            .expect("oneOf");
        let animation_oneof = oneof.as_sequence().expect("oneOf as sequence");

        assert_eq!(
            animation_oneof.len(),
            animation_catalog.len(),
            "Generated schema should have oneOf entry for each animation in catalog"
        );
    }

    #[test]
    fn test_metadata_coverage() {
        // Verify that every behavior/animation metadata has required fields
        use engine_core::authoring::catalog::{animation_catalog, behavior_catalog};

        for (name, fields) in behavior_catalog() {
            assert!(
                !fields.is_empty(),
                "Behavior '{}' should have metadata fields",
                name
            );

            // Verify each field has description
            for field in fields {
                assert!(
                    !field.description.is_empty(),
                    "Behavior '{}' field '{}' should have description",
                    name,
                    field.name
                );
            }
        }

        for (name, fields) in animation_catalog() {
            assert!(
                !fields.is_empty(),
                "Animation '{}' should have metadata fields",
                name
            );

            for field in fields {
                assert!(
                    !field.description.is_empty(),
                    "Animation '{}' field '{}' should have description",
                    name,
                    field.name
                );
            }
        }
    }

    #[test]
    fn test_behavior_schema_preserves_required_and_stage_list_shapes() {
        let schema = build_behavior_schema();
        let defs = schema
            .get("$defs")
            .and_then(Value::as_mapping)
            .expect("$defs mapping");
        let behavior = defs
            .get(Value::String("behavior".to_string()))
            .and_then(Value::as_mapping)
            .expect("behavior def");
        let variants = behavior
            .get(Value::String("oneOf".to_string()))
            .and_then(Value::as_sequence)
            .expect("behavior oneOf");

        let follow = variants
            .iter()
            .find(|variant| {
                variant
                    .as_mapping()
                    .and_then(|variant| variant.get(Value::String("properties".to_string())))
                    .and_then(Value::as_mapping)
                    .and_then(|props| props.get(Value::String("name".to_string())))
                    .and_then(Value::as_mapping)
                    .and_then(|name| name.get(Value::String("const".to_string())))
                    .and_then(Value::as_str)
                    == Some("follow")
            })
            .and_then(Value::as_mapping)
            .expect("follow variant");
        let follow_params = follow
            .get(Value::String("properties".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|props| props.get(Value::String("params".to_string())))
            .and_then(Value::as_mapping)
            .expect("follow params");
        let follow_required = follow_params
            .get(Value::String("required".to_string()))
            .and_then(Value::as_sequence)
            .expect("follow required");
        assert!(follow_required
            .iter()
            .any(|value| value.as_str() == Some("target")));

        let stage_visibility = variants
            .iter()
            .find(|variant| {
                variant
                    .as_mapping()
                    .and_then(|variant| variant.get(Value::String("properties".to_string())))
                    .and_then(Value::as_mapping)
                    .and_then(|props| props.get(Value::String("name".to_string())))
                    .and_then(Value::as_mapping)
                    .and_then(|name| name.get(Value::String("const".to_string())))
                    .and_then(Value::as_str)
                    == Some("stage-visibility")
            })
            .and_then(Value::as_mapping)
            .expect("stage-visibility variant");
        let stages_schema = stage_visibility
            .get(Value::String("properties".to_string()))
            .and_then(Value::as_mapping)
            .and_then(|props| props.get(Value::String("params".to_string())))
            .and_then(Value::as_mapping)
            .and_then(|params| params.get(Value::String("properties".to_string())))
            .and_then(Value::as_mapping)
            .and_then(|props| props.get(Value::String("stages".to_string())))
            .and_then(Value::as_mapping)
            .expect("stages schema");

        assert_eq!(
            stages_schema
                .get(Value::String("type".to_string()))
                .and_then(Value::as_str),
            Some("array")
        );
        let items = stages_schema
            .get(Value::String("items".to_string()))
            .and_then(Value::as_mapping)
            .expect("stages items");
        let enum_values = items
            .get(Value::String("enum".to_string()))
            .and_then(Value::as_sequence)
            .expect("stages enum");
        assert!(enum_values
            .iter()
            .any(|value| value.as_str() == Some("on-leave")));
        assert!(enum_values
            .iter()
            .any(|value| value.as_str() == Some("done")));
    }
}

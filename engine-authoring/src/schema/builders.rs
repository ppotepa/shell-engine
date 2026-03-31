//! Core schema builders and file-level overlay schema generators.

use anyhow::Result;
use engine_core::effects::{shared_dispatcher, ParamControl};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;

use super::helpers::{mapping_with, non_empty_string_schema, object_schema, schema_ref};
use super::overlays::{
    object_doc_overlay_patch, object_instance_overlay_patch, object_logic_overlay_def,
    scene_overlay_patch, shared_overlay_defs,
};
use super::GeneratedSchemaFile;

/// Renders one schema document as YAML with a trailing newline.
pub fn render_schema_file(value: &Value) -> Result<String> {
    let mut yaml = serde_yaml::to_string(value)?;
    yaml = quote_problematic_schema_scalars(&yaml);
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }
    Ok(yaml)
}

pub(super) fn quote_problematic_schema_scalars(yaml: &str) -> String {
    yaml.lines()
        .map(|line| sanitize_schema_line(line))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn sanitize_schema_line(line: &str) -> String {
    let Some(colon_idx) = line.find(':') else {
        return line.to_string();
    };
    let (prefix, rest) = line.split_at(colon_idx + 1);
    let Some(value) = rest.strip_prefix(' ') else {
        return line.to_string();
    };

    let key = prefix.trim_end_matches(':').trim();
    if !matches!(key, "description" | "title") {
        return line.to_string();
    }
    if value.is_empty()
        || value.starts_with('"')
        || value.starts_with('\'')
        || value.starts_with('[')
        || value.starts_with('{')
        || value.starts_with('|')
        || value.starts_with('>')
    {
        return line.to_string();
    }
    if !value.contains(": ") {
        return line.to_string();
    }

    format!("{prefix} '{}'", value.replace('\'', "''"))
}

pub(super) fn output_file(file_name: String, value: Value) -> GeneratedSchemaFile {
    GeneratedSchemaFile { file_name, value }
}

/// Builds schema for all built-in behaviors with their parameter schemas.
#[cfg(test)]
pub(super) fn build_behavior_schema() -> Value {
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

/// Builds schema for all built-in animations with their parameter schemas.
#[cfg(test)]
pub(super) fn build_animation_schema() -> Value {
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
pub(super) fn build_input_profile_schema() -> Value {
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
pub(super) fn build_required_if_allof(
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
pub(super) fn build_scene_fields_schema() -> Value {
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
pub(super) fn build_sugar_schema() -> Value {
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

pub(super) fn build_mod_overlay_schema(mod_name: &str) -> Value {
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
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    root.insert(
        Value::String("additionalProperties".to_string()),
        Value::Bool(true),
    );
    root.insert(
        Value::String("required".to_string()),
        Value::Sequence(vec![
            Value::String("name".to_string()),
            Value::String("version".to_string()),
            Value::String("entrypoint".to_string()),
        ]),
    );
    let mut props = Mapping::new();
    props.insert(
        Value::String("name".to_string()),
        schema_ref("../../../schemas/mod.schema.yaml#/properties/name"),
    );
    props.insert(
        Value::String("version".to_string()),
        schema_ref("../../../schemas/mod.schema.yaml#/properties/version"),
    );
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
    props.insert(
        Value::String("output_backend".to_string()),
        schema_ref("../../../schemas/mod.schema.yaml#/properties/output_backend"),
    );
    props.insert(
        Value::String("splash".to_string()),
        schema_ref("../../../schemas/mod.schema.yaml#/properties/splash"),
    );
    props.insert(
        Value::String("terminal".to_string()),
        schema_ref("../../../schemas/mod.schema.yaml#/properties/terminal"),
    );
    root.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    Value::Mapping(root)
}

pub(super) fn build_scene_overlay_schema(mod_name: &str) -> Value {
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

pub(super) fn build_object_doc_overlay_schema(mod_name: &str) -> Value {
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

pub(super) fn build_objects_file_overlay_schema(mod_name: &str) -> Value {
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

pub(super) fn build_layers_file_overlay_schema(mod_name: &str) -> Value {
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
    Value::Mapping(root)
}

pub(super) fn build_templates_file_overlay_schema(mod_name: &str) -> Value {
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
    Value::Mapping(root)
}

pub(super) fn build_sprites_file_overlay_schema(mod_name: &str) -> Value {
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
    Value::Mapping(root)
}

pub(super) fn build_effect_file_overlay_schema(
    mod_name: &str,
    effect_names: &BTreeSet<String>,
) -> Value {
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

pub(super) fn effect_variant_schemas(
    mod_name: &str,
    effect_names: &BTreeSet<String>,
) -> Vec<Value> {
    let mut variants = vec![effect_preset_alias_schema()];
    variants.extend(effect_names.iter().map(|effect_name| {
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
    }));
    variants
}

pub(super) fn effect_preset_alias_schema() -> Value {
    let mut props = Mapping::new();
    props.insert(Value::String("use".to_string()), non_empty_string_schema());
    props.insert(
        Value::String("preset".to_string()),
        non_empty_string_schema(),
    );
    props.insert(Value::String("ref".to_string()), non_empty_string_schema());
    props.insert(Value::String("overrides".to_string()), object_schema());

    let mut alias = Mapping::new();
    alias.insert(
        Value::String("type".to_string()),
        Value::String("object".to_string()),
    );
    alias.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    alias.insert(
        Value::String("additionalProperties".to_string()),
        Value::Bool(true),
    );
    alias.insert(
        Value::String("anyOf".to_string()),
        Value::Sequence(vec![
            Value::Mapping(mapping_with(
                "required",
                Value::Sequence(vec![Value::String("use".to_string())]),
            )),
            Value::Mapping(mapping_with(
                "required",
                Value::Sequence(vec![Value::String("preset".to_string())]),
            )),
            Value::Mapping(mapping_with(
                "required",
                Value::Sequence(vec![Value::String("ref".to_string())]),
            )),
        ]),
    );
    alias.insert(
        Value::String("title".to_string()),
        Value::String("effect preset alias".to_string()),
    );
    Value::Mapping(alias)
}

pub(super) fn effect_params_schema(
    params: &'static [engine_core::effects::ParamMetadata],
) -> Value {
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

pub(super) fn param_control_schema(control: &ParamControl) -> Mapping {
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
            if *step > 0.0 {
                map.insert(
                    Value::String("multipleOf".to_string()),
                    serde_yaml::to_value(*step).expect("step value"),
                );
            }
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

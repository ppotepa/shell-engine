//! Schema source-of-truth for authoring files.
//!
//! This module owns generated authoring schema fragments so the generator CLI,
//! tests, and future editor integrations all consume the same descriptors.

use anyhow::{Context, Result};
use engine_core::authoring::catalog::static_catalog;
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
    let object_names = collect_object_names(mod_root)?;
    let mut effect_names = collect_effect_names(mod_root)?;
    for name in static_catalog().effect_names {
        effect_names.insert((*name).to_string());
    }
    let layer_refs = collect_scene_partial_refs(mod_root, "layers")?;
    let sprite_refs = collect_scene_partial_refs(mod_root, "sprites")?;
    let template_refs = collect_scene_partial_refs(mod_root, "templates")?;
    let object_refs = collect_scene_partial_refs(mod_root, "objects")?;
    let effect_refs = collect_scene_partial_refs(mod_root, "effects")?;

    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/schemas/generated/{mod_name}.schema.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("{mod_name} generated schema fragment")),
    );

    let mut defs = Mapping::new();
    defs.insert(
        Value::String("scene_ids".to_string()),
        enum_schema(scene_ids.into_iter().collect()),
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
    root.insert(Value::String("$defs".to_string()), Value::Mapping(defs));

    Ok(vec![
        output_file(format!("{mod_name}.schema.yaml"), Value::Mapping(root)),
        output_file(
            format!("{mod_name}.scene.schema.yaml"),
            build_scene_overlay_schema(mod_name),
        ),
        output_file(
            format!("{mod_name}.scene-file.schema.yaml"),
            build_scene_file_overlay_schema(mod_name),
        ),
        output_file(
            format!("{mod_name}.objects-file.schema.yaml"),
            build_objects_file_overlay_schema(mod_name),
        ),
        output_file(
            format!("{mod_name}.layers-file.schema.yaml"),
            build_layers_file_overlay_schema(mod_name),
        ),
        output_file(
            format!("{mod_name}.templates-file.schema.yaml"),
            build_templates_file_overlay_schema(mod_name),
        ),
        output_file(
            format!("{mod_name}.sprites-file.schema.yaml"),
            build_sprites_file_overlay_schema(mod_name),
        ),
        output_file(
            format!("{mod_name}.effect-file.schema.yaml"),
            build_effect_file_overlay_schema(mod_name, &effect_names),
        ),
        output_file("behaviors.schema.yaml".to_string(), build_behavior_schema()),
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
fn build_behavior_schema() -> Value {
    use engine_core::authoring::catalog::behavior_catalog;
    
    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String("https://shell-quest.local/schemas/generated/behaviors.schema.yaml".to_string()),
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
        
        // Required name field matching this behavior
        let mut required = vec![Value::String("name".to_string())];
        let mut properties = Mapping::new();
        
        // name property with const
        let mut name_prop = Mapping::new();
        name_prop.insert(Value::String("const".to_string()), Value::String(name.to_string()));
        properties.insert(Value::String("name".to_string()), Value::Mapping(name_prop));
        
        // params property with nested fields
        if !fields.is_empty() {
            let mut params_schema = Mapping::new();
            params_schema.insert(
                Value::String("type".to_string()),
                Value::String("object".to_string()),
            );
            
            let mut param_props = Mapping::new();
            for field in fields {
                param_props.insert(
                    Value::String(field.name.to_string()),
                    field_metadata_to_schema(&field),
                );
            }
            
            params_schema.insert(
                Value::String("properties".to_string()),
                Value::Mapping(param_props),
            );
            
            properties.insert(Value::String("params".to_string()), Value::Mapping(params_schema));
        }
        
        variant.insert(Value::String("properties".to_string()), Value::Mapping(properties));
        variant.insert(Value::String("required".to_string()), Value::Sequence(required));
        
        one_of_variants.push(Value::Mapping(variant));
    }

    let mut defs = Mapping::new();
    let mut behavior_def = Mapping::new();
    behavior_def.insert(Value::String("oneOf".to_string()), Value::Sequence(one_of_variants));
    defs.insert(Value::String("behavior".to_string()), Value::Mapping(behavior_def));
    
    root.insert(Value::String("$defs".to_string()), Value::Mapping(defs));
    Value::Mapping(root)
}

/// Converts FieldMetadata to JSON Schema property definition.
fn field_metadata_to_schema(field: &engine_core::authoring::metadata::FieldMetadata) -> Value {
    use engine_core::authoring::metadata::{Requirement, ValueKind};
    
    let mut prop = Mapping::new();
    
    // Type
    let type_str = match field.value_kind {
        ValueKind::Number => "number",
        ValueKind::Integer => "integer",
        ValueKind::Boolean => "boolean",
        ValueKind::Text | ValueKind::Colour => "string",
        ValueKind::Select => "string",
    };
    prop.insert(Value::String("type".to_string()), Value::String(type_str.to_string()));
    
    // Description
    if !field.description.is_empty() {
        prop.insert(
            Value::String("description".to_string()),
            Value::String(field.description.to_string()),
        );
    }
    
    // Default
    if let Some(default) = field.default_text {
        prop.insert(Value::String("default".to_string()), Value::String(default.to_string()));
    } else if let Some(default) = field.default_number {
        prop.insert(
            Value::String("default".to_string()),
            serde_yaml::to_value(default).unwrap(),
        );
    }
    
    // Enum options
    if let Some(options) = field.enum_options {
        prop.insert(
            Value::String("enum".to_string()),
            Value::Sequence(options.iter().map(|s| Value::String(s.to_string())).collect()),
        );
    }
    
    // Number constraints
    if let Some(min) = field.min {
        prop.insert(Value::String("minimum".to_string()), serde_yaml::to_value(min).unwrap());
    }
    if let Some(max) = field.max {
        prop.insert(Value::String("maximum".to_string()), serde_yaml::to_value(max).unwrap());
    }
    if let Some(step) = field.step {
        prop.insert(Value::String("multipleOf".to_string()), serde_yaml::to_value(step).unwrap());
    }
    
    Value::Mapping(prop)
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
            "https://shell-quest.local/schemas/generated/{mod_name}.scene.schema.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("{mod_name} scene overlay schema")),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../scene.schema.yaml"),
            Value::Mapping(scene_overlay_patch(mod_name)),
        ]),
    );
    Value::Mapping(root)
}

fn build_scene_file_overlay_schema(mod_name: &str) -> Value {
    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/schemas/generated/{mod_name}.scene-file.schema.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("{mod_name} scene-file overlay schema")),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![
            schema_ref("../scene-file.schema.yaml"),
            Value::Mapping(scene_overlay_patch(mod_name)),
        ]),
    );
    Value::Mapping(root)
}

fn build_objects_file_overlay_schema(mod_name: &str) -> Value {
    let mut items_patch = Mapping::new();
    let mut use_props = Mapping::new();
    use_props.insert(
        Value::String("use".to_string()),
        schema_ref(&format!("./{mod_name}.schema.yaml#/$defs/object_names")),
    );
    items_patch.insert(
        Value::String("properties".to_string()),
        Value::Mapping(use_props),
    );

    let mut root = Mapping::new();
    root.insert(
        Value::String("$schema".to_string()),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
    );
    root.insert(
        Value::String("$id".to_string()),
        Value::String(format!(
            "https://shell-quest.local/schemas/generated/{mod_name}.objects-file.schema.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("{mod_name} objects-file overlay schema")),
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
                schema_ref("../objects-file.schema.yaml#/items"),
                Value::Mapping(items_patch),
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
            "https://shell-quest.local/schemas/generated/{mod_name}.layers-file.schema.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("{mod_name} layers-file overlay schema")),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref("../layers-file.schema.yaml")]),
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
            "https://shell-quest.local/schemas/generated/{mod_name}.templates-file.schema.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("{mod_name} templates-file overlay schema")),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref("../templates-file.schema.yaml")]),
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
            "https://shell-quest.local/schemas/generated/{mod_name}.sprites-file.schema.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("{mod_name} sprites-file overlay schema")),
    );
    root.insert(
        Value::String("allOf".to_string()),
        Value::Sequence(vec![schema_ref("../sprites-file.schema.yaml")]),
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
            "https://shell-quest.local/schemas/generated/{mod_name}.effect-file.schema.yaml"
        )),
    );
    root.insert(
        Value::String("title".to_string()),
        Value::String(format!("{mod_name} effect-file overlay schema")),
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
                    schema_ref("../effect-file.schema.yaml#/items"),
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

fn scene_overlay_patch(mod_name: &str) -> Mapping {
    let mut props = Mapping::new();
    props.insert(
        Value::String("next".to_string()),
        nullable_ref(&format!("./{mod_name}.schema.yaml#/$defs/scene_ids")),
    );
    props.insert(
        Value::String("menu-options".to_string()),
        menu_options_overlay(mod_name),
    );
    props.insert(
        Value::String("menu_options".to_string()),
        menu_options_overlay(mod_name),
    );
    props.insert(
        Value::String("objects".to_string()),
        objects_overlay(mod_name),
    );

    let mut root = Mapping::new();
    root.insert(
        Value::String("properties".to_string()),
        Value::Mapping(props),
    );
    root
}

fn menu_options_overlay(mod_name: &str) -> Value {
    Value::Mapping(mapping_with(
        "items",
        Value::Mapping(mapping_with(
            "properties",
            Value::Mapping(mapping_with(
                "next",
                schema_ref(&format!("./{mod_name}.schema.yaml#/$defs/scene_ids")),
            )),
        )),
    ))
}

fn objects_overlay(mod_name: &str) -> Value {
    Value::Mapping(mapping_with(
        "items",
        Value::Mapping(mapping_with(
            "properties",
            Value::Mapping(mapping_with(
                "use",
                schema_ref(&format!("./{mod_name}.schema.yaml#/$defs/object_names")),
            )),
        )),
    ))
}

fn schema_ref(target: &str) -> Value {
    Value::Mapping(mapping_with("$ref", Value::String(target.to_string())))
}

fn nullable_ref(target: &str) -> Value {
    Value::Mapping(mapping_with(
        "oneOf",
        Value::Sequence(vec![
            schema_ref(target),
            Value::Mapping(mapping_with("type", Value::String("null".to_string()))),
        ]),
    ))
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
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to read {}", current.display()))?
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
                out.insert(rel.to_string_lossy().replace('\\', "/"));
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
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to read {}", current.display()))?
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
                out.insert(format!("{}/{}", prefix, rel.to_string_lossy().replace('\\', "/")));
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
            if let Some(id) = map.get(Value::String("id".to_string())).and_then(Value::as_str) {
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
    
    for scene_dir_entry in fs::read_dir(&scenes_root)
        .with_context(|| format!("failed to read {}", scenes_root.display()))?
    {
        let scene_dir = scene_dir_entry?;
        let scene_path = scene_dir.path();
        if !scene_path.is_dir() {
            continue;
        }
        
        let templates_root = scene_path.join("templates");
        if !templates_root.exists() {
            continue;
        }
        
        for file in yaml_files_under(&templates_root)? {
            if let Ok(raw) = fs::read_to_string(&file) {
                if let Ok(v) = serde_yaml::from_str::<Value>(&raw) {
                    if let Some(name) = v.get("name").and_then(Value::as_str) {
                        names.insert(name.to_string());
                    }
                }
            }
        }
    }
    Ok(names)
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
    use super::generate_mod_schema_files;
    use serde_yaml::Value;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn generate_fragment_contains_dynamic_defs() {
        let temp_root = unique_temp_dir("authoring-schema-test");
        let mod_root = temp_root.join("playground");
        fs::create_dir_all(mod_root.join("scenes/intro/layers")).expect("create layers");
        fs::create_dir_all(mod_root.join("scenes/intro/sprites")).expect("create sprites");
        fs::create_dir_all(mod_root.join("objects")).expect("create objects");
        fs::write(mod_root.join("mod.yaml"), "name: playground\n").expect("write mod");
        fs::write(
            mod_root.join("scenes/intro/scene.yml"),
            "id: intro\neffects:\n  - name: fade-in\n    duration: 1.0\n",
        )
        .expect("write scene");
        fs::write(
            mod_root.join("scenes/intro/layers/bg.yml"),
            "name: background\n",
        )
        .expect("write layer partial");
        fs::write(mod_root.join("objects/npc.yml"), "name: npc\n").expect("write object");

        let files = generate_mod_schema_files(&mod_root).expect("generate schemas");
        let root = files
            .iter()
            .find(|file| file.file_name == "playground.schema.yaml")
            .expect("root schema");
        let yaml = root.value.as_mapping().expect("schema mapping");
        let defs = yaml
            .get(Value::String("$defs".to_string()))
            .and_then(Value::as_mapping)
            .expect("defs mapping");

        assert!(defs.contains_key(Value::String("scene_ids".to_string())));
        assert!(defs.contains_key(Value::String("object_names".to_string())));
        assert!(defs.contains_key(Value::String("effect_names".to_string())));
        assert!(defs.contains_key(Value::String("layer_refs".to_string())));
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
        fs::write(temp.join("assets/fonts/mono/manifest.yaml"), "name: mono-bold\n").unwrap();
        
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
        assert!(paths.contains("logo.png"));
        assert!(paths.contains("ui/button.png"));
    }

    #[test]
    fn test_collect_model_paths() {
        let temp = unique_temp_dir("model-test");
        fs::create_dir_all(temp.join("scenes/intro")).unwrap();
        fs::create_dir_all(temp.join("assets/models")).unwrap();
        fs::write(temp.join("scenes/intro/cube.obj"), "").unwrap();
        fs::write(temp.join("assets/models/sphere.obj"), "").unwrap();
        
        let paths = super::collect_model_paths(&temp).unwrap();
        assert!(paths.contains("scenes/intro/cube.obj"));
        assert!(paths.contains("assets/models/sphere.obj"));
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
            "name: menu-button\n",
        )
        .unwrap();
        
        let names = super::collect_template_names(&temp).unwrap();
        assert!(names.contains("menu-button"));
    }

    #[test]
    fn test_behavior_schema_generation() {
        let temp = unique_temp_dir("behavior-schema-test");
        fs::write(temp.join("mod.yaml"), "name: test\n").unwrap();
        fs::create_dir_all(temp.join("scenes")).unwrap();
        
        let files = generate_mod_schema_files(&temp).unwrap();
        let behavior_schema = files
            .iter()
            .find(|f| f.file_name == "behaviors.schema.yaml")
            .expect("behaviors.schema.yaml generated");
        
        let defs = behavior_schema
            .value
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
}

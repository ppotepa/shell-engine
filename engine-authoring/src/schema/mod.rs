//! Schema source-of-truth for authoring files.
//!
//! This module owns generated authoring schema fragments so the generator CLI,
//! tests, and future editor integrations all consume the same descriptors.

use anyhow::Result;
use engine_core::authoring::catalog::static_catalog;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;
use std::path::Path;

mod builders;
mod collectors;
mod helpers;
mod overlays;
#[cfg(test)]
mod tests;

use builders::*;
use collectors::*;
use helpers::*;

pub use builders::render_schema_file;


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
    let effect_refs = collect_effect_refs(mod_root)?;

    // Asset catalogs for autocomplete
    let font_names = collect_font_names(mod_root)?;
    let font_specs = collect_font_specs(&font_names);
    let image_paths = collect_image_paths(mod_root)?;
    let model_paths = collect_model_paths(mod_root)?;
    let scene3d_paths = collect_scene3d_paths(mod_root)?;
    let cutscene_refs = collect_cutscene_refs(mod_root)?;
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
        Value::String("scene3d_paths".to_string()),
        enum_schema(scene3d_paths.into_iter().collect()),
    );
    defs.insert(
        Value::String("cutscene_refs".to_string()),
        enum_schema(cutscene_refs.into_iter().collect()),
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

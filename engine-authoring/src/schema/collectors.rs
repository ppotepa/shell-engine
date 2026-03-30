//! Data collection functions for schema generation (collect_*, walk_*, yaml_files_under).

use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::fs;

use crate::repository::is_discoverable_scene_path;

pub(super) fn collect_scene_ids(mod_root: &Path) -> Result<BTreeSet<String>> {
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

pub(super) fn collect_scene_paths(mod_root: &Path) -> Result<BTreeSet<String>> {
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

pub(super) fn collect_object_names(mod_root: &Path) -> Result<BTreeSet<String>> {
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

pub(super) fn collect_object_ref_values(
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

pub(super) fn collect_effect_names(mod_root: &Path) -> Result<BTreeSet<String>> {
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

pub(super) fn collect_effect_refs(mod_root: &Path) -> Result<BTreeSet<String>> {
    let mut refs = collect_scene_partial_refs(mod_root, "effects")?;
    for file in yaml_files_under(&mod_root.join("effects"))? {
        if let Ok(rel) = file.strip_prefix(mod_root) {
            refs.insert(format!("/{}", rel.to_string_lossy().replace('\\', "/")));
        }
    }
    Ok(refs)
}

pub(super) fn collect_effect_names_from_value(value: &Value, out: &mut BTreeSet<String>) {
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

pub(super) fn collect_scene_partial_refs(mod_root: &Path, part_dir: &str) -> Result<BTreeSet<String>> {
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
pub(super) fn collect_font_names(mod_root: &Path) -> Result<BTreeSet<String>> {
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
                if let Some(name) = v
                    .get("font_label")
                    .or_else(|| v.get("font_family"))
                    .or_else(|| v.get("name"))
                    .and_then(Value::as_str)
                {
                    names.insert(name.to_string());
                }
            }
        }
    }
    Ok(names)
}

pub(super) fn collect_font_specs(font_names: &BTreeSet<String>) -> BTreeSet<String> {
    let mut specs = BTreeSet::from([
        "default".to_string(),
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
pub(super) fn collect_image_paths(mod_root: &Path) -> Result<BTreeSet<String>> {
    let images_root = mod_root.join("assets/images");
    let mut paths = BTreeSet::new();
    if !images_root.exists() {
        return Ok(paths);
    }
    walk_images(&images_root, &images_root, &mut paths)?;
    Ok(paths)
}

pub(super) fn walk_images(root: &Path, current: &Path, out: &mut BTreeSet<String>) -> Result<()> {
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
pub(super) fn collect_model_paths(mod_root: &Path) -> Result<BTreeSet<String>> {
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

pub(super) fn walk_models(
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

/// Collects `.scene3d.yml` paths from assets/3d/**/*.scene3d.yml files.
pub(super) fn collect_scene3d_paths(mod_root: &Path) -> Result<BTreeSet<String>> {
    let root = mod_root.join("assets/3d");
    let mut paths = BTreeSet::new();
    if !root.exists() {
        return Ok(paths);
    }
    walk_scene3d(&root, &root, &mut paths)?;
    Ok(paths)
}

pub(super) fn walk_scene3d(root: &Path, current: &Path, out: &mut BTreeSet<String>) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read {}", current.display()))?
    {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            walk_scene3d(root, &p, out)?;
            continue;
        }
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        if name.ends_with(".scene3d.yml") {
            if let Ok(rel) = p.strip_prefix(root) {
                out.insert(format!(
                    "/assets/3d/{}",
                    rel.to_string_lossy().replace('\\', "/")
                ));
            }
        }
    }
    Ok(())
}

/// Collects cutscene references from cutscenes/**/*.yml files.
pub(super) fn collect_cutscene_refs(mod_root: &Path) -> Result<BTreeSet<String>> {
    let cutscenes_root = mod_root.join("cutscenes");
    let mut refs = BTreeSet::new();
    if !cutscenes_root.exists() {
        return Ok(refs);
    }

    for file in yaml_files_under(&cutscenes_root)? {
        let rel_from_mod = match file.strip_prefix(mod_root) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        let rel_from_cutscenes = match file.strip_prefix(&cutscenes_root) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        refs.insert(format!("/{rel_from_mod}"));
        let no_ext = rel_from_cutscenes
            .trim_end_matches(".yml")
            .trim_end_matches(".yaml");
        if !no_ext.is_empty() {
            refs.insert(no_ext.to_string());
        }
    }

    Ok(refs)
}

/// Collects sprite IDs from all scene YAML files.
pub(super) fn collect_sprite_ids(mod_root: &Path) -> Result<BTreeSet<String>> {
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

pub(super) fn collect_sprite_ids_from_value(value: &Value, out: &mut BTreeSet<String>) {
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
pub(super) fn collect_template_names(mod_root: &Path) -> Result<BTreeSet<String>> {
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

pub(super) fn collect_template_names_from_mapping(map: &Mapping, out: &mut BTreeSet<String>) {
    for key in map.keys() {
        if let Some(name) = key.as_str() {
            out.insert(name.to_string());
        }
    }
}

pub(super) fn yaml_files_under(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    walk_yaml(root, &mut out)?;
    out.sort();
    Ok(out)
}

pub(super) fn walk_yaml(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
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

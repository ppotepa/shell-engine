use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::SchemaTargetArgs;
use crate::fs_utils::resolve_mod_roots;

#[derive(Debug, Default, Clone)]
struct ArchitectureSnapshot {
    mods_scanned: usize,
    scene_files_scanned: usize,
    root_scene_files: usize,
    scene_contract_files: usize,
    scene_contract_space_files: usize,
    scene_contract_spatial_files: usize,
    scene_contract_lighting_files: usize,
    scene_contract_view_files: usize,
    scene_contract_celestial_files: usize,
    world_model_counts: BTreeMap<String, usize>,
    controller_defaults_files: usize,
    controller_default_field_counts: BTreeMap<String, usize>,
    legacy_camera_field_counts: BTreeMap<String, usize>,
    object_ref_files: usize,
    object_ref_sequences: usize,
    object_ref_instances: usize,
    object_repeat_groups: usize,
    object_repeat_expanded_instances: usize,
}

#[derive(Debug, Default)]
struct FileSnapshot {
    has_scene_contract_fields: bool,
    has_space: bool,
    has_spatial: bool,
    has_lighting: bool,
    has_view: bool,
    has_celestial: bool,
    world_model: Option<String>,
    controller_defaults_present: bool,
    controller_default_field_counts: BTreeMap<String, usize>,
    legacy_camera_field_counts: BTreeMap<String, usize>,
    has_objects: bool,
    object_sequences: usize,
    object_instances: usize,
    object_repeat_groups: usize,
    object_repeat_expanded_instances: usize,
}

pub fn write_snapshot(repo_root: &Path, args: &SchemaTargetArgs) -> Result<()> {
    let mod_roots = resolve_mod_roots(repo_root, args)?;
    let mut total = ArchitectureSnapshot::default();

    for mod_root in mod_roots {
        let snapshot = collect_mod_snapshot(&mod_root)?;
        total.merge(snapshot);
    }

    print_snapshot(&total);
    Ok(())
}

fn collect_mod_snapshot(mod_root: &Path) -> Result<ArchitectureSnapshot> {
    let scenes_dir = mod_root.join("scenes");
    let mut snapshot = ArchitectureSnapshot::default();
    snapshot.mods_scanned = 1;

    if !scenes_dir.is_dir() {
        return Ok(snapshot);
    }

    let scene_files = collect_yaml_files(&scenes_dir)?;
    for scene_file in scene_files {
        let content = fs::read_to_string(&scene_file)
            .with_context(|| format!("failed to read {}", scene_file.display()))?;
        let value: Value = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse {}", scene_file.display()))?;
        let is_root_scene = is_root_scene_file(&scene_file, &scenes_dir);
        let file_snapshot = collect_file_snapshot(&value, is_root_scene);
        snapshot.merge_file(file_snapshot, is_root_scene);
    }

    Ok(snapshot)
}

fn collect_yaml_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_yaml_files_inner(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_yaml_files_inner(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))?;
    for entry in entries {
        let entry = entry.with_context(|| format!("failed to read {}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_yaml_files_inner(&path, out)?;
            continue;
        }
        if is_yaml_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml"))
}

fn is_root_scene_file(path: &Path, scenes_dir: &Path) -> bool {
    if path.file_name().and_then(|name| name.to_str()) == Some("scene.yml")
        || path.file_name().and_then(|name| name.to_str()) == Some("scene.yaml")
    {
        return true;
    }

    path.parent().is_some_and(|parent| parent == scenes_dir)
}

fn collect_file_snapshot(value: &Value, is_root_scene: bool) -> FileSnapshot {
    let mut snapshot = FileSnapshot::default();
    if let Some(root) = value.as_mapping() {
        if is_root_scene {
            snapshot.world_model = read_root_string(root, &["world-model", "world_model"]);
            snapshot.controller_defaults_present = root
                .get(Value::String("controller-defaults".to_string()))
                .or_else(|| root.get(Value::String("controller_defaults".to_string())))
                .is_some();
            if let Some(input) = root
                .get(Value::String("input".to_string()))
                .and_then(Value::as_mapping)
            {
                snapshot
                    .legacy_camera_field_counts
                    .extend(count_legacy_camera_fields(input));
            }
            if let Some(ctrl) = root
                .get(Value::String("controller-defaults".to_string()))
                .or_else(|| root.get(Value::String("controller_defaults".to_string())))
                .and_then(Value::as_mapping)
            {
                snapshot
                    .controller_default_field_counts
                    .extend(count_controller_default_fields(ctrl));
            }
        }
        visit_mapping(root, &mut snapshot);
    }
    snapshot
}

fn read_root_string(root: &Mapping, keys: &[&str]) -> Option<String> {
    for key in keys {
        let value = root
            .get(Value::String((*key).to_string()))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        if value.is_some() {
            return value;
        }
    }
    None
}

fn count_controller_default_fields(ctrl: &Mapping) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for (field, aliases) in [
        ("camera-preset", ["camera-preset", "camera_preset"]),
        ("player-preset", ["player-preset", "player_preset"]),
        ("ui-preset", ["ui-preset", "ui_preset"]),
        ("spawn-preset", ["spawn-preset", "spawn_preset"]),
        ("gravity-preset", ["gravity-preset", "gravity_preset"]),
        ("surface-preset", ["surface-preset", "surface_preset"]),
    ] {
        if aliases
            .iter()
            .any(|alias| ctrl.get(Value::String((*alias).to_string())).is_some())
        {
            *counts.entry(field.to_string()).or_default() += 1;
        }
    }
    counts
}

fn count_legacy_camera_fields(input: &Mapping) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for field in ["obj-viewer", "orbit-camera", "free-look-camera"] {
        if input.get(Value::String(field.to_string())).is_some() {
            *counts.entry(format!("input.{field}")).or_default() += 1;
        }
    }
    counts
}

fn visit_value(value: &Value, snapshot: &mut FileSnapshot) {
    match value {
        Value::Mapping(map) => visit_mapping(map, snapshot),
        Value::Sequence(seq) => {
            for item in seq {
                visit_value(item, snapshot);
            }
        }
        _ => {}
    }
}

fn visit_mapping(map: &Mapping, snapshot: &mut FileSnapshot) {
    for (key, value) in map {
        let Some(key) = key.as_str() else {
            continue;
        };

        match key {
            "space" | "render-space" | "render_space" => {
                snapshot.has_scene_contract_fields = true;
                snapshot.has_space = true;
            }
            "spatial" => {
                snapshot.has_scene_contract_fields = true;
                snapshot.has_spatial = true;
            }
            "lighting" => {
                snapshot.has_scene_contract_fields = true;
                snapshot.has_lighting = true;
            }
            "view" => {
                snapshot.has_scene_contract_fields = true;
                snapshot.has_view = true;
            }
            "celestial" => {
                snapshot.has_scene_contract_fields = true;
                snapshot.has_celestial = true;
            }
            "objects" => {
                if let Some(seq) = value.as_sequence() {
                    snapshot.has_objects = true;
                    snapshot.object_sequences += 1;
                    count_object_instances(seq, snapshot);
                }
            }
            _ => {}
        }

        visit_value(value, snapshot);
    }
}

fn count_object_instances(seq: &[Value], snapshot: &mut FileSnapshot) {
    for entry in seq {
        let Some(map) = entry.as_mapping() else {
            continue;
        };

        if let Some(repeat) = map
            .get(Value::String("repeat".to_string()))
            .and_then(Value::as_mapping)
        {
            let Some(count) = repeat
                .get(Value::String("count".to_string()))
                .and_then(Value::as_i64)
            else {
                continue;
            };
            if count <= 0 {
                continue;
            }
            if repeat
                .get(Value::String("ref".to_string()))
                .or_else(|| repeat.get(Value::String("use".to_string())))
                .and_then(Value::as_str)
                .is_none()
            {
                continue;
            }

            snapshot.object_repeat_groups += 1;
            snapshot.object_repeat_expanded_instances += count as usize;
            continue;
        }

        if map.get(Value::String("ref".to_string())).is_some()
            || map.get(Value::String("use".to_string())).is_some()
        {
            snapshot.object_instances += 1;
        }
    }
}

impl ArchitectureSnapshot {
    fn merge_file(&mut self, file: FileSnapshot, is_root_scene: bool) {
        self.scene_files_scanned += 1;
        if is_root_scene {
            self.root_scene_files += 1;
        }
        if file.has_scene_contract_fields {
            self.scene_contract_files += 1;
        }
        if file.has_space {
            self.scene_contract_space_files += 1;
        }
        if file.has_spatial {
            self.scene_contract_spatial_files += 1;
        }
        if file.has_lighting {
            self.scene_contract_lighting_files += 1;
        }
        if file.has_view {
            self.scene_contract_view_files += 1;
        }
        if file.has_celestial {
            self.scene_contract_celestial_files += 1;
        }
        if let Some(world_model) = file.world_model {
            *self.world_model_counts.entry(world_model).or_default() += 1;
        }
        if file.controller_defaults_present {
            self.controller_defaults_files += 1;
        }
        for (field, count) in file.controller_default_field_counts {
            *self
                .controller_default_field_counts
                .entry(field)
                .or_default() += count;
        }
        for (field, count) in file.legacy_camera_field_counts {
            *self.legacy_camera_field_counts.entry(field).or_default() += count;
        }
        if file.has_objects {
            self.object_ref_files += 1;
        }
        self.object_ref_sequences += file.object_sequences;
        self.object_ref_instances += file.object_instances;
        self.object_repeat_groups += file.object_repeat_groups;
        self.object_repeat_expanded_instances += file.object_repeat_expanded_instances;
    }

    fn merge(&mut self, other: ArchitectureSnapshot) {
        self.mods_scanned += other.mods_scanned;
        self.scene_files_scanned += other.scene_files_scanned;
        self.root_scene_files += other.root_scene_files;
        self.scene_contract_files += other.scene_contract_files;
        self.scene_contract_space_files += other.scene_contract_space_files;
        self.scene_contract_spatial_files += other.scene_contract_spatial_files;
        self.scene_contract_lighting_files += other.scene_contract_lighting_files;
        self.scene_contract_view_files += other.scene_contract_view_files;
        self.scene_contract_celestial_files += other.scene_contract_celestial_files;
        for (world_model, count) in other.world_model_counts {
            *self.world_model_counts.entry(world_model).or_default() += count;
        }
        self.controller_defaults_files += other.controller_defaults_files;
        for (field, count) in other.controller_default_field_counts {
            *self
                .controller_default_field_counts
                .entry(field)
                .or_default() += count;
        }
        for (field, count) in other.legacy_camera_field_counts {
            *self.legacy_camera_field_counts.entry(field).or_default() += count;
        }
        self.object_ref_files += other.object_ref_files;
        self.object_ref_sequences += other.object_ref_sequences;
        self.object_ref_instances += other.object_ref_instances;
        self.object_repeat_groups += other.object_repeat_groups;
        self.object_repeat_expanded_instances += other.object_repeat_expanded_instances;
    }
}

fn print_snapshot(snapshot: &ArchitectureSnapshot) {
    println!("Architecture snapshot");
    println!(
        "mods scanned: {}, scene files: {}, root scenes: {}",
        snapshot.mods_scanned, snapshot.scene_files_scanned, snapshot.root_scene_files
    );
    println!(
        "scene contract files: {} (space: {}, spatial: {}, lighting: {}, view: {}, celestial: {})",
        snapshot.scene_contract_files,
        snapshot.scene_contract_space_files,
        snapshot.scene_contract_spatial_files,
        snapshot.scene_contract_lighting_files,
        snapshot.scene_contract_view_files,
        snapshot.scene_contract_celestial_files
    );
    println!("world-model counts:");
    for (world_model, count) in &snapshot.world_model_counts {
        println!("  {world_model}: {count}");
    }
    println!(
        "controller-defaults files: {}",
        snapshot.controller_defaults_files
    );
    for (field, count) in &snapshot.controller_default_field_counts {
        println!("  {field}: {count}");
    }
    println!("legacy camera input files:");
    for (field, count) in &snapshot.legacy_camera_field_counts {
        println!("  {field}: {count}");
    }
    println!(
        "object refs: files with objects={}, object sequences={}, direct instances={}, repeat groups={}, expanded repeat instances={}",
        snapshot.object_ref_files,
        snapshot.object_ref_sequences,
        snapshot.object_ref_instances,
        snapshot.object_repeat_groups,
        snapshot.object_repeat_expanded_instances
    );
}

#[cfg(test)]
mod tests {
    use super::collect_mod_snapshot;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn collects_scene_contract_world_model_and_object_ref_counts() {
        let dir = tempdir().expect("tempdir");
        let mod_root = dir.path();
        fs::create_dir_all(mod_root.join("scenes/intro/layers")).expect("scenes dir");
        fs::write(
            mod_root.join("scenes/intro/scene.yml"),
            r#"
id: intro
title: Intro
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: orbit-camera
input:
  free-look-camera: {}
layers:
  - ref: main
objects:
  - ref: /objects/probe.yml
  - repeat:
      count: 3
      ref: /objects/star.yml
"#,
        )
        .expect("scene");
        fs::write(
            mod_root.join("scenes/intro/layers/main.yml"),
            r#"
space: screen
objects:
  - ref: /objects/banner.yml
"#,
        )
        .expect("layer");

        let snapshot = collect_mod_snapshot(mod_root).expect("snapshot");
        assert_eq!(snapshot.root_scene_files, 1);
        assert_eq!(snapshot.scene_contract_files, 2);
        assert_eq!(snapshot.scene_contract_space_files, 2);
        assert_eq!(snapshot.world_model_counts.get("celestial-3d"), Some(&1));
        assert_eq!(snapshot.controller_defaults_files, 1);
        assert_eq!(
            snapshot
                .legacy_camera_field_counts
                .get("input.free-look-camera"),
            Some(&1)
        );
        assert_eq!(snapshot.object_ref_files, 2);
        assert_eq!(snapshot.object_ref_sequences, 2);
        assert_eq!(snapshot.object_ref_instances, 2);
        assert_eq!(snapshot.object_repeat_groups, 1);
        assert_eq!(snapshot.object_repeat_expanded_instances, 3);
    }
}

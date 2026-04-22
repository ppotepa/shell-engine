//! Raw scene migration snapshot used by startup validation.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::{Mapping, Value};

#[derive(Debug, Default, Clone)]
pub struct ArchitectureSnapshot {
    pub scene_files_scanned: usize,
    pub root_scene_files: usize,
    pub scene_contract_files: usize,
    pub scene_contract_space_files: usize,
    pub scene_contract_spatial_files: usize,
    pub scene_contract_lighting_files: usize,
    pub scene_contract_view_files: usize,
    pub scene_contract_celestial_files: usize,
    pub world_model_counts: BTreeMap<String, usize>,
    pub controller_defaults_files: usize,
    pub controller_default_field_counts: BTreeMap<String, usize>,
    pub legacy_camera_field_counts: BTreeMap<String, usize>,
    pub object_ref_files: usize,
    pub object_ref_sequences: usize,
    pub object_ref_instances: usize,
    pub object_repeat_groups: usize,
    pub object_repeat_expanded_instances: usize,
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

    pub fn format_summary(&self) -> String {
        format!(
            "scene files={}, root scenes={}, scene contract files={} (space={}, spatial={}, lighting={}, view={}, celestial={}), world-models={}, controller-defaults={} files / fields={}, legacy camera blocks={}, objects={} files / {} sequences / {} instances / {} repeat groups / {} repeat instances",
            self.scene_files_scanned,
            self.root_scene_files,
            self.scene_contract_files,
            self.scene_contract_space_files,
            self.scene_contract_spatial_files,
            self.scene_contract_lighting_files,
            self.scene_contract_view_files,
            self.scene_contract_celestial_files,
            format_world_models(&self.world_model_counts),
            self.controller_defaults_files,
            format_field_counts(&self.controller_default_field_counts),
            format_field_counts(&self.legacy_camera_field_counts),
            self.object_ref_files,
            self.object_ref_sequences,
            self.object_ref_instances,
            self.object_repeat_groups,
            self.object_repeat_expanded_instances,
        )
    }
}

pub fn collect_mod_snapshot(mod_source: &Path) -> Result<Option<ArchitectureSnapshot>, String> {
    if !mod_source.is_dir() {
        return Ok(None);
    }

    let scenes_dir = mod_source.join("scenes");
    let mut snapshot = ArchitectureSnapshot::default();
    if !scenes_dir.is_dir() {
        return Ok(Some(snapshot));
    }

    let scene_files = collect_yaml_files(&scenes_dir)?;
    for scene_file in scene_files {
        let content = fs::read_to_string(&scene_file)
            .map_err(|err| format!("failed to read {}: {err}", scene_file.display()))?;
        let value: Value = serde_yaml::from_str(&content)
            .map_err(|err| format!("failed to parse {}: {err}", scene_file.display()))?;
        let is_root_scene = is_root_scene_file(&scene_file, &scenes_dir);
        let file_snapshot = collect_file_snapshot(&value, is_root_scene);
        snapshot.merge_file(file_snapshot, is_root_scene);
    }

    Ok(Some(snapshot))
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
            if let Some(ctrl) = root
                .get(Value::String("controller-defaults".to_string()))
                .or_else(|| root.get(Value::String("controller_defaults".to_string())))
                .and_then(Value::as_mapping)
            {
                snapshot
                    .controller_default_field_counts
                    .extend(count_controller_default_fields(ctrl));
            }
            if let Some(input) = root
                .get(Value::String("input".to_string()))
                .and_then(Value::as_mapping)
            {
                snapshot
                    .legacy_camera_field_counts
                    .extend(count_legacy_camera_fields(input));
            }
        }
        visit_mapping(root, &mut snapshot);
    }
    snapshot
}

fn collect_yaml_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    collect_yaml_files_inner(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_yaml_files_inner(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(root).map_err(|err| format!("failed to read {}: {err}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read {}: {err}", root.display()))?;
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

fn format_world_models(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }
    counts
        .iter()
        .map(|(name, count)| format!("{name}:{count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_field_counts(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }
    counts
        .iter()
        .map(|(name, count)| format!("{name}:{count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::collect_mod_snapshot;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn snapshot_ignores_runtime_objects_when_counting_legacy_object_refs() {
        let mod_dir = tempdir().expect("temp dir");
        let scenes_dir = mod_dir.path().join("scenes");
        let runtime_object_dir = scenes_dir.join("runtime-object");
        let legacy_objects_dir = scenes_dir.join("legacy-objects");
        fs::create_dir_all(&runtime_object_dir).expect("runtime-object dir");
        fs::create_dir_all(&legacy_objects_dir).expect("legacy-objects dir");

        fs::write(
            runtime_object_dir.join("scene.yml"),
            r#"
id: runtime-object-scene
title: Runtime Object Scene
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: orbit-camera
runtime-objects:
  - name: runtime-root
    kind: runtime-object
    transform:
      space: 3d
      translation: [1.0, 2.0, 3.0]
layers: []
"#,
        )
        .expect("write runtime-object scene");

        fs::write(
            legacy_objects_dir.join("scene.yml"),
            r#"
id: legacy-objects-scene
title: Legacy Objects Scene
objects:
  - ref: /objects/ship.yml
  - repeat:
      ref: /objects/marker.yml
      count: 3
layers: []
"#,
        )
        .expect("write legacy-objects scene");

        let snapshot = collect_mod_snapshot(mod_dir.path())
            .expect("snapshot collection should succeed")
            .expect("directory snapshot");

        assert_eq!(snapshot.scene_files_scanned, 2);
        assert_eq!(snapshot.root_scene_files, 2);
        assert_eq!(snapshot.controller_defaults_files, 1);
        assert_eq!(snapshot.object_ref_files, 1);
        assert_eq!(snapshot.object_ref_sequences, 1);
        assert_eq!(snapshot.object_ref_instances, 1);
        assert_eq!(snapshot.object_repeat_groups, 1);
        assert_eq!(snapshot.object_repeat_expanded_instances, 3);
        assert_eq!(
            snapshot.world_model_counts.get("euclidean-3d"),
            Some(&1usize),
            "runtime-object scenes should still count toward authored world-model migration"
        );
    }

    #[test]
    fn snapshot_does_not_treat_camera_rig_only_scenes_as_legacy_camera_authoring() {
        let mod_dir = tempdir().expect("temp dir");
        let scene_dir = mod_dir.path().join("scenes").join("camera-rig-only");
        fs::create_dir_all(&scene_dir).expect("scene dir");

        fs::write(
            scene_dir.join("scene.yml"),
            r#"
id: camera-rig-only
title: Camera Rig Only
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: surface-free-look
camera-rig:
  preset: surface-free-look
  surface:
    mode: locked
  free-look-camera: {}
runtime-objects:
  - name: pilot
    kind: runtime-object
    transform:
      space: celestial
      body: earth
      altitude-m: 1200.0
layers: []
"#,
        )
        .expect("write camera-rig scene");

        let snapshot = collect_mod_snapshot(mod_dir.path())
            .expect("snapshot collection should succeed")
            .expect("directory snapshot");

        assert_eq!(snapshot.scene_files_scanned, 1);
        assert_eq!(snapshot.controller_defaults_files, 1);
        assert_eq!(snapshot.legacy_camera_field_counts.len(), 0);
        assert_eq!(snapshot.object_ref_files, 0);
        assert_eq!(snapshot.object_ref_sequences, 0);
        assert_eq!(snapshot.object_ref_instances, 0);
        assert!(
            snapshot
                .format_summary()
                .contains("legacy camera blocks=none"),
            "camera-rig-only scenes should not look like raw legacy input debt in architecture summaries"
        );
    }

    #[test]
    fn snapshot_keeps_prefab_first_runtime_object_trees_out_of_legacy_object_counts() {
        let mod_dir = tempdir().expect("temp dir");
        let scene_dir = mod_dir.path().join("scenes").join("runtime-prefab-tree");
        fs::create_dir_all(&scene_dir).expect("scene dir");

        fs::write(
            scene_dir.join("scene.yml"),
            r#"
id: runtime-prefab-tree
title: Runtime Prefab Tree
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  player-preset: flight-player
runtime-objects:
  - name: pilot-root
    kind: runtime-object
    prefab: prefabs/flight-player
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
    overrides:
      components:
        gameplay:
          module: "test.player"
    children:
      - name: cockpit
        prefab: prefabs/cockpit
        transform:
          space: 3d
          translation: [0.0, 0.0, 0.5]
        overrides:
          components:
            render:
              mesh: "cockpit://test"
layers: []
"#,
        )
        .expect("write scene");

        let snapshot = collect_mod_snapshot(mod_dir.path())
            .expect("snapshot collection should succeed")
            .expect("directory snapshot");

        assert_eq!(snapshot.scene_files_scanned, 1);
        assert_eq!(snapshot.controller_defaults_files, 1);
        assert_eq!(
            snapshot
                .controller_default_field_counts
                .get("player-preset"),
            Some(&1usize)
        );
        assert_eq!(snapshot.object_ref_files, 0);
        assert_eq!(snapshot.object_ref_sequences, 0);
        assert_eq!(snapshot.object_ref_instances, 0);
        assert_eq!(snapshot.object_repeat_groups, 0);
        assert_eq!(snapshot.object_repeat_expanded_instances, 0);
        assert!(
            snapshot
                .format_summary()
                .contains("objects=0 files / 0 sequences / 0 instances / 0 repeat groups / 0 repeat instances"),
            "prefab-first runtime-object trees should remain outside legacy object expansion accounting"
        );
    }
}

//! Authored reusable object documents and vNext runtime-object scaffolding.

#[allow(unused_imports)]
pub use engine_core::scene::model::{RuntimeObjectDocument, RuntimeObjectTransform};
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
/// Authored reusable object definition loaded before scene materialization.
///
/// This is the legacy prefab surface that scene compilation still lowers into
/// layers and sprites. Object documents provide exported defaults, optional
/// logic metadata, and scene content for that transitional path.
pub struct ObjectDocument {
    pub name: String,
    /// Optional explicit document kind marker for authored tooling.
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub exports: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub state: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub logic: Option<LogicSpec>,
}

#[derive(Debug, Clone, Deserialize)]
/// Authored logic metadata attached to an object document.
///
/// Native logic is still the only variant lowered into layer behaviors during
/// scene compilation; the other kinds preserve the authored boundary for
/// future runtimes.
pub struct LogicSpec {
    #[serde(default, rename = "type", alias = "kind")]
    pub kind: LogicKind,
    #[serde(default)]
    pub behavior: Option<String>,
    #[serde(default)]
    pub params: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
/// Declares which runtime should interpret an object's authored logic block.
pub enum LogicKind {
    #[default]
    Native,
    Graph,
    Script,
}

const RUNTIME_OBJECT_FAMILY_ALIASES: &[(&str, &[&str])] = &[
    ("reference-frame", &["reference_frame"]),
    ("follow-anchor-3d", &["follow_anchor_3d"]),
    ("linear-motor-3d", &["linear_motor_3d"]),
    ("angular-motor-3d", &["angular_motor_3d"]),
    ("character-motor-3d", &["character_motor_3d"]),
    ("flight-motor-3d", &["flight_motor_3d"]),
    ("camera-rig", &["camera_rig"]),
    ("celestial-binding", &["celestial_binding"]),
    ("extra-data", &["extra_data"]),
];

fn string_key(key: &str) -> Value {
    Value::String(key.to_string())
}

fn promote_alias(map: &mut Mapping, canonical: &str, aliases: &[&str]) {
    let canonical_key = string_key(canonical);
    let canonical_present = map.contains_key(&canonical_key);
    let mut promoted = None;
    for alias in aliases {
        let alias_key = string_key(alias);
        if let Some(value) = map.remove(&alias_key) {
            if !canonical_present && promoted.is_none() {
                promoted = Some(value);
            }
        }
    }
    if let Some(value) = promoted {
        map.insert(canonical_key, value);
    }
}

fn normalize_runtime_object_components(map: &mut Mapping) {
    for (canonical, aliases) in RUNTIME_OBJECT_FAMILY_ALIASES {
        promote_alias(map, canonical, aliases);
    }
}

fn normalize_runtime_object_node(node: &mut Value) {
    let Some(node_map) = node.as_mapping_mut() else {
        return;
    };

    promote_alias(node_map, "prefab", &["ref", "use"]);
    promote_alias(node_map, "overrides", &["with"]);

    if let Some(components) = node_map
        .get_mut(string_key("components"))
        .and_then(Value::as_mapping_mut)
    {
        normalize_runtime_object_components(components);
    }

    if let Some(overrides) = node_map
        .get_mut(string_key("overrides"))
        .and_then(Value::as_mapping_mut)
    {
        if let Some(components) = overrides
            .get_mut(string_key("components"))
            .and_then(Value::as_mapping_mut)
        {
            normalize_runtime_object_components(components);
        }
    }

    if let Some(children) = node_map
        .get_mut(string_key("children"))
        .and_then(Value::as_sequence_mut)
    {
        for child in children {
            normalize_runtime_object_node(child);
        }
    }
}

/// Normalizes the authored `runtime-objects` bridge surface in raw scene YAML.
///
/// This keeps the prefab-first path ergonomic while the vNext runtime-object
/// model is still a pass-through bridge:
/// - `ref` / `use` are promoted to canonical `prefab`
/// - `with` is promoted to canonical `overrides`
/// - known component-family underscore aliases are canonicalized to kebab-case
pub(crate) fn normalize_runtime_objects_surface(root: &mut Value) {
    let Some(scene_map) = root.as_mapping_mut() else {
        return;
    };

    let runtime_objects_value = if scene_map.contains_key(string_key("runtime-objects")) {
        scene_map.get_mut(string_key("runtime-objects"))
    } else {
        scene_map.get_mut(string_key("runtime_objects"))
    };
    let Some(runtime_objects) = runtime_objects_value.and_then(Value::as_sequence_mut) else {
        return;
    };

    for node in runtime_objects {
        normalize_runtime_object_node(node);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_runtime_objects_surface, ObjectDocument, RuntimeObjectDocument,
        RuntimeObjectTransform,
    };
    use serde_yaml::Value;

    fn runtime_objects_from_value(raw: &Value) -> &Vec<Value> {
        raw.as_mapping()
            .and_then(|map| {
                map.get(Value::String("runtime-objects".to_string()))
                    .or_else(|| map.get(Value::String("runtime_objects".to_string())))
            })
            .and_then(Value::as_sequence)
            .expect("runtime-objects sequence")
    }

    #[test]
    fn parses_object_document_kind_marker() {
        let doc = serde_yaml::from_str::<ObjectDocument>(
            r#"
name: sample
kind: object
"#,
        )
        .expect("object document");

        assert_eq!(doc.kind.as_deref(), Some("object"));
    }

    #[test]
    fn parses_runtime_object_kind_marker() {
        let doc = serde_yaml::from_str::<RuntimeObjectDocument>(
            r#"
name: runtime-root
kind: runtime-object
transform:
  space: 2d
  x: 0
  y: 0
"#,
        )
        .expect("runtime-object document");

        assert_eq!(doc.kind.as_deref(), Some("runtime-object"));
    }

    #[test]
    fn normalize_runtime_objects_surface_promotes_prefab_and_overrides_aliases() {
        let mut raw: Value = serde_yaml::from_str(
            r#"
runtime-objects:
  - name: root
    ref: /prefabs/player.yml
    transform:
      space: 3d
    with:
      slot: pilot
    children:
      - name: child
        use: /prefabs/cockpit.yml
        transform:
          space: 2d
          x: 1
          y: 2
        with:
          seat: gunner
"#,
        )
        .expect("raw scene value");

        normalize_runtime_objects_surface(&mut raw);
        let runtime_objects = runtime_objects_from_value(&raw);
        let root = serde_yaml::from_value::<RuntimeObjectDocument>(runtime_objects[0].clone())
            .expect("normalized runtime object");
        assert_eq!(root.prefab.as_deref(), Some("/prefabs/player.yml"));
        assert_eq!(
            root.overrides.get("slot").and_then(Value::as_str),
            Some("pilot")
        );
        assert_eq!(
            root.children[0].prefab.as_deref(),
            Some("/prefabs/cockpit.yml")
        );
        assert_eq!(
            root.children[0]
                .overrides
                .get("seat")
                .and_then(Value::as_str),
            Some("gunner")
        );
    }

    #[test]
    fn normalize_runtime_objects_surface_canonicalizes_component_family_aliases() {
        let mut raw: Value = serde_yaml::from_str(
            r#"
runtime-objects:
  - name: root
    transform:
      space: 3d
    components:
      reference_frame:
        mode: LocalHorizon
        body_id: earth
      linear_motor_3d:
        space: ReferenceFrame
        accel: 24
      camera_rig:
        preset: cockpit
      celestial_binding:
        body_id: earth
"#,
        )
        .expect("raw scene value");

        normalize_runtime_objects_surface(&mut raw);
        let runtime_objects = runtime_objects_from_value(&raw);
        let root = serde_yaml::from_value::<RuntimeObjectDocument>(runtime_objects[0].clone())
            .expect("normalized runtime object");
        assert!(matches!(
            root.transform,
            RuntimeObjectTransform::ThreeD { .. }
        ));
        assert!(root.components.contains_key("reference-frame"));
        assert!(root.components.contains_key("linear-motor-3d"));
        assert!(root.components.contains_key("camera-rig"));
        assert!(root.components.contains_key("celestial-binding"));
        assert!(!root.components.contains_key("reference_frame"));
        assert!(!root.components.contains_key("linear_motor_3d"));
        assert!(!root.components.contains_key("camera_rig"));
        assert!(!root.components.contains_key("celestial_binding"));
    }

    #[test]
    fn normalize_runtime_objects_surface_canonicalizes_override_component_family_aliases() {
        let mut raw: Value = serde_yaml::from_str(
            r#"
runtime-objects:
  - name: root
    transform:
      space: 3d
    with:
      components:
        linear_motor_3d:
          space: ReferenceFrame
          accel: 18
        camera_rig:
          preset: chase
    children:
      - name: child
        transform:
          space: 2d
          x: 1
          y: 2
        with:
          components:
            follow_anchor_3d:
              local_offset: [1, 2, 3]
"#,
        )
        .expect("raw scene value");

        normalize_runtime_objects_surface(&mut raw);
        let runtime_objects = runtime_objects_from_value(&raw);
        let root = serde_yaml::from_value::<RuntimeObjectDocument>(runtime_objects[0].clone())
            .expect("normalized runtime object");
        let root_components = root
            .overrides
            .get("components")
            .and_then(Value::as_mapping)
            .expect("root override components");
        let child_components = root.children[0]
            .overrides
            .get("components")
            .and_then(Value::as_mapping)
            .expect("child override components");
        assert!(root_components.contains_key(Value::String("linear-motor-3d".to_string())));
        assert!(root_components.contains_key(Value::String("camera-rig".to_string())));
        assert!(!root_components.contains_key(Value::String("linear_motor_3d".to_string())));
        assert!(!root_components.contains_key(Value::String("camera_rig".to_string())));
        assert!(child_components.contains_key(Value::String("follow-anchor-3d".to_string())));
        assert!(!child_components.contains_key(Value::String("follow_anchor_3d".to_string())));
    }

    #[test]
    fn normalize_runtime_objects_surface_handles_runtime_objects_alias_and_nested_prefab_subtrees()
    {
        let mut raw: Value = serde_yaml::from_str(
            r#"
runtime_objects:
  - name: root
    ref: /prefabs/player.yml
    preset: cockpit
    transform:
      space: 3d
    with:
      seat: pilot
      components:
        extra_data:
          role: command
    children:
      - name: child
        use: /prefabs/camera.yml
        transform:
          space: 3d
        with:
          components:
            linear_motor_3d:
              accel: 12
        children:
          - name: marker
            prefab: /prefabs/marker.yml
            transform:
              space: 2d
              x: 1
              y: 2
            with:
              metadata:
                keep: true
"#,
        )
        .expect("raw scene value");

        normalize_runtime_objects_surface(&mut raw);
        let runtime_objects = runtime_objects_from_value(&raw);
        let root = serde_yaml::from_value::<RuntimeObjectDocument>(runtime_objects[0].clone())
            .expect("normalized runtime object");
        assert_eq!(root.prefab.as_deref(), Some("/prefabs/player.yml"));
        assert_eq!(root.preset.as_deref(), Some("cockpit"));
        assert_eq!(
            root.overrides.get("seat").and_then(Value::as_str),
            Some("pilot")
        );
        assert!(root
            .overrides
            .get("components")
            .and_then(Value::as_mapping)
            .is_some_and(|map| map.contains_key(Value::String("extra-data".to_string()))));
        assert_eq!(root.children.len(), 1);
        assert_eq!(
            root.children[0].prefab.as_deref(),
            Some("/prefabs/camera.yml")
        );
        assert!(root.children[0]
            .overrides
            .get("components")
            .and_then(Value::as_mapping)
            .is_some_and(|map| map.contains_key(Value::String("linear-motor-3d".to_string()))));
        assert_eq!(root.children[0].children.len(), 1);
        assert_eq!(
            root.children[0].children[0].prefab.as_deref(),
            Some("/prefabs/marker.yml")
        );
        assert_eq!(
            root.children[0].children[0]
                .overrides
                .get("metadata")
                .and_then(Value::as_mapping)
                .and_then(|map| map.get(Value::String("keep".to_string())))
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn normalize_runtime_objects_surface_preserves_child_lifecycle_override_payload() {
        let mut raw: Value = serde_yaml::from_str(
            r#"
runtime-objects:
  - name: root
    prefab: /prefabs/carrier.yml
    transform:
      space: 3d
    children:
      - name: escort
        use: /prefabs/drone.yml
        transform:
          space: 3d
        with:
          inherit_owner_lifecycle: true
          ttl_ms: 250
          components:
            lifecycle: TtlFollowOwner
            extra_data:
              trail: true
"#,
        )
        .expect("raw scene value");

        normalize_runtime_objects_surface(&mut raw);
        let runtime_objects = runtime_objects_from_value(&raw);
        let root = serde_yaml::from_value::<RuntimeObjectDocument>(runtime_objects[0].clone())
            .expect("normalized runtime object");
        let escort = &root.children[0];
        assert_eq!(escort.prefab.as_deref(), Some("/prefabs/drone.yml"));
        assert_eq!(
            escort
                .overrides
                .get("inherit_owner_lifecycle")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            escort.overrides.get("ttl_ms").and_then(Value::as_i64),
            Some(250)
        );
        assert_eq!(
            escort
                .overrides
                .get("components")
                .and_then(Value::as_mapping)
                .and_then(|map| map.get(Value::String("lifecycle".to_string())))
                .and_then(Value::as_str),
            Some("TtlFollowOwner")
        );
        assert!(escort
            .overrides
            .get("components")
            .and_then(Value::as_mapping)
            .and_then(|map| map.get(Value::String("extra-data".to_string())))
            .and_then(Value::as_mapping)
            .is_some_and(|map| {
                map.get(Value::String("trail".to_string()))
                    .and_then(Value::as_bool)
                    == Some(true)
            }));
    }
}

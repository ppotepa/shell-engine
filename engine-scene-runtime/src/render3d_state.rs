use super::*;
use engine_core::render_types::DirtyMask3D;
use engine_core::render_types::MaterialValue;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Render3dRebuildDiagnostics {
    pub mesh_dirty_events: u64,
    pub worldgen_dirty_events: u64,
}

impl Render3dRebuildDiagnostics {
    pub fn is_empty(self) -> bool {
        self.mesh_dirty_events == 0 && self.worldgen_dirty_events == 0
    }
}

impl SceneRuntime {
    pub(crate) fn apply_render3d_property_for_target(
        &mut self,
        object_id: &str,
        target: &str,
        path: &str,
        value: &JsonValue,
    ) -> bool {
        match path {
            "scene3d.frame" => {
                let Some(next_frame) = value.as_str() else {
                    return false;
                };
                self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                    runtime.set_scene3d_sprite_frame(alias, next_frame)
                })
            }
            path if path.starts_with("planet.") => {
                self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                    runtime.set_planet_sprite_property(alias, path, value)
                })
            }
            path if path.starts_with("obj.")
                || path.starts_with("terrain.")
                || path.starts_with("world.") =>
            {
                self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                    runtime.set_obj_sprite_property(alias, path, value)
                })
            }
            _ => false,
        }
    }

    pub(crate) fn track_render3d_rebuild_cause(&mut self, dirty: DirtyMask3D) {
        if dirty.contains(DirtyMask3D::MESH) {
            self.render3d_rebuild_diagnostics.mesh_dirty_events = self
                .render3d_rebuild_diagnostics
                .mesh_dirty_events
                .saturating_add(1);
        }
        if dirty.contains(DirtyMask3D::WORLDGEN) {
            self.render3d_rebuild_diagnostics.worldgen_dirty_events = self
                .render3d_rebuild_diagnostics
                .worldgen_dirty_events
                .saturating_add(1);
        }
    }
}

pub fn scene_mutation_from_set_property_3d(
    target: &str,
    path: &str,
    value: &serde_json::Value,
) -> Option<SceneMutation> {
    if !is_render3d_set_property_path(path) {
        return None;
    }
    Some(SceneMutation::SetRender3D(
        Render3DMutation::SetWorldgenParam {
            target: target.to_string(),
            param: path.to_string(),
            value: material_value_from_json(value)?,
        },
    ))
}

fn is_render3d_set_property_path(path: &str) -> bool {
    path == "scene3d.frame"
        || path.starts_with("planet.")
        || path.starts_with("obj.")
        || path.starts_with("terrain.")
        || path.starts_with("world.")
}

pub(crate) fn render3d_material_value_to_json(
    value: &engine_core::render_types::MaterialValue,
) -> Option<JsonValue> {
    match value {
        engine_core::render_types::MaterialValue::Scalar(v) => {
            serde_json::Number::from_f64(*v as f64).map(JsonValue::Number)
        }
        engine_core::render_types::MaterialValue::ColorRgb(rgb) => Some(JsonValue::Array(vec![
            JsonValue::from(rgb[0]),
            JsonValue::from(rgb[1]),
            JsonValue::from(rgb[2]),
        ])),
        engine_core::render_types::MaterialValue::Bool(v) => Some(JsonValue::Bool(*v)),
        engine_core::render_types::MaterialValue::Text(v) => Some(JsonValue::String(v.clone())),
    }
}

pub(crate) fn material_value_from_json(value: &serde_json::Value) -> Option<MaterialValue> {
    if let Some(n) = value.as_f64() {
        return Some(MaterialValue::Scalar(n as f32));
    }
    if let Some(b) = value.as_bool() {
        return Some(MaterialValue::Bool(b));
    }
    if let Some(s) = value.as_str() {
        return Some(MaterialValue::Text(s.to_string()));
    }
    if let Some(arr) = value.as_array() {
        if arr.len() == 3 {
            let r = arr.first()?.as_u64().and_then(|v| u8::try_from(v).ok())?;
            let g = arr.get(1)?.as_u64().and_then(|v| u8::try_from(v).ok())?;
            let b = arr.get(2)?.as_u64().and_then(|v| u8::try_from(v).ok())?;
            return Some(MaterialValue::ColorRgb([r, g, b]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_obj_namespace_set_property_to_typed_worldgen_mutation() {
        let mutation =
            scene_mutation_from_set_property_3d("ship", "obj.scale", &serde_json::json!(1.25))
                .expect("typed mutation");
        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetWorldgenParam {
                target,
                param,
                value,
            }) => {
                assert_eq!(target, "ship");
                assert_eq!(param, "obj.scale");
                assert_eq!(value, MaterialValue::Scalar(1.25));
            }
            _ => panic!("expected worldgen mutation"),
        }
    }

    #[test]
    fn leaves_non_render3d_set_property_unmapped() {
        let mutation =
            scene_mutation_from_set_property_3d("hud", "text.content", &serde_json::json!("hello"));
        assert!(mutation.is_none());
    }
}

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
    pub(crate) fn apply_render3d_compat_property_for_target(
        &mut self,
        object_id: &str,
        target: &str,
        property: &Render3DCompatProperty,
    ) -> bool {
        match property {
            Render3DCompatProperty::Scene3dFrame { frame } => {
                self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                    runtime.set_scene3d_sprite_frame(alias, frame)
                })
            }
            Render3DCompatProperty::PlanetParam { path, value } => {
                let Some(json_value) = render3d_material_value_to_json(value) else {
                    return false;
                };
                self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                    runtime.set_planet_sprite_property(alias, path, &json_value)
                })
            }
            Render3DCompatProperty::ObjParam { path, value } => {
                let Some(json_value) = render3d_material_value_to_json(value) else {
                    return false;
                };
                self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                    runtime.set_obj_sprite_property(alias, path, &json_value)
                })
            }
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
    let value = material_value_from_json(value)?;
    let property = render3d_compat_property_from_param(path, value)?;
    Some(SceneMutation::SetRender3D(
        Render3DMutation::SetCompatProperty {
            target: target.to_string(),
            property,
        },
    ))
}

pub(crate) fn render3d_compat_property_from_param(
    path: &str,
    value: MaterialValue,
) -> Option<Render3DCompatProperty> {
    if path == "scene3d.frame" {
        if let MaterialValue::Text(frame) = value {
            return Some(Render3DCompatProperty::Scene3dFrame { frame });
        }
        return None;
    }
    if path.starts_with("planet.") {
        return Some(Render3DCompatProperty::PlanetParam {
            path: path.to_string(),
            value,
        });
    }
    if path.starts_with("obj.") || path.starts_with("terrain.") || path.starts_with("world.") {
        return Some(Render3DCompatProperty::ObjParam {
            path: path.to_string(),
            value,
        });
    }
    None
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
            SceneMutation::SetRender3D(Render3DMutation::SetCompatProperty {
                target,
                property,
            }) => {
                assert_eq!(target, "ship");
                assert_eq!(
                    property,
                    Render3DCompatProperty::ObjParam {
                        path: "obj.scale".to_string(),
                        value: MaterialValue::Scalar(1.25),
                    }
                );
            }
            _ => panic!("expected compatibility property mutation"),
        }
    }

    #[test]
    fn leaves_non_render3d_set_property_unmapped() {
        let mutation =
            scene_mutation_from_set_property_3d("hud", "text.content", &serde_json::json!("hello"));
        assert!(mutation.is_none());
    }
}

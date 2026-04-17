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
    use crate::mutations::{AtmosphereParam, ObjMaterialParam, PlanetParam, TerrainParam, WorldgenParam};
    let mat_value = material_value_from_json(value)?;

    if path == "scene3d.frame" {
        if let MaterialValue::Text(frame) = mat_value {
            return Some(SceneMutation::SetRender3D(Render3DMutation::SetCompatProperty {
                target: target.to_string(),
                property: Render3DCompatProperty::Scene3dFrame { frame },
            }));
        }
        return None;
    }

    if path.starts_with("planet.") {
        if let Some(param) = PlanetParam::from_full_path(path) {
            return Some(SceneMutation::SetRender3D(Render3DMutation::SetPlanetParamTyped {
                target: target.to_string(),
                param,
                value: mat_value,
            }));
        }
        return None;
    }

    if path.starts_with("obj.atmo.") {
        if let Some(param) = AtmosphereParam::from_full_path(path) {
            return Some(SceneMutation::SetRender3D(Render3DMutation::SetAtmosphereParamTyped {
                target: target.to_string(),
                param,
                value: mat_value,
            }));
        }
        return None;
    }

    if path.starts_with("obj.") {
        if let Some(param) = ObjMaterialParam::from_full_path(path) {
            return Some(SceneMutation::SetRender3D(Render3DMutation::SetObjMaterialParam {
                target: target.to_string(),
                param,
                value: mat_value,
            }));
        }
        return None;
    }

    if path.starts_with("terrain.") {
        if let Some(param) = TerrainParam::from_full_path(path) {
            return Some(SceneMutation::SetRender3D(Render3DMutation::SetTerrainParamTyped {
                target: target.to_string(),
                param,
                value: mat_value,
            }));
        }
        return None;
    }

    if path.starts_with("world.") {
        if let Some(param) = WorldgenParam::from_full_path(path) {
            return Some(SceneMutation::SetRender3D(Render3DMutation::SetWorldgenParamTyped {
                target: target.to_string(),
                param,
                value: mat_value,
            }));
        }
        return None;
    }

    None
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
    fn maps_obj_namespace_set_property_to_typed_material_mutation() {
        let mutation =
            scene_mutation_from_set_property_3d("ship", "obj.scale", &serde_json::json!(1.25))
                .expect("typed mutation");
        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetObjMaterialParam {
                target,
                param,
                value,
            }) => {
                assert_eq!(target, "ship");
                assert_eq!(param, crate::mutations::ObjMaterialParam::Scale);
                assert_eq!(value, MaterialValue::Scalar(1.25));
            }
            _ => panic!("expected SetObjMaterialParam"),
        }
    }

    #[test]
    fn leaves_non_render3d_set_property_unmapped() {
        let mutation =
            scene_mutation_from_set_property_3d("hud", "text.content", &serde_json::json!("hello"));
        assert!(mutation.is_none());
    }
}

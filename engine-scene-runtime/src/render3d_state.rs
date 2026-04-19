use super::*;
use engine_core::render_types::DirtyMask3D;
use engine_core::render_types::MaterialValue;
use engine_core::scene::{LightingProfile, SpaceEnvironmentProfile, TonemapOperator};

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
    pub(crate) fn apply_profile_selection(
        &mut self,
        slot: crate::mutations::Render3DProfileSlot,
        profile: &str,
    ) -> bool {
        let view = self.scene.view.get_or_insert_with(Default::default);
        match slot {
            crate::mutations::Render3DProfileSlot::View => {
                view.profile = Some(profile.to_string());
            }
            crate::mutations::Render3DProfileSlot::Lighting => {
                view.lighting_profile = Some(profile.to_string());
            }
            crate::mutations::Render3DProfileSlot::SpaceEnvironment => {
                view.space_environment_profile = Some(profile.to_string());
            }
        }
        self.refresh_resolved_view_profile();
        true
    }

    pub(crate) fn apply_profile_param(
        &mut self,
        param: &crate::mutations::Render3DProfileParam,
        value: &MaterialValue,
    ) -> bool {
        match param {
            crate::mutations::Render3DProfileParam::Lighting(param) => {
                self.apply_lighting_profile_param(param, value)
            }
            crate::mutations::Render3DProfileParam::SpaceEnvironment(param) => {
                self.apply_space_environment_param(param, value)
            }
        }
    }

    pub(crate) fn refresh_resolved_view_profile(&mut self) {
        let mut resolved = engine_core::scene::resolve_scene_view_profile(&self.scene);
        if let Some(override_profile) = self.runtime_lighting_profile_override.as_ref() {
            resolved.lighting =
                engine_core::scene::merge_lighting_profile(resolved.lighting, override_profile);
        }
        if let Some(override_profile) = self.runtime_space_environment_override.as_ref() {
            resolved.environment = engine_core::scene::merge_space_environment_profile(
                resolved.environment,
                override_profile,
            );
        }
        self.resolved_view_profile = resolved;
    }

    pub(crate) fn apply_lighting_profile_param(
        &mut self,
        param: &crate::mutations::LightingProfileParam,
        value: &MaterialValue,
    ) -> bool {
        let profile = self
            .runtime_lighting_profile_override
            .get_or_insert_with(|| LightingProfile {
                id: if self.resolved_view_profile.lighting.id.is_empty() {
                    "runtime-lighting-override".to_string()
                } else {
                    self.resolved_view_profile.lighting.id.clone()
                },
                ..Default::default()
            });

        match param {
            crate::mutations::LightingProfileParam::AmbientIntensity => {
                profile.ambient_intensity = scalar_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::KeyLightIntensity => {
                profile.key_light_intensity = scalar_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::FillLightIntensity => {
                profile.fill_light_intensity = scalar_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::RimLightIntensity => {
                profile.rim_light_intensity = scalar_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::BlackLevel => {
                profile.black_level = scalar_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::ShadowContrast => {
                profile.shadow_contrast = scalar_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::Exposure => {
                profile.exposure = scalar_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::Tonemap => {
                profile.tonemap = tonemap_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::Gamma => {
                profile.gamma = scalar_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::NightGlowScale => {
                profile.night_glow_scale = scalar_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::HazeNightLeak => {
                profile.haze_night_leak = scalar_from_material_value(value);
            }
            crate::mutations::LightingProfileParam::SpecularFloor => {
                profile.specular_floor = scalar_from_material_value(value);
            }
        }

        self.refresh_resolved_view_profile();
        true
    }

    pub(crate) fn apply_space_environment_param(
        &mut self,
        param: &crate::mutations::SpaceEnvironmentParam,
        value: &MaterialValue,
    ) -> bool {
        let profile = self
            .runtime_space_environment_override
            .get_or_insert_with(|| SpaceEnvironmentProfile {
                id: if self.resolved_view_profile.environment.id.is_empty() {
                    "runtime-space-environment-override".to_string()
                } else {
                    self.resolved_view_profile.environment.id.clone()
                },
                ..Default::default()
            });

        match param {
            crate::mutations::SpaceEnvironmentParam::BackgroundColor => {
                profile.background_color = text_or_hex_from_material_value(value);
            }
            crate::mutations::SpaceEnvironmentParam::BackgroundFloor => {
                profile.background_floor = scalar_from_material_value(value);
            }
            crate::mutations::SpaceEnvironmentParam::StarfieldDensity => {
                profile.starfield_density = scalar_from_material_value(value);
            }
            crate::mutations::SpaceEnvironmentParam::StarfieldBrightness => {
                profile.starfield_brightness = scalar_from_material_value(value);
            }
            crate::mutations::SpaceEnvironmentParam::StarfieldSizeMin => {
                profile.starfield_size_min = scalar_from_material_value(value);
            }
            crate::mutations::SpaceEnvironmentParam::StarfieldSizeMax => {
                profile.starfield_size_max = scalar_from_material_value(value);
            }
            crate::mutations::SpaceEnvironmentParam::PrimaryStarColor => {
                profile.primary_star_color = text_or_hex_from_material_value(value);
            }
            crate::mutations::SpaceEnvironmentParam::PrimaryStarGlareStrength => {
                profile.primary_star_glare_strength = scalar_from_material_value(value);
            }
            crate::mutations::SpaceEnvironmentParam::PrimaryStarGlareWidth => {
                profile.primary_star_glare_width = scalar_from_material_value(value);
            }
            crate::mutations::SpaceEnvironmentParam::NebulaStrength => {
                profile.nebula_strength = scalar_from_material_value(value);
            }
            crate::mutations::SpaceEnvironmentParam::DustBandStrength => {
                profile.dust_band_strength = scalar_from_material_value(value);
            }
        }

        self.refresh_resolved_view_profile();
        true
    }

    pub(crate) fn apply_scene3d_frame_for_target(
        &mut self,
        object_id: &str,
        target: &str,
        frame: &str,
    ) -> bool {
        self.apply_text_property_for_target(object_id, target, |runtime, alias| {
            runtime.set_scene3d_sprite_frame(alias, frame)
        })
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

pub fn scene_mutation_from_render_path(
    target: &str,
    path: &str,
    value: &serde_json::Value,
) -> Option<SceneMutation> {
    use crate::mutations::{
        AtmosphereParam, ObjMaterialParam, PlanetParam, TerrainParam, WorldgenParam,
    };
    let mat_value = material_value_from_json(value)?;

    if path == "scene3d.frame" {
        if let MaterialValue::Text(frame) = mat_value {
            return Some(SceneMutation::SetRender3D(
                Render3DMutation::SetScene3DFrame {
                    target: target.to_string(),
                    frame,
                },
            ));
        }
        return None;
    }

    if path.starts_with("planet.") {
        if let Some(param) = PlanetParam::from_full_path(path) {
            return Some(SceneMutation::SetRender3D(
                Render3DMutation::SetGroupedParams {
                    target: Some(target.to_string()),
                    params: vec![(
                        crate::mutations::Render3DGroupedParam::Body(param),
                        mat_value,
                    )],
                },
            ));
        }
        return None;
    }

    if path.starts_with("obj.atmo.") {
        if let Some(param) = AtmosphereParam::from_full_path(path) {
            return Some(SceneMutation::SetRender3D(
                Render3DMutation::SetGroupedParams {
                    target: Some(target.to_string()),
                    params: vec![(
                        crate::mutations::Render3DGroupedParam::Atmosphere(param),
                        mat_value,
                    )],
                },
            ));
        }
        return None;
    }

    if path.starts_with("obj.") {
        if let Some(param) = ObjMaterialParam::from_full_path(path) {
            return Some(SceneMutation::SetRender3D(
                Render3DMutation::SetGroupedParams {
                    target: Some(target.to_string()),
                    params: vec![(
                        crate::mutations::Render3DGroupedParam::Material(param),
                        mat_value,
                    )],
                },
            ));
        }
        return None;
    }

    if path.starts_with("terrain.") {
        if let Some(param) = TerrainParam::from_full_path(path) {
            return Some(SceneMutation::SetRender3D(
                Render3DMutation::SetGroupedParams {
                    target: Some(target.to_string()),
                    params: vec![(
                        crate::mutations::Render3DGroupedParam::Surface(param),
                        mat_value,
                    )],
                },
            ));
        }
        return None;
    }

    if path.starts_with("world.") {
        if let Some(param) = WorldgenParam::from_full_path(path) {
            return Some(SceneMutation::SetRender3D(
                Render3DMutation::SetGroupedParams {
                    target: Some(target.to_string()),
                    params: vec![(
                        crate::mutations::Render3DGroupedParam::Generator(param),
                        mat_value,
                    )],
                },
            ));
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

fn scalar_from_material_value(value: &MaterialValue) -> Option<f32> {
    match value {
        MaterialValue::Scalar(v) => Some(*v),
        _ => None,
    }
}

fn text_or_hex_from_material_value(value: &MaterialValue) -> Option<String> {
    match value {
        MaterialValue::Text(text) => Some(text.clone()),
        MaterialValue::ColorRgb([r, g, b]) => Some(format!("#{r:02x}{g:02x}{b:02x}")),
        _ => None,
    }
}

fn tonemap_from_material_value(value: &MaterialValue) -> Option<TonemapOperator> {
    match value {
        MaterialValue::Text(text) => match text.as_str() {
            "linear" => Some(TonemapOperator::Linear),
            "reinhard" => Some(TonemapOperator::Reinhard),
            "aces_approx" | "aces-approx" => Some(TonemapOperator::AcesApprox),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::scene::{Scene, SceneView};

    fn test_scene_runtime() -> SceneRuntime {
        SceneRuntime::new(Scene {
            id: "test".to_string(),
            title: "Test".to_string(),
            cutscene: false,
            target_fps: None,
            space: Default::default(),
            spatial: Default::default(),
            celestial: Default::default(),
            lighting: None,
            view: Some(SceneView {
                profile: Some("orbit-realistic".to_string()),
                lighting_profile: None,
                space_environment_profile: None,
                resolved_view_profile_asset: None,
                resolved_lighting_profile_asset: None,
                resolved_space_environment_profile_asset: None,
            }),
            virtual_size_override: None,
            bg_colour: None,
            stages: Default::default(),
            behaviors: Vec::new(),
            audio: Default::default(),
            ui: Default::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            input: Default::default(),
            postfx: Vec::new(),
            next: None,
            prerender: false,
            palette_bindings: Vec::new(),
            game_state_bindings: Vec::new(),
            gui: Default::default(),
        })
    }

    #[test]
    fn maps_obj_namespace_set_property_to_typed_material_mutation() {
        let mutation =
            scene_mutation_from_render_path("ship", "obj.scale", &serde_json::json!(1.25))
                .expect("typed mutation");
        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetGroupedParams {
                target,
                params,
            }) => {
                assert_eq!(target.as_deref(), Some("ship"));
                assert_eq!(
                    params,
                    vec![(
                        crate::mutations::Render3DGroupedParam::Material(
                            crate::mutations::ObjMaterialParam::Scale
                        ),
                        MaterialValue::Scalar(1.25),
                    )]
                );
            }
            _ => panic!("expected SetGroupedParams"),
        }
    }

    #[test]
    fn leaves_non_render3d_set_property_unmapped() {
        let mutation =
            scene_mutation_from_render_path("hud", "text.content", &serde_json::json!("hello"));
        assert!(mutation.is_none());
    }

    #[test]
    fn apply_scene_level_profile_param_mutations_refreshes_resolved_view() {
        let mut runtime = test_scene_runtime();
        assert!(runtime.apply_profile_selection(
            crate::mutations::Render3DProfileSlot::Lighting,
            "lab-neutral",
        ));
        assert!(runtime.apply_profile_param(
            &crate::mutations::Render3DProfileParam::Lighting(
                crate::mutations::LightingProfileParam::Exposure,
            ),
            &MaterialValue::Scalar(0.81),
        ));
        assert!(runtime.apply_profile_param(
            &crate::mutations::Render3DProfileParam::SpaceEnvironment(
                crate::mutations::SpaceEnvironmentParam::BackgroundColor,
            ),
            &MaterialValue::Text("#010203".to_string()),
        ));

        assert_eq!(
            runtime
                .scene
                .view
                .as_ref()
                .and_then(|view| view.lighting_profile.as_deref()),
            Some("lab-neutral")
        );
        assert_eq!(
            runtime.resolved_view_profile().lighting.exposure,
            Some(0.81)
        );
        assert_eq!(
            runtime
                .resolved_view_profile()
                .environment
                .background_color
                .as_deref(),
            Some("#010203")
        );
    }
}

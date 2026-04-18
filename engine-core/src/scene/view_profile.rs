use serde::{Deserialize, Serialize};

use super::model::Scene;

/// Supported scene-level tone mapping operators for resolved 3D view profiles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TonemapOperator {
    #[default]
    Linear,
    Reinhard,
    AcesApprox,
}

/// Reusable renderer-agnostic lighting profile for 3D scenes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LightingProfile {
    pub id: String,
    #[serde(default)]
    pub ambient_intensity: Option<f32>,
    #[serde(default)]
    pub key_light_intensity: Option<f32>,
    #[serde(default)]
    pub fill_light_intensity: Option<f32>,
    #[serde(default)]
    pub rim_light_intensity: Option<f32>,
    #[serde(default)]
    pub black_level: Option<f32>,
    #[serde(default)]
    pub shadow_contrast: Option<f32>,
    #[serde(default)]
    pub exposure: Option<f32>,
    #[serde(default)]
    pub tonemap: Option<TonemapOperator>,
    #[serde(default)]
    pub gamma: Option<f32>,
    #[serde(default)]
    pub night_glow_scale: Option<f32>,
    #[serde(default)]
    pub haze_night_leak: Option<f32>,
    #[serde(default)]
    pub specular_floor: Option<f32>,
}

/// Reusable observation environment profile for vacuum/space scenes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SpaceEnvironmentProfile {
    pub id: String,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub background_floor: Option<f32>,
    #[serde(default)]
    pub starfield_density: Option<f32>,
    #[serde(default)]
    pub starfield_brightness: Option<f32>,
    #[serde(default)]
    pub starfield_size_min: Option<f32>,
    #[serde(default)]
    pub starfield_size_max: Option<f32>,
    #[serde(default)]
    pub primary_star_color: Option<String>,
    #[serde(default)]
    pub primary_star_glare_strength: Option<f32>,
    #[serde(default)]
    pub primary_star_glare_width: Option<f32>,
    #[serde(default)]
    pub nebula_strength: Option<f32>,
    #[serde(default)]
    pub dust_band_strength: Option<f32>,
}

/// Small, explicit top-level overrides applied by a scene-facing view profile.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewProfileOverrides {
    #[serde(default)]
    pub exposure: Option<f32>,
    #[serde(default)]
    pub black_level: Option<f32>,
    #[serde(default)]
    pub background_floor: Option<f32>,
    #[serde(default)]
    pub starfield_brightness: Option<f32>,
    #[serde(default)]
    pub primary_star_glare_strength: Option<f32>,
}

/// Top-level reusable scene view profile.
///
/// This is the preferred authoring entry-point: it composes lower-level
/// lighting/environment profiles and allows a deliberately small override set.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewProfile {
    pub id: String,
    #[serde(default)]
    pub lighting_profile: Option<String>,
    #[serde(default)]
    pub space_environment_profile: Option<String>,
    #[serde(default)]
    pub overrides: ViewProfileOverrides,
}

/// Fully resolved 3D view contract consumed by runtime/render systems.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ResolvedViewProfile {
    pub lighting: LightingProfile,
    pub environment: SpaceEnvironmentProfile,
    #[serde(default)]
    pub overrides: ViewProfileOverrides,
}

pub fn builtin_lighting_profile(id: &str) -> Option<LightingProfile> {
    match id {
        "space-hard-vacuum" => Some(LightingProfile {
            id: id.to_string(),
            ambient_intensity: Some(0.02),
            key_light_intensity: Some(1.0),
            fill_light_intensity: Some(0.0),
            rim_light_intensity: Some(0.08),
            black_level: Some(0.0),
            shadow_contrast: Some(1.0),
            exposure: Some(0.95),
            tonemap: Some(TonemapOperator::Reinhard),
            gamma: Some(2.2),
            night_glow_scale: Some(0.0),
            haze_night_leak: Some(0.0),
            specular_floor: Some(0.0),
        }),
        "space-cinematic-soft" => Some(LightingProfile {
            id: id.to_string(),
            ambient_intensity: Some(0.08),
            key_light_intensity: Some(1.0),
            fill_light_intensity: Some(0.18),
            rim_light_intensity: Some(0.22),
            black_level: Some(0.03),
            shadow_contrast: Some(0.82),
            exposure: Some(1.05),
            tonemap: Some(TonemapOperator::Reinhard),
            gamma: Some(2.2),
            night_glow_scale: Some(0.08),
            haze_night_leak: Some(0.06),
            specular_floor: Some(0.03),
        }),
        "lab-neutral" => Some(LightingProfile {
            id: id.to_string(),
            ambient_intensity: Some(0.15),
            key_light_intensity: Some(0.85),
            fill_light_intensity: Some(0.25),
            rim_light_intensity: Some(0.12),
            black_level: Some(0.06),
            shadow_contrast: Some(0.7),
            exposure: Some(1.0),
            tonemap: Some(TonemapOperator::Linear),
            gamma: Some(2.2),
            night_glow_scale: Some(0.0),
            haze_night_leak: Some(0.0),
            specular_floor: Some(0.04),
        }),
        _ => None,
    }
}

pub fn builtin_space_environment_profile(id: &str) -> Option<SpaceEnvironmentProfile> {
    match id {
        "deep-space-sparse" => Some(SpaceEnvironmentProfile {
            id: id.to_string(),
            background_color: Some("#000008".to_string()),
            background_floor: Some(0.0),
            starfield_density: Some(0.18),
            starfield_brightness: Some(0.75),
            starfield_size_min: Some(0.6),
            starfield_size_max: Some(1.2),
            primary_star_color: Some("#fff4d6".to_string()),
            primary_star_glare_strength: Some(0.12),
            primary_star_glare_width: Some(0.18),
            nebula_strength: Some(0.0),
            dust_band_strength: Some(0.0),
        }),
        "deep-space-rich" => Some(SpaceEnvironmentProfile {
            id: id.to_string(),
            background_color: Some("#00000a".to_string()),
            background_floor: Some(0.0),
            starfield_density: Some(0.45),
            starfield_brightness: Some(0.92),
            starfield_size_min: Some(0.6),
            starfield_size_max: Some(1.6),
            primary_star_color: Some("#fff1cf".to_string()),
            primary_star_glare_strength: Some(0.2),
            primary_star_glare_width: Some(0.24),
            nebula_strength: Some(0.04),
            dust_band_strength: Some(0.02),
        }),
        _ => None,
    }
}

pub fn builtin_view_profile(id: &str) -> Option<ViewProfile> {
    match id {
        "orbit-realistic" => Some(ViewProfile {
            id: id.to_string(),
            lighting_profile: Some("space-hard-vacuum".to_string()),
            space_environment_profile: Some("deep-space-sparse".to_string()),
            overrides: ViewProfileOverrides {
                exposure: Some(0.92),
                black_level: Some(0.0),
                background_floor: Some(0.0),
                starfield_brightness: Some(0.7),
                primary_star_glare_strength: Some(0.1),
            },
        }),
        "orbit-cinematic" => Some(ViewProfile {
            id: id.to_string(),
            lighting_profile: Some("space-cinematic-soft".to_string()),
            space_environment_profile: Some("deep-space-rich".to_string()),
            overrides: ViewProfileOverrides {
                exposure: Some(1.08),
                black_level: Some(0.025),
                background_floor: Some(0.0),
                starfield_brightness: Some(0.95),
                primary_star_glare_strength: Some(0.24),
            },
        }),
        "deep-space-harsh" => Some(ViewProfile {
            id: id.to_string(),
            lighting_profile: Some("space-hard-vacuum".to_string()),
            space_environment_profile: Some("deep-space-sparse".to_string()),
            overrides: ViewProfileOverrides {
                exposure: Some(0.86),
                black_level: Some(0.0),
                background_floor: Some(0.0),
                starfield_brightness: Some(0.62),
                primary_star_glare_strength: Some(0.08),
            },
        }),
        _ => None,
    }
}

fn default_lighting_profile() -> LightingProfile {
    builtin_lighting_profile("lab-neutral").unwrap_or_default()
}

fn default_environment_profile() -> SpaceEnvironmentProfile {
    builtin_space_environment_profile("deep-space-sparse").unwrap_or_default()
}

pub fn merge_lighting_profile(mut base: LightingProfile, other: &LightingProfile) -> LightingProfile {
    base.id = other.id.clone();
    if other.ambient_intensity.is_some() {
        base.ambient_intensity = other.ambient_intensity;
    }
    if other.key_light_intensity.is_some() {
        base.key_light_intensity = other.key_light_intensity;
    }
    if other.fill_light_intensity.is_some() {
        base.fill_light_intensity = other.fill_light_intensity;
    }
    if other.rim_light_intensity.is_some() {
        base.rim_light_intensity = other.rim_light_intensity;
    }
    if other.black_level.is_some() {
        base.black_level = other.black_level;
    }
    if other.shadow_contrast.is_some() {
        base.shadow_contrast = other.shadow_contrast;
    }
    if other.exposure.is_some() {
        base.exposure = other.exposure;
    }
    if other.tonemap.is_some() {
        base.tonemap = other.tonemap;
    }
    if other.gamma.is_some() {
        base.gamma = other.gamma;
    }
    if other.night_glow_scale.is_some() {
        base.night_glow_scale = other.night_glow_scale;
    }
    if other.haze_night_leak.is_some() {
        base.haze_night_leak = other.haze_night_leak;
    }
    if other.specular_floor.is_some() {
        base.specular_floor = other.specular_floor;
    }
    base
}

pub fn merge_space_environment_profile(
    mut base: SpaceEnvironmentProfile,
    other: &SpaceEnvironmentProfile,
) -> SpaceEnvironmentProfile {
    base.id = other.id.clone();
    if other.background_color.is_some() {
        base.background_color = other.background_color.clone();
    }
    if other.background_floor.is_some() {
        base.background_floor = other.background_floor;
    }
    if other.starfield_density.is_some() {
        base.starfield_density = other.starfield_density;
    }
    if other.starfield_brightness.is_some() {
        base.starfield_brightness = other.starfield_brightness;
    }
    if other.starfield_size_min.is_some() {
        base.starfield_size_min = other.starfield_size_min;
    }
    if other.starfield_size_max.is_some() {
        base.starfield_size_max = other.starfield_size_max;
    }
    if other.primary_star_color.is_some() {
        base.primary_star_color = other.primary_star_color.clone();
    }
    if other.primary_star_glare_strength.is_some() {
        base.primary_star_glare_strength = other.primary_star_glare_strength;
    }
    if other.primary_star_glare_width.is_some() {
        base.primary_star_glare_width = other.primary_star_glare_width;
    }
    if other.nebula_strength.is_some() {
        base.nebula_strength = other.nebula_strength;
    }
    if other.dust_band_strength.is_some() {
        base.dust_band_strength = other.dust_band_strength;
    }
    base
}

fn apply_view_overrides(
    lighting: &mut LightingProfile,
    environment: &mut SpaceEnvironmentProfile,
    overrides: &ViewProfileOverrides,
) {
    if overrides.exposure.is_some() {
        lighting.exposure = overrides.exposure;
    }
    if overrides.black_level.is_some() {
        lighting.black_level = overrides.black_level;
    }
    if overrides.background_floor.is_some() {
        environment.background_floor = overrides.background_floor;
    }
    if overrides.starfield_brightness.is_some() {
        environment.starfield_brightness = overrides.starfield_brightness;
    }
    if overrides.primary_star_glare_strength.is_some() {
        environment.primary_star_glare_strength = overrides.primary_star_glare_strength;
    }
}

fn normalize_lighting_profile(profile: &mut LightingProfile) {
    profile.ambient_intensity = profile.ambient_intensity.map(|v| v.clamp(0.0, 1.0));
    profile.key_light_intensity = profile.key_light_intensity.map(|v| v.clamp(0.0, 8.0));
    profile.fill_light_intensity = profile.fill_light_intensity.map(|v| v.clamp(0.0, 4.0));
    profile.rim_light_intensity = profile.rim_light_intensity.map(|v| v.clamp(0.0, 4.0));
    profile.black_level = profile.black_level.map(|v| v.clamp(0.0, 1.0));
    profile.shadow_contrast = profile.shadow_contrast.map(|v| v.clamp(0.25, 4.0));
    profile.exposure = profile.exposure.map(|v| v.clamp(0.0, 8.0));
    profile.gamma = profile.gamma.map(|v| v.clamp(0.1, 4.0));
    profile.night_glow_scale = profile.night_glow_scale.map(|v| v.clamp(0.0, 2.0));
    profile.haze_night_leak = profile.haze_night_leak.map(|v| v.clamp(0.0, 1.0));
    profile.specular_floor = profile.specular_floor.map(|v| v.clamp(0.0, 1.0));
}

fn normalize_space_environment_profile(profile: &mut SpaceEnvironmentProfile) {
    profile.background_floor = profile.background_floor.map(|v| v.clamp(0.0, 1.0));
    profile.starfield_density = profile.starfield_density.map(|v| v.clamp(0.0, 1.0));
    profile.starfield_brightness = profile.starfield_brightness.map(|v| v.clamp(0.0, 1.5));
    profile.starfield_size_min = profile.starfield_size_min.map(|v| v.clamp(0.5, 3.0));
    profile.starfield_size_max = profile.starfield_size_max.map(|v| v.clamp(0.5, 4.0));
    if let (Some(min), Some(max)) = (profile.starfield_size_min, profile.starfield_size_max) {
        profile.starfield_size_max = Some(max.max(min));
    }
    profile.primary_star_glare_strength =
        profile.primary_star_glare_strength.map(|v| v.clamp(0.0, 1.5));
    profile.primary_star_glare_width = profile.primary_star_glare_width.map(|v| v.clamp(0.02, 1.0));
    profile.nebula_strength = profile.nebula_strength.map(|v| v.clamp(0.0, 1.0));
    profile.dust_band_strength = profile.dust_band_strength.map(|v| v.clamp(0.0, 1.0));
}

fn normalize_view_overrides(overrides: &mut ViewProfileOverrides) {
    overrides.exposure = overrides.exposure.map(|v| v.clamp(0.0, 8.0));
    overrides.black_level = overrides.black_level.map(|v| v.clamp(0.0, 1.0));
    overrides.background_floor = overrides.background_floor.map(|v| v.clamp(0.0, 1.0));
    overrides.starfield_brightness = overrides.starfield_brightness.map(|v| v.clamp(0.0, 1.5));
    overrides.primary_star_glare_strength =
        overrides.primary_star_glare_strength.map(|v| v.clamp(0.0, 1.5));
}

pub fn resolve_scene_view_profile(scene: &Scene) -> ResolvedViewProfile {
    let mut lighting = default_lighting_profile();
    let mut environment = default_environment_profile();
    let mut overrides = ViewProfileOverrides::default();

    if let Some(view) = scene.view.as_ref() {
        if let Some(view_id) = view.profile.as_deref() {
            if let Some(view_profile) = view
                .resolved_view_profile_asset
                .clone()
                .or_else(|| builtin_view_profile(view_id))
            {
                if let Some(lighting_id) = view_profile.lighting_profile.as_deref() {
                    if let Some(profile) = view
                        .resolved_lighting_profile_asset
                        .clone()
                        .or_else(|| builtin_lighting_profile(lighting_id))
                    {
                        lighting = merge_lighting_profile(lighting, &profile);
                    }
                }
                if let Some(env_id) = view_profile.space_environment_profile.as_deref() {
                    if let Some(profile) = view
                        .resolved_space_environment_profile_asset
                        .clone()
                        .or_else(|| builtin_space_environment_profile(env_id))
                    {
                        environment = merge_space_environment_profile(environment, &profile);
                    }
                }
                overrides = view_profile.overrides;
            }
        }

        if let Some(lighting_id) = view.lighting_profile.as_deref() {
            if let Some(profile) = view
                .resolved_lighting_profile_asset
                .clone()
                .or_else(|| builtin_lighting_profile(lighting_id))
            {
                lighting = merge_lighting_profile(lighting, &profile);
            }
        }
        if let Some(env_id) = view.space_environment_profile.as_deref() {
            if let Some(profile) = view
                .resolved_space_environment_profile_asset
                .clone()
                .or_else(|| builtin_space_environment_profile(env_id))
            {
                environment = merge_space_environment_profile(environment, &profile);
            }
        }
    }

    apply_view_overrides(&mut lighting, &mut environment, &overrides);
    normalize_view_overrides(&mut overrides);

    if let Some(scene_lighting) = scene.lighting.as_ref() {
        if let Some(ambient_floor) = scene_lighting.ambient_floor {
            lighting.black_level = Some(ambient_floor.clamp(0.0, 1.0));
        }
    }

    normalize_lighting_profile(&mut lighting);
    normalize_space_environment_profile(&mut environment);

    ResolvedViewProfile {
        lighting,
        environment,
        overrides,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{SceneLighting, SceneView};

    fn base_scene() -> Scene {
        Scene {
            id: "test".to_string(),
            title: "Test".to_string(),
            cutscene: false,
            target_fps: None,
            space: Default::default(),
            spatial: Default::default(),
            celestial: Default::default(),
            lighting: None,
            view: None,
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
        }
    }

    #[test]
    fn resolve_scene_view_profile_uses_view_profile_hierarchy() {
        let mut scene = base_scene();
        scene.view = Some(SceneView {
            profile: Some("orbit-realistic".to_string()),
            lighting_profile: None,
            space_environment_profile: None,
            resolved_view_profile_asset: None,
            resolved_lighting_profile_asset: None,
            resolved_space_environment_profile_asset: None,
        });

        let resolved = resolve_scene_view_profile(&scene);

        assert_eq!(resolved.lighting.id, "space-hard-vacuum");
        assert_eq!(resolved.environment.id, "deep-space-sparse");
        assert_eq!(resolved.lighting.black_level, Some(0.0));
        assert_eq!(resolved.environment.starfield_brightness, Some(0.7));
    }

    #[test]
    fn explicit_scene_refs_override_view_profile_inputs() {
        let mut scene = base_scene();
        scene.view = Some(SceneView {
            profile: Some("orbit-realistic".to_string()),
            lighting_profile: Some("space-cinematic-soft".to_string()),
            space_environment_profile: Some("deep-space-rich".to_string()),
            resolved_view_profile_asset: None,
            resolved_lighting_profile_asset: None,
            resolved_space_environment_profile_asset: None,
        });

        let resolved = resolve_scene_view_profile(&scene);

        assert_eq!(resolved.lighting.id, "space-hard-vacuum");
        assert_eq!(resolved.environment.id, "deep-space-sparse");
        assert_eq!(resolved.lighting.fill_light_intensity, Some(0.18));
        assert_eq!(resolved.environment.nebula_strength, Some(0.04));
    }

    #[test]
    fn scene_lighting_ambient_floor_overrides_black_level() {
        let mut scene = base_scene();
        scene.view = Some(SceneView {
            profile: Some("deep-space-harsh".to_string()),
            lighting_profile: None,
            space_environment_profile: None,
            resolved_view_profile_asset: None,
            resolved_lighting_profile_asset: None,
            resolved_space_environment_profile_asset: None,
        });
        scene.lighting = Some(SceneLighting {
            ambient_floor: Some(0.14),
        });

        let resolved = resolve_scene_view_profile(&scene);

        assert_eq!(resolved.lighting.black_level, Some(0.14));
    }

    #[test]
    fn embedded_asset_profiles_override_builtin_lookup() {
        let mut scene = base_scene();
        scene.view = Some(SceneView {
            profile: Some("missing-view".to_string()),
            lighting_profile: None,
            space_environment_profile: None,
            resolved_view_profile_asset: Some(ViewProfile {
                id: "mod-view".to_string(),
                lighting_profile: Some("mod-light".to_string()),
                space_environment_profile: Some("mod-space".to_string()),
                overrides: ViewProfileOverrides {
                    exposure: Some(0.77),
                    black_level: Some(0.01),
                    background_floor: Some(0.02),
                    starfield_brightness: Some(0.33),
                    primary_star_glare_strength: Some(0.44),
                },
            }),
            resolved_lighting_profile_asset: Some(LightingProfile {
                id: "mod-light".to_string(),
                black_level: Some(0.05),
                exposure: Some(0.88),
                gamma: Some(2.0),
                ..Default::default()
            }),
            resolved_space_environment_profile_asset: Some(SpaceEnvironmentProfile {
                id: "mod-space".to_string(),
                background_color: Some("#010203".to_string()),
                starfield_brightness: Some(0.22),
                ..Default::default()
            }),
        });

        let resolved = resolve_scene_view_profile(&scene);

        assert_eq!(resolved.lighting.id, "mod-light");
        assert_eq!(resolved.environment.id, "mod-space");
        assert_eq!(resolved.lighting.exposure, Some(0.77));
        assert_eq!(resolved.environment.starfield_brightness, Some(0.33));
        assert_eq!(
            resolved.environment.background_color.as_deref(),
            Some("#010203")
        );
    }

    #[test]
    fn resolved_view_profiles_clamp_out_of_range_values() {
        let mut scene = base_scene();
        scene.view = Some(SceneView {
            profile: None,
            lighting_profile: Some("mod-light".to_string()),
            space_environment_profile: Some("mod-space".to_string()),
            resolved_view_profile_asset: None,
            resolved_lighting_profile_asset: Some(LightingProfile {
                id: "mod-light".to_string(),
                black_level: Some(-0.5),
                exposure: Some(99.0),
                gamma: Some(9.0),
                night_glow_scale: Some(4.0),
                haze_night_leak: Some(-1.0),
                ..Default::default()
            }),
            resolved_space_environment_profile_asset: Some(SpaceEnvironmentProfile {
                id: "mod-space".to_string(),
                starfield_density: Some(2.0),
                starfield_brightness: Some(-0.2),
                starfield_size_min: Some(3.5),
                starfield_size_max: Some(1.0),
                primary_star_glare_strength: Some(4.0),
                primary_star_glare_width: Some(0.0),
                ..Default::default()
            }),
        });

        let resolved = resolve_scene_view_profile(&scene);
        assert_eq!(resolved.lighting.black_level, Some(0.0));
        assert_eq!(resolved.lighting.exposure, Some(8.0));
        assert_eq!(resolved.lighting.gamma, Some(4.0));
        assert_eq!(resolved.lighting.night_glow_scale, Some(2.0));
        assert_eq!(resolved.lighting.haze_night_leak, Some(0.0));
        assert_eq!(resolved.environment.starfield_density, Some(1.0));
        assert_eq!(resolved.environment.starfield_brightness, Some(0.0));
        assert_eq!(resolved.environment.starfield_size_min, Some(3.0));
        assert_eq!(resolved.environment.starfield_size_max, Some(3.0));
        assert_eq!(resolved.environment.primary_star_glare_strength, Some(1.5));
        assert_eq!(resolved.environment.primary_star_glare_width, Some(0.02));
    }
}

use crate::scene_pipeline::ScenePreparationStep;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_behavior::catalog::{ModCatalogs, PlayerPreset};
use engine_core::scene::model::{
    CelestialClockSource, CelestialFrame, CelestialScope, SceneSpace, SceneWorldModel,
};
use engine_core::scene::Scene;
use engine_game::components::{
    AngularMotor3D, AngularMotorMode, Assembly3D, AtmosphereAffected2D, AttachmentBundle3D,
    BootstrapAssembly3D, CharacterMotor3D, CharacterUpMode, ControlBundle3D, GravityAffected2D,
    GravityMode2D, LinearMotor3D, MotorBundle3D, MotorSpace, ReferenceFrameBinding3D,
    ReferenceFrameMode, SpatialBundle3D,
};
use engine_game::{GameplayWorld, SpatialKind};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedSceneCameraPreset {
    pub preset_id: String,
    pub source: SceneCameraPresetSource,
    pub controller_kind: String,
    pub target: Option<String>,
    pub config: std::collections::HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SceneCameraPresetSource {
    #[default]
    Catalog,
    BuiltInCompat,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppliedSceneUiPreset {
    pub preset_id: String,
    pub layout: Option<String>,
    pub config: std::collections::HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppliedScenePlayerPreset {
    pub preset_id: String,
    pub controlled: bool,
    pub has_bootstrap_assembly: bool,
    pub input_profile: Option<String>,
    pub controller_type: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedScenePlayerPreset {
    pub preset_id: String,
    pub controlled: bool,
    pub has_bootstrap_assembly: bool,
    pub input_profile: Option<String>,
    pub controller_type: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SceneRuntimePresetSource {
    #[default]
    Catalog,
    BuiltInCompat,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedSceneSpawnPreset {
    pub preset_id: String,
    pub source: SceneRuntimePresetSource,
    pub spawn_type: String,
    pub config: std::collections::HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedSceneGravityPreset {
    pub preset_id: String,
    pub source: SceneRuntimePresetSource,
    pub gravity_type: String,
    pub config: std::collections::HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedSceneSurfacePreset {
    pub preset_id: String,
    pub source: SceneRuntimePresetSource,
    pub surface_type: String,
    pub config: std::collections::HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneBootstrapCelestialBinding {
    pub scope: CelestialScope,
    pub region: Option<String>,
    pub system: Option<String>,
    pub focus_body: Option<String>,
    pub focus_site: Option<String>,
    pub frame: CelestialFrame,
    pub clock_source: CelestialClockSource,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SceneBootstrapDomainFlags {
    pub uses_2d_render_space: bool,
    pub uses_3d_render_space: bool,
    pub is_planar_2d: bool,
    pub is_euclidean_3d: bool,
    pub is_celestial_3d: bool,
    pub has_celestial_binding: bool,
}

impl SceneBootstrapDomainFlags {
    fn from_scene(scene: &Scene) -> Self {
        Self {
            uses_2d_render_space: matches!(scene.space, SceneSpace::TwoD),
            uses_3d_render_space: matches!(scene.space, SceneSpace::ThreeD),
            is_planar_2d: matches!(scene.world_model, SceneWorldModel::Planar2D),
            is_euclidean_3d: matches!(scene.world_model, SceneWorldModel::Euclidean3D),
            is_celestial_3d: matches!(scene.world_model, SceneWorldModel::Celestial3D),
            has_celestial_binding: scene.world_model == SceneWorldModel::Celestial3D
                || scene.celestial != Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneBootstrapSelectionHints {
    pub camera: Option<String>,
    pub player: Option<String>,
    pub ui: Option<String>,
    pub spawn: Option<String>,
    pub gravity: Option<String>,
    pub surface: Option<String>,
}

fn celestial_scope_label(scope: CelestialScope) -> &'static str {
    match scope {
        CelestialScope::Local => "local",
        CelestialScope::System => "system",
        CelestialScope::Region => "region",
    }
}

fn celestial_frame_label(frame: CelestialFrame) -> &'static str {
    match frame {
        CelestialFrame::FocusRelative => "focus-relative",
        CelestialFrame::Barycentric => "barycentric",
        CelestialFrame::SurfaceLocal => "surface-local",
    }
}

fn celestial_clock_source_label(clock_source: CelestialClockSource) -> &'static str {
    match clock_source {
        CelestialClockSource::Scene => "scene",
        CelestialClockSource::Campaign => "campaign",
        CelestialClockSource::Fixed => "fixed",
    }
}

fn reference_frame_mode_label(mode: ReferenceFrameMode) -> &'static str {
    match mode {
        ReferenceFrameMode::World => "world",
        ReferenceFrameMode::ParentEntity => "parent-entity",
        ReferenceFrameMode::Orbital => "orbital",
        ReferenceFrameMode::LocalHorizon => "local-horizon",
        ReferenceFrameMode::CelestialBody => "celestial-body",
    }
}

fn reference_frame_binding_summary(binding: &ReferenceFrameBinding3D) -> String {
    format!(
        "{}(body={})",
        reference_frame_mode_label(binding.mode),
        binding.body_id.as_deref().unwrap_or("-"),
    )
}

fn gravity_summary(gravity: &GravityAffected2D) -> String {
    format!("point(body={})", gravity.body_id.as_deref().unwrap_or("-"),)
}

fn resolve_catalog_camera_preset(
    catalogs: Option<&ModCatalogs>,
    camera_preset: Option<&str>,
) -> Option<ResolvedSceneCameraPreset> {
    let preset_id = camera_preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    let preset = catalogs?.presets.camera(preset_id)?;
    let controller_kind = preset.controller_kind.as_deref()?.trim();
    if controller_kind.is_empty() {
        return None;
    }
    Some(ResolvedSceneCameraPreset {
        preset_id: preset_id.to_string(),
        source: SceneCameraPresetSource::Catalog,
        controller_kind: controller_kind.to_string(),
        target: preset.target.clone(),
        config: preset.config.clone(),
    })
}

fn resolve_builtin_camera_preset(camera_preset: Option<&str>) -> Option<ResolvedSceneCameraPreset> {
    let preset_id = camera_preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    let controller_kind = match preset_id.to_ascii_lowercase().as_str() {
        "orbit-camera" => "orbit-camera",
        "free-look-camera" => "free-look-camera",
        "surface-free-look" => "surface-free-look",
        _ => return None,
    };
    Some(ResolvedSceneCameraPreset {
        preset_id: preset_id.to_string(),
        source: SceneCameraPresetSource::BuiltInCompat,
        controller_kind: controller_kind.to_string(),
        target: None,
        config: Default::default(),
    })
}

fn resolve_camera_preset(
    catalogs: Option<&ModCatalogs>,
    camera_preset: Option<&str>,
) -> Option<ResolvedSceneCameraPreset> {
    resolve_catalog_camera_preset(catalogs, camera_preset)
        .or_else(|| resolve_builtin_camera_preset(camera_preset))
}

fn resolve_catalog_player_preset(
    catalogs: Option<&ModCatalogs>,
    player_preset: Option<&str>,
) -> Option<ResolvedScenePlayerPreset> {
    let preset_id = player_preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    let preset = catalogs?.presets.player(preset_id)?;
    Some(ResolvedScenePlayerPreset {
        preset_id: preset_id.to_string(),
        controlled: preset
            .config
            .get("controlled")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        has_bootstrap_assembly: bootstrap_assembly3d_from_player_preset(preset).is_some(),
        input_profile: preset.input_profile.clone(),
        controller_type: preset
            .controller
            .as_ref()
            .map(|controller| controller.controller_type.clone()),
    })
}

impl From<&ResolvedScenePlayerPreset> for AppliedScenePlayerPreset {
    fn from(value: &ResolvedScenePlayerPreset) -> Self {
        Self {
            preset_id: value.preset_id.clone(),
            controlled: value.controlled,
            has_bootstrap_assembly: value.has_bootstrap_assembly,
            input_profile: value.input_profile.clone(),
            controller_type: value.controller_type.clone(),
        }
    }
}

fn camera_selection_hint(
    camera_preset: Option<&str>,
    resolved_camera_preset: Option<&ResolvedSceneCameraPreset>,
) -> Option<String> {
    if let Some(resolved) = resolved_camera_preset {
        return Some(match resolved.source {
            SceneCameraPresetSource::Catalog => format!(
                "catalog camera preset `{}` -> {}(target={})",
                resolved.preset_id,
                resolved.controller_kind,
                resolved.target.as_deref().unwrap_or("-"),
            ),
            SceneCameraPresetSource::BuiltInCompat => format!(
                "built-in camera preset `{}` -> {}(target={})",
                resolved.preset_id,
                resolved.controller_kind,
                resolved.target.as_deref().unwrap_or("-"),
            ),
        });
    }

    let preset = camera_preset?.trim();
    if preset.is_empty() {
        return None;
    }

    let lower = preset.to_ascii_lowercase();
    let route = match lower.as_str() {
        "obj-viewer" => "legacy input.obj-viewer compatibility route".to_string(),
        "orbit-camera" => "legacy input.orbit-camera compatibility route".to_string(),
        "free-look-camera" => "legacy input.free-look-camera compatibility route".to_string(),
        "surface-free-look" => {
            "legacy input.free-look-camera compatibility route (surface-mode)".to_string()
        }
        _ => format!("pending runtime preset registry for `{preset}`"),
    };
    Some(route)
}

fn player_selection_hint(
    player_preset: Option<&str>,
    resolved_player_preset: Option<&ResolvedScenePlayerPreset>,
    _catalog_lookup_attempted: bool,
) -> Option<String> {
    if let Some(resolved) = resolved_player_preset {
        let mut actions = Vec::new();
        if resolved.controlled {
            actions.push("controlled gameplay entity".to_string());
        }
        if resolved.has_bootstrap_assembly {
            actions.push("bootstrap assembly".to_string());
        }
        if let Some(controller_type) = resolved.controller_type.as_deref() {
            actions.push(format!("controller={controller_type}"));
        }
        if let Some(input_profile) = resolved.input_profile.as_deref() {
            actions.push(format!("input={input_profile}"));
        }
        let action_summary = if actions.is_empty() {
            "metadata only".to_string()
        } else {
            actions.join(" + ")
        };
        return Some(format!(
            "catalog player preset `{}` -> {action_summary}",
            resolved.preset_id
        ));
    }

    player_preset.map(|preset| format!("pending runtime preset registry for `{preset}`"))
}

fn camera_preset_needs_runtime_registry(camera_preset: &str) -> bool {
    let lower = camera_preset.trim().to_ascii_lowercase();
    !matches!(
        lower.as_str(),
        "obj-viewer" | "orbit-camera" | "free-look-camera" | "surface-free-look"
    )
}

fn builtin_spawn_preset(preset: Option<&str>) -> Option<&'static str> {
    match preset
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "planetary-surface" | "planetary_surface" => Some("planetary-surface"),
        _ => None,
    }
}

fn builtin_gravity_preset(preset: Option<&str>) -> Option<&'static str> {
    match preset
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "radial-body" | "radial_body" => Some("radial-body"),
        _ => None,
    }
}

fn builtin_surface_preset(preset: Option<&str>) -> Option<&'static str> {
    match preset
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "local-horizon" | "local_horizon" => Some("local-horizon"),
        "planetary-surface" | "planetary_surface" => Some("planetary-surface"),
        _ => None,
    }
}

fn resolve_builtin_spawn_preset(preset: Option<&str>) -> Option<ResolvedSceneSpawnPreset> {
    let preset_id = preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    let spawn_type = builtin_spawn_preset(Some(preset_id))?;
    Some(ResolvedSceneSpawnPreset {
        preset_id: preset_id.to_string(),
        source: SceneRuntimePresetSource::BuiltInCompat,
        spawn_type: spawn_type.to_string(),
        config: Default::default(),
    })
}

fn resolve_catalog_spawn_preset(
    catalogs: Option<&ModCatalogs>,
    preset: Option<&str>,
) -> Option<ResolvedSceneSpawnPreset> {
    let preset_id = preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    let preset = catalogs?.presets.spawn(preset_id)?;
    let spawn_type = builtin_spawn_preset(preset.spawn_type.as_deref())?;
    Some(ResolvedSceneSpawnPreset {
        preset_id: preset_id.to_string(),
        source: SceneRuntimePresetSource::Catalog,
        spawn_type: spawn_type.to_string(),
        config: preset.config.clone(),
    })
}

fn resolve_spawn_preset(
    catalogs: Option<&ModCatalogs>,
    preset: Option<&str>,
) -> Option<ResolvedSceneSpawnPreset> {
    if has_catalog_spawn_preset(catalogs, preset) {
        resolve_catalog_spawn_preset(catalogs, preset)
    } else {
        resolve_builtin_spawn_preset(preset)
    }
}

fn resolve_builtin_gravity_preset(preset: Option<&str>) -> Option<ResolvedSceneGravityPreset> {
    let preset_id = preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    let gravity_type = builtin_gravity_preset(Some(preset_id))?;
    Some(ResolvedSceneGravityPreset {
        preset_id: preset_id.to_string(),
        source: SceneRuntimePresetSource::BuiltInCompat,
        gravity_type: gravity_type.to_string(),
        config: Default::default(),
    })
}

fn resolve_catalog_gravity_preset(
    catalogs: Option<&ModCatalogs>,
    preset: Option<&str>,
) -> Option<ResolvedSceneGravityPreset> {
    let preset_id = preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    let preset = catalogs?.presets.gravity(preset_id)?;
    let gravity_type = builtin_gravity_preset(preset.gravity_type.as_deref())?;
    Some(ResolvedSceneGravityPreset {
        preset_id: preset_id.to_string(),
        source: SceneRuntimePresetSource::Catalog,
        gravity_type: gravity_type.to_string(),
        config: preset.config.clone(),
    })
}

fn resolve_gravity_preset(
    catalogs: Option<&ModCatalogs>,
    preset: Option<&str>,
) -> Option<ResolvedSceneGravityPreset> {
    if has_catalog_gravity_preset(catalogs, preset) {
        resolve_catalog_gravity_preset(catalogs, preset)
    } else {
        resolve_builtin_gravity_preset(preset)
    }
}

fn resolve_builtin_surface_preset(preset: Option<&str>) -> Option<ResolvedSceneSurfacePreset> {
    let preset_id = preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    let surface_type = builtin_surface_preset(Some(preset_id))?;
    Some(ResolvedSceneSurfacePreset {
        preset_id: preset_id.to_string(),
        source: SceneRuntimePresetSource::BuiltInCompat,
        surface_type: surface_type.to_string(),
        config: Default::default(),
    })
}

fn resolve_catalog_surface_preset(
    catalogs: Option<&ModCatalogs>,
    preset: Option<&str>,
) -> Option<ResolvedSceneSurfacePreset> {
    let preset_id = preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    let preset = catalogs?.presets.surface(preset_id)?;
    let surface_type = builtin_surface_preset(preset.surface_type.as_deref())?;
    Some(ResolvedSceneSurfacePreset {
        preset_id: preset_id.to_string(),
        source: SceneRuntimePresetSource::Catalog,
        surface_type: surface_type.to_string(),
        config: preset.config.clone(),
    })
}

fn resolve_surface_preset(
    catalogs: Option<&ModCatalogs>,
    preset: Option<&str>,
) -> Option<ResolvedSceneSurfacePreset> {
    if has_catalog_surface_preset(catalogs, preset) {
        resolve_catalog_surface_preset(catalogs, preset)
    } else {
        resolve_builtin_surface_preset(preset)
    }
}

fn has_catalog_spawn_preset(catalogs: Option<&ModCatalogs>, preset: Option<&str>) -> bool {
    let Some(preset_id) = preset
        .map(str::trim)
        .filter(|preset_id| !preset_id.is_empty())
    else {
        return false;
    };
    catalogs
        .and_then(|catalogs| catalogs.presets.spawn(preset_id))
        .is_some()
}

fn has_catalog_gravity_preset(catalogs: Option<&ModCatalogs>, preset: Option<&str>) -> bool {
    let Some(preset_id) = preset
        .map(str::trim)
        .filter(|preset_id| !preset_id.is_empty())
    else {
        return false;
    };
    catalogs
        .and_then(|catalogs| catalogs.presets.gravity(preset_id))
        .is_some()
}

fn has_catalog_surface_preset(catalogs: Option<&ModCatalogs>, preset: Option<&str>) -> bool {
    let Some(preset_id) = preset
        .map(str::trim)
        .filter(|preset_id| !preset_id.is_empty())
    else {
        return false;
    };
    catalogs
        .and_then(|catalogs| catalogs.presets.surface(preset_id))
        .is_some()
}

fn spawn_selection_hint(
    preset: Option<&str>,
    catalogs: Option<&ModCatalogs>,
    resolved_spawn_preset: Option<&ResolvedSceneSpawnPreset>,
    default_reference_frame: Option<&ReferenceFrameBinding3D>,
) -> Option<String> {
    let preset = preset?.trim();
    if preset.is_empty() {
        return None;
    }
    if let Some(resolved) = resolved_spawn_preset {
        let source = match resolved.source {
            SceneRuntimePresetSource::Catalog => "catalog",
            SceneRuntimePresetSource::BuiltInCompat => "built-in",
        };
        let summary = format!(
            "{source} spawn preset `{}` -> {}",
            resolved.preset_id, resolved.spawn_type
        );
        return Some(
            default_reference_frame
                .map(reference_frame_binding_summary)
                .map(|binding| format!("{summary} + {binding}"))
                .unwrap_or_else(|| {
                    format!("{summary} (pending runtime bootstrap resolver for `{preset}`)")
                }),
        );
    }
    if has_catalog_spawn_preset(catalogs, Some(preset)) {
        return Some(format!(
            "catalog spawn preset `{preset}` is pending runtime bootstrap resolver"
        ));
    }
    Some(format!("pending runtime preset registry for `{preset}`"))
}

fn gravity_selection_hint(
    preset: Option<&str>,
    catalogs: Option<&ModCatalogs>,
    resolved_gravity_preset: Option<&ResolvedSceneGravityPreset>,
    default_gravity: Option<&GravityAffected2D>,
) -> Option<String> {
    let preset = preset?.trim();
    if preset.is_empty() {
        return None;
    }
    if let Some(resolved) = resolved_gravity_preset {
        let source = match resolved.source {
            SceneRuntimePresetSource::Catalog => "catalog",
            SceneRuntimePresetSource::BuiltInCompat => "built-in",
        };
        let summary = format!(
            "{source} gravity preset `{}` -> {}",
            resolved.preset_id, resolved.gravity_type
        );
        return Some(
            default_gravity
                .map(gravity_summary)
                .map(|binding| format!("{summary} + {binding}"))
                .unwrap_or_else(|| {
                    format!("{summary} (pending runtime bootstrap resolver for `{preset}`)")
                }),
        );
    }
    if has_catalog_gravity_preset(catalogs, Some(preset)) {
        return Some(format!(
            "catalog gravity preset `{preset}` is pending runtime bootstrap resolver"
        ));
    }
    Some(format!("pending runtime preset registry for `{preset}`"))
}

fn surface_selection_hint(
    preset: Option<&str>,
    catalogs: Option<&ModCatalogs>,
    resolved_surface_preset: Option<&ResolvedSceneSurfacePreset>,
    default_reference_frame: Option<&ReferenceFrameBinding3D>,
) -> Option<String> {
    let preset = preset?.trim();
    if preset.is_empty() {
        return None;
    }
    if let Some(resolved) = resolved_surface_preset {
        let source = match resolved.source {
            SceneRuntimePresetSource::Catalog => "catalog",
            SceneRuntimePresetSource::BuiltInCompat => "built-in",
        };
        let summary = format!(
            "{source} surface preset `{}` -> {}",
            resolved.preset_id, resolved.surface_type
        );
        return Some(
            default_reference_frame
                .map(reference_frame_binding_summary)
                .map(|binding| format!("{summary} + {binding}"))
                .unwrap_or_else(|| {
                    format!("{summary} (pending runtime bootstrap resolver for `{preset}`)")
                }),
        );
    }
    if has_catalog_surface_preset(catalogs, Some(preset)) {
        return Some(format!(
            "catalog surface preset `{preset}` is pending runtime bootstrap resolver"
        ));
    }
    Some(format!("pending runtime preset registry for `{preset}`"))
}

fn resolve_ui_selection_hint(
    catalogs: Option<&ModCatalogs>,
    ui_preset: Option<&str>,
) -> Option<String> {
    let preset_id = ui_preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    if let Some(preset) = catalogs.and_then(|catalogs| catalogs.presets.ui(preset_id)) {
        return Some(format!(
            "catalog ui preset `{}` -> layout={}",
            preset_id,
            preset.layout.as_deref().unwrap_or("-"),
        ));
    }
    Some(format!("pending runtime preset registry for `{preset_id}`"))
}

fn resolve_ui_preset_resource(
    catalogs: Option<&ModCatalogs>,
    ui_preset: Option<&str>,
) -> Option<AppliedSceneUiPreset> {
    let preset_id = ui_preset?.trim();
    if preset_id.is_empty() {
        return None;
    }
    let preset = catalogs?.presets.ui(preset_id)?;
    Some(AppliedSceneUiPreset {
        preset_id: preset_id.to_string(),
        layout: preset.layout.clone(),
        config: preset.config.clone(),
    })
}

fn motor_space_from_str(value: Option<&str>) -> MotorSpace {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "world" => MotorSpace::World,
        _ => MotorSpace::Local,
    }
}

fn angular_motor_mode_from_str(value: Option<&str>) -> AngularMotorMode {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "torque" => AngularMotorMode::Torque,
        _ => AngularMotorMode::Rate,
    }
}

fn character_up_mode_from_str(value: Option<&str>) -> CharacterUpMode {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "radial" | "local-horizon" | "local_horizon" | "surface-normal" | "surface_normal" => {
            CharacterUpMode::SurfaceNormal
        }
        "reference-frame" | "reference_frame" => CharacterUpMode::ReferenceFrameUp,
        _ => CharacterUpMode::WorldUp,
    }
}

fn bootstrap_assembly3d_from_player_preset(
    player_preset: &PlayerPreset,
) -> Option<BootstrapAssembly3D> {
    let components = player_preset.components.as_ref()?;
    let assembly = Assembly3D {
        spatial: SpatialBundle3D::default(),
        control: ControlBundle3D {
            control_intent: None,
            reference_frame: components.reference_frame.as_ref().map(|frame| {
                ReferenceFrameBinding3D {
                    mode: match frame
                        .mode
                        .as_deref()
                        .unwrap_or_default()
                        .trim()
                        .to_ascii_lowercase()
                        .as_str()
                    {
                        "parent-entity" | "parent_entity" => ReferenceFrameMode::ParentEntity,
                        "orbital" => ReferenceFrameMode::Orbital,
                        "local-horizon" | "local_horizon" => ReferenceFrameMode::LocalHorizon,
                        "celestial-body" | "celestial_body" => ReferenceFrameMode::CelestialBody,
                        _ => ReferenceFrameMode::World,
                    },
                    entity_id: frame.entity_id,
                    body_id: frame.body_id.clone(),
                    inherit_linear_velocity: frame.inherit_linear_velocity.unwrap_or(false),
                    inherit_angular_velocity: frame.inherit_angular_velocity.unwrap_or(false),
                }
            }),
            reference_frame_state: None,
        },
        attachments: AttachmentBundle3D {
            follow_anchor: components.follow_anchor_3d.as_ref().map(|follow| {
                engine_game::components::FollowAnchor3D {
                    local_offset: follow
                        .local_offset
                        .unwrap_or([0.0, 0.0, 0.0])
                        .map(|value| value as f32),
                    inherit_orientation: follow.inherit_orientation.unwrap_or(true),
                }
            }),
        },
        motors: MotorBundle3D {
            linear_motor: components
                .linear_motor_3d
                .as_ref()
                .map(|motor| LinearMotor3D {
                    space: motor_space_from_str(motor.space.as_deref()),
                    accel: motor.accel.unwrap_or(0.0) as f32,
                    decel: motor.decel.unwrap_or(0.0) as f32,
                    max_speed: motor.max_speed.unwrap_or(0.0) as f32,
                    boost_scale: motor.boost_scale.unwrap_or(1.0) as f32,
                    air_control: motor.air_control.unwrap_or(1.0) as f32,
                }),
            angular_motor: components
                .angular_motor_3d
                .as_ref()
                .map(|motor| AngularMotor3D {
                    mode: angular_motor_mode_from_str(motor.mode.as_deref()),
                    yaw_rate: motor.yaw_rate.unwrap_or(0.0) as f32,
                    pitch_rate: motor.pitch_rate.unwrap_or(0.0) as f32,
                    roll_rate: motor.roll_rate.unwrap_or(0.0) as f32,
                    torque_scale: motor.torque_scale.unwrap_or(1.0) as f32,
                    look_sensitivity: motor.look_sensitivity.unwrap_or(1.0) as f32,
                }),
            character_motor: components
                .character_motor_3d
                .as_ref()
                .map(|motor| CharacterMotor3D {
                    up_mode: character_up_mode_from_str(motor.up_mode.as_deref()),
                    jump_speed: motor.jump_speed.unwrap_or(0.0) as f32,
                    stick_to_ground: motor.stick_to_ground.unwrap_or(false),
                    max_slope_deg: motor.max_slope_deg.unwrap_or(45.0) as f32,
                }),
            flight_motor: components.flight_motor_3d.as_ref().map(|motor| {
                engine_game::components::FlightMotor3D {
                    translational_dofs: motor.translational_dofs.unwrap_or([true, true, true]),
                    rotational_dofs: motor.rotational_dofs.unwrap_or([true, true, true]),
                    horizon_lock_strength: motor.horizon_lock_strength.unwrap_or(0.0) as f32,
                }
            }),
        },
    };

    let controlled = player_preset
        .config
        .get("controlled")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    let bootstrap = BootstrapAssembly3D {
        assembly,
        controlled,
        owner_id: None,
        inherit_owner_lifecycle: false,
        lifecycle: None,
    };

    (!bootstrap.is_empty()).then_some(bootstrap)
}

fn bootstrap_selection_hints(
    authored: &SceneSimulationBootstrap,
    catalogs: Option<&ModCatalogs>,
    resolved_camera_preset: Option<&ResolvedSceneCameraPreset>,
    resolved_player_preset: Option<&ResolvedScenePlayerPreset>,
    resolved_ui_preset: Option<&AppliedSceneUiPreset>,
    resolved_spawn_preset: Option<&ResolvedSceneSpawnPreset>,
    resolved_gravity_preset: Option<&ResolvedSceneGravityPreset>,
    resolved_surface_preset: Option<&ResolvedSceneSurfacePreset>,
    catalog_lookup_attempted: bool,
    default_reference_frame: Option<&ReferenceFrameBinding3D>,
    default_gravity: Option<&GravityAffected2D>,
) -> SceneBootstrapSelectionHints {
    SceneBootstrapSelectionHints {
        camera: camera_selection_hint(authored.camera_preset.as_deref(), resolved_camera_preset),
        player: player_selection_hint(
            authored.player_preset.as_deref(),
            resolved_player_preset,
            catalog_lookup_attempted,
        ),
        ui: resolved_ui_preset
            .map(|preset| {
                format!(
                    "catalog ui preset `{}` -> layout={}",
                    preset.preset_id,
                    preset.layout.as_deref().unwrap_or("-"),
                )
            })
            .or_else(|| resolve_ui_selection_hint(None, authored.ui_preset.as_deref())),
        spawn: spawn_selection_hint(
            authored.spawn_preset.as_deref(),
            catalogs,
            resolved_spawn_preset,
            default_reference_frame,
        ),
        gravity: gravity_selection_hint(
            authored.gravity_preset.as_deref(),
            catalogs,
            resolved_gravity_preset,
            default_gravity,
        ),
        surface: surface_selection_hint(
            authored.surface_preset.as_deref(),
            catalogs,
            resolved_surface_preset,
            default_reference_frame,
        ),
    }
}

fn player_preset_requests_controlled_entity(
    authored: &SceneSimulationBootstrap,
    resolved_player_preset: Option<&ResolvedScenePlayerPreset>,
) -> bool {
    let _ = authored;
    resolved_player_preset
        .map(|preset| preset.controlled)
        .unwrap_or(false)
}

fn selection_hints_summary(hints: &SceneBootstrapSelectionHints) -> String {
    format!(
        "selection[camera={},player={},ui={},spawn={},gravity={},surface={}]",
        hints.camera.as_deref().unwrap_or("-"),
        hints.player.as_deref().unwrap_or("-"),
        hints.ui.as_deref().unwrap_or("-"),
        hints.spawn.as_deref().unwrap_or("-"),
        hints.gravity.as_deref().unwrap_or("-"),
        hints.surface.as_deref().unwrap_or("-"),
    )
}

fn push_diagnostic(diagnostics: &mut Vec<String>, message: impl Into<String>) {
    let message = message.into();
    if !diagnostics.iter().any(|existing| existing == &message) {
        diagnostics.push(message);
    }
}

fn celestial_binding_summary(binding: &SceneBootstrapCelestialBinding) -> String {
    format!(
        "celestial[scope={},region={},system={},focus_body={},focus_site={},frame={},clock={}]",
        celestial_scope_label(binding.scope),
        binding.region.as_deref().unwrap_or("-"),
        binding.system.as_deref().unwrap_or("-"),
        binding.focus_body.as_deref().unwrap_or("-"),
        binding.focus_site.as_deref().unwrap_or("-"),
        celestial_frame_label(binding.frame),
        celestial_clock_source_label(binding.clock_source),
    )
}

fn scene_bootstrap_target_detail(
    source: SceneBootstrapTargetSource,
    target_entity: Option<u64>,
    ids_3d: &[u64],
) -> String {
    match source {
        SceneBootstrapTargetSource::NoneRequired => {
            "scene does not request a gameplay target".to_string()
        }
        SceneBootstrapTargetSource::ControlledEntity => target_entity
            .map(|id| format!("using existing controlled 3D entity {id}"))
            .unwrap_or_else(|| "using existing controlled 3D entity".to_string()),
        SceneBootstrapTargetSource::Sole3dEntity => target_entity
            .map(|id| format!("selected sole 3D gameplay entity {id}"))
            .unwrap_or_else(|| "selected sole 3D gameplay entity".to_string()),
        SceneBootstrapTargetSource::DeferredNo3dEntity => {
            "waiting for at least one 3D gameplay entity before controller/default bindings can apply"
                .to_string()
        }
        SceneBootstrapTargetSource::DeferredAmbiguous3dEntities => {
            let ids = ids_3d
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            if ids.is_empty() {
                "waiting for a single 3D gameplay entity".to_string()
            } else {
                format!("waiting for a single 3D gameplay entity; found ids=[{ids}]")
            }
        }
        SceneBootstrapTargetSource::DeferredNoGameplayWorld => {
            "waiting for gameplay world registration before controller/default bindings can apply"
                .to_string()
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneSimulationBootstrap {
    pub scene_id: String,
    pub render_space: SceneSpace,
    pub world_model: SceneWorldModel,
    pub camera_preset: Option<String>,
    pub player_preset: Option<String>,
    pub ui_preset: Option<String>,
    pub spawn_preset: Option<String>,
    pub gravity_preset: Option<String>,
    pub surface_preset: Option<String>,
    pub celestial: Option<SceneBootstrapCelestialBinding>,
    pub resolved_clock_source: Option<CelestialClockSource>,
    pub flags: SceneBootstrapDomainFlags,
}

impl SceneSimulationBootstrap {
    pub fn from_scene(scene: &Scene) -> Self {
        debug_assert!(
            scene.world_model == SceneWorldModel::Celestial3D
                || scene.celestial == Default::default(),
            "celestial bindings require `world-model: celestial-3d`"
        );
        let flags = SceneBootstrapDomainFlags::from_scene(scene);
        let celestial = flags
            .has_celestial_binding
            .then(|| SceneBootstrapCelestialBinding {
                scope: scene.celestial.scope,
                region: scene.celestial.region.clone(),
                system: scene.celestial.system.clone(),
                focus_body: scene.celestial.focus_body.clone(),
                focus_site: scene.celestial.focus_site.clone(),
                frame: scene.celestial.frame,
                clock_source: scene.celestial.clock_source,
            });
        Self {
            scene_id: scene.id.clone(),
            render_space: scene.space,
            world_model: scene.world_model,
            camera_preset: scene.controller_defaults.camera_preset.clone(),
            player_preset: scene.controller_defaults.player_preset.clone(),
            ui_preset: scene.controller_defaults.ui_preset.clone(),
            spawn_preset: scene.controller_defaults.spawn_preset.clone(),
            gravity_preset: scene.controller_defaults.gravity_preset.clone(),
            surface_preset: scene.controller_defaults.surface_preset.clone(),
            resolved_clock_source: celestial.as_ref().map(|binding| binding.clock_source),
            celestial,
            flags,
        }
    }

    pub fn has_authored_defaults(&self) -> bool {
        self.camera_preset.is_some()
            || self.player_preset.is_some()
            || self.ui_preset.is_some()
            || self.spawn_preset.is_some()
            || self.gravity_preset.is_some()
            || self.surface_preset.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneBootstrapTargetSource {
    NoneRequired,
    ControlledEntity,
    Sole3dEntity,
    DeferredNo3dEntity,
    DeferredAmbiguous3dEntities,
    DeferredNoGameplayWorld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapApplyState {
    NotRequested,
    Applied,
    AlreadyPresent,
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneBootstrapRouteState {
    NotRequested,
    Resolved,
    PendingRegistry,
    PendingResolver,
}

impl SceneBootstrapRouteState {
    fn label(self) -> &'static str {
        match self {
            SceneBootstrapRouteState::NotRequested => "not-requested",
            SceneBootstrapRouteState::Resolved => "resolved",
            SceneBootstrapRouteState::PendingRegistry => "pending-registry",
            SceneBootstrapRouteState::PendingResolver => "pending-resolver",
        }
    }
}

fn route_state_from_hint(hint: Option<&str>) -> SceneBootstrapRouteState {
    let Some(hint) = hint.map(str::trim).filter(|hint| !hint.is_empty()) else {
        return SceneBootstrapRouteState::NotRequested;
    };
    if hint.contains("pending runtime preset registry") {
        SceneBootstrapRouteState::PendingRegistry
    } else if hint.contains("pending runtime bootstrap resolver") {
        SceneBootstrapRouteState::PendingResolver
    } else {
        SceneBootstrapRouteState::Resolved
    }
}

fn route_states_summary(
    camera: SceneBootstrapRouteState,
    player: SceneBootstrapRouteState,
    ui: SceneBootstrapRouteState,
    spawn: SceneBootstrapRouteState,
    gravity: SceneBootstrapRouteState,
    surface: SceneBootstrapRouteState,
) -> String {
    format!(
        "routes[camera={},player={},ui={},spawn={},gravity={},surface={}]",
        camera.label(),
        player.label(),
        ui.label(),
        spawn.label(),
        gravity.label(),
        surface.label(),
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSceneBootstrap {
    pub authored: SceneSimulationBootstrap,
    pub resolved_camera_preset: Option<ResolvedSceneCameraPreset>,
    pub resolved_player_preset: Option<ResolvedScenePlayerPreset>,
    pub resolved_ui_preset: Option<AppliedSceneUiPreset>,
    pub resolved_spawn_preset: Option<ResolvedSceneSpawnPreset>,
    pub resolved_gravity_preset: Option<ResolvedSceneGravityPreset>,
    pub resolved_surface_preset: Option<ResolvedSceneSurfacePreset>,
    pub player_bootstrap: Option<BootstrapAssembly3D>,
    pub default_reference_frame: Option<ReferenceFrameBinding3D>,
    pub default_gravity: Option<GravityAffected2D>,
    pub default_atmosphere: Option<AtmosphereAffected2D>,
    pub selection_hints: SceneBootstrapSelectionHints,
    pub wants_gameplay_target: bool,
    pub diagnostics: Vec<String>,
}

impl ResolvedSceneBootstrap {
    pub fn from_scene(scene: &Scene) -> Self {
        Self::from_scene_with_catalogs(scene, None)
    }

    pub fn from_authored(authored: SceneSimulationBootstrap) -> Self {
        Self::from_authored_with_catalogs(authored, None)
    }

    pub fn from_scene_with_catalogs(scene: &Scene, catalogs: Option<&ModCatalogs>) -> Self {
        Self::from_authored_with_catalogs(SceneSimulationBootstrap::from_scene(scene), catalogs)
    }

    pub fn from_authored_with_catalogs(
        authored: SceneSimulationBootstrap,
        catalogs: Option<&ModCatalogs>,
    ) -> Self {
        let focus_body = authored
            .celestial
            .as_ref()
            .and_then(|binding| binding.focus_body.clone());
        let resolved_camera_preset =
            resolve_camera_preset(catalogs, authored.camera_preset.as_deref());
        let resolved_player_preset =
            resolve_catalog_player_preset(catalogs, authored.player_preset.as_deref());
        let resolved_ui_preset =
            resolve_ui_preset_resource(catalogs, authored.ui_preset.as_deref());
        let resolved_spawn_preset =
            resolve_spawn_preset(catalogs, authored.spawn_preset.as_deref());
        let resolved_gravity_preset =
            resolve_gravity_preset(catalogs, authored.gravity_preset.as_deref());
        let resolved_surface_preset =
            resolve_surface_preset(catalogs, authored.surface_preset.as_deref());
        let player_bootstrap = authored
            .player_preset
            .as_deref()
            .and_then(|preset_id| catalogs.and_then(|catalogs| catalogs.presets.player(preset_id)))
            .and_then(bootstrap_assembly3d_from_player_preset);
        let default_reference_frame = resolve_reference_frame_default(
            &authored,
            focus_body.clone(),
            resolved_spawn_preset.as_ref(),
            resolved_surface_preset.as_ref(),
        );
        let default_gravity = resolve_gravity_default(
            &authored,
            focus_body.clone(),
            resolved_gravity_preset.as_ref(),
        );
        let default_atmosphere =
            focus_body
                .filter(|_| authored.flags.is_celestial_3d)
                .map(|body_id| AtmosphereAffected2D {
                    body_id: Some(body_id),
                    ..Default::default()
                });
        let selection_hints = bootstrap_selection_hints(
            &authored,
            catalogs,
            resolved_camera_preset.as_ref(),
            resolved_player_preset.as_ref(),
            resolved_ui_preset.as_ref(),
            resolved_spawn_preset.as_ref(),
            resolved_gravity_preset.as_ref(),
            resolved_surface_preset.as_ref(),
            catalogs.is_some(),
            default_reference_frame.as_ref(),
            default_gravity.as_ref(),
        );
        let wants_gameplay_target =
            player_preset_requests_controlled_entity(&authored, resolved_player_preset.as_ref())
                || player_bootstrap.is_some()
                || default_reference_frame.is_some()
                || default_gravity.is_some()
                || default_atmosphere.is_some();

        let mut diagnostics = Vec::new();
        if authored.flags.is_celestial_3d
            && authored
                .celestial
                .as_ref()
                .and_then(|binding| binding.focus_body.as_deref())
                .is_none()
        {
            diagnostics.push("celestial world-model without focus-body binding".to_string());
        }
        if let Some(camera_preset) = authored.camera_preset.as_deref() {
            if resolved_camera_preset.is_none()
                && camera_preset_needs_runtime_registry(camera_preset)
            {
                push_diagnostic(
                    &mut diagnostics,
                    format!("camera-preset=`{camera_preset}` is pending runtime preset registry"),
                );
            }
        }
        if authored.player_preset.is_some() && resolved_player_preset.is_none() {
            push_diagnostic(
                &mut diagnostics,
                format!(
                    "player-preset=`{}` is pending runtime preset registry",
                    authored.player_preset.as_deref().unwrap_or_default()
                ),
            );
        }
        if authored.ui_preset.is_some() && resolved_ui_preset.is_none() {
            push_diagnostic(
                &mut diagnostics,
                format!(
                    "ui-preset=`{}` is pending runtime preset registry",
                    authored.ui_preset.as_deref().unwrap_or_default()
                ),
            );
        }
        push_preset_family_diagnostic(
            &mut diagnostics,
            "spawn",
            authored.spawn_preset.as_deref(),
            selection_hints.spawn.as_deref(),
        );
        push_preset_family_diagnostic(
            &mut diagnostics,
            "gravity",
            authored.gravity_preset.as_deref(),
            selection_hints.gravity.as_deref(),
        );
        push_preset_family_diagnostic(
            &mut diagnostics,
            "surface",
            authored.surface_preset.as_deref(),
            selection_hints.surface.as_deref(),
        );
        if let Some(clock_source) = authored.resolved_clock_source {
            if clock_source != CelestialClockSource::Scene {
                push_diagnostic(
                    &mut diagnostics,
                    format!(
                        "clock-source=`{}` is runtime-backed; celestial runtime reads `/runtime/celestial/{}_clock_sec` or `/runtime/celestial/{}_clock_ms`",
                        celestial_clock_source_label(clock_source),
                        celestial_clock_source_label(clock_source),
                        celestial_clock_source_label(clock_source)
                    ),
                );
            }
        }

        Self {
            authored,
            resolved_camera_preset,
            resolved_player_preset,
            resolved_ui_preset,
            resolved_spawn_preset,
            resolved_gravity_preset,
            resolved_surface_preset,
            player_bootstrap,
            default_reference_frame,
            default_gravity,
            default_atmosphere,
            selection_hints,
            wants_gameplay_target,
            diagnostics,
        }
    }

    pub fn summary(&self) -> String {
        let celestial = self
            .authored
            .celestial
            .as_ref()
            .map(celestial_binding_summary)
            .unwrap_or_else(|| "-".to_string());
        let frame = self
            .default_reference_frame
            .as_ref()
            .map(reference_frame_binding_summary)
            .unwrap_or_else(|| "-".to_string());
        let gravity = self
            .default_gravity
            .as_ref()
            .map(gravity_summary)
            .unwrap_or_else(|| "-".to_string());
        let atmosphere = self
            .default_atmosphere
            .as_ref()
            .map(|binding| format!("body={}", binding.body_id.as_deref().unwrap_or("-")))
            .unwrap_or_else(|| "-".to_string());
        format!(
            "scene={} render_space={:?} world_model={:?} celestial={} camera={} player={} ui={} spawn={} gravity={} surface={} wants_target={} {} defaults[frame={},gravity={},atmo={}] notes={}",
            self.authored.scene_id,
            self.authored.render_space,
            self.authored.world_model,
            celestial,
            self.authored.camera_preset.as_deref().unwrap_or("-"),
            self.authored.player_preset.as_deref().unwrap_or("-"),
            self.authored.ui_preset.as_deref().unwrap_or("-"),
            self.authored.spawn_preset.as_deref().unwrap_or("-"),
            self.authored.gravity_preset.as_deref().unwrap_or("-"),
            self.authored.surface_preset.as_deref().unwrap_or("-"),
            if self.wants_gameplay_target { "yes" } else { "no" },
            selection_hints_summary(&self.selection_hints),
            frame,
            gravity,
            atmosphere,
            if self.diagnostics.is_empty() {
                "-".to_string()
            } else {
                self.diagnostics.join(" | ")
            },
        )
    }
}

fn push_preset_family_diagnostic(
    diagnostics: &mut Vec<String>,
    family: &str,
    preset: Option<&str>,
    hint: Option<&str>,
) {
    let Some(preset) = preset.map(str::trim).filter(|preset| !preset.is_empty()) else {
        return;
    };
    match route_state_from_hint(hint) {
        SceneBootstrapRouteState::PendingRegistry => push_diagnostic(
            diagnostics,
            format!("{family}-preset=`{preset}` is pending runtime preset registry"),
        ),
        SceneBootstrapRouteState::PendingResolver => push_diagnostic(
            diagnostics,
            format!("{family}-preset=`{preset}` is pending runtime bootstrap resolver"),
        ),
        _ => {}
    }
}

fn resolve_gravity_default(
    authored: &SceneSimulationBootstrap,
    focus_body: Option<String>,
    resolved_gravity_preset: Option<&ResolvedSceneGravityPreset>,
) -> Option<GravityAffected2D> {
    let body_id = focus_body?;
    if !authored.flags.is_celestial_3d {
        return None;
    }
    if authored.gravity_preset.is_some() && resolved_gravity_preset.is_none() {
        return None;
    }
    if authored.gravity_preset.is_some()
        && !matches!(
            resolved_gravity_preset.map(|preset| preset.gravity_type.as_str()),
            Some("radial-body")
        )
    {
        return None;
    }
    Some(GravityAffected2D {
        mode: GravityMode2D::Point,
        body_id: Some(body_id),
        gravity_scale: 1.0,
        flat_ax: 0.0,
        flat_ay: 0.0,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppliedSceneBootstrap {
    pub scene_id: String,
    pub target_entity: Option<u64>,
    pub target_source: SceneBootstrapTargetSource,
    pub target_source_detail: Option<String>,
    pub clock_source: Option<CelestialClockSource>,
    pub selection_hints: SceneBootstrapSelectionHints,
    pub camera_applied: BootstrapApplyState,
    pub ui_applied: BootstrapApplyState,
    pub player_applied: BootstrapApplyState,
    pub controlled_entity_applied: BootstrapApplyState,
    pub reference_frame_applied: BootstrapApplyState,
    pub gravity_applied: BootstrapApplyState,
    pub atmosphere_applied: BootstrapApplyState,
    pub camera_route_state: SceneBootstrapRouteState,
    pub player_route_state: SceneBootstrapRouteState,
    pub ui_route_state: SceneBootstrapRouteState,
    pub spawn_route_state: SceneBootstrapRouteState,
    pub gravity_route_state: SceneBootstrapRouteState,
    pub surface_route_state: SceneBootstrapRouteState,
    pub diagnostics: Vec<String>,
}

impl AppliedSceneBootstrap {
    fn from_resolved(resolved: &ResolvedSceneBootstrap) -> Self {
        let camera_route_state = route_state_from_hint(resolved.selection_hints.camera.as_deref());
        let player_route_state = route_state_from_hint(resolved.selection_hints.player.as_deref());
        let ui_route_state = route_state_from_hint(resolved.selection_hints.ui.as_deref());
        let spawn_route_state = route_state_from_hint(resolved.selection_hints.spawn.as_deref());
        let gravity_route_state =
            route_state_from_hint(resolved.selection_hints.gravity.as_deref());
        let surface_route_state =
            route_state_from_hint(resolved.selection_hints.surface.as_deref());
        Self {
            scene_id: resolved.authored.scene_id.clone(),
            target_entity: None,
            target_source: if resolved.wants_gameplay_target {
                SceneBootstrapTargetSource::DeferredNoGameplayWorld
            } else {
                SceneBootstrapTargetSource::NoneRequired
            },
            target_source_detail: Some(scene_bootstrap_target_detail(
                if resolved.wants_gameplay_target {
                    SceneBootstrapTargetSource::DeferredNoGameplayWorld
                } else {
                    SceneBootstrapTargetSource::NoneRequired
                },
                None,
                &[],
            )),
            clock_source: resolved.authored.resolved_clock_source,
            selection_hints: resolved.selection_hints.clone(),
            camera_applied: if resolved.resolved_camera_preset.is_some() {
                BootstrapApplyState::Deferred
            } else {
                BootstrapApplyState::NotRequested
            },
            ui_applied: if resolved.resolved_ui_preset.is_some() {
                BootstrapApplyState::Deferred
            } else {
                BootstrapApplyState::NotRequested
            },
            player_applied: if resolved.resolved_player_preset.is_some() {
                BootstrapApplyState::Deferred
            } else {
                BootstrapApplyState::NotRequested
            },
            controlled_entity_applied: BootstrapApplyState::NotRequested,
            reference_frame_applied: if resolved.default_reference_frame.is_some() {
                BootstrapApplyState::Deferred
            } else {
                BootstrapApplyState::NotRequested
            },
            gravity_applied: if resolved.default_gravity.is_some() {
                BootstrapApplyState::Deferred
            } else {
                BootstrapApplyState::NotRequested
            },
            atmosphere_applied: if resolved.default_atmosphere.is_some() {
                BootstrapApplyState::Deferred
            } else {
                BootstrapApplyState::NotRequested
            },
            camera_route_state,
            player_route_state,
            ui_route_state,
            spawn_route_state,
            gravity_route_state,
            surface_route_state,
            diagnostics: resolved.diagnostics.clone(),
        }
    }

    pub fn has_pending_work(&self) -> bool {
        matches!(
            self.target_source,
            SceneBootstrapTargetSource::DeferredNo3dEntity
                | SceneBootstrapTargetSource::DeferredAmbiguous3dEntities
                | SceneBootstrapTargetSource::DeferredNoGameplayWorld
        ) || matches!(self.reference_frame_applied, BootstrapApplyState::Deferred)
            || matches!(self.gravity_applied, BootstrapApplyState::Deferred)
            || matches!(self.atmosphere_applied, BootstrapApplyState::Deferred)
            || matches!(self.camera_applied, BootstrapApplyState::Deferred)
            || matches!(self.ui_applied, BootstrapApplyState::Deferred)
            || matches!(self.player_applied, BootstrapApplyState::Deferred)
    }

    fn has_pending_registry_route(&self) -> bool {
        matches!(
            self.camera_route_state,
            SceneBootstrapRouteState::PendingRegistry
        ) || matches!(
            self.player_route_state,
            SceneBootstrapRouteState::PendingRegistry
        ) || matches!(
            self.ui_route_state,
            SceneBootstrapRouteState::PendingRegistry
        ) || matches!(
            self.spawn_route_state,
            SceneBootstrapRouteState::PendingRegistry
        ) || matches!(
            self.gravity_route_state,
            SceneBootstrapRouteState::PendingRegistry
        ) || matches!(
            self.surface_route_state,
            SceneBootstrapRouteState::PendingRegistry
        )
    }

    pub fn summary(&self) -> String {
        format!(
            "scene={} target={:?} target_source={:?} target_reason={} clock={} {} {} camera={:?} ui={:?} player={:?} controlled={:?} frame={:?} gravity={:?} atmo={:?} notes={}",
            self.scene_id,
            self.target_entity,
            self.target_source,
            self.target_source_detail.as_deref().unwrap_or("-"),
            self.clock_source
                .map(celestial_clock_source_label)
                .unwrap_or("-"),
            selection_hints_summary(&self.selection_hints),
            route_states_summary(
                self.camera_route_state,
                self.player_route_state,
                self.ui_route_state,
                self.spawn_route_state,
                self.gravity_route_state,
                self.surface_route_state,
            ),
            self.camera_applied,
            self.ui_applied,
            self.player_applied,
            self.controlled_entity_applied,
            self.reference_frame_applied,
            self.gravity_applied,
            self.atmosphere_applied,
            if self.diagnostics.is_empty() {
                "-".to_string()
            } else {
                self.diagnostics.join(" | ")
            },
        )
    }
}

pub struct SceneBootstrapPreparationStep;

impl ScenePreparationStep for SceneBootstrapPreparationStep {
    fn name(&self) -> &'static str {
        "scene-bootstrap"
    }

    fn run(&self, scene: &Scene, world: &mut World) {
        prepare_scene_bootstrap(scene, world);
    }
}

pub fn prepare_scene_bootstrap(scene: &Scene, world: &mut World) {
    let resolved = {
        let catalogs = world.get::<ModCatalogs>();
        ResolvedSceneBootstrap::from_scene_with_catalogs(scene, catalogs)
    };
    world.register_scoped(resolved);
}

pub fn activate_scene_bootstrap(world: &mut World) {
    let Some(resolved) = world.get::<ResolvedSceneBootstrap>().cloned().or_else(|| {
        let catalogs = world.get::<ModCatalogs>();
        world.scene_runtime().map(|runtime| {
            ResolvedSceneBootstrap::from_scene_with_catalogs(runtime.scene(), catalogs)
        })
    }) else {
        return;
    };
    let applied = apply_resolved_bootstrap(world, &resolved);
    world.register_scoped(resolved);
    world.register_scoped(applied);
}

pub fn refresh_scene_bootstrap(world: &mut World) {
    let Some(scene) = world.scene_runtime().map(|runtime| runtime.scene().clone()) else {
        return;
    };
    prepare_scene_bootstrap(&scene, world);
    activate_scene_bootstrap(world);
}

pub fn apply_pending_scene_bootstrap(world: &mut World) {
    let _ = apply_pending_scene_bootstrap_core(world);
}

pub fn apply_pending_scene_bootstrap_core(
    world: &mut engine_core::world::World,
) -> Option<AppliedSceneBootstrap> {
    let Some(existing) = world.get::<AppliedSceneBootstrap>().cloned() else {
        return None;
    };
    let retry_pending_registry =
        existing.has_pending_registry_route() && world.get::<ModCatalogs>().is_some();
    if !existing.has_pending_work() && !retry_pending_registry {
        return None;
    }
    let Some(existing_resolved) = world.get::<ResolvedSceneBootstrap>().cloned() else {
        return None;
    };
    let resolved = if retry_pending_registry {
        let catalogs = world.get::<ModCatalogs>();
        let refreshed = ResolvedSceneBootstrap::from_authored_with_catalogs(
            existing_resolved.authored.clone(),
            catalogs,
        );
        if refreshed != existing_resolved {
            world.register_scoped(refreshed.clone());
        }
        refreshed
    } else {
        existing_resolved
    };
    let applied = apply_resolved_bootstrap(world, &resolved);
    if applied != existing {
        world.register_scoped(applied.clone());
        Some(applied)
    } else {
        None
    }
}

fn resolve_reference_frame_default(
    authored: &SceneSimulationBootstrap,
    focus_body: Option<String>,
    resolved_spawn_preset: Option<&ResolvedSceneSpawnPreset>,
    resolved_surface_preset: Option<&ResolvedSceneSurfacePreset>,
) -> Option<ReferenceFrameBinding3D> {
    let body_id = focus_body?;
    if !authored.flags.is_celestial_3d {
        return None;
    }

    let explicit_surface_or_spawn =
        authored.surface_preset.is_some() || authored.spawn_preset.is_some();

    let mode = if matches!(
        resolved_surface_preset.map(|preset| preset.surface_type.as_str()),
        Some("local-horizon" | "planetary-surface")
    ) || matches!(
        resolved_spawn_preset.map(|preset| preset.spawn_type.as_str()),
        Some("planetary-surface")
    ) {
        ReferenceFrameMode::LocalHorizon
    } else if explicit_surface_or_spawn {
        return None;
    } else if matches!(
        authored.celestial.as_ref().map(|binding| binding.frame),
        Some(CelestialFrame::SurfaceLocal)
    ) {
        ReferenceFrameMode::LocalHorizon
    } else {
        ReferenceFrameMode::CelestialBody
    };

    Some(ReferenceFrameBinding3D {
        mode,
        entity_id: None,
        body_id: Some(body_id),
        inherit_linear_velocity: true,
        inherit_angular_velocity: false,
    })
}

fn select_bootstrap_target(
    gameplay_world: &GameplayWorld,
    resolved: &ResolvedSceneBootstrap,
    report: &mut AppliedSceneBootstrap,
) -> Option<u64> {
    if !resolved.wants_gameplay_target {
        report.target_source = SceneBootstrapTargetSource::NoneRequired;
        report.target_source_detail = Some(scene_bootstrap_target_detail(
            report.target_source,
            None,
            &[],
        ));
        return None;
    }

    if let Some(id) = gameplay_world.controlled_entity() {
        if gameplay_world.spatial_kind(id) == Some(SpatialKind::ThreeD) {
            report.target_entity = Some(id);
            report.target_source = SceneBootstrapTargetSource::ControlledEntity;
            report.target_source_detail = Some(scene_bootstrap_target_detail(
                report.target_source,
                Some(id),
                &[],
            ));
            report.controlled_entity_applied = BootstrapApplyState::AlreadyPresent;
            return Some(id);
        }
    }

    let ids_3d = gameplay_world.ids_with_any_3d();
    match ids_3d.as_slice() {
        [] => {
            report.target_source = SceneBootstrapTargetSource::DeferredNo3dEntity;
            let detail = scene_bootstrap_target_detail(report.target_source, None, &ids_3d);
            report.target_source_detail = Some(detail.clone());
            push_diagnostic(
                &mut report.diagnostics,
                "bootstrap target deferred: no 3D gameplay entity is registered yet",
            );
            None
        }
        [id] => {
            report.target_entity = Some(*id);
            report.target_source = SceneBootstrapTargetSource::Sole3dEntity;
            report.target_source_detail = Some(scene_bootstrap_target_detail(
                report.target_source,
                Some(*id),
                &ids_3d,
            ));
            report.controlled_entity_applied = if player_preset_requests_controlled_entity(
                &resolved.authored,
                resolved.resolved_player_preset.as_ref(),
            ) {
                if gameplay_world.set_controlled_entity(*id) {
                    BootstrapApplyState::Applied
                } else {
                    BootstrapApplyState::Deferred
                }
            } else {
                BootstrapApplyState::NotRequested
            };
            Some(*id)
        }
        _ => {
            report.target_source = SceneBootstrapTargetSource::DeferredAmbiguous3dEntities;
            let detail = scene_bootstrap_target_detail(report.target_source, None, &ids_3d);
            report.target_source_detail = Some(detail);
            push_diagnostic(
                &mut report.diagnostics,
                "bootstrap target deferred: multiple 3D gameplay entities are registered",
            );
            None
        }
    }
}

fn apply_resolved_bootstrap(
    world: &mut engine_core::world::World,
    resolved: &ResolvedSceneBootstrap,
) -> AppliedSceneBootstrap {
    let mut report = AppliedSceneBootstrap::from_resolved(resolved);

    if let Some(spawn_preset) = resolved.resolved_spawn_preset.clone() {
        let existing = world.get::<ResolvedSceneSpawnPreset>();
        if existing != Some(&spawn_preset) {
            world.register_scoped(spawn_preset);
        }
    }

    if let Some(gravity_preset) = resolved.resolved_gravity_preset.clone() {
        let existing = world.get::<ResolvedSceneGravityPreset>();
        if existing != Some(&gravity_preset) {
            world.register_scoped(gravity_preset);
        }
    }

    if let Some(surface_preset) = resolved.resolved_surface_preset.clone() {
        let existing = world.get::<ResolvedSceneSurfacePreset>();
        if existing != Some(&surface_preset) {
            world.register_scoped(surface_preset);
        }
    }

    if let Some(player_preset) = resolved.resolved_player_preset.as_ref() {
        let applied_player_preset = AppliedScenePlayerPreset::from(player_preset);
        let existing = world.get::<AppliedScenePlayerPreset>();
        report.player_applied = if existing == Some(&applied_player_preset) {
            BootstrapApplyState::AlreadyPresent
        } else {
            world.register_scoped(applied_player_preset);
            BootstrapApplyState::Applied
        };
    }

    if let Some(ui_preset) = resolved.resolved_ui_preset.clone() {
        let existing = world.get::<AppliedSceneUiPreset>();
        report.ui_applied = if existing == Some(&ui_preset) {
            BootstrapApplyState::AlreadyPresent
        } else {
            world.register_scoped(ui_preset);
            BootstrapApplyState::Applied
        };
    }

    if let Some(camera_preset) = resolved.resolved_camera_preset.as_ref() {
        let controller_id = camera_preset
            .target
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("scene-bootstrap:{}", camera_preset.preset_id));
        report.camera_applied = if let Some(runtime) =
            world.get_mut::<engine_scene_runtime::SceneRuntime>()
        {
            if runtime.apply_catalog_camera_preset(&camera_preset.controller_kind, &controller_id) {
                BootstrapApplyState::Applied
            } else if matches!(camera_preset.source, SceneCameraPresetSource::BuiltInCompat) {
                BootstrapApplyState::AlreadyPresent
            } else {
                push_diagnostic(
                    &mut report.diagnostics,
                    format!(
                        "camera-preset=`{}` resolved via catalog but runtime could not activate controller-kind=`{}`",
                        camera_preset.preset_id, camera_preset.controller_kind
                    ),
                );
                BootstrapApplyState::Deferred
            }
        } else {
            BootstrapApplyState::Deferred
        };
    }

    let Some(gameplay_world) = world.get::<GameplayWorld>() else {
        if resolved.wants_gameplay_target {
            push_diagnostic(
                &mut report.diagnostics,
                "bootstrap target deferred: gameplay world is not registered yet",
            );
        }
        return report;
    };

    let Some(target) = select_bootstrap_target(gameplay_world, resolved, &mut report) else {
        return report;
    };

    if let Some(player_bootstrap) = resolved.player_bootstrap.clone() {
        if !player_bootstrap.is_empty()
            && !gameplay_world.bootstrap_assembly3d(target, player_bootstrap)
        {
            push_diagnostic(
                &mut report.diagnostics,
                format!(
                    "player-preset=`{}` resolved via catalog but bootstrap assembly could not attach to entity {target}",
                    resolved.authored.player_preset.as_deref().unwrap_or_default()
                ),
            );
        }
    }

    if let Some(binding) = resolved.default_reference_frame.clone() {
        report.reference_frame_applied = if gameplay_world.reference_frame3d(target).is_some() {
            BootstrapApplyState::AlreadyPresent
        } else if gameplay_world.attach_reference_frame3d(target, binding) {
            BootstrapApplyState::Applied
        } else {
            BootstrapApplyState::Deferred
        };
    }

    if let Some(gravity) = resolved.default_gravity.clone() {
        report.gravity_applied = if gameplay_world.gravity(target).is_some() {
            BootstrapApplyState::AlreadyPresent
        } else if gameplay_world.attach_gravity(target, gravity) {
            BootstrapApplyState::Applied
        } else {
            BootstrapApplyState::Deferred
        };
    }

    if let Some(atmosphere) = resolved.default_atmosphere.clone() {
        report.atmosphere_applied = if gameplay_world.atmosphere(target).is_some() {
            BootstrapApplyState::AlreadyPresent
        } else if gameplay_world.attach_atmosphere(target, atmosphere) {
            BootstrapApplyState::Applied
        } else {
            BootstrapApplyState::Deferred
        };
    }

    report
}

#[cfg(test)]
mod tests {
    use super::{
        activate_scene_bootstrap, apply_pending_scene_bootstrap_core, prepare_scene_bootstrap,
        AppliedSceneBootstrap, AppliedScenePlayerPreset, BootstrapApplyState,
        ResolvedSceneBootstrap, ResolvedSceneGravityPreset, ResolvedSceneSpawnPreset,
        ResolvedSceneSurfacePreset, SceneBootstrapRouteState, SceneBootstrapTargetSource,
        SceneCameraPresetSource,
    };
    use crate::world::World;
    use engine_behavior::catalog::{
        CameraPreset, CatalogPresets, GravityPreset, ModCatalogs, PlayerPreset, SpawnPreset,
        SurfacePreset, UiPreset,
    };
    use engine_core::scene::model::{
        CelestialClockSource, CelestialFrame, CelestialScope, SceneSpace, SceneWorldModel,
    };
    use engine_core::scene::Scene;
    use engine_game::{
        AtmosphereAffected2D, GameplayWorld, GravityAffected2D, GravityMode2D,
        ReferenceFrameBinding3D, ReferenceFrameMode, Transform3D,
    };

    #[test]
    fn bootstrap_captures_world_model_and_controller_defaults() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-scene
title: Bootstrap
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: cockpit-flight
  player-preset: celestial-free-flight
  ui-preset: flight-hud
  spawn-preset: planetary-surface
  gravity-preset: radial-body
  surface-preset: local-horizon
celestial:
  scope: system
  system: sol
  focus-body: earth
  focus-site: iss
  frame: surface-local
  clock-source: campaign
layers: []
"#,
        )
        .expect("scene parse");

        let bootstrap = ResolvedSceneBootstrap::from_scene(&scene);
        assert_eq!(bootstrap.authored.scene_id, "bootstrap-scene");
        assert_eq!(bootstrap.authored.render_space, SceneSpace::ThreeD);
        assert_eq!(bootstrap.authored.world_model, SceneWorldModel::Celestial3D);
        assert_eq!(
            bootstrap.authored.resolved_clock_source,
            Some(CelestialClockSource::Campaign)
        );
        assert!(bootstrap.summary().contains("clock=campaign"));
        assert!(bootstrap
            .summary()
            .contains("celestial=celestial[scope=system"));
        assert_eq!(
            bootstrap.authored.camera_preset.as_deref(),
            Some("cockpit-flight")
        );
        assert_eq!(
            bootstrap.authored.player_preset.as_deref(),
            Some("celestial-free-flight")
        );
        assert_eq!(bootstrap.authored.ui_preset.as_deref(), Some("flight-hud"));
        assert_eq!(
            bootstrap.authored.spawn_preset.as_deref(),
            Some("planetary-surface")
        );
        assert_eq!(
            bootstrap
                .default_reference_frame
                .as_ref()
                .expect("default reference frame")
                .mode,
            engine_game::ReferenceFrameMode::LocalHorizon
        );
        assert_eq!(
            bootstrap
                .default_gravity
                .as_ref()
                .and_then(|gravity| gravity.body_id.as_deref()),
            Some("earth")
        );
        assert_eq!(
            bootstrap
                .default_atmosphere
                .as_ref()
                .and_then(|atmo| atmo.body_id.as_deref()),
            Some("earth")
        );
        assert!(bootstrap.authored.flags.uses_3d_render_space);
        assert!(bootstrap.authored.flags.is_celestial_3d);
        assert!(bootstrap.authored.flags.has_celestial_binding);
        let celestial = bootstrap
            .authored
            .celestial
            .as_ref()
            .expect("celestial binding");
        assert_eq!(celestial.scope, CelestialScope::System);
        assert_eq!(celestial.system.as_deref(), Some("sol"));
        assert_eq!(celestial.focus_body.as_deref(), Some("earth"));
        assert_eq!(celestial.focus_site.as_deref(), Some("iss"));
        assert_eq!(celestial.frame, CelestialFrame::SurfaceLocal);
        assert!(bootstrap.authored.has_authored_defaults());
        assert!(bootstrap
            .selection_hints
            .camera
            .as_deref()
            .expect("camera selection hint")
            .contains("pending runtime preset registry"));
        assert!(bootstrap
            .selection_hints
            .player
            .as_deref()
            .expect("player selection hint")
            .contains("pending runtime preset registry"));
        assert!(bootstrap
            .selection_hints
            .spawn
            .as_deref()
            .expect("spawn selection hint")
            .contains("local-horizon(body=earth)"));
        assert!(bootstrap
            .selection_hints
            .gravity
            .as_deref()
            .expect("gravity selection hint")
            .contains("point(body=earth)"));
        assert!(bootstrap
            .selection_hints
            .surface
            .as_deref()
            .expect("surface selection hint")
            .contains("local-horizon(body=earth)"));
        assert!(bootstrap.diagnostics.iter().any(|note| {
            note.contains("camera-preset=`cockpit-flight` is pending runtime preset registry")
        }));
        assert!(bootstrap.diagnostics.iter().any(|note| {
            note.contains("ui-preset=`flight-hud` is pending runtime preset registry")
        }));
        assert!(bootstrap.diagnostics.iter().any(|note| {
            note.contains(
                "player-preset=`celestial-free-flight` is pending runtime preset registry",
            )
        }));
        assert!(bootstrap
            .summary()
            .contains("selection[camera=pending runtime preset registry for `cockpit-flight`"));
        assert!(bootstrap
            .summary()
            .contains("player=pending runtime preset registry for `celestial-free-flight`"));
    }

    #[test]
    fn bootstrap_reports_camera_selection_hints_for_legacy_and_custom_presets() {
        let cases = [
            (
                "playground-obj-viewer",
                r#"
id: playground-obj-viewer
title: Obj Viewer
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: obj-viewer
input:
  obj-viewer:
    sprite_id: probe
layers: []
"#,
                Some("legacy input.obj-viewer compatibility route"),
            ),
            (
                "playground-surface-free-look",
                r#"
id: playground-surface-free-look
title: Surface Free Look
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: surface-free-look
input:
  free-look-camera:
    surface-mode: true
celestial:
  focus-body: earth
layers: []
"#,
                Some("built-in camera preset `surface-free-look` -> surface-free-look(target=-)"),
            ),
            (
                "playground-orbit-inspector",
                r#"
id: playground-orbit-inspector
title: Orbit Inspector
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: celestial-inspector
celestial:
  focus-body: earth
layers: []
"#,
                Some("pending runtime preset registry for `celestial-inspector`"),
            ),
            (
                "planet-generator-cockpit",
                r#"
id: planet-generator-cockpit
title: Cockpit
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: cockpit-view
celestial:
  focus-body: earth
layers: []
"#,
                Some("pending runtime preset registry for `cockpit-view`"),
            ),
            (
                "missing-controller-defaults",
                r#"
id: missing-controller-defaults
title: Missing
render-space: 3d
world-model: euclidean-3d
layers: []
"#,
                None,
            ),
        ];

        for (scene_id, yaml, expected_camera_hint) in cases {
            let scene: Scene = serde_yaml::from_str(yaml).expect("scene parse");
            let bootstrap = ResolvedSceneBootstrap::from_scene(&scene);
            assert_eq!(bootstrap.authored.scene_id, scene_id);
            assert_eq!(
                bootstrap.selection_hints.camera.as_deref(),
                expected_camera_hint
            );
        }
    }

    #[test]
    fn bootstrap_resolves_builtin_camera_preset_without_catalog_registry() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-built-in-camera
title: Bootstrap Built-in Camera
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: surface-free-look
input:
  free-look-camera:
    surface-mode: true
celestial:
  focus-body: earth
layers: []
"#,
        )
        .expect("scene parse");

        let bootstrap = ResolvedSceneBootstrap::from_scene(&scene);
        let camera = bootstrap
            .resolved_camera_preset
            .as_ref()
            .expect("resolved built-in camera");
        assert_eq!(camera.source, SceneCameraPresetSource::BuiltInCompat);
        assert_eq!(camera.controller_kind, "surface-free-look");
        assert_eq!(
            bootstrap.selection_hints.camera.as_deref(),
            Some("built-in camera preset `surface-free-look` -> surface-free-look(target=-)")
        );
        assert!(!bootstrap
            .diagnostics
            .iter()
            .any(|note| note.contains("pending runtime preset registry")));
    }

    #[test]
    fn bootstrap_reports_missing_focus_body_for_celestial_world_model() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-missing-focus-body
title: Bootstrap Missing Focus Body
render-space: 3d
world-model: celestial-3d
controller-defaults:
  player-preset: celestial-free-flight
layers: []
"#,
        )
        .expect("scene parse");

        let bootstrap = ResolvedSceneBootstrap::from_scene(&scene);
        assert!(bootstrap
            .diagnostics
            .iter()
            .any(|note| note.contains("celestial world-model without focus-body binding")));
        assert!(bootstrap.default_reference_frame.is_none());
        assert!(bootstrap.default_gravity.is_none());
        assert!(bootstrap.default_atmosphere.is_none());
        assert!(!bootstrap.wants_gameplay_target);
        assert!(bootstrap.diagnostics.iter().any(|note| note
            .contains("player-preset=`celestial-free-flight` is pending runtime preset registry")));
        assert!(bootstrap
            .summary()
            .contains("notes=celestial world-model without focus-body binding"));
    }

    #[test]
    fn bootstrap_reports_pending_registry_for_custom_player_and_surface_gravity_spawn_presets() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-custom-runtime-presets
title: Bootstrap Custom Runtime Presets
render-space: 3d
world-model: celestial-3d
controller-defaults:
  player-preset: custom-flight
  spawn-preset: custom-surface-spawn
  gravity-preset: custom-radial
  surface-preset: custom-surface
celestial:
  focus-body: earth
  frame: surface-local
layers: []
"#,
        )
        .expect("scene parse");

        let bootstrap = ResolvedSceneBootstrap::from_scene(&scene);
        assert_eq!(
            bootstrap.selection_hints.player.as_deref(),
            Some("pending runtime preset registry for `custom-flight`")
        );
        assert_eq!(
            bootstrap.selection_hints.spawn.as_deref(),
            Some("pending runtime preset registry for `custom-surface-spawn`")
        );
        assert_eq!(
            bootstrap.selection_hints.gravity.as_deref(),
            Some("pending runtime preset registry for `custom-radial`")
        );
        assert_eq!(
            bootstrap.selection_hints.surface.as_deref(),
            Some("pending runtime preset registry for `custom-surface`")
        );
        assert!(bootstrap.diagnostics.iter().any(|note| note
            .contains("player-preset=`custom-flight` is pending runtime preset registry")));
        assert!(bootstrap.diagnostics.iter().any(|note| note
            .contains("spawn-preset=`custom-surface-spawn` is pending runtime preset registry")));
        assert!(bootstrap.diagnostics.iter().any(|note| note
            .contains("gravity-preset=`custom-radial` is pending runtime preset registry")));
        assert!(bootstrap.diagnostics.iter().any(|note| note
            .contains("surface-preset=`custom-surface` is pending runtime preset registry")));
        assert!(bootstrap.default_reference_frame.is_none());
        assert!(bootstrap.default_gravity.is_none());
        assert!(bootstrap.default_atmosphere.is_some());
        assert!(bootstrap.wants_gameplay_target);
    }

    #[test]
    fn bootstrap_resolves_catalog_backed_spawn_gravity_and_surface_presets() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-catalog-backed-world-presets
title: Bootstrap Catalog-backed World Presets
render-space: 3d
world-model: celestial-3d
controller-defaults:
  spawn-preset: touchdown
  gravity-preset: radial-default
  surface-preset: terrain-horizon
celestial:
  focus-body: earth
layers: []
"#,
        )
        .expect("scene parse");

        let mut catalogs = ModCatalogs::default();
        catalogs.presets = CatalogPresets {
            spawns: [(
                "touchdown".to_string(),
                SpawnPreset {
                    spawn_type: Some("planetary-surface".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            gravity: [(
                "radial-default".to_string(),
                GravityPreset {
                    gravity_type: Some("radial-body".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            surfaces: [(
                "terrain-horizon".to_string(),
                SurfacePreset {
                    surface_type: Some("local-horizon".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let bootstrap = ResolvedSceneBootstrap::from_scene_with_catalogs(&scene, Some(&catalogs));
        assert_eq!(
            bootstrap.selection_hints.spawn.as_deref(),
            Some(
                "catalog spawn preset `touchdown` -> planetary-surface + local-horizon(body=earth)"
            )
        );
        assert_eq!(
            bootstrap.selection_hints.gravity.as_deref(),
            Some("catalog gravity preset `radial-default` -> radial-body + point(body=earth)")
        );
        assert_eq!(
            bootstrap.selection_hints.surface.as_deref(),
            Some("catalog surface preset `terrain-horizon` -> local-horizon + local-horizon(body=earth)")
        );
        assert!(bootstrap.default_reference_frame.is_some());
        assert!(bootstrap.default_gravity.is_some());
        assert!(!bootstrap
            .diagnostics
            .iter()
            .any(|note| note.contains("pending runtime")));
    }

    #[test]
    fn bootstrap_applies_catalog_backed_world_presets_as_scoped_resources() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-catalog-backed-world-presets-apply
title: Bootstrap Catalog-backed World Presets Apply
render-space: 3d
world-model: celestial-3d
controller-defaults:
  spawn-preset: touchdown
  gravity-preset: radial-default
  surface-preset: terrain-horizon
celestial:
  focus-body: earth
layers: []
"#,
        )
        .expect("scene parse");

        let mut catalogs = ModCatalogs::default();
        catalogs.presets = CatalogPresets {
            spawns: [(
                "touchdown".to_string(),
                SpawnPreset {
                    spawn_type: Some("planetary-surface".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            gravity: [(
                "radial-default".to_string(),
                GravityPreset {
                    gravity_type: Some("radial-body".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            surfaces: [(
                "terrain-horizon".to_string(),
                SurfacePreset {
                    surface_type: Some("local-horizon".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let mut world = World::new();
        world.register(catalogs);
        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        assert_eq!(
            world.get::<ResolvedSceneSpawnPreset>(),
            Some(&ResolvedSceneSpawnPreset {
                preset_id: "touchdown".to_string(),
                source: super::SceneRuntimePresetSource::Catalog,
                spawn_type: "planetary-surface".to_string(),
                config: Default::default(),
            })
        );
        assert_eq!(
            world.get::<ResolvedSceneGravityPreset>(),
            Some(&ResolvedSceneGravityPreset {
                preset_id: "radial-default".to_string(),
                source: super::SceneRuntimePresetSource::Catalog,
                gravity_type: "radial-body".to_string(),
                config: Default::default(),
            })
        );
        assert_eq!(
            world.get::<ResolvedSceneSurfacePreset>(),
            Some(&ResolvedSceneSurfacePreset {
                preset_id: "terrain-horizon".to_string(),
                source: super::SceneRuntimePresetSource::Catalog,
                surface_type: "local-horizon".to_string(),
                config: Default::default(),
            })
        );
    }

    #[test]
    fn catalog_world_preset_ids_shadow_builtin_aliases_when_type_is_not_consumable() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-shadowed-builtins
title: Bootstrap Shadowed Builtins
render-space: 3d
world-model: celestial-3d
controller-defaults:
  spawn-preset: planetary-surface
  gravity-preset: radial-body
  surface-preset: local-horizon
celestial:
  focus-body: earth
layers: []
"#,
        )
        .expect("scene parse");

        let mut catalogs = ModCatalogs::default();
        catalogs.presets = CatalogPresets {
            spawns: [(
                "planetary-surface".to_string(),
                SpawnPreset {
                    spawn_type: Some("scripted-spawn".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            gravity: [(
                "radial-body".to_string(),
                GravityPreset {
                    gravity_type: Some("field-volume".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            surfaces: [(
                "local-horizon".to_string(),
                SurfacePreset {
                    surface_type: Some("mesh-surface".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let bootstrap = ResolvedSceneBootstrap::from_scene_with_catalogs(&scene, Some(&catalogs));
        assert_eq!(
            bootstrap.selection_hints.spawn.as_deref(),
            Some("catalog spawn preset `planetary-surface` is pending runtime bootstrap resolver")
        );
        assert_eq!(
            bootstrap.selection_hints.gravity.as_deref(),
            Some("catalog gravity preset `radial-body` is pending runtime bootstrap resolver")
        );
        assert_eq!(
            bootstrap.selection_hints.surface.as_deref(),
            Some("catalog surface preset `local-horizon` is pending runtime bootstrap resolver")
        );
        assert!(bootstrap.default_reference_frame.is_none());
        assert!(bootstrap.default_gravity.is_none());
        assert!(bootstrap.diagnostics.iter().any(|note| note
            .contains("spawn-preset=`planetary-surface` is pending runtime bootstrap resolver")));
        assert!(bootstrap.diagnostics.iter().any(|note| note
            .contains("gravity-preset=`radial-body` is pending runtime bootstrap resolver")));
        assert!(bootstrap.diagnostics.iter().any(|note| note
            .contains("surface-preset=`local-horizon` is pending runtime bootstrap resolver")));
    }

    #[test]
    fn bootstrap_ui_only_diagnostics_do_not_require_gameplay_retry() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-ui-only
title: Bootstrap UI Only
render-space: 2d
world-model: planar-2d
controller-defaults:
  ui-preset: pause-menu
layers: []
"#,
        )
        .expect("scene parse");

        let mut world = World::new();
        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let applied = world
            .get::<AppliedSceneBootstrap>()
            .expect("applied bootstrap");
        assert!(!applied.has_pending_work());
        assert_eq!(
            applied.ui_route_state,
            SceneBootstrapRouteState::PendingRegistry
        );
        assert_eq!(
            applied.camera_route_state,
            SceneBootstrapRouteState::NotRequested
        );
        assert!(
            applied
                .diagnostics
                .iter()
                .any(|note| note
                    .contains("ui-preset=`pause-menu` is pending runtime preset registry"))
        );
        assert!(applied
            .summary()
            .contains("routes[camera=not-requested,player=not-requested,ui=pending-registry,spawn=not-requested,gravity=not-requested,surface=not-requested]"));
    }

    #[test]
    fn bootstrap_resolves_catalog_backed_camera_ui_and_player_presets() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-catalog-backed-presets
title: Bootstrap Catalog-backed Presets
render-space: 3d
world-model: celestial-3d
controller-defaults:
  camera-preset: cockpit-flight
  player-preset: celestial-free-flight
  ui-preset: flight-hud
celestial:
  focus-body: earth
layers: []
"#,
        )
        .expect("scene parse");

        let mut catalogs = ModCatalogs::default();
        catalogs.presets = CatalogPresets {
            cameras: [(
                "cockpit-flight".to_string(),
                CameraPreset {
                    controller_kind: Some("cockpit".to_string()),
                    target: Some("player".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            players: [(
                "celestial-free-flight".to_string(),
                PlayerPreset {
                    input_profile: Some("flight".to_string()),
                    config: [("controlled".to_string(), serde_json::json!(true))]
                        .into_iter()
                        .collect(),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ui: [(
                "flight-hud".to_string(),
                UiPreset {
                    layout: Some("cockpit".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let mut world = World::new();
        world.register(catalogs);
        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let resolved = world
            .get::<ResolvedSceneBootstrap>()
            .expect("resolved bootstrap");
        assert_eq!(
            resolved.selection_hints.camera.as_deref(),
            Some("catalog camera preset `cockpit-flight` -> cockpit(target=player)")
        );
        assert_eq!(
            resolved.selection_hints.player.as_deref(),
            Some("catalog player preset `celestial-free-flight` -> controlled gameplay entity + input=flight")
        );
        assert_eq!(
            resolved.selection_hints.ui.as_deref(),
            Some("catalog ui preset `flight-hud` -> layout=cockpit")
        );

        let applied = world
            .get::<AppliedSceneBootstrap>()
            .expect("applied bootstrap");
        assert_eq!(
            applied.camera_route_state,
            SceneBootstrapRouteState::Resolved
        );
        assert_eq!(applied.ui_route_state, SceneBootstrapRouteState::Resolved);
        assert_eq!(applied.ui_applied, BootstrapApplyState::Applied);
        assert_eq!(applied.player_applied, BootstrapApplyState::Applied);
        assert_eq!(applied.camera_applied, BootstrapApplyState::Applied);
        assert_eq!(
            world.get::<AppliedScenePlayerPreset>(),
            Some(&AppliedScenePlayerPreset {
                preset_id: "celestial-free-flight".to_string(),
                controlled: true,
                has_bootstrap_assembly: false,
                input_profile: Some("flight".to_string()),
                controller_type: None,
            })
        );
        assert!(!applied
            .diagnostics
            .iter()
            .any(|note| note.contains("pending runtime preset registry")));
    }

    #[test]
    fn bootstrap_applies_builtin_camera_preset_under_bootstrap_path() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-built-in-camera-apply
title: Bootstrap Built-in Camera Apply
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: free-look-camera
input:
  free-look-camera: {}
layers: []
"#,
        )
        .expect("scene parse");

        let mut world = World::new();
        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let resolved = world
            .get::<ResolvedSceneBootstrap>()
            .expect("resolved bootstrap");
        assert_eq!(
            resolved.selection_hints.camera.as_deref(),
            Some("built-in camera preset `free-look-camera` -> free-look-camera(target=-)")
        );

        let applied = world
            .get::<AppliedSceneBootstrap>()
            .expect("applied bootstrap");
        assert_eq!(
            applied.camera_route_state,
            SceneBootstrapRouteState::Resolved
        );
        assert_eq!(applied.camera_applied, BootstrapApplyState::Applied);
        assert_eq!(applied.player_applied, BootstrapApplyState::NotRequested);
        assert!(!applied.has_pending_work());
    }

    #[test]
    fn metadata_only_catalog_player_preset_does_not_force_a_bootstrap_target() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-metadata-player-preset
title: Bootstrap Metadata Player Preset
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  player-preset: observer
layers: []
"#,
        )
        .expect("scene parse");

        let mut catalogs = ModCatalogs::default();
        catalogs.presets = CatalogPresets {
            players: [(
                "observer".to_string(),
                PlayerPreset {
                    input_profile: Some("look".to_string()),
                    controller: Some(engine_behavior::catalog::ControllerComponent {
                        controller_type: "ObserverController".to_string(),
                        config: Default::default(),
                    }),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let mut world = World::new();
        world.register(catalogs);
        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let resolved = world
            .get::<ResolvedSceneBootstrap>()
            .expect("resolved bootstrap");
        assert_eq!(
            resolved.selection_hints.player.as_deref(),
            Some("catalog player preset `observer` -> controller=ObserverController + input=look")
        );
        assert!(!resolved.wants_gameplay_target);

        let applied = world
            .get::<AppliedSceneBootstrap>()
            .expect("applied bootstrap");
        assert_eq!(
            applied.target_source,
            SceneBootstrapTargetSource::NoneRequired
        );
        assert_eq!(applied.target_entity, None);
        assert_eq!(applied.player_applied, BootstrapApplyState::Applied);
        assert_eq!(
            applied.controlled_entity_applied,
            BootstrapApplyState::NotRequested
        );
        assert!(!applied.has_pending_work());
        assert_eq!(
            world.get::<AppliedScenePlayerPreset>(),
            Some(&AppliedScenePlayerPreset {
                preset_id: "observer".to_string(),
                controlled: false,
                has_bootstrap_assembly: false,
                input_profile: Some("look".to_string()),
                controller_type: Some("ObserverController".to_string()),
            })
        );
    }

    #[test]
    fn bootstrap_reports_pending_player_registry_when_catalogs_do_not_define_player_preset() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-missing-player-preset
title: Bootstrap Missing Player Preset
render-space: 3d
world-model: celestial-3d
controller-defaults:
  player-preset: missing-flight
celestial:
  focus-body: earth
layers: []
"#,
        )
        .expect("scene parse");

        let catalogs = ModCatalogs::default();
        let bootstrap = ResolvedSceneBootstrap::from_scene_with_catalogs(&scene, Some(&catalogs));

        assert_eq!(
            bootstrap.selection_hints.player.as_deref(),
            Some("pending runtime preset registry for `missing-flight`")
        );
        assert!(bootstrap.diagnostics.iter().any(|note| note
            .contains("player-preset=`missing-flight` is pending runtime preset registry")));
    }

    #[test]
    fn pending_registry_routes_re_resolve_when_catalogs_register_late() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-late-catalogs
title: Bootstrap Late Catalogs
render-space: 3d
world-model: euclidean-3d
controller-defaults:
  camera-preset: cockpit-flight
  player-preset: observer
  ui-preset: flight-hud
layers: []
"#,
        )
        .expect("scene parse");

        let mut world = World::new();
        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let initial = world
            .get::<AppliedSceneBootstrap>()
            .expect("initial applied bootstrap");
        assert_eq!(
            initial.camera_route_state,
            SceneBootstrapRouteState::PendingRegistry
        );
        assert_eq!(
            initial.player_route_state,
            SceneBootstrapRouteState::PendingRegistry
        );
        assert_eq!(
            initial.ui_route_state,
            SceneBootstrapRouteState::PendingRegistry
        );
        assert!(!initial.has_pending_work());

        let mut catalogs = ModCatalogs::default();
        catalogs.presets = CatalogPresets {
            cameras: [(
                "cockpit-flight".to_string(),
                CameraPreset {
                    controller_kind: Some("cockpit".to_string()),
                    target: Some("player".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            players: [(
                "observer".to_string(),
                PlayerPreset {
                    input_profile: Some("look".to_string()),
                    controller: Some(engine_behavior::catalog::ControllerComponent {
                        controller_type: "ObserverController".to_string(),
                        config: Default::default(),
                    }),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ui: [(
                "flight-hud".to_string(),
                UiPreset {
                    layout: Some("cockpit".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        world.register(catalogs);

        let applied = apply_pending_scene_bootstrap_core(&mut world)
            .expect("bootstrap should re-resolve after catalogs register");
        assert_eq!(
            applied.camera_route_state,
            SceneBootstrapRouteState::Resolved
        );
        assert_eq!(
            applied.player_route_state,
            SceneBootstrapRouteState::Resolved
        );
        assert_eq!(applied.ui_route_state, SceneBootstrapRouteState::Resolved);
        assert_eq!(applied.camera_applied, BootstrapApplyState::Applied);
        assert_eq!(applied.ui_applied, BootstrapApplyState::Applied);
        assert_eq!(applied.player_applied, BootstrapApplyState::Applied);
        assert_eq!(
            applied.target_source,
            SceneBootstrapTargetSource::NoneRequired
        );

        let resolved = world
            .get::<ResolvedSceneBootstrap>()
            .expect("resolved bootstrap");
        assert_eq!(
            resolved.selection_hints.camera.as_deref(),
            Some("catalog camera preset `cockpit-flight` -> cockpit(target=player)")
        );
        assert_eq!(
            resolved.selection_hints.player.as_deref(),
            Some("catalog player preset `observer` -> controller=ObserverController + input=look")
        );
        assert_eq!(
            resolved.selection_hints.ui.as_deref(),
            Some("catalog ui preset `flight-hud` -> layout=cockpit")
        );
        assert_eq!(
            world.get::<AppliedScenePlayerPreset>(),
            Some(&AppliedScenePlayerPreset {
                preset_id: "observer".to_string(),
                controlled: false,
                has_bootstrap_assembly: false,
                input_profile: Some("look".to_string()),
                controller_type: Some("ObserverController".to_string()),
            })
        );
    }

    #[test]
    fn pending_registry_world_preset_routes_re_resolve_when_catalogs_register_late() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: bootstrap-late-world-catalogs
title: Bootstrap Late World Catalogs
render-space: 3d
world-model: celestial-3d
controller-defaults:
  spawn-preset: touchdown
  gravity-preset: radial-default
  surface-preset: terrain-horizon
celestial:
  focus-body: earth
layers: []
"#,
        )
        .expect("scene parse");

        let mut world = World::new();
        let gameplay = GameplayWorld::default();
        let entity = gameplay
            .spawn("pilot", serde_json::json!({}))
            .expect("entity");
        assert!(gameplay.set_transform3d(entity, Transform3D::default()));
        world.register(gameplay.clone());

        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let initial = world
            .get::<AppliedSceneBootstrap>()
            .expect("initial applied bootstrap");
        assert_eq!(
            initial.spawn_route_state,
            SceneBootstrapRouteState::PendingRegistry
        );
        assert_eq!(
            initial.gravity_route_state,
            SceneBootstrapRouteState::PendingRegistry
        );
        assert_eq!(
            initial.surface_route_state,
            SceneBootstrapRouteState::PendingRegistry
        );
        assert_eq!(
            initial.reference_frame_applied,
            BootstrapApplyState::NotRequested
        );
        assert_eq!(initial.gravity_applied, BootstrapApplyState::NotRequested);

        let mut catalogs = ModCatalogs::default();
        catalogs.presets = CatalogPresets {
            spawns: [(
                "touchdown".to_string(),
                SpawnPreset {
                    spawn_type: Some("planetary-surface".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            gravity: [(
                "radial-default".to_string(),
                GravityPreset {
                    gravity_type: Some("radial-body".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            surfaces: [(
                "terrain-horizon".to_string(),
                SurfacePreset {
                    surface_type: Some("local-horizon".to_string()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        world.register(catalogs);

        let applied = apply_pending_scene_bootstrap_core(&mut world)
            .expect("bootstrap should re-resolve after world catalogs register");
        assert_eq!(
            applied.spawn_route_state,
            SceneBootstrapRouteState::Resolved
        );
        assert_eq!(
            applied.gravity_route_state,
            SceneBootstrapRouteState::Resolved
        );
        assert_eq!(
            applied.surface_route_state,
            SceneBootstrapRouteState::Resolved
        );
        assert_eq!(
            applied.reference_frame_applied,
            BootstrapApplyState::Applied
        );
        assert_eq!(applied.gravity_applied, BootstrapApplyState::Applied);

        let resolved = world
            .get::<ResolvedSceneBootstrap>()
            .expect("resolved bootstrap");
        assert_eq!(
            resolved.selection_hints.spawn.as_deref(),
            Some(
                "catalog spawn preset `touchdown` -> planetary-surface + local-horizon(body=earth)"
            )
        );
        assert_eq!(
            resolved.selection_hints.gravity.as_deref(),
            Some("catalog gravity preset `radial-default` -> radial-body + point(body=earth)")
        );
        assert_eq!(
            resolved.selection_hints.surface.as_deref(),
            Some("catalog surface preset `terrain-horizon` -> local-horizon + local-horizon(body=earth)")
        );
    }

    #[test]
    fn bootstrap_reports_target_waits_and_retries_as_state_changes() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: gameplay-bootstrap-pending
title: Gameplay Bootstrap Pending
render-space: 3d
world-model: celestial-3d
controller-defaults:
  player-preset: celestial-free-flight
  surface-preset: local-horizon
celestial:
  focus-body: earth
  frame: surface-local
layers: []
"#,
        )
        .expect("scene parse");

        let mut world = World::new();
        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let initial = world
            .get::<AppliedSceneBootstrap>()
            .expect("initial applied bootstrap");
        assert_eq!(initial.clock_source, Some(CelestialClockSource::Scene));
        assert_eq!(
            initial.target_source,
            SceneBootstrapTargetSource::DeferredNoGameplayWorld
        );
        assert!(matches!(
            initial.target_source_detail.as_deref(),
            Some(detail) if detail.contains("waiting for gameplay world registration")
        ));
        assert!(initial
            .diagnostics
            .iter()
            .any(|note| note.contains("gameplay world is not registered yet")));

        let gameplay = GameplayWorld::default();
        world.register(gameplay.clone());

        let first_retry = apply_pending_scene_bootstrap_core(&mut world);
        assert!(
            first_retry.is_some(),
            "state should update when gameplay world appears"
        );

        let after_world = world
            .get::<AppliedSceneBootstrap>()
            .expect("bootstrap after world registration");
        assert_eq!(
            after_world.target_source,
            SceneBootstrapTargetSource::DeferredNo3dEntity
        );
        assert!(matches!(
            after_world.target_source_detail.as_deref(),
            Some(detail) if detail.contains("waiting for at least one 3D gameplay entity")
        ));
        assert!(after_world
            .diagnostics
            .iter()
            .any(|note| note.contains("no 3D gameplay entity is registered yet")));

        let entity = gameplay
            .spawn("pilot", serde_json::json!({}))
            .expect("entity");
        assert!(gameplay.set_transform3d(entity, Transform3D::default()));

        let second_retry = apply_pending_scene_bootstrap_core(&mut world);
        assert!(
            second_retry.is_some(),
            "state should update when a 3D entity appears"
        );

        let after_entity = world
            .get::<AppliedSceneBootstrap>()
            .expect("bootstrap after entity spawn");
        assert_eq!(after_entity.target_entity, Some(entity));
        assert_eq!(
            after_entity.target_source,
            SceneBootstrapTargetSource::Sole3dEntity
        );
        assert!(matches!(
            after_entity.target_source_detail.as_deref(),
            Some(detail) if detail.contains("selected sole 3D gameplay entity")
        ));
        assert_eq!(
            after_entity.controlled_entity_applied,
            BootstrapApplyState::NotRequested
        );
        assert_eq!(
            after_entity.reference_frame_applied,
            BootstrapApplyState::Applied
        );
        assert_eq!(after_entity.gravity_applied, BootstrapApplyState::Applied);
        assert_eq!(
            after_entity.atmosphere_applied,
            BootstrapApplyState::Applied
        );
    }

    #[test]
    fn activate_scene_bootstrap_claims_sole_3d_entity_and_applies_defaults() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: gameplay-bootstrap
title: Gameplay Bootstrap
render-space: 3d
world-model: celestial-3d
controller-defaults:
  player-preset: celestial-free-flight
  surface-preset: local-horizon
celestial:
  focus-body: earth
  frame: surface-local
layers: []
"#,
        )
        .expect("scene parse");

        let mut world = World::new();
        let gameplay = GameplayWorld::default();
        let entity = gameplay
            .spawn("pilot", serde_json::json!({}))
            .expect("entity");
        assert!(gameplay.set_transform3d(entity, Transform3D::default()));
        world.register(gameplay.clone());

        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let applied = world
            .get::<AppliedSceneBootstrap>()
            .expect("applied bootstrap");
        assert_eq!(applied.target_entity, Some(entity));
        assert_eq!(
            applied.target_source,
            SceneBootstrapTargetSource::Sole3dEntity
        );
        assert_eq!(
            applied.controlled_entity_applied,
            BootstrapApplyState::NotRequested
        );
        assert_eq!(
            applied.reference_frame_applied,
            BootstrapApplyState::Applied
        );
        assert_eq!(applied.gravity_applied, BootstrapApplyState::Applied);
        assert_eq!(applied.atmosphere_applied, BootstrapApplyState::Applied);
        assert_eq!(gameplay.controlled_entity(), None);
        assert!(gameplay.reference_frame3d(entity).is_some());
        assert!(gameplay.gravity(entity).is_some());
        assert!(gameplay.atmosphere(entity).is_some());
        assert_eq!(
            applied.player_route_state,
            SceneBootstrapRouteState::PendingRegistry
        );
        assert_eq!(
            applied.surface_route_state,
            SceneBootstrapRouteState::Resolved
        );
        assert!(applied
            .summary()
            .contains("routes[camera=not-requested,player=pending-registry,ui=not-requested,spawn=not-requested,gravity=not-requested,surface=resolved]"));
    }

    #[test]
    fn activate_scene_bootstrap_keeps_existing_target_and_defaults_as_already_present() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: gameplay-bootstrap-existing
title: Gameplay Bootstrap Existing
render-space: 3d
world-model: celestial-3d
controller-defaults:
  player-preset: celestial-free-flight
  surface-preset: local-horizon
celestial:
  focus-body: earth
  frame: surface-local
layers: []
"#,
        )
        .expect("scene parse");

        let mut world = World::new();
        let gameplay = GameplayWorld::default();
        let entity = gameplay
            .spawn("pilot", serde_json::json!({}))
            .expect("entity");
        assert!(gameplay.set_transform3d(entity, Transform3D::default()));
        assert!(gameplay.set_controlled_entity(entity));
        assert!(gameplay.attach_reference_frame3d(
            entity,
            ReferenceFrameBinding3D {
                mode: ReferenceFrameMode::LocalHorizon,
                entity_id: None,
                body_id: Some("earth".to_string()),
                inherit_linear_velocity: true,
                inherit_angular_velocity: false,
            }
        ));
        assert!(gameplay.attach_gravity(
            entity,
            GravityAffected2D {
                mode: GravityMode2D::Point,
                body_id: Some("earth".to_string()),
                gravity_scale: 1.0,
                flat_ax: 0.0,
                flat_ay: 0.0,
            }
        ));
        assert!(gameplay.attach_atmosphere(
            entity,
            AtmosphereAffected2D {
                body_id: Some("earth".to_string()),
                ..Default::default()
            }
        ));
        world.register(gameplay.clone());

        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let applied = world
            .get::<AppliedSceneBootstrap>()
            .expect("applied bootstrap");
        assert_eq!(applied.target_entity, Some(entity));
        assert_eq!(
            applied.target_source,
            SceneBootstrapTargetSource::ControlledEntity
        );
        assert_eq!(
            applied.controlled_entity_applied,
            BootstrapApplyState::AlreadyPresent
        );
        assert_eq!(
            applied.reference_frame_applied,
            BootstrapApplyState::AlreadyPresent
        );
        assert_eq!(applied.gravity_applied, BootstrapApplyState::AlreadyPresent);
        assert_eq!(
            applied.atmosphere_applied,
            BootstrapApplyState::AlreadyPresent
        );
        assert_eq!(gameplay.controlled_entity(), Some(entity));
    }

    #[test]
    fn pending_bootstrap_retries_after_3d_entity_spawns() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: gameplay-bootstrap-pending
title: Gameplay Bootstrap Pending
render-space: 3d
world-model: celestial-3d
controller-defaults:
  player-preset: celestial-free-flight
  surface-preset: local-horizon
celestial:
  focus-body: earth
  frame: surface-local
layers: []
"#,
        )
        .expect("scene parse");

        let mut world = World::new();
        let gameplay = GameplayWorld::default();
        world.register(gameplay.clone());

        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let applied = world
            .get::<AppliedSceneBootstrap>()
            .expect("initial applied bootstrap");
        assert_eq!(
            applied.target_source,
            SceneBootstrapTargetSource::DeferredNo3dEntity
        );

        let entity = gameplay
            .spawn("pilot", serde_json::json!({}))
            .expect("entity");
        assert!(gameplay.set_transform3d(entity, Transform3D::default()));

        let _ = apply_pending_scene_bootstrap_core(&mut world);

        let applied = world
            .get::<AppliedSceneBootstrap>()
            .expect("retried applied bootstrap");
        assert_eq!(applied.target_entity, Some(entity));
        assert_eq!(
            applied.target_source,
            SceneBootstrapTargetSource::Sole3dEntity
        );
        assert_eq!(
            applied.reference_frame_applied,
            BootstrapApplyState::Applied
        );
        assert_eq!(applied.gravity_applied, BootstrapApplyState::Applied);
        assert_eq!(applied.atmosphere_applied, BootstrapApplyState::Applied);
        assert_eq!(
            applied.controlled_entity_applied,
            BootstrapApplyState::NotRequested
        );
        assert_eq!(gameplay.controlled_entity(), None);
    }

    #[test]
    fn pending_bootstrap_retries_after_gameplay_world_is_registered() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: gameplay-bootstrap-no-world
title: Gameplay Bootstrap No World
render-space: 3d
world-model: celestial-3d
controller-defaults:
  player-preset: celestial-free-flight
  surface-preset: local-horizon
celestial:
  focus-body: earth
  frame: surface-local
layers: []
"#,
        )
        .expect("scene parse");

        let mut world = World::new();

        prepare_scene_bootstrap(&scene, &mut world);
        world.register_scoped(crate::scene_runtime::SceneRuntime::new(scene));
        activate_scene_bootstrap(&mut world);

        let applied = world
            .get::<AppliedSceneBootstrap>()
            .expect("initial applied bootstrap");
        assert_eq!(
            applied.target_source,
            SceneBootstrapTargetSource::DeferredNoGameplayWorld
        );
        assert!(applied.has_pending_work());

        let gameplay = GameplayWorld::default();
        let entity = gameplay
            .spawn("pilot", serde_json::json!({}))
            .expect("entity");
        assert!(gameplay.set_transform3d(entity, Transform3D::default()));
        world.register(gameplay.clone());

        let _ = apply_pending_scene_bootstrap_core(&mut world);

        let applied = world
            .get::<AppliedSceneBootstrap>()
            .expect("retried applied bootstrap");
        assert_eq!(applied.target_entity, Some(entity));
        assert_eq!(
            applied.target_source,
            SceneBootstrapTargetSource::Sole3dEntity
        );
        assert_eq!(
            applied.controlled_entity_applied,
            BootstrapApplyState::NotRequested
        );
        assert_eq!(
            applied.reference_frame_applied,
            BootstrapApplyState::Applied
        );
        assert_eq!(applied.gravity_applied, BootstrapApplyState::Applied);
        assert_eq!(applied.atmosphere_applied, BootstrapApplyState::Applied);
        assert_eq!(gameplay.controlled_entity(), None);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "celestial bindings require `world-model: celestial-3d`")]
    fn bootstrap_rejects_celestial_binding_in_euclidean_scene() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: mismatch
title: Mismatch
world-model: euclidean-3d
celestial:
  focus-body: earth
layers: []
"#,
        )
        .expect("scene parse");

        let _ = ResolvedSceneBootstrap::from_scene(&scene);
    }
}

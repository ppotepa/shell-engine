//! ScriptGameplayApi and ScriptGameplayEntityApi implementation - large standalone module.
//! This module contains the full impl blocks extracted from lib.rs.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use engine_celestial::{find_planet_spawn_from_params, BodyPatch, WorldPoint3};
use engine_core::scene::model::{RuntimeObjectDocument, RuntimeObjectTransform};
use engine_terrain::{Biome, PlanetGenParams};
use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Map as RhaiMap};
use serde_json::{Map as JsonMap, Value as JsonValue};

use engine_api::commands::planet_apply_spec_from_rhai_map;
use engine_api::{
    filter_hits_by_kind, filter_hits_of_kind, follow_anchor_from_args, is_ephemeral_lifecycle,
    map_int, map_number, map_string, parse_lifecycle_policy, Camera3dMutationRequest, EmitResolved,
    EphemeralPrefabResolved, SceneMutationRequest, ScriptEntityContext, ScriptWorldContext,
};
use engine_game::components::{
    AngularMotor3D, AngularMotorMode, Assembly3D, AtmosphereAffected2D, AttachmentBundle3D,
    BootstrapAssembly3D, CharacterMotor3D, CharacterUpMode, ControlBundle3D, DespawnVisual,
    FollowAnchor3D, GravityAffected2D, GravityMode2D, LifecyclePolicy, LinearMotor3D,
    MotorBundle3D, MotorSpace, ParticleColorRamp, ParticlePhysics, ParticleThreadMode,
    ReferenceFrameBinding3D, ReferenceFrameMode, SpatialBundle3D, Transform3D,
};
use engine_game::{
    Collider2D, ColliderShape, CollisionHit, GameplayWorld, Lifetime, PhysicsBody2D, Transform2D,
    VisualBinding,
};

use engine_persistence::PersistenceStore;

use crate::palette::PaletteStore;
use crate::rhai_util::{json_to_rhai_dynamic, rhai_dynamic_to_json};
use crate::scripting::ephemeral::{spawn_ephemeral_visual, EphemeralSpawn};
use crate::scripting::physics::ScriptEntityPhysicsApi;
use crate::scripting::vehicle::{
    angular_body_from_rhai_map, attach_vehicle_stack, linear_brake_from_rhai_map,
    thruster_ramp_from_rhai_map,
};
use crate::{catalog, BehaviorCommand, EmitterState};

fn queue_set_property_or_mutation(
    queue: &mut Vec<BehaviorCommand>,
    target: String,
    path: String,
    value: JsonValue,
) {
    if let Some(request) =
        engine_api::commands::scene_mutation_request_from_set_path(&target, &path, &value, None)
    {
        queue.push(BehaviorCommand::ApplySceneMutation { request });
    }
}

fn merge_rhai_dynamic(base: RhaiDynamic, overlay: RhaiDynamic) -> RhaiDynamic {
    match (
        base.clone().try_cast::<RhaiMap>(),
        overlay.clone().try_cast::<RhaiMap>(),
    ) {
        (Some(base_map), Some(overlay_map)) => {
            RhaiDynamic::from(merge_rhai_maps(base_map, &overlay_map))
        }
        _ => overlay,
    }
}

fn merge_rhai_maps(mut base: RhaiMap, overlay: &RhaiMap) -> RhaiMap {
    for (key, overlay_value) in overlay {
        let merged = match base.remove(key.as_str()) {
            Some(base_value) => merge_rhai_dynamic(base_value, overlay_value.clone()),
            None => overlay_value.clone(),
        };
        base.insert(key.clone(), merged);
    }
    base
}

fn merge_json_values(base: JsonValue, overlay: JsonValue) -> JsonValue {
    match (base, overlay) {
        (JsonValue::Object(mut base_map), JsonValue::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                let merged = match base_map.remove(&key) {
                    Some(base_value) => merge_json_values(base_value, overlay_value),
                    None => overlay_value,
                };
                base_map.insert(key, merged);
            }
            JsonValue::Object(base_map)
        }
        (_, overlay_value) => overlay_value,
    }
}

fn merge_json_map(
    mut base: BTreeMap<String, JsonValue>,
    overlay: &RhaiMap,
    skip_keys: &[&str],
) -> BTreeMap<String, JsonValue> {
    for (key, overlay_value) in overlay {
        if skip_keys.iter().any(|skip| *skip == key.as_str()) {
            continue;
        }
        let Some(overlay_json) = rhai_dynamic_to_json(overlay_value) else {
            continue;
        };
        let merged = match base.remove(key.as_str()) {
            Some(base_value) => merge_json_values(base_value, overlay_json),
            None => overlay_json,
        };
        base.insert(key.to_string(), merged);
    }
    base
}

fn normalize_token(value: Option<&str>) -> String {
    value
        .unwrap_or_default()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

fn reference_frame_mode_from_str(value: Option<&str>) -> ReferenceFrameMode {
    match normalize_token(value).as_str() {
        "parententity" => ReferenceFrameMode::ParentEntity,
        "celestialbody" => ReferenceFrameMode::CelestialBody,
        "localhorizon" => ReferenceFrameMode::LocalHorizon,
        "orbital" => ReferenceFrameMode::Orbital,
        _ => ReferenceFrameMode::World,
    }
}

fn motor_space_from_str(value: Option<&str>) -> MotorSpace {
    match normalize_token(value).as_str() {
        "world" => MotorSpace::World,
        "referenceframe" => MotorSpace::ReferenceFrame,
        _ => MotorSpace::Local,
    }
}

fn angular_motor_mode_from_str(value: Option<&str>) -> AngularMotorMode {
    match normalize_token(value).as_str() {
        "torque" => AngularMotorMode::Torque,
        _ => AngularMotorMode::Rate,
    }
}

fn character_up_mode_from_str(value: Option<&str>) -> CharacterUpMode {
    match normalize_token(value).as_str() {
        "surfacenormal" => CharacterUpMode::SurfaceNormal,
        "referenceframeup" => CharacterUpMode::ReferenceFrameUp,
        _ => CharacterUpMode::WorldUp,
    }
}

fn prefab_spawn_args(prefab: &catalog::PrefabTemplate, args: &RhaiMap) -> RhaiMap {
    let mut merged = args.clone();
    if let Some(transform) = &prefab.transform {
        if let Some(value) = transform.x {
            merged
                .entry("x".into())
                .or_insert_with(|| (value as rhai::FLOAT).into());
        }
        if let Some(value) = transform.y {
            merged
                .entry("y".into())
                .or_insert_with(|| (value as rhai::FLOAT).into());
        }
        if let Some(value) = transform.heading {
            merged
                .entry("heading".into())
                .or_insert_with(|| (value as rhai::FLOAT).into());
        }
        if let Some(value) = transform.z {
            merged
                .entry("z".into())
                .or_insert_with(|| (value as rhai::FLOAT).into());
        }
        if let Some(value) = transform.pitch {
            merged
                .entry("pitch".into())
                .or_insert_with(|| (value as rhai::FLOAT).into());
        }
        if let Some(value) = transform.roll {
            merged
                .entry("roll".into())
                .or_insert_with(|| (value as rhai::FLOAT).into());
        }
        if let Some(value) = transform.scale_x {
            merged
                .entry("scale_x".into())
                .or_insert_with(|| (value as rhai::FLOAT).into());
        }
        if let Some(value) = transform.scale_y {
            merged
                .entry("scale_y".into())
                .or_insert_with(|| (value as rhai::FLOAT).into());
        }
        if let Some(value) = transform.scale_z {
            merged
                .entry("scale_z".into())
                .or_insert_with(|| (value as rhai::FLOAT).into());
        }
    }
    merged
}

fn prefab_lifecycle_name(prefab: &catalog::PrefabTemplate, args: &RhaiMap) -> Option<String> {
    match map_string(args, "lifecycle") {
        Some(name) if !name.trim().is_empty() => Some(name),
        _ => prefab
            .components
            .as_ref()
            .and_then(|components| components.lifecycle.clone()),
    }
}

fn yaml_value_to_json(value: &serde_yaml::Value) -> Option<JsonValue> {
    serde_json::to_value(value).ok()
}

fn yaml_value_to_rhai_dynamic(value: &serde_yaml::Value) -> Option<RhaiDynamic> {
    yaml_value_to_json(value).map(|json| json_to_rhai_dynamic(&json))
}

fn runtime_object_transform_args(transform: &RuntimeObjectTransform) -> RhaiMap {
    let mut args = RhaiMap::new();
    match transform {
        RuntimeObjectTransform::TwoD {
            x, y, rotation_deg, ..
        } => {
            args.insert("x".into(), (*x as rhai::FLOAT).into());
            args.insert("y".into(), (*y as rhai::FLOAT).into());
            args.insert(
                "heading".into(),
                ((*rotation_deg as f64).to_radians() as rhai::FLOAT).into(),
            );
        }
        RuntimeObjectTransform::ThreeD {
            translation,
            rotation_deg,
            ..
        }
        | RuntimeObjectTransform::Celestial {
            translation,
            rotation_deg,
            ..
        } => {
            args.insert("x".into(), (translation[0] as rhai::FLOAT).into());
            args.insert("y".into(), (translation[1] as rhai::FLOAT).into());
            args.insert("z".into(), (translation[2] as rhai::FLOAT).into());
            args.insert(
                "pitch".into(),
                ((rotation_deg[0] as f64).to_radians() as rhai::FLOAT).into(),
            );
            args.insert(
                "heading".into(),
                ((rotation_deg[1] as f64).to_radians() as rhai::FLOAT).into(),
            );
            args.insert(
                "roll".into(),
                ((rotation_deg[2] as f64).to_radians() as rhai::FLOAT).into(),
            );
        }
    }
    args
}

fn euler_deg_xyz_to_quat(rotation_deg: [f32; 3]) -> [f32; 4] {
    let [x_deg, y_deg, z_deg] = rotation_deg;
    let (sx, cx) = (x_deg.to_radians() * 0.5).sin_cos();
    let (sy, cy) = (y_deg.to_radians() * 0.5).sin_cos();
    let (sz, cz) = (z_deg.to_radians() * 0.5).sin_cos();
    [
        sx * cy * cz - cx * sy * sz,
        cx * sy * cz + sx * cy * sz,
        cx * cy * sz - sx * sy * cz,
        cx * cy * cz + sx * sy * sz,
    ]
}

fn runtime_object_transform3d(transform: &RuntimeObjectTransform) -> Option<Transform3D> {
    match transform {
        RuntimeObjectTransform::TwoD { .. } => None,
        RuntimeObjectTransform::ThreeD {
            translation,
            rotation_deg,
            ..
        }
        | RuntimeObjectTransform::Celestial {
            translation,
            rotation_deg,
            ..
        } => Some(Transform3D {
            position: *translation,
            orientation: euler_deg_xyz_to_quat(*rotation_deg),
        }),
    }
}

fn runtime_object_component_families_json(node: &RuntimeObjectDocument) -> JsonValue {
    let mut families = serde_json::to_value(&node.components)
        .unwrap_or_else(|_| JsonValue::Object(JsonMap::new()));
    if let Some(overlay) = node
        .overrides
        .get("components")
        .and_then(yaml_value_to_json)
    {
        families = merge_json_values(families, overlay);
    }
    families
}

fn canonicalize_runtime_object_family_json(value: JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => JsonValue::Object(
            map.into_iter()
                .map(|(key, value)| {
                    (
                        key.replace('-', "_"),
                        canonicalize_runtime_object_family_json(value),
                    )
                })
                .collect(),
        ),
        JsonValue::Array(values) => JsonValue::Array(
            values
                .into_iter()
                .map(canonicalize_runtime_object_family_json)
                .collect(),
        ),
        other => other,
    }
}

fn runtime_object_prefab_components(
    node: &RuntimeObjectDocument,
) -> Option<catalog::PrefabComponents> {
    let families = runtime_object_component_families_json(node);
    prefab_components_from_runtime_object_families(&families)
}

fn runtime_object_args(node: &RuntimeObjectDocument, owner_id: Option<u64>) -> RhaiMap {
    let mut args = runtime_object_transform_args(&node.transform);
    for (key, value) in &node.overrides {
        if key == "components" {
            continue;
        }
        let Some(value) = yaml_value_to_rhai_dynamic(value) else {
            continue;
        };
        let merged = match args.remove(key.as_str()) {
            Some(existing) => merge_rhai_dynamic(existing, value),
            None => value,
        };
        args.insert(key.clone().into(), merged);
    }

    if let Some(owner_id) = owner_id {
        args.entry("owner_id".into())
            .or_insert_with(|| (owner_id as rhai::INT).into());
        args.entry("inherit_owner_lifecycle".into())
            .or_insert_with(|| true.into());
    }

    if !args.contains_key("lifecycle") {
        if let Some(lifecycle) =
            runtime_object_prefab_components(node).and_then(|components| components.lifecycle)
        {
            args.insert("lifecycle".into(), lifecycle.into());
        }
    }

    args
}

fn runtime_object_bootstrap_assembly3d_from_document(
    node: &RuntimeObjectDocument,
    args: &RhaiMap,
) -> Option<BootstrapAssembly3D> {
    let mut bootstrap = runtime_object_prefab_components(node)
        .and_then(|components| prefab_bootstrap_assembly3d_from_components(&components, args))
        .unwrap_or_default();
    if let Some(transform) = runtime_object_transform3d(&node.transform) {
        bootstrap.assembly.spatial.transform = Some(transform);
    }
    (!bootstrap.is_empty()).then_some(bootstrap)
}

fn runtime_object_spawn_kind(node: &RuntimeObjectDocument) -> String {
    node.kind
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("runtime-object")
        .to_string()
}

fn runtime_object_component_prefab(node: &RuntimeObjectDocument) -> Option<catalog::PrefabTemplate> {
    Some(catalog::PrefabTemplate {
        kind: runtime_object_spawn_kind(node),
        sprite_template: None,
        transform: None,
        init_fields: Default::default(),
        components: Some(runtime_object_prefab_components(node)?),
        fg_colour: None,
        default_tags: Vec::new(),
    })
}

fn apply_runtime_object_owner_lifecycle(
    world: &GameplayWorld,
    entity_id: u64,
    args: &RhaiMap,
) -> bool {
    let owner_id = map_int(args, "owner_id", 0);
    let inherit_owner_lifecycle = args
        .get("inherit_owner_lifecycle")
        .and_then(|value| value.clone().try_cast::<bool>())
        .unwrap_or(false);
    let lifecycle = map_string(args, "lifecycle")
        .filter(|value| !value.trim().is_empty())
        .map(|name| parse_lifecycle_policy(&name, LifecyclePolicy::Persistent));

    if owner_id > 0 && !world.register_child(owner_id as u64, entity_id) {
        return false;
    }

    if let Some(policy) = lifecycle {
        return world.set_lifecycle(entity_id, policy);
    }

    if inherit_owner_lifecycle && owner_id > 0 {
        let inherited = world
            .lifecycle(owner_id as u64)
            .unwrap_or(LifecyclePolicy::FollowOwner);
        return world.set_lifecycle(entity_id, inherited);
    }

    true
}

/// Runtime-owned bridge seam for a single materialized `runtime-object` node.
///
/// This binds a gameplay entity to an already-materialized scene-runtime target
/// instead of asking the scene runtime to spawn a fresh clone. Callers own any
/// parent/child recursion and idempotence checks.
pub fn bridge_runtime_object_node_to_gameplay(
    world: GameplayWorld,
    catalogs: Arc<catalog::ModCatalogs>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    palette_store: Arc<PaletteStore>,
    palette_default_id: Option<String>,
    spatial_meters_per_world_unit: Option<f64>,
    node: &RuntimeObjectDocument,
    runtime_target: &str,
    owner_id: Option<u64>,
) -> rhai::INT {
    let runtime_target = runtime_target.trim();
    if runtime_target.is_empty() {
        return 0;
    }

    let empty_hits: Arc<Vec<CollisionHit>> = Arc::new(Vec::new());
    let mut api = ScriptGameplayApi::new(
        Some(world),
        Arc::clone(&empty_hits),
        Arc::clone(&empty_hits),
        Arc::clone(&empty_hits),
        empty_hits,
        spatial_meters_per_world_unit,
        catalogs,
        None,
        queue,
        palette_store,
        None,
        palette_default_id,
    );
    api.spawn_runtime_object_node_bound_to_visual(node, runtime_target, owner_id)
}

fn prefab_assembly3d_from_components(components: &catalog::PrefabComponents) -> Option<Assembly3D> {
    let assembly = Assembly3D {
        spatial: SpatialBundle3D::default(),
        control: ControlBundle3D {
            control_intent: None,
            reference_frame: components.reference_frame.as_ref().map(|frame| {
                ReferenceFrameBinding3D {
                    mode: reference_frame_mode_from_str(frame.mode.as_deref()),
                    entity_id: frame.entity_id,
                    body_id: frame.body_id.clone(),
                    inherit_linear_velocity: frame.inherit_linear_velocity.unwrap_or(false),
                    inherit_angular_velocity: frame.inherit_angular_velocity.unwrap_or(false),
                }
            }),
            reference_frame_state: None,
        },
        attachments: AttachmentBundle3D {
            follow_anchor: components
                .follow_anchor_3d
                .as_ref()
                .map(|follow| FollowAnchor3D {
                    local_offset: follow
                        .local_offset
                        .unwrap_or([0.0, 0.0, 0.0])
                        .map(|value| value as f32),
                    inherit_orientation: follow.inherit_orientation.unwrap_or(true),
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

    (!assembly.is_empty()).then_some(assembly)
}

#[cfg_attr(not(test), allow(dead_code))]
fn prefab_components_from_runtime_object_families(
    value: &JsonValue,
) -> Option<catalog::PrefabComponents> {
    serde_json::from_value::<catalog::PrefabComponents>(canonicalize_runtime_object_family_json(
        value.clone(),
    ))
    .ok()
}

fn prefab_bootstrap_assembly3d_from_components(
    components: &catalog::PrefabComponents,
    args: &RhaiMap,
) -> Option<BootstrapAssembly3D> {
    let assembly = prefab_assembly3d_from_components(components).unwrap_or_default();
    let owner_id = map_int(args, "owner_id", 0);
    let inherit_owner_lifecycle = args
        .get("inherit_owner_lifecycle")
        .and_then(|value| value.clone().try_cast::<bool>())
        .unwrap_or(false);
    let lifecycle_name = {
        let override_name = map_string(args, "lifecycle");
        if override_name.as_deref().unwrap_or("").trim().is_empty() {
            components.lifecycle.clone()
        } else {
            override_name
        }
    };
    let lifecycle = lifecycle_name
        .as_deref()
        .map(|name| parse_lifecycle_policy(name, LifecyclePolicy::Persistent));

    if assembly.is_empty() && owner_id <= 0 && !inherit_owner_lifecycle && lifecycle.is_none() {
        return None;
    }

    Some(BootstrapAssembly3D {
        assembly,
        controlled: false,
        owner_id: (owner_id > 0).then_some(owner_id as u64),
        inherit_owner_lifecycle,
        lifecycle,
    })
}

// ── Struct Definitions ───────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptGameplayApi {
    pub(crate) ctx: ScriptWorldContext,
    pub(crate) catalogs: Arc<catalog::ModCatalogs>,
    pub(crate) emitter_state: Option<EmitterState>,
    pub(crate) palette_store: Arc<PaletteStore>,
    pub(crate) palette_persistence: Option<PersistenceStore>,
    pub(crate) palette_default_id: Option<String>,
    /// Per-frame cache: (kind_a, kind_b) → filtered collision result.
    /// Populated on first `collision.enters(a, b)` call; reused if called again
    /// with the same pair in the same frame. Cleared when the struct is re-created.
    collision_enters_cache: std::collections::HashMap<(String, String), RhaiArray>,
}

#[derive(Clone)]
pub(crate) struct ScriptGameplayEntityApi {
    pub(crate) ctx: ScriptEntityContext,
    pub(crate) physics: ScriptEntityPhysicsApi,
}

#[derive(Clone)]
pub(crate) struct ScriptGameplayObjectsApi {
    pub(crate) world: Option<GameplayWorld>,
}

#[derive(Clone)]
pub(crate) struct ScriptGameplayObjectApi {
    pub(crate) world: Option<GameplayWorld>,
    pub(crate) id: u64,
}

#[derive(Clone, Default)]
pub(crate) struct ScriptGameplayBodySnapshotApi {
    pub(crate) id: String,
    pub(crate) body: Option<catalog::BodyDef>,
    pub(crate) spatial_meters_per_world_unit: Option<f64>,
}

// ── ScriptGameplayApi Implementation ──────────────────────────────────────
impl ScriptGameplayApi {
    fn map_patch_value<'a>(patch: &'a RhaiMap, keys: &[&str]) -> Option<&'a RhaiDynamic> {
        keys.iter().find_map(|key| patch.get(*key))
    }

    fn dynamic_to_bool(value: &RhaiDynamic) -> Option<bool> {
        value
            .clone()
            .try_cast::<bool>()
            .or_else(|| value.clone().try_cast::<rhai::INT>().map(|i| i != 0))
            .or_else(|| value.clone().try_cast::<rhai::FLOAT>().map(|f| f != 0.0))
            .or_else(|| {
                let normalized = value.clone().try_cast::<String>()?;
                match normalized.trim().to_ascii_lowercase().as_str() {
                    "1" | "true" | "yes" | "on" => Some(true),
                    "0" | "false" | "no" | "off" => Some(false),
                    _ => None,
                }
            })
    }

    fn dynamic_to_number(value: &RhaiDynamic) -> Option<f64> {
        value
            .clone()
            .try_cast::<rhai::FLOAT>()
            .map(|f| f as f64)
            .or_else(|| value.clone().try_cast::<rhai::INT>().map(|i| i as f64))
            .or_else(|| {
                value
                    .clone()
                    .try_cast::<String>()
                    .and_then(|s| s.trim().parse::<f64>().ok())
            })
    }

    fn map_patch_bool_any(patch: &RhaiMap, keys: &[&str]) -> Option<bool> {
        Self::map_patch_value(patch, keys).and_then(Self::dynamic_to_bool)
    }

    fn map_patch_u64_any(patch: &RhaiMap, keys: &[&str]) -> Option<u64> {
        Self::map_patch_value(patch, keys)
            .and_then(Self::dynamic_to_number)
            .map(|value| value.max(0.0) as u64)
    }

    fn map_patch_u8_any(patch: &RhaiMap, keys: &[&str]) -> Option<u8> {
        Self::map_patch_value(patch, keys)
            .and_then(Self::dynamic_to_number)
            .map(|value| value.clamp(0.0, u8::MAX as f64) as u8)
    }

    fn biome_from_label(label: &str) -> Option<Biome> {
        match label.trim().to_ascii_lowercase().as_str() {
            "ocean" => Some(Biome::Ocean),
            "shallow" | "shallow_water" | "shallow-water" => Some(Biome::ShallowWater),
            "beach" => Some(Biome::Beach),
            "desert" => Some(Biome::Desert),
            "grass" | "grassland" => Some(Biome::Grassland),
            "forest" => Some(Biome::Forest),
            "tundra" | "cold" => Some(Biome::Tundra),
            "snow" | "ice" => Some(Biome::Snow),
            "mountain" => Some(Biome::Mountain),
            "volcanic" | "volcano" => Some(Biome::Volcanic),
            _ => None,
        }
    }

    fn biome_label(biome: Biome) -> &'static str {
        match biome {
            Biome::Ocean => "ocean",
            Biome::ShallowWater => "shallow",
            Biome::Beach => "beach",
            Biome::Desert => "desert",
            Biome::Grassland => "grassland",
            Biome::Forest => "forest",
            Biome::Tundra => "tundra",
            Biome::Snow => "snow",
            Biome::Mountain => "mountain",
            Biome::Volcanic => "volcanic",
        }
    }

    fn displacement_scale_from_rhai_map(config: &RhaiMap) -> f32 {
        Self::map_patch_number_any(
            config,
            &["disp", "displacement_scale", "displacement-scale"],
        )
        .unwrap_or(0.22)
        .clamp(0.0, 1.0) as f32
    }

    fn planet_gen_params_from_rhai_map(config: &RhaiMap) -> PlanetGenParams {
        let mut params = PlanetGenParams::default();
        if let Some(seed) = Self::map_patch_u64_any(config, &["seed"]) {
            params.seed = seed;
        }
        if let Some(has_ocean) = Self::map_patch_bool_any(config, &["has_ocean", "has-ocean"]) {
            params.has_ocean = has_ocean;
        }
        if let Some(ocean) =
            Self::map_patch_number_any(config, &["ocean", "ocean_fraction", "ocean-fraction"])
        {
            params.ocean_fraction = ocean.clamp(0.0, 1.0);
        }
        if let Some(v) =
            Self::map_patch_number_any(config, &["cscale", "continent_scale", "continent-scale"])
        {
            params.continent_scale = v.clamp(0.5, 10.0);
        }
        if let Some(v) =
            Self::map_patch_number_any(config, &["cwarp", "continent_warp", "continent-warp"])
        {
            params.continent_warp = v.clamp(0.0, 2.0);
        }
        if let Some(v) =
            Self::map_patch_u8_any(config, &["coct", "continent_octaves", "continent-octaves"])
        {
            params.continent_octaves = v.clamp(2, 8);
        }
        if let Some(v) =
            Self::map_patch_number_any(config, &["mscale", "mountain_scale", "mountain-scale"])
        {
            params.mountain_scale = v.clamp(1.0, 20.0);
        }
        if let Some(v) =
            Self::map_patch_number_any(config, &["mstr", "mountain_strength", "mountain-strength"])
        {
            params.mountain_strength = v.clamp(0.0, 1.0);
        }
        if let Some(v) = Self::map_patch_u8_any(
            config,
            &["mroct", "mountain_ridge_octaves", "mountain-ridge-octaves"],
        ) {
            params.mountain_ridge_octaves = v.clamp(2, 8);
        }
        if let Some(v) =
            Self::map_patch_number_any(config, &["moisture", "moisture_scale", "moisture-scale"])
        {
            params.moisture_scale = v.clamp(0.5, 10.0);
        }
        if let Some(v) =
            Self::map_patch_number_any(config, &["ice", "ice_cap_strength", "ice-cap-strength"])
        {
            params.ice_cap_strength = v.clamp(0.0, 3.0);
        }
        if let Some(v) = Self::map_patch_number_any(config, &["lapse", "lapse_rate", "lapse-rate"])
        {
            params.lapse_rate = v.clamp(0.0, 1.0);
        }
        if let Some(v) = Self::map_patch_number_any(config, &["rain", "rain_shadow", "rain-shadow"])
        {
            params.rain_shadow = v.clamp(0.0, 1.0);
        }
        params
    }

    fn default_spawn_biomes() -> Vec<Biome> {
        engine_celestial::default_spawn_biomes()
    }

    fn preferred_biomes_from_array(preferred_biomes: RhaiArray) -> Vec<Biome> {
        let mut biomes: Vec<Biome> = preferred_biomes
            .into_iter()
            .filter_map(|entry| entry.try_cast::<String>())
            .filter_map(|label| Self::biome_from_label(&label))
            .collect();
        if biomes.is_empty() {
            biomes = Self::default_spawn_biomes();
        }
        biomes
    }

    fn find_planet_spawn_angle_for_params(
        params: &PlanetGenParams,
        preferred_biomes: &[Biome],
    ) -> rhai::FLOAT {
        find_planet_spawn_from_params(params, 0.22, preferred_biomes).longitude_deg as rhai::FLOAT
    }

    fn find_planet_spawn_for_params(
        params: &PlanetGenParams,
        displacement_scale: f32,
        preferred_biomes: &[Biome],
    ) -> RhaiMap {
        let sample = find_planet_spawn_from_params(params, displacement_scale, preferred_biomes);

        let mut map = RhaiMap::new();
        map.insert("angle_deg".into(), sample.longitude_deg.into());
        map.insert("longitude_deg".into(), sample.longitude_deg.into());
        map.insert("latitude_deg".into(), sample.latitude_deg.into());
        map.insert("row".into(), (sample.row as i64).into());
        map.insert("col".into(), (sample.col as i64).into());
        map.insert("normal_x".into(), sample.normal.x.into());
        map.insert("normal_y".into(), sample.normal.y.into());
        map.insert("normal_z".into(), sample.normal.z.into());
        map.insert(
            "surface_radius_scale".into(),
            sample.surface_radius_scale.into(),
        );
        map.insert("surface_offset".into(), sample.surface_offset.into());
        map.insert("elevation".into(), (sample.elevation as rhai::FLOAT).into());
        map.insert("moisture".into(), (sample.moisture as rhai::FLOAT).into());
        map.insert(
            "temperature".into(),
            (sample.temperature as rhai::FLOAT).into(),
        );
        map.insert(
            "biome".into(),
            sample
                .biome
                .map(Self::biome_label)
                .unwrap_or_default()
                .into(),
        );
        map
    }

    pub(crate) fn find_planet_spawn_angle(
        &mut self,
        config: RhaiMap,
        preferred_biomes: RhaiArray,
    ) -> rhai::FLOAT {
        let params = Self::planet_gen_params_from_rhai_map(&config);
        let preferred = Self::preferred_biomes_from_array(preferred_biomes);
        Self::find_planet_spawn_angle_for_params(&params, &preferred)
    }

    pub(crate) fn find_planet_spawn(
        &mut self,
        config: RhaiMap,
        preferred_biomes: RhaiArray,
    ) -> RhaiMap {
        let params = Self::planet_gen_params_from_rhai_map(&config);
        let preferred = Self::preferred_biomes_from_array(preferred_biomes);
        let displacement_scale = Self::displacement_scale_from_rhai_map(&config);
        Self::find_planet_spawn_for_params(&params, displacement_scale, &preferred)
    }

    fn map_patch_number(patch: &RhaiMap, key: &str) -> Option<f64> {
        Self::map_patch_value(patch, &[key]).and_then(Self::dynamic_to_number)
    }

    fn map_patch_number_any(patch: &RhaiMap, keys: &[&str]) -> Option<f64> {
        Self::map_patch_value(patch, keys).and_then(Self::dynamic_to_number)
    }

    fn map_patch_string(patch: &RhaiMap, key: &str) -> Option<String> {
        patch.get(key).and_then(|v| v.clone().try_cast::<String>())
    }

    fn map_patch_opt_number(patch: &RhaiMap, key: &str) -> Option<Option<f64>> {
        let value = patch.get(key)?;
        if value.is_unit() {
            return Some(None);
        }
        Self::map_patch_number(patch, key).map(Some)
    }

    fn map_patch_opt_string(patch: &RhaiMap, key: &str) -> Option<Option<String>> {
        let value = patch.get(key)?;
        if value.is_unit() {
            return Some(None);
        }
        Self::map_patch_string(patch, key).map(Some)
    }

    fn apply_body_patch(body: &mut catalog::BodyDef, patch: &RhaiMap) {
        body.apply_patch(&BodyPatch {
            planet_type: Self::map_patch_opt_string(patch, "planet_type"),
            center_x: Self::map_patch_number(patch, "center_x"),
            center_y: Self::map_patch_number(patch, "center_y"),
            parent: Self::map_patch_opt_string(patch, "parent"),
            orbit_radius: Self::map_patch_number(patch, "orbit_radius"),
            orbit_period_sec: Self::map_patch_number(patch, "orbit_period_sec"),
            orbit_phase_deg: Self::map_patch_number(patch, "orbit_phase_deg"),
            radius_px: Self::map_patch_number(patch, "radius_px"),
            radius_km: Self::map_patch_opt_number(patch, "radius_km"),
            km_per_px: Self::map_patch_opt_number(patch, "km_per_px"),
            gravity_mu: Self::map_patch_number(patch, "gravity_mu"),
            gravity_mu_km3_s2: Self::map_patch_opt_number(patch, "gravity_mu_km3_s2")
                .or_else(|| Self::map_patch_opt_number(patch, "gravity-mu-km3-s2")),
            surface_radius: Self::map_patch_number(patch, "surface_radius"),
            atmosphere_top: Self::map_patch_opt_number(patch, "atmosphere_top"),
            atmosphere_dense_start: Self::map_patch_opt_number(patch, "atmosphere_dense_start"),
            atmosphere_drag_max: Self::map_patch_opt_number(patch, "atmosphere_drag_max"),
            atmosphere_top_km: Self::map_patch_opt_number(patch, "atmosphere_top_km"),
            atmosphere_dense_start_km: Self::map_patch_opt_number(
                patch,
                "atmosphere_dense_start_km",
            ),
            cloud_bottom_km: Self::map_patch_opt_number(patch, "cloud_bottom_km"),
            cloud_top_km: Self::map_patch_opt_number(patch, "cloud_top_km"),
        });
    }

    pub(crate) fn body_upsert(&mut self, id: &str, patch: RhaiMap) -> bool {
        let body_id = id.trim();
        if body_id.is_empty() {
            return false;
        }
        let catalogs = Arc::make_mut(&mut self.catalogs);
        let body = catalogs
            .celestial
            .bodies
            .entry(body_id.to_string())
            .or_default();
        Self::apply_body_patch(body, &patch);
        true
    }

    pub(crate) fn body_patch(&mut self, id: &str, patch: RhaiMap) -> bool {
        self.body_upsert(id, patch)
    }

    pub(crate) fn apply_planet_spec(
        &mut self,
        target: &str,
        body_id: &str,
        spec_map: RhaiMap,
    ) -> bool {
        let target = target.trim();
        let body_id = body_id.trim();
        if target.is_empty() || body_id.is_empty() {
            return false;
        }
        let Some(spec) = planet_apply_spec_from_rhai_map(spec_map) else {
            return false;
        };
        if spec.is_empty() {
            return false;
        }
        let Ok(mut commands) = self.ctx.queue.lock() else {
            return false;
        };
        commands.push(BehaviorCommand::ApplyPlanetSpec {
            target: target.to_string(),
            body_id: body_id.to_string(),
            spec,
        });
        true
    }

    pub(crate) fn map_number(args: &RhaiMap, key: &str, fallback: rhai::FLOAT) -> rhai::FLOAT {
        map_number(args, key, fallback)
    }

    pub(crate) fn map_int(args: &RhaiMap, key: &str, fallback: rhai::INT) -> rhai::INT {
        map_int(args, key, fallback)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        world: Option<GameplayWorld>,
        collisions: std::sync::Arc<Vec<CollisionHit>>,
        collision_enters: std::sync::Arc<Vec<CollisionHit>>,
        collision_stays: std::sync::Arc<Vec<CollisionHit>>,
        collision_exits: std::sync::Arc<Vec<CollisionHit>>,
        spatial_meters_per_world_unit: Option<f64>,
        catalogs: Arc<catalog::ModCatalogs>,
        emitter_state: Option<EmitterState>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
        palette_store: Arc<PaletteStore>,
        palette_persistence: Option<PersistenceStore>,
        palette_default_id: Option<String>,
    ) -> Self {
        Self {
            ctx: ScriptWorldContext::new(
                world,
                collisions,
                collision_enters,
                collision_stays,
                collision_exits,
                spatial_meters_per_world_unit,
                queue,
            ),
            catalogs,
            emitter_state,
            palette_store,
            palette_persistence,
            palette_default_id,
            collision_enters_cache: std::collections::HashMap::new(),
        }
    }

    pub(crate) fn entity(&mut self, id: rhai::INT) -> ScriptGameplayEntityApi {
        let id_u64 = if id < 0 { 0 } else { id as u64 };
        let world = self.ctx.world.clone();
        ScriptGameplayEntityApi {
            physics: ScriptEntityPhysicsApi::new(world.clone(), id_u64),
            ctx: ScriptEntityContext::new(world, id_u64, Arc::clone(&self.ctx.queue)),
        }
    }

    pub(crate) fn objects(&mut self) -> ScriptGameplayObjectsApi {
        ScriptGameplayObjectsApi {
            world: self.ctx.world.clone(),
        }
    }

    pub(crate) fn body(&mut self, id: &str) -> ScriptGameplayBodySnapshotApi {
        let id = id.trim();
        let body = self.catalogs.celestial.bodies.get(id).cloned();
        ScriptGameplayBodySnapshotApi::new(
            if body.is_some() {
                id.to_string()
            } else {
                String::new()
            },
            body,
            self.ctx.spatial_meters_per_world_unit,
        )
    }

    pub(crate) fn clear(&mut self) {
        if let Some(world) = self.ctx.world.as_ref() {
            world.clear();
        }
    }

    /// Look up a particle ramp from the active palette by name.
    /// Returns None if no palette is active or the ramp name is not found.
    fn resolve_palette_ramp(&self, ramp_name: &str) -> Option<Vec<String>> {
        let persisted = self
            .palette_persistence
            .as_ref()
            .and_then(|p| p.get("/__palette__"))
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        self.palette_store
            .resolve(persisted.as_deref(), self.palette_default_id.as_deref())
            .and_then(|palette| palette.particles.get(ramp_name).cloned())
    }

    pub(crate) fn count(&mut self) -> rhai::INT {
        self.ctx
            .world
            .as_ref()
            .map(|world| world.count() as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn count_kind(&mut self, kind: &str) -> rhai::INT {
        self.ctx
            .world
            .as_ref()
            .map(|world| world.count_kind(kind) as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn count_tag(&mut self, tag: &str) -> rhai::INT {
        self.ctx
            .world
            .as_ref()
            .map(|world| world.count_tag(tag) as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn first_kind(&mut self, kind: &str) -> rhai::INT {
        self.ctx
            .world
            .as_ref()
            .and_then(|world| world.first_kind(kind))
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn first_tag(&mut self, tag: &str) -> rhai::INT {
        self.ctx
            .world
            .as_ref()
            .and_then(|world| world.first_tag(tag))
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn set_controlled_entity(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_controlled_entity(id as u64)
    }

    pub(crate) fn controlled_entity(&mut self) -> rhai::INT {
        self.ctx
            .world
            .as_ref()
            .and_then(|world| world.controlled_entity())
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn clear_controlled_entity(&mut self) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.clear_controlled_entity()
    }

    /// Returns a Rhai map with diagnostic info about current entity counts.
    /// Useful for tracking object growth: { total: N, by_kind: { ... }, by_policy: { ... } }
    pub(crate) fn diagnostic_info(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let snapshot = world.diagnostic_snapshot();
        let mut result = RhaiMap::new();
        result.insert("total".into(), (snapshot.total as i64).into());

        let mut by_kind = RhaiMap::new();
        for (kind, count) in snapshot.by_kind {
            by_kind.insert(kind.into(), (count as i64).into());
        }
        result.insert("by_kind".into(), by_kind.into());

        let mut by_policy = RhaiMap::new();
        for (policy, count) in snapshot.by_policy {
            by_policy.insert(policy.into(), (count as i64).into());
        }
        result.insert("by_policy".into(), by_policy.into());

        result
    }

    pub(crate) fn spawn(&mut self, kind: &str, payload: RhaiDynamic) -> rhai::INT {
        let Some(world) = self.ctx.world.clone() else {
            return 0;
        };
        let Some(payload) = rhai_dynamic_to_json(&payload) else {
            return 0;
        };
        world
            .spawn(kind, payload)
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn despawn(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let uid = id as u64;
        let tree_ids = world.despawn_tree_ids(uid);
        if let Ok(mut commands) = self.ctx.queue.lock() {
            for tree_id in &tree_ids {
                if let Some(binding) = world.visual(*tree_id) {
                    for vid in binding.all_visual_ids() {
                        commands.push(BehaviorCommand::ApplySceneMutation {
                            request: SceneMutationRequest::DespawnObject {
                                target: vid.to_string(),
                            },
                        });
                    }
                }
            }
        }
        world.despawn(uid)
    }

    /// Cleanup-aware reset that despawns all dynamic entities and their visuals,
    /// unlike raw `clear()` which only wipes the store.
    pub(crate) fn reset_dynamic_entities(&mut self) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let all_ids = world.ids();
        if let Ok(mut commands) = self.ctx.queue.lock() {
            for id in &all_ids {
                if let Some(binding) = world.visual(*id) {
                    for vid in binding.all_visual_ids() {
                        commands.push(BehaviorCommand::ApplySceneMutation {
                            request: SceneMutationRequest::DespawnObject {
                                target: vid.to_string(),
                            },
                        });
                    }
                }
            }
        }
        world.clear();
        true
    }

    pub(crate) fn bind_visual(&mut self, id: rhai::INT, visual_id: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 || visual_id.trim().is_empty() {
            return false;
        }
        world.add_visual(id as u64, visual_id.to_string())
    }

    pub(crate) fn exists(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.exists(id as u64)
    }

    pub(crate) fn kind(&mut self, id: rhai::INT) -> String {
        let Some(world) = self.ctx.world.as_ref() else {
            return String::new();
        };
        if id < 0 {
            return String::new();
        }
        world.kind_of(id as u64).unwrap_or_default()
    }

    pub(crate) fn tags(&mut self, id: rhai::INT) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        if id < 0 {
            return RhaiArray::new();
        }
        world.tags(id as u64).into_iter().map(Into::into).collect()
    }

    pub(crate) fn ids(&mut self) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .ids()
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn query_kind(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_kind(kind)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn query_tag(&mut self, tag: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_tag(tag)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn query_circle(
        &mut self,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        radius: rhai::FLOAT,
    ) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_circle(x as f32, y as f32, radius as f32)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn query_rect(
        &mut self,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        w: rhai::FLOAT,
        h: rhai::FLOAT,
    ) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_rect(x as f32, y as f32, w as f32, h as f32)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn query_nearest(
        &mut self,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        max_dist: rhai::FLOAT,
    ) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        world
            .query_nearest(x as f32, y as f32, max_dist as f32)
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn query_nearest_kind(
        &mut self,
        kind: &str,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        max_dist: rhai::FLOAT,
    ) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        world
            .query_nearest_kind(kind, x as f32, y as f32, max_dist as f32)
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn get(&mut self, id: rhai::INT, path: &str) -> RhaiDynamic {
        let Some(world) = self.ctx.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        world
            .get(id as u64, path)
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    pub(crate) fn set(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.set(id as u64, path, value)
    }

    pub(crate) fn has(&mut self, id: rhai::INT, path: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.has(id as u64, path)
    }

    pub(crate) fn remove(&mut self, id: rhai::INT, path: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.remove(id as u64, path)
    }

    pub(crate) fn push(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.push(id as u64, path, value)
    }

    pub(crate) fn set_transform(
        &mut self,
        id: rhai::INT,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        heading: rhai::FLOAT,
    ) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let existing = world.transform(id as u64).unwrap_or_default();
        world.set_transform(
            id as u64,
            Transform2D {
                x: x as f32,
                y: y as f32,
                z: existing.z,
                heading: heading as f32,
            },
        )
    }

    pub(crate) fn set_transform_3d(
        &mut self,
        id: rhai::INT,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        z: rhai::FLOAT,
        heading: rhai::FLOAT,
    ) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_transform(
            id as u64,
            Transform2D {
                x: x as f32,
                y: y as f32,
                z: z as f32,
                heading: heading as f32,
            },
        )
    }

    pub(crate) fn transform(&mut self, id: rhai::INT) -> RhaiDynamic {
        let Some(world) = self.ctx.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        if let Some(xf) = world.transform(id as u64) {
            let mut map = RhaiMap::new();
            map.insert("x".into(), (xf.x as rhai::FLOAT).into());
            map.insert("y".into(), (xf.y as rhai::FLOAT).into());
            map.insert("z".into(), (xf.z as rhai::FLOAT).into());
            map.insert("heading".into(), (xf.heading as rhai::FLOAT).into());
            return map.into();
        }
        ().into()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn set_physics(
        &mut self,
        id: rhai::INT,
        vx: rhai::FLOAT,
        vy: rhai::FLOAT,
        ax: rhai::FLOAT,
        ay: rhai::FLOAT,
        drag: rhai::FLOAT,
        max_speed: rhai::FLOAT,
    ) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let existing = world.physics(id as u64).unwrap_or_default();
        world.set_physics(
            id as u64,
            PhysicsBody2D {
                vx: vx as f32,
                vy: vy as f32,
                vz: existing.vz,
                ax: ax as f32,
                ay: ay as f32,
                az: existing.az,
                drag: drag as f32,
                max_speed: max_speed as f32,
                mass: existing.mass,
                restitution: existing.restitution,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn set_physics_3d(
        &mut self,
        id: rhai::INT,
        vx: rhai::FLOAT,
        vy: rhai::FLOAT,
        vz: rhai::FLOAT,
        ax: rhai::FLOAT,
        ay: rhai::FLOAT,
        az: rhai::FLOAT,
        drag: rhai::FLOAT,
        max_speed: rhai::FLOAT,
    ) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let existing = world.physics(id as u64).unwrap_or_default();
        world.set_physics(
            id as u64,
            PhysicsBody2D {
                vx: vx as f32,
                vy: vy as f32,
                vz: vz as f32,
                ax: ax as f32,
                ay: ay as f32,
                az: az as f32,
                drag: drag as f32,
                max_speed: max_speed as f32,
                mass: existing.mass,
                restitution: existing.restitution,
            },
        )
    }

    pub(crate) fn physics(&mut self, id: rhai::INT) -> RhaiDynamic {
        let Some(world) = self.ctx.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        if let Some(body) = world.physics(id as u64) {
            let mut map = RhaiMap::new();
            map.insert("vx".into(), (body.vx as rhai::FLOAT).into());
            map.insert("vy".into(), (body.vy as rhai::FLOAT).into());
            map.insert("vz".into(), (body.vz as rhai::FLOAT).into());
            map.insert("ax".into(), (body.ax as rhai::FLOAT).into());
            map.insert("ay".into(), (body.ay as rhai::FLOAT).into());
            map.insert("az".into(), (body.az as rhai::FLOAT).into());
            map.insert("drag".into(), (body.drag as rhai::FLOAT).into());
            map.insert("max_speed".into(), (body.max_speed as rhai::FLOAT).into());
            map.insert("mass".into(), (body.mass as rhai::FLOAT).into());
            map.insert(
                "restitution".into(),
                (body.restitution as rhai::FLOAT).into(),
            );
            return map.into();
        }
        ().into()
    }

    pub(crate) fn gravity_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let mode = match config
            .get("mode")
            .and_then(|v| v.clone().try_cast::<String>())
        {
            Some(mode) if mode.eq_ignore_ascii_case("flat") => GravityMode2D::Flat,
            _ => GravityMode2D::Point,
        };
        let body_id = config
            .get("body")
            .and_then(|v| v.clone().try_cast::<String>())
            .or_else(|| {
                config
                    .get("body_id")
                    .and_then(|v| v.clone().try_cast::<String>())
            });
        let gravity_scale = Self::map_number(&config, "gravity_scale", 1.0) as f32;
        let flat_ax = Self::map_number(&config, "flat_ax", 0.0) as f32;
        let flat_ay = Self::map_number(&config, "flat_ay", 0.0) as f32;
        world.attach_gravity(
            id as u64,
            GravityAffected2D {
                mode,
                body_id,
                gravity_scale,
                flat_ax,
                flat_ay,
            },
        )
    }

    pub(crate) fn gravity(&mut self, id: rhai::INT) -> RhaiDynamic {
        let Some(world) = self.ctx.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        let Some(gravity) = world.gravity(id as u64) else {
            return ().into();
        };
        let mut map = RhaiMap::new();
        map.insert(
            "mode".into(),
            match gravity.mode {
                GravityMode2D::Flat => "flat",
                GravityMode2D::Point => "point",
            }
            .into(),
        );
        map.insert(
            "gravity_scale".into(),
            (gravity.gravity_scale as rhai::FLOAT).into(),
        );
        map.insert("flat_ax".into(), (gravity.flat_ax as rhai::FLOAT).into());
        map.insert("flat_ay".into(), (gravity.flat_ay as rhai::FLOAT).into());
        if let Some(body_id) = gravity.body_id {
            map.insert("body".into(), body_id.into());
        }
        map.into()
    }

    pub(crate) fn body_gravity(
        &mut self,
        body_id: &str,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        z: rhai::FLOAT,
    ) -> RhaiMap {
        let mut map = RhaiMap::new();
        let Some(sample) = self.catalogs.celestial.gravity_sample(
            body_id,
            WorldPoint3 { x, y, z },
            0.0,
            self.ctx.spatial_meters_per_world_unit,
        ) else {
            return map;
        };
        map.insert("ax".into(), (sample.accel.x as rhai::FLOAT).into());
        map.insert("ay".into(), (sample.accel.y as rhai::FLOAT).into());
        map.insert("az".into(), (sample.accel.z as rhai::FLOAT).into());
        map.insert(
            "distance".into(),
            (sample.distance_world as rhai::FLOAT).into(),
        );
        map.insert(
            "altitude".into(),
            (sample.altitude_world as rhai::FLOAT).into(),
        );
        map.insert(
            "altitude_km".into(),
            (sample.altitude_km as rhai::FLOAT).into(),
        );
        map
    }

    pub(crate) fn atmosphere_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let body_id = config
            .get("body")
            .and_then(|v| v.clone().try_cast::<String>())
            .or_else(|| {
                config
                    .get("body_id")
                    .and_then(|v| v.clone().try_cast::<String>())
            });
        world.attach_atmosphere(
            id as u64,
            AtmosphereAffected2D {
                body_id,
                drag_scale: Self::map_number(&config, "drag_scale", 1.0) as f32,
                heat_scale: Self::map_number(&config, "heat_scale", 1.0) as f32,
                cooling: Self::map_number(&config, "cooling", 0.20) as f32,
                ..AtmosphereAffected2D::default()
            },
        )
    }

    pub(crate) fn atmosphere(&mut self, id: rhai::INT) -> RhaiDynamic {
        let Some(world) = self.ctx.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        let Some(atmo) = world.atmosphere(id as u64) else {
            return ().into();
        };
        let mut map = RhaiMap::new();
        map.insert("drag_scale".into(), (atmo.drag_scale as rhai::FLOAT).into());
        map.insert("heat_scale".into(), (atmo.heat_scale as rhai::FLOAT).into());
        map.insert("cooling".into(), (atmo.cooling as rhai::FLOAT).into());
        map.insert("heat".into(), (atmo.heat as rhai::FLOAT).into());
        map.insert("density".into(), (atmo.density as rhai::FLOAT).into());
        map.insert(
            "dense_density".into(),
            (atmo.dense_density as rhai::FLOAT).into(),
        );
        map.insert(
            "altitude_km".into(),
            (atmo.altitude_km as rhai::FLOAT).into(),
        );
        if let Some(body_id) = atmo.body_id {
            map.insert("body".into(), body_id.into());
        }
        map.into()
    }

    pub(crate) fn set_lifetime(&mut self, id: rhai::INT, ttl_ms: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_lifetime(
            id as u64,
            Lifetime {
                ttl_ms: ttl_ms as i32,
                original_ttl_ms: ttl_ms as i32,
                on_expire: DespawnVisual::None,
            },
        )
    }

    pub(crate) fn set_visual(&mut self, id: rhai::INT, visual_id: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_visual(
            id as u64,
            VisualBinding {
                visual_id: if visual_id.trim().is_empty() {
                    None
                } else {
                    Some(visual_id.to_string())
                },
                additional_visuals: Vec::new(),
            },
        )
    }

    pub(crate) fn spawn_visual(&mut self, kind: &str, template: &str, data: RhaiMap) -> rhai::INT {
        let Some(world) = self.ctx.world.clone() else {
            return 0;
        };

        // Step 1: Spawn gameplay entity with empty payload
        let Some(entity_id) = world.spawn(kind, JsonValue::Object(JsonMap::new())) else {
            return 0;
        };

        // Step 2: Generate visual_id (format: "{kind}-{entity_id}")
        let visual_id = format!("{}-{}", kind, entity_id);

        // Step 3: Emit typed scene spawn mutation for the visual.
        {
            let mut commands = match self.ctx.queue.lock() {
                Ok(cmds) => cmds,
                Err(_) => {
                    world.despawn(entity_id);
                    return 0;
                }
            };
            commands.push(BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SpawnObject {
                    template: template.to_string(),
                    target: visual_id.clone(),
                },
            });
        }

        // Step 4: Set visual binding
        if !world.set_visual(
            entity_id,
            VisualBinding {
                visual_id: Some(visual_id.clone()),
                additional_visuals: Vec::new(),
            },
        ) {
            world.despawn(entity_id);
            return 0;
        }

        // Step 5: Set transform from data
        let x = data
            .get("x")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;
        let y = data
            .get("y")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;
        let heading = data
            .get("heading")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;

        if !world.set_transform(
            entity_id,
            Transform2D {
                x,
                y,
                z: 0.0,
                heading,
            },
        ) {
            world.despawn(entity_id);
            return 0;
        }

        // Step 6: Set collider if provided
        if let Some(radius_val) = data.get("collider_radius") {
            let radius_opt = radius_val.clone().try_cast::<rhai::FLOAT>().or_else(|| {
                radius_val
                    .clone()
                    .try_cast::<rhai::INT>()
                    .map(|i| i as rhai::FLOAT)
            });
            if let Some(radius) = radius_opt {
                let layer = data
                    .get("collider_layer")
                    .and_then(|v| v.clone().try_cast::<rhai::INT>())
                    .unwrap_or(-1) as u32;
                let mask = data
                    .get("collider_mask")
                    .and_then(|v| v.clone().try_cast::<rhai::INT>())
                    .unwrap_or(-1) as u32;

                if !world.set_collider(
                    entity_id,
                    Collider2D {
                        shape: ColliderShape::Circle {
                            radius: radius as f32,
                        },
                        layer,
                        mask,
                    },
                ) {
                    world.despawn(entity_id);
                    return 0;
                }
            }
        }

        // Step 6b: Set polygon collider if provided
        if let Some(poly_val) = data.get("collider_polygon") {
            if let Some(poly_arr) = poly_val.clone().try_cast::<RhaiArray>() {
                let mut points: Vec<[f32; 2]> = Vec::new();
                for point in poly_arr {
                    if let Some(point_arr) = point.try_cast::<RhaiArray>() {
                        if point_arr.len() >= 2 {
                            let px = point_arr[0].clone().try_cast::<rhai::FLOAT>().or_else(|| {
                                point_arr[0]
                                    .clone()
                                    .try_cast::<rhai::INT>()
                                    .map(|v| v as rhai::FLOAT)
                            });
                            let py = point_arr[1].clone().try_cast::<rhai::FLOAT>().or_else(|| {
                                point_arr[1]
                                    .clone()
                                    .try_cast::<rhai::INT>()
                                    .map(|v| v as rhai::FLOAT)
                            });
                            if let (Some(px), Some(py)) = (px, py) {
                                points.push([px as f32, py as f32]);
                            }
                        }
                    }
                }
                if !points.is_empty() {
                    let layer = data
                        .get("collider_layer")
                        .and_then(|v| v.clone().try_cast::<rhai::INT>())
                        .unwrap_or(-1) as u32;
                    let mask = data
                        .get("collider_mask")
                        .and_then(|v| v.clone().try_cast::<rhai::INT>())
                        .unwrap_or(-1) as u32;

                    if !world.set_collider(
                        entity_id,
                        Collider2D {
                            shape: ColliderShape::Polygon { points },
                            layer,
                            mask,
                        },
                    ) {
                        world.despawn(entity_id);
                        return 0;
                    }
                }
            }
        }

        // Step 7: Set lifetime if provided (skip if zero — means no expiry)
        if let Some(ttl_val) = data.get("lifetime_ms") {
            if let Some(ttl) = ttl_val.clone().try_cast::<rhai::INT>() {
                if ttl > 0
                    && !world.set_lifetime(
                        entity_id,
                        Lifetime {
                            ttl_ms: ttl as i32,
                            original_ttl_ms: ttl as i32,
                            on_expire: DespawnVisual::None,
                        },
                    )
                {
                    world.despawn(entity_id);
                    return 0;
                }
            }
        }

        // Step 8: Create physics body if velocity/physics fields are present
        let has_physics = data.contains_key("vx")
            || data.contains_key("vy")
            || data.contains_key("drag")
            || data.contains_key("max_speed");
        if has_physics {
            let extract_f = |key: &str| -> f32 {
                data.get(key)
                    .and_then(|v| {
                        v.clone()
                            .try_cast::<rhai::FLOAT>()
                            .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
                    })
                    .unwrap_or(0.0) as f32
            };
            let body = PhysicsBody2D {
                vx: extract_f("vx"),
                vy: extract_f("vy"),
                vz: 0.0,
                ax: extract_f("ax"),
                ay: extract_f("ay"),
                az: 0.0,
                drag: extract_f("drag"),
                max_speed: extract_f("max_speed"),
                mass: extract_f("mass").max(0.0),
                restitution: extract_f("restitution").clamp(0.0, 1.0),
            };
            if !world.set_physics(entity_id, body) {
                world.despawn(entity_id);
                return 0;
            }
        }

        entity_id as rhai::INT
    }

    /// Generic prefab applicator: reads catalog components and applies them to an entity.
    /// This centralizes all prefab component logic (physics, collider, controller, 3D bundle, lifecycle)
    /// in one place, eliminating hardcoded match arms.
    fn apply_prefab_components(
        &mut self,
        entity_id: rhai::INT,
        prefab: &catalog::PrefabTemplate,
        args: &RhaiMap,
    ) -> bool {
        let Some(components) = &prefab.components else {
            return true; // No components to apply; entity spawned successfully
        };
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };

        // Apply physics component - check args for overrides
        if let Some(phys) = &components.physics {
            let mut vx = phys.vx.unwrap_or(0.0);
            let mut vy = phys.vy.unwrap_or(0.0);
            let mut vz = phys.vz.unwrap_or(0.0);
            let ax = phys.ax.unwrap_or(0.0);
            let ay = phys.ay.unwrap_or(0.0);
            let az = phys.az.unwrap_or(0.0);
            let drag = phys.drag.unwrap_or(0.0);
            let max_speed = phys.max_speed.unwrap_or(0.0);
            let mass = phys.mass.unwrap_or(1.0) as f32;
            let restitution = phys.restitution.unwrap_or(0.7) as f32;

            if let Some(arg_vx) = args.get("vx").and_then(|v| v.as_float().ok()) {
                vx = arg_vx;
            }
            if let Some(arg_vy) = args.get("vy").and_then(|v| v.as_float().ok()) {
                vy = arg_vy;
            }
            if let Some(arg_vz) = args.get("vz").and_then(|v| v.as_float().ok()) {
                vz = arg_vz;
            }

            if !world.set_physics(
                entity_id as u64,
                PhysicsBody2D {
                    vx: vx as f32,
                    vy: vy as f32,
                    vz: vz as f32,
                    ax: ax as f32,
                    ay: ay as f32,
                    az: az as f32,
                    drag: drag as f32,
                    max_speed: max_speed as f32,
                    mass,
                    restitution,
                },
            ) {
                return false;
            }

            if phys.gravity_scale.unwrap_or(0.0) > 0.0 || phys.gravity_mode.is_some() {
                let gravity_mode = match phys.gravity_mode.as_deref() {
                    Some("flat") => GravityMode2D::Flat,
                    _ => GravityMode2D::Point,
                };
                if !world.attach_gravity(
                    entity_id as u64,
                    GravityAffected2D {
                        mode: gravity_mode,
                        body_id: phys.gravity_body.clone(),
                        gravity_scale: phys.gravity_scale.unwrap_or(1.0) as f32,
                        flat_ax: phys.gravity_flat_x.unwrap_or(0.0) as f32,
                        flat_ay: phys.gravity_flat_y.unwrap_or(0.0) as f32,
                    },
                ) {
                    return false;
                }
            }

            if phys.atmosphere_body.is_some()
                || phys.atmosphere_drag_scale.is_some()
                || phys.atmosphere_heat_scale.is_some()
            {
                if !world.attach_atmosphere(
                    entity_id as u64,
                    AtmosphereAffected2D {
                        body_id: phys.atmosphere_body.clone(),
                        drag_scale: phys.atmosphere_drag_scale.unwrap_or(1.0) as f32,
                        heat_scale: phys.atmosphere_heat_scale.unwrap_or(1.0) as f32,
                        cooling: phys.atmosphere_cooling.unwrap_or(0.20) as f32,
                        ..AtmosphereAffected2D::default()
                    },
                ) {
                    return false;
                }
            }
        }

        if let Some(bootstrap) = prefab_bootstrap_assembly3d_from_components(components, args) {
            if !world.bootstrap_assembly3d(entity_id as u64, bootstrap) {
                return false;
            }
        }

        // Apply collider component - check args for radius override
        if let Some(coll) = &components.collider {
            if coll.shape.as_str() == "circle" {
                let mut radius = coll.radius.unwrap_or(1.0);
                let layer = coll.layer.unwrap_or(0xFFFF) as rhai::INT;
                let mask = coll.mask.unwrap_or(0xFFFF) as rhai::INT;

                // Check args for collider_radius override
                if let Some(arg_radius) = args.get("collider_radius") {
                    if let Ok(r) = arg_radius.as_float() {
                        radius = r;
                    }
                }

                if !world.set_collider(
                    entity_id as u64,
                    Collider2D {
                        shape: ColliderShape::Circle {
                            radius: radius as f32,
                        },
                        layer: layer as u32,
                        mask: mask as u32,
                    },
                ) {
                    return false;
                }
            }
        }

        // Apply controller component - merge catalog config with args["cfg"] overrides
        if let Some(ctrl) = &components.controller {
            if matches!(
                ctrl.controller_type.as_str(),
                "ArcadeController" | "VehicleAssembly"
            ) {
                let mut config_map = if let Some(cfg) = &ctrl.config {
                    let mut m = RhaiMap::new();
                    for (k, v) in cfg {
                        m.insert(k.clone().into(), json_to_rhai_dynamic(v));
                    }
                    m
                } else {
                    RhaiMap::new()
                };

                // Merge runtime overrides from args["cfg"] (e.g. per-level difficulty)
                if let Some(cfg_dyn) = args.get("cfg") {
                    if let Some(cfg_map) = cfg_dyn.clone().try_cast::<RhaiMap>() {
                        config_map = merge_rhai_maps(config_map, &cfg_map);
                    }
                }

                if !attach_vehicle_stack(world, entity_id as u64, config_map) {
                    return false;
                }
            }
        }

        // Apply wrappable flag
        if components.wrappable.unwrap_or(false) && !world.enable_wrap_bounds(entity_id as u64) {
            return false;
        }

        // Apply extra data fields from catalog and args overrides
        let mut data = RhaiMap::new();

        // Start with catalog extra_data
        if let Some(extra) = &components.extra_data {
            for (k, v) in extra {
                data.insert(k.clone().into(), json_to_rhai_dynamic(v));
            }
        }

        // Apply init_fields from prefab
        for (k, v) in &prefab.init_fields {
            data.insert(k.clone().into(), json_to_rhai_dynamic(v));
        }

        // Apply args overrides for shape, size, and any other fields
        for (k, v) in args {
            // Skip position/velocity args that are handled separately
            if ![
                "x",
                "y",
                "heading",
                "vx",
                "vy",
                "ttl_ms",
                "radius",
                "owner_id",
                "cfg",
                "invulnerable_ms",
                "collider_radius",
            ]
            .contains(&k.as_str())
            {
                let merged = match data.remove(k) {
                    Some(existing) => merge_rhai_dynamic(existing, v.clone()),
                    None => v.clone(),
                };
                data.insert(k.clone(), merged);
            }
        }

        if !data.is_empty() && !self.entity(entity_id).set_many(data) {
            return false;
        }

        true
    }

    pub(crate) fn spawn_prefab(&mut self, name: &str, args: RhaiMap) -> rhai::INT {
        // Look up prefab in catalog
        let Some(prefab) = self.catalogs.prefabs.get(name).cloned() else {
            return 0; // Prefab not found in catalog
        };
        let spawn_args = prefab_spawn_args(&prefab, &args);

        // Extract position from args
        let x = Self::map_number(&spawn_args, "x", 0.0);
        let y = Self::map_number(&spawn_args, "y", 0.0);
        let heading = Self::map_number(&spawn_args, "heading", 0.0);

        // Determine spawn approach based on lifecycle
        let lifecycle_name = prefab_lifecycle_name(&prefab, &spawn_args);
        let lifecycle_str = lifecycle_name.as_deref().unwrap_or("");

        let sprite_template = prefab.sprite_template.as_deref().unwrap_or(&prefab.kind);

        let id = if is_ephemeral_lifecycle(lifecycle_str) {
            // Ephemeral spawn for TTL-based entities (bullets, smoke, short-lived particles)
            self.spawn_prefab_ephemeral(&prefab, x, y, heading, &spawn_args)
        } else {
            // Regular spawn for persistent entities
            let mut visual_args = RhaiMap::new();
            visual_args.insert("x".into(), x.into());
            visual_args.insert("y".into(), y.into());
            visual_args.insert("heading".into(), heading.into());

            let id = self.spawn_visual(&prefab.kind, sprite_template, visual_args);
            if id <= 0 {
                return 0;
            }

            // Apply catalog components generically
            if !self.apply_prefab_components(id, &prefab, &spawn_args) {
                let _ = self.despawn(id);
                return 0;
            }

            // Handle mod-specific initialization (passed as args, not hardcoded)
            let invulnerable_ms = Self::map_int(&args, "invulnerable_ms", 0);
            if invulnerable_ms > 0 {
                let _ = self.entity(id).status_add("invulnerable", invulnerable_ms);
            }

            id
        };

        if id <= 0 {
            return 0;
        }

        // Apply default_tags from prefab catalog, then override/extend with args tags
        {
            let Some(world) = self.ctx.world.as_ref() else {
                return id;
            };
            for tag in &prefab.default_tags {
                world.tag_add(id as u64, tag);
            }
            if let Some(tags_val) = spawn_args.get("tags") {
                if let Ok(tags_arr) = tags_val.clone().into_array() {
                    for t in tags_arr {
                        if let Ok(s) = t.into_string() {
                            world.tag_add(id as u64, &s);
                        }
                    }
                }
            }
        }

        // Apply fg_colour from prefab (supports "@palette.<key>" and literal colors)
        self.apply_prefab_fg_colour(id, &prefab.fg_colour, &spawn_args);

        id
    }

    fn spawn_prefab_bound_visual(
        &mut self,
        name: &str,
        runtime_target: &str,
        args: RhaiMap,
        transform: &RuntimeObjectTransform,
    ) -> rhai::INT {
        let Some(prefab) = self.catalogs.prefabs.get(name).cloned() else {
            return 0;
        };
        let spawn_args = prefab_spawn_args(&prefab, &args);
        let Some(world) = self.ctx.world.clone() else {
            return 0;
        };

        let Some(entity_id) = world.spawn(&prefab.kind, JsonValue::Object(JsonMap::new())) else {
            return 0;
        };

        if !world.set_visual(
            entity_id,
            VisualBinding {
                visual_id: Some(runtime_target.to_string()),
                additional_visuals: Vec::new(),
            },
        ) {
            world.despawn(entity_id);
            return 0;
        }

        if let RuntimeObjectTransform::TwoD { .. } = transform {
            let x = Self::map_number(&spawn_args, "x", 0.0) as f32;
            let y = Self::map_number(&spawn_args, "y", 0.0) as f32;
            let z = Self::map_number(&spawn_args, "z", 0.0) as f32;
            let heading = Self::map_number(&spawn_args, "heading", 0.0) as f32;
            if !world.set_transform(entity_id, Transform2D { x, y, z, heading }) {
                world.despawn(entity_id);
                return 0;
            }
        }

        let ttl_ms = Self::map_int(&spawn_args, "ttl_ms", Self::map_int(&spawn_args, "lifetime_ms", 0));
        if ttl_ms > 0
            && !world.set_lifetime(
                entity_id,
                Lifetime {
                    ttl_ms: ttl_ms as i32,
                    original_ttl_ms: ttl_ms as i32,
                    on_expire: DespawnVisual::None,
                },
            )
        {
            world.despawn(entity_id);
            return 0;
        }

        let entity_id_rhai = entity_id as rhai::INT;
        if !self.apply_prefab_components(entity_id_rhai, &prefab, &spawn_args) {
            world.despawn(entity_id);
            return 0;
        }

        let invulnerable_ms = Self::map_int(&spawn_args, "invulnerable_ms", 0);
        if invulnerable_ms > 0 {
            let _ = self
                .entity(entity_id_rhai)
                .status_add("invulnerable", invulnerable_ms);
        }

        for tag in &prefab.default_tags {
            world.tag_add(entity_id, tag);
        }
        if let Some(tags_val) = spawn_args.get("tags") {
            if let Ok(tags_arr) = tags_val.clone().into_array() {
                for t in tags_arr {
                    if let Ok(s) = t.into_string() {
                        world.tag_add(entity_id, &s);
                    }
                }
            }
        }

        self.apply_prefab_fg_colour(entity_id_rhai, &prefab.fg_colour, &spawn_args);
        entity_id_rhai
    }

    fn spawn_runtime_object_node_bound_to_visual(
        &mut self,
        node: &RuntimeObjectDocument,
        runtime_target: &str,
        owner_id: Option<u64>,
    ) -> rhai::INT {
        let args = runtime_object_args(node, owner_id);
        if let Some(prefab) = node.prefab.as_deref() {
            let id =
                self.spawn_prefab_bound_visual(prefab, runtime_target, args.clone(), &node.transform);
            if id <= 0 {
                return 0;
            }

            if let Some(world) = self.ctx.world.as_ref() {
                if let Some(bootstrap) =
                    runtime_object_bootstrap_assembly3d_from_document(node, &args)
                {
                    if !world.bootstrap_assembly3d(id as u64, bootstrap) {
                        let _ = self.despawn(id);
                        return 0;
                    }
                }
            }

            return id;
        }

        let Some(world) = self.ctx.world.clone() else {
            return 0;
        };
        let kind = runtime_object_spawn_kind(node);
        let Some(entity_id) = world.spawn(&kind, JsonValue::Object(JsonMap::new())) else {
            return 0;
        };

        if !world.set_visual(
            entity_id,
            VisualBinding {
                visual_id: Some(runtime_target.to_string()),
                additional_visuals: Vec::new(),
            },
        ) {
            let _ = world.despawn(entity_id);
            return 0;
        }

        match &node.transform {
            RuntimeObjectTransform::TwoD { .. } => {
                let x = Self::map_number(&args, "x", 0.0) as f32;
                let y = Self::map_number(&args, "y", 0.0) as f32;
                let z = Self::map_number(&args, "z", 0.0) as f32;
                let heading = Self::map_number(&args, "heading", 0.0) as f32;
                if !world.set_transform(entity_id, Transform2D { x, y, z, heading }) {
                    let _ = world.despawn(entity_id);
                    return 0;
                }
            }
            _ => {
                let Some(transform) = runtime_object_transform3d(&node.transform) else {
                    let _ = world.despawn(entity_id);
                    return 0;
                };
                if !world.set_transform3d(entity_id, transform) {
                    let _ = world.despawn(entity_id);
                    return 0;
                }
            }
        }

        let ttl_ms = Self::map_int(
            &args,
            "ttl_ms",
            Self::map_int(&args, "lifetime_ms", 0),
        );
        if ttl_ms > 0
            && !world.set_lifetime(
                entity_id,
                Lifetime {
                    ttl_ms: ttl_ms as i32,
                    original_ttl_ms: ttl_ms as i32,
                    on_expire: DespawnVisual::None,
                },
            )
        {
            let _ = world.despawn(entity_id);
            return 0;
        }

        let entity_id_rhai = entity_id as rhai::INT;
        if let Some(prefab) = runtime_object_component_prefab(node) {
            if !self.apply_prefab_components(entity_id_rhai, &prefab, &args) {
                let _ = world.despawn(entity_id);
                return 0;
            }
        }

        if let Some(bootstrap) = runtime_object_bootstrap_assembly3d_from_document(node, &args) {
            if !world.bootstrap_assembly3d(entity_id, bootstrap) {
                let _ = world.despawn(entity_id);
                return 0;
            }
        } else if !apply_runtime_object_owner_lifecycle(&world, entity_id, &args) {
            let _ = world.despawn(entity_id);
            return 0;
        }

        entity_id_rhai
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn spawn_runtime_object_document(
        &mut self,
        node: &RuntimeObjectDocument,
        owner_id: Option<u64>,
    ) -> rhai::INT {
        let Some(prefab) = node.prefab.as_deref() else {
            return 0;
        };

        let args = runtime_object_args(node, owner_id);
        let id = self.spawn_prefab(prefab, args.clone());
        if id <= 0 {
            return 0;
        }

        if let Some(world) = self.ctx.world.as_ref() {
            if let Some(bootstrap) = runtime_object_bootstrap_assembly3d_from_document(node, &args)
            {
                if !world.bootstrap_assembly3d(id as u64, bootstrap) {
                    let _ = self.despawn(id);
                    return 0;
                }
            }
        }

        for child in &node.children {
            if self.spawn_runtime_object_document(child, Some(id as u64)) <= 0 {
                let _ = self.despawn(id);
                return 0;
            }
        }

        id
    }

    /// Spawn a prefab along the owner entity's heading direction.
    ///
    /// Eliminates manual sin/cos trig from scripts for projectile spawning.
    ///
    /// Args map keys:
    /// - `speed`: projectile speed (default 0.0)
    /// - `offset`: forward distance from owner origin to spawn point (default 0.0)
    /// - `inherit_velocity`: add owner velocity to projectile (default false)
    /// - Any other keys are forwarded to `spawn_prefab` (e.g. `ttl_ms`, `heading`)
    pub(crate) fn spawn_from_heading(
        &mut self,
        owner_id: rhai::INT,
        prefab: &str,
        args: RhaiMap,
    ) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        let Some(xf) = world.transform(owner_id as u64) else {
            return 0;
        };
        let heading = xf.heading;
        let (fwd_x, fwd_y) = (heading.sin(), -heading.cos());
        let speed = Self::map_number(&args, "speed", 0.0) as f32;
        let offset = Self::map_number(&args, "offset", 0.0) as f32;
        let inherit_velocity = args
            .get("inherit_velocity")
            .and_then(|v| v.as_bool().ok())
            .unwrap_or(false);
        let (svx, svy) = if inherit_velocity {
            world
                .physics(owner_id as u64)
                .map(|b| (b.vx, b.vy))
                .unwrap_or((0.0, 0.0))
        } else {
            (0.0f32, 0.0f32)
        };
        let spawn_x = xf.x + fwd_x * offset;
        let spawn_y = xf.y + fwd_y * offset;
        let vx = fwd_x * speed + svx;
        let vy = fwd_y * speed + svy;
        let mut merged = args.clone();
        merged.insert("x".into(), (spawn_x as rhai::FLOAT).into());
        merged.insert("y".into(), (spawn_y as rhai::FLOAT).into());
        merged.insert("vx".into(), (vx as rhai::FLOAT).into());
        merged.insert("vy".into(), (vy as rhai::FLOAT).into());
        merged.insert("heading".into(), (heading as rhai::FLOAT).into());
        merged.remove("speed");
        merged.remove("offset");
        merged.remove("inherit_velocity");
        self.spawn_prefab(prefab, merged)
    }

    /// Return velocity decomposed into heading-relative components.
    ///
    /// Returns `#{fwd, right, drift, speed}`:
    /// - `fwd`   – component of velocity along heading (+ = forward, − = backward)
    /// - `right` – lateral component (+ = drifting right/clockwise, − = left)
    /// - `drift` – |right| / speed, normalised 0-1 (0 = perfectly aligned)
    /// - `speed` – total speed magnitude
    pub(crate) fn heading_drift(&mut self, id: rhai::INT) -> RhaiMap {
        let mut out = RhaiMap::new();
        let Some(world) = self.ctx.world.as_ref() else {
            return out;
        };
        let Some(xf) = world.transform(id as u64) else {
            return out;
        };
        let heading = xf.heading;
        let (fwd_x, fwd_y) = (heading.sin(), -heading.cos());
        let (vx, vy) = match world.physics(id as u64) {
            Some(b) => (b.vx, b.vy),
            None => (0.0, 0.0),
        };
        let fwd = vx * fwd_x + vy * fwd_y;
        let right = vx * (-fwd_y) + vy * fwd_x;
        let speed = (vx * vx + vy * vy).sqrt();
        let drift = if speed > 0.001 {
            right.abs() / speed
        } else {
            0.0
        };
        out.insert("fwd".into(), (fwd as rhai::FLOAT).into());
        out.insert("right".into(), (right as rhai::FLOAT).into());
        out.insert("drift".into(), (drift as rhai::FLOAT).into());
        out.insert("speed".into(), (speed as rhai::FLOAT).into());
        out
    }

    fn spawn_prefab_ephemeral(
        &mut self,
        prefab: &catalog::PrefabTemplate,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        heading: rhai::FLOAT,
        args: &RhaiMap,
    ) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };

        let vx = Self::map_number(args, "vx", 0.0);
        let vy = Self::map_number(args, "vy", 0.0);
        let sprite_template = prefab.sprite_template.as_deref().unwrap_or(&prefab.kind);

        // Extract physics for drag/max_speed
        let (drag, max_speed) = prefab
            .components
            .as_ref()
            .and_then(|c| c.physics.as_ref())
            .map(|p| (p.drag.unwrap_or(0.0), p.max_speed.unwrap_or(0.0)))
            .unwrap_or((0.0, 0.0));

        // Determine lifecycle policy
        let lifecycle_name = prefab_lifecycle_name(prefab, args);
        let lifecycle_str = lifecycle_name.as_deref().unwrap_or("");

        let resolved =
            self.resolve_ephemeral_prefab(prefab, args, lifecycle_str, vx, vy, drag, max_speed);

        let Some(id) = spawn_ephemeral_visual(
            world,
            &self.ctx.queue,
            EphemeralSpawn {
                kind: prefab.kind.clone(),
                template: sprite_template.to_string(),
                x: x as f32,
                y: y as f32,
                heading: heading as f32,
                vx: resolved.vx,
                vy: resolved.vy,
                drag: resolved.drag,
                max_speed: resolved.max_speed,
                ttl_ms: Some(resolved.ttl_ms),
                owner_id: resolved.owner_id,
                lifecycle: resolved.lifecycle,
                follow_anchor: resolved.follow_anchor,
                extra_data: resolved.extra_data,
            },
        ) else {
            return 0;
        };

        if let Some(components) = &prefab.components {
            if let Some(assembly) = prefab_assembly3d_from_components(components) {
                if !world.attach_assembly3d(id as u64, assembly) {
                    let _ = world.despawn(id);
                    return 0;
                }
            }
        }

        // Apply collider if specified in prefab
        if let Some(components) = &prefab.components {
            if let Some(coll) = &components.collider {
                if coll.shape.as_str() == "circle" {
                    let radius = coll.radius.unwrap_or(1.0);
                    let layer = coll.layer.unwrap_or(0xFFFF) as rhai::INT;
                    let mask = coll.mask.unwrap_or(0xFFFF) as rhai::INT;
                    if !world.set_collider(
                        id as u64,
                        Collider2D {
                            shape: ColliderShape::Circle {
                                radius: radius as f32,
                            },
                            layer: layer as u32,
                            mask: mask as u32,
                        },
                    ) {
                        let _ = world.despawn(id);
                        return 0;
                    }
                }
            }
        }

        // Apply wrap if specified
        if prefab
            .components
            .as_ref()
            .and_then(|c| c.wrappable)
            .unwrap_or(false)
            && !world.enable_wrap_bounds(id as u64)
        {
            let _ = world.despawn(id);
            return 0;
        }

        // Apply default_tags from prefab catalog, then extend with args tags
        {
            if let Some(world) = self.ctx.world.as_ref() {
                for tag in &prefab.default_tags {
                    world.tag_add(id, tag);
                }
                if let Some(tags_val) = args.get("tags") {
                    if let Ok(tags_arr) = tags_val.clone().into_array() {
                        for t in tags_arr {
                            if let Ok(s) = t.into_string() {
                                world.tag_add(id, &s);
                            }
                        }
                    }
                }
            }
        }

        // Apply fg_colour from prefab (supports "@palette.<key>" and literal colors)
        self.apply_prefab_fg_colour(id as rhai::INT, &prefab.fg_colour, args);

        id as rhai::INT
    }

    /// Resolves a prefab `fg_colour` string (which may be `"@palette.<key>"` or a literal color)
    /// and applies it to the spawned entity. Args `fg` override takes precedence.
    fn apply_prefab_fg_colour(
        &mut self,
        id: rhai::INT,
        fg_colour: &Option<String>,
        args: &RhaiMap,
    ) {
        // Args-level override wins over catalog default
        let color_str =
            if let Some(arg_fg) = args.get("fg").and_then(|v| v.clone().into_string().ok()) {
                if !arg_fg.is_empty() {
                    arg_fg
                } else {
                    return;
                }
            } else if let Some(cfg) = fg_colour {
                if let Some(key) = cfg.strip_prefix("@palette.") {
                    let resolved = self
                        .palette_store
                        .resolve(
                            self.palette_persistence
                                .as_ref()
                                .and_then(|p| p.get("/__palette__"))
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                                .as_deref(),
                            self.palette_default_id.as_deref(),
                        )
                        .and_then(|pal| pal.colors.get(key))
                        .cloned();
                    match resolved {
                        Some(c) => c,
                        None => return,
                    }
                } else {
                    cfg.clone()
                }
            } else {
                return;
            };
        self.set(id, "style.fg", rhai::Dynamic::from(color_str));
    }

    pub(crate) fn spawn_group(&mut self, group_name: &str, prefab_name: &str) -> RhaiArray {
        // Try to load from catalog first
        if let Some(group) = self.catalogs.groups.get(group_name) {
            if group.prefab == prefab_name {
                let spawns = group.spawns.clone();
                return spawns
                    .iter()
                    .map(|spec| {
                        let mut args = RhaiMap::new();
                        args.insert("x".into(), (spec.x).into());
                        args.insert("y".into(), (spec.y).into());
                        args.insert("vx".into(), (spec.vx).into());
                        args.insert("vy".into(), (spec.vy).into());
                        args.insert("shape".into(), (spec.shape).into());
                        args.insert("size".into(), (spec.size).into());
                        self.spawn_prefab(prefab_name, args).into()
                    })
                    .collect();
            }
        }
        RhaiArray::new()
    }

    /// Returns body world position at `elapsed_sec`, including parent orbit chain.
    /// Returns empty map when body id is unknown or parent chain is invalid.
    pub(crate) fn body_position(&mut self, id: &str, elapsed_sec: rhai::FLOAT) -> RhaiMap {
        let mut map = RhaiMap::new();
        let Some(pose) = self.catalogs.celestial.body_pose(
            id,
            elapsed_sec as f64,
            self.ctx.spatial_meters_per_world_unit,
        ) else {
            return map;
        };
        map.insert("x".into(), (pose.center.x as rhai::FLOAT).into());
        map.insert("y".into(), (pose.center.y as rhai::FLOAT).into());
        map.insert("z".into(), (pose.center.z as rhai::FLOAT).into());
        map.insert(
            "surface_radius".into(),
            (pose.surface_radius_world as rhai::FLOAT).into(),
        );
        map.insert(
            "render_radius".into(),
            (pose.render_radius_world as rhai::FLOAT).into(),
        );
        map
    }

    pub(crate) fn body_pose(&mut self, id: &str, elapsed_sec: rhai::FLOAT) -> RhaiMap {
        let mut map = RhaiMap::new();
        let Some(pose) = self.catalogs.celestial.body_pose(
            id,
            elapsed_sec as f64,
            self.ctx.spatial_meters_per_world_unit,
        ) else {
            return map;
        };
        map.insert("x".into(), (pose.center.x as rhai::FLOAT).into());
        map.insert("y".into(), (pose.center.y as rhai::FLOAT).into());
        map.insert("z".into(), (pose.center.z as rhai::FLOAT).into());
        map.insert(
            "orbit_angle_rad".into(),
            (pose.orbit_angle_rad as rhai::FLOAT).into(),
        );
        map.insert(
            "render_radius".into(),
            (pose.render_radius_world as rhai::FLOAT).into(),
        );
        map.insert(
            "surface_radius".into(),
            (pose.surface_radius_world as rhai::FLOAT).into(),
        );
        map.insert(
            "gravity_mu".into(),
            (pose.gravity_mu_world_units as rhai::FLOAT).into(),
        );
        if let Some(km) = pose.radius_km {
            map.insert("radius_km".into(), (km as rhai::FLOAT).into());
        }
        if let Some(km_per_world_unit) = pose.km_per_world_unit {
            map.insert(
                "km_per_world_unit".into(),
                (km_per_world_unit as rhai::FLOAT).into(),
            );
        }
        if let Some(parent_center) = pose.parent_center {
            map.insert("parent_x".into(), (parent_center.x as rhai::FLOAT).into());
            map.insert("parent_y".into(), (parent_center.y as rhai::FLOAT).into());
            map.insert("parent_z".into(), (parent_center.z as rhai::FLOAT).into());
        }
        map
    }

    pub(crate) fn body_surface(
        &mut self,
        body_id: &str,
        latitude_deg: rhai::FLOAT,
        longitude_deg: rhai::FLOAT,
        altitude_world: rhai::FLOAT,
        elapsed_sec: rhai::FLOAT,
    ) -> RhaiMap {
        let mut map = RhaiMap::new();
        let Some(surface) = self.catalogs.celestial.surface_point(
            body_id,
            latitude_deg,
            longitude_deg,
            altitude_world,
            elapsed_sec as f64,
            self.ctx.spatial_meters_per_world_unit,
        ) else {
            return map;
        };
        map.insert("x".into(), (surface.point.x as rhai::FLOAT).into());
        map.insert("y".into(), (surface.point.y as rhai::FLOAT).into());
        map.insert("z".into(), (surface.point.z as rhai::FLOAT).into());
        map.insert("normal_x".into(), (surface.normal.x as rhai::FLOAT).into());
        map.insert("normal_y".into(), (surface.normal.y as rhai::FLOAT).into());
        map.insert("normal_z".into(), (surface.normal.z as rhai::FLOAT).into());
        map.insert(
            "radius_world".into(),
            (surface.radius_world as rhai::FLOAT).into(),
        );
        map.insert(
            "altitude".into(),
            (surface.altitude_world as rhai::FLOAT).into(),
        );
        map.insert(
            "altitude_km".into(),
            (surface.altitude_km as rhai::FLOAT).into(),
        );
        map.insert(
            "latitude_deg".into(),
            (surface.latitude_deg as rhai::FLOAT).into(),
        );
        map.insert(
            "longitude_deg".into(),
            (surface.longitude_deg as rhai::FLOAT).into(),
        );
        map
    }

    pub(crate) fn body_frame(
        &mut self,
        body_id: &str,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        z: rhai::FLOAT,
        elapsed_sec: rhai::FLOAT,
    ) -> RhaiMap {
        let mut map = RhaiMap::new();
        let Some(frame) = self.catalogs.celestial.local_frame(
            body_id,
            WorldPoint3 { x, y, z },
            elapsed_sec as f64,
        ) else {
            return map;
        };
        map.insert("origin_x".into(), (frame.origin.x as rhai::FLOAT).into());
        map.insert("origin_y".into(), (frame.origin.y as rhai::FLOAT).into());
        map.insert("origin_z".into(), (frame.origin.z as rhai::FLOAT).into());
        map.insert("up_x".into(), (frame.up.x as rhai::FLOAT).into());
        map.insert("up_y".into(), (frame.up.y as rhai::FLOAT).into());
        map.insert("up_z".into(), (frame.up.z as rhai::FLOAT).into());
        map.insert("east_x".into(), (frame.east.x as rhai::FLOAT).into());
        map.insert("east_y".into(), (frame.east.y as rhai::FLOAT).into());
        map.insert("east_z".into(), (frame.east.z as rhai::FLOAT).into());
        map.insert("north_x".into(), (frame.north.x as rhai::FLOAT).into());
        map.insert("north_y".into(), (frame.north.y as rhai::FLOAT).into());
        map.insert("north_z".into(), (frame.north.z as rhai::FLOAT).into());
        map.insert(
            "forward_x".into(),
            (frame.tangent_forward.x as rhai::FLOAT).into(),
        );
        map.insert(
            "forward_y".into(),
            (frame.tangent_forward.y as rhai::FLOAT).into(),
        );
        map.insert(
            "forward_z".into(),
            (frame.tangent_forward.z as rhai::FLOAT).into(),
        );
        map
    }

    pub(crate) fn body_atmosphere(
        &mut self,
        body_id: &str,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        z: rhai::FLOAT,
        elapsed_sec: rhai::FLOAT,
    ) -> RhaiMap {
        let mut map = RhaiMap::new();
        let Some(sample) = self.catalogs.celestial.atmosphere_sample(
            body_id,
            WorldPoint3 { x, y, z },
            elapsed_sec as f64,
            self.ctx.spatial_meters_per_world_unit,
        ) else {
            return map;
        };
        map.insert("density".into(), (sample.density as rhai::FLOAT).into());
        map.insert(
            "dense_density".into(),
            (sample.dense_density as rhai::FLOAT).into(),
        );
        map.insert("drag".into(), (sample.drag as rhai::FLOAT).into());
        map.insert("heat_band".into(), (sample.heat_band as rhai::FLOAT).into());
        map.insert(
            "altitude".into(),
            (sample.altitude_world as rhai::FLOAT).into(),
        );
        map.insert(
            "altitude_km".into(),
            (sample.altitude_km as rhai::FLOAT).into(),
        );
        map
    }

    pub(crate) fn site_pose(&mut self, site_id: &str, elapsed_sec: rhai::FLOAT) -> RhaiMap {
        let mut map = RhaiMap::new();
        let Some(site) = self.catalogs.celestial.site_pose(
            site_id,
            elapsed_sec as f64,
            self.ctx.spatial_meters_per_world_unit,
        ) else {
            return map;
        };
        map.insert("x".into(), (site.position.x as rhai::FLOAT).into());
        map.insert("y".into(), (site.position.y as rhai::FLOAT).into());
        map.insert("z".into(), (site.position.z as rhai::FLOAT).into());
        map.insert("up_x".into(), (site.up.x as rhai::FLOAT).into());
        map.insert("up_y".into(), (site.up.y as rhai::FLOAT).into());
        map.insert("up_z".into(), (site.up.z as rhai::FLOAT).into());
        map.insert(
            "altitude".into(),
            (site.altitude_world as rhai::FLOAT).into(),
        );
        map.insert(
            "altitude_km".into(),
            (site.altitude_km as rhai::FLOAT).into(),
        );
        if let Some(body_id) = site.body_id {
            map.insert("body".into(), body_id.into());
        }
        if let Some(body_center) = site.body_center {
            map.insert("body_x".into(), (body_center.x as rhai::FLOAT).into());
            map.insert("body_y".into(), (body_center.y as rhai::FLOAT).into());
            map.insert("body_z".into(), (body_center.z as rhai::FLOAT).into());
        }
        if let Some(lat) = site.latitude_deg {
            map.insert("latitude_deg".into(), (lat as rhai::FLOAT).into());
        }
        if let Some(lon) = site.longitude_deg {
            map.insert("longitude_deg".into(), (lon as rhai::FLOAT).into());
        }
        map
    }

    pub(crate) fn system_query(&mut self, system_id: &str, elapsed_sec: rhai::FLOAT) -> RhaiMap {
        let mut map = RhaiMap::new();
        let Some(system) = self
            .catalogs
            .celestial
            .system_query(system_id, elapsed_sec as f64)
        else {
            return map;
        };
        map.insert("id".into(), system.id.into());
        if let Some(region) = system.region {
            map.insert("region".into(), region.into());
        }
        if let Some(star_body_id) = system.star_body_id {
            map.insert("star_body".into(), star_body_id.into());
        }
        if let Some(star_center) = system.star_center {
            map.insert("star_x".into(), (star_center.x as rhai::FLOAT).into());
            map.insert("star_y".into(), (star_center.y as rhai::FLOAT).into());
            map.insert("star_z".into(), (star_center.z as rhai::FLOAT).into());
        }
        if let Some(map_position) = system.map_position {
            map.insert("map_x".into(), (map_position.x as rhai::FLOAT).into());
            map.insert("map_y".into(), (map_position.y as rhai::FLOAT).into());
        }
        map.insert(
            "bodies".into(),
            system
                .bodies
                .into_iter()
                .map(RhaiDynamic::from)
                .collect::<RhaiArray>()
                .into(),
        );
        map.insert(
            "sites".into(),
            system
                .sites
                .into_iter()
                .map(RhaiDynamic::from)
                .collect::<RhaiArray>()
                .into(),
        );
        map
    }

    /// Returns a map of planet visual preset parameters for the given planet type id.
    /// Returns a map with all default values when the id is not found.
    pub(crate) fn planet_type_info(&mut self, id: &str) -> RhaiMap {
        let p = self
            .catalogs
            .celestial
            .planet_types
            .get(id)
            .cloned()
            .unwrap_or_default();
        let mut map = RhaiMap::new();
        map.insert("ocean_color".into(), p.ocean_color.into());
        map.insert("land_color".into(), p.land_color.into());
        map.insert(
            "terrain_threshold".into(),
            (p.terrain_threshold as rhai::FLOAT).into(),
        );
        map.insert(
            "terrain_noise_scale".into(),
            (p.terrain_noise_scale as rhai::FLOAT).into(),
        );
        map.insert(
            "terrain_noise_octaves".into(),
            (p.terrain_noise_octaves as rhai::INT).into(),
        );
        map.insert(
            "marble_depth".into(),
            (p.marble_depth as rhai::FLOAT).into(),
        );
        map.insert("ambient".into(), (p.ambient as rhai::FLOAT).into());
        map.insert(
            "latitude_bands".into(),
            (p.latitude_bands as rhai::INT).into(),
        );
        map.insert(
            "latitude_band_depth".into(),
            (p.latitude_band_depth as rhai::FLOAT).into(),
        );
        map.insert(
            "polar_ice_start".into(),
            (p.polar_ice_start as rhai::FLOAT).into(),
        );
        map.insert(
            "polar_ice_end".into(),
            (p.polar_ice_end as rhai::FLOAT).into(),
        );
        map.insert(
            "desert_strength".into(),
            (p.desert_strength as rhai::FLOAT).into(),
        );
        map.insert(
            "atmo_strength".into(),
            (p.atmo_strength as rhai::FLOAT).into(),
        );
        map.insert(
            "atmo_rim_power".into(),
            (p.atmo_rim_power as rhai::FLOAT).into(),
        );
        map.insert(
            "night_light_threshold".into(),
            (p.night_light_threshold as rhai::FLOAT).into(),
        );
        map.insert(
            "night_light_intensity".into(),
            (p.night_light_intensity as rhai::FLOAT).into(),
        );
        map.insert("sun_dir_x".into(), (p.sun_dir_x as rhai::FLOAT).into());
        map.insert("sun_dir_y".into(), (p.sun_dir_y as rhai::FLOAT).into());
        map.insert("sun_dir_z".into(), (p.sun_dir_z as rhai::FLOAT).into());
        map.insert(
            "surface_spin_dps".into(),
            (p.surface_spin_dps as rhai::FLOAT).into(),
        );
        map.insert(
            "cloud_spin_dps".into(),
            (p.cloud_spin_dps as rhai::FLOAT).into(),
        );
        map.insert(
            "cloud_spin_2_dps".into(),
            (p.cloud_spin_2_dps as rhai::FLOAT).into(),
        );
        map.insert(
            "cloud_threshold".into(),
            (p.cloud_threshold as rhai::FLOAT).into(),
        );
        map.insert(
            "cloud_noise_scale".into(),
            (p.cloud_noise_scale as rhai::FLOAT).into(),
        );
        map.insert(
            "cloud_noise_octaves".into(),
            (p.cloud_noise_octaves as rhai::INT).into(),
        );
        if let Some(c) = p.polar_ice_color {
            map.insert("polar_ice_color".into(), c.into());
        }
        if let Some(c) = p.desert_color {
            map.insert("desert_color".into(), c.into());
        }
        if let Some(c) = p.atmo_color {
            map.insert("atmo_color".into(), c.into());
        }
        if let Some(c) = p.night_light_color {
            map.insert("night_light_color".into(), c.into());
        }
        if let Some(c) = p.cloud_color {
            map.insert("cloud_color".into(), c.into());
        }
        if let Some(c) = p.shadow_color {
            map.insert("shadow_color".into(), c.into());
        }
        if let Some(c) = p.midtone_color {
            map.insert("midtone_color".into(), c.into());
        }
        if let Some(c) = p.highlight_color {
            map.insert("highlight_color".into(), c.into());
        }
        map
    }

    pub(crate) fn collisions(&mut self) -> RhaiArray {
        self.ctx
            .collisions
            .iter()
            .map(|hit| {
                let mut map = RhaiMap::new();
                map.insert("a".into(), (hit.a as rhai::INT).into());
                map.insert("b".into(), (hit.b as rhai::INT).into());
                map.into()
            })
            .collect()
    }

    pub(crate) fn collisions_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return vec![];
        };
        filter_hits_by_kind(&self.ctx.collisions, world, kind_a, kind_b)
    }

    pub(crate) fn collisions_of(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return vec![];
        };
        filter_hits_of_kind(&self.ctx.collisions, world, kind)
    }

    pub(crate) fn collision_enters_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return vec![];
        };
        let key = (kind_a.to_string(), kind_b.to_string());
        if let Some(cached) = self.collision_enters_cache.get(&key) {
            return cached.clone();
        }
        let result = filter_hits_by_kind(&self.ctx.collision_enters, world, kind_a, kind_b);
        self.collision_enters_cache.insert(key, result.clone());
        result
    }

    pub(crate) fn collision_stays_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return vec![];
        };
        filter_hits_by_kind(&self.ctx.collision_stays, world, kind_a, kind_b)
    }

    pub(crate) fn collision_exits_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return vec![];
        };
        filter_hits_by_kind(&self.ctx.collision_exits, world, kind_a, kind_b)
    }

    pub(crate) fn spawn_child_entity(
        &mut self,
        parent_id: rhai::INT,
        kind: &str,
        template: &str,
        data: RhaiMap,
    ) -> rhai::INT {
        if parent_id < 0 {
            return 0;
        }
        // Check parent exists before taking &mut self via spawn_visual
        let parent_uid = parent_id as u64;
        let parent_exists = self
            .ctx
            .world
            .as_ref()
            .map(|w| w.exists(parent_uid))
            .unwrap_or(false);
        if !parent_exists {
            return 0;
        }
        let child_id = self.spawn_visual(kind, template, data);
        if child_id > 0 {
            if let Some(world) = self.ctx.world.as_ref() {
                world.register_child(parent_uid, child_id as u64);
            }
        }
        child_id
    }

    pub(crate) fn despawn_children_of(&mut self, parent_id: rhai::INT) {
        if parent_id < 0 {
            return;
        }
        let Some(world) = self.ctx.world.as_ref() else {
            return;
        };
        world.despawn_children(parent_id as u64);
    }

    pub(crate) fn enable_wrap(
        &mut self,
        id: rhai::INT,
        min_x: rhai::FLOAT,
        max_x: rhai::FLOAT,
        min_y: rhai::FLOAT,
        max_y: rhai::FLOAT,
    ) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        let bounds =
            engine_game::WrapBounds::new(min_x as f32, max_x as f32, min_y as f32, max_y as f32);
        world.set_wrap_bounds(uid, bounds)
    }

    pub(crate) fn disable_wrap(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        world.remove_wrap_bounds(uid);
        true
    }

    pub(crate) fn set_world_bounds(
        &mut self,
        min_x: rhai::FLOAT,
        min_y: rhai::FLOAT,
        max_x: rhai::FLOAT,
        max_y: rhai::FLOAT,
    ) {
        let Some(world) = self.ctx.world.as_ref() else {
            return;
        };
        world.set_world_bounds(min_x as f32, max_x as f32, min_y as f32, max_y as f32);
    }

    pub(crate) fn world_bounds(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        match world.world_bounds() {
            Some(b) => {
                let mut map = RhaiMap::new();
                map.insert("min_x".into(), (b.min_x as rhai::FLOAT).into());
                map.insert("max_x".into(), (b.max_x as rhai::FLOAT).into());
                map.insert("min_y".into(), (b.min_y as rhai::FLOAT).into());
                map.insert("max_y".into(), (b.max_y as rhai::FLOAT).into());
                map
            }
            None => RhaiMap::new(),
        }
    }

    /// Returns just the world width (`max_x - min_x`) as a scalar.
    /// Avoids allocating a Rhai map when only dimensions are needed.
    pub(crate) fn world_width(&mut self) -> rhai::FLOAT {
        self.ctx
            .world
            .as_ref()
            .and_then(|w| w.world_bounds())
            .map(|b| (b.max_x - b.min_x) as rhai::FLOAT)
            .unwrap_or(0.0)
    }

    /// Returns just the world height (`max_y - min_y`) as a scalar.
    pub(crate) fn world_height(&mut self) -> rhai::FLOAT {
        self.ctx
            .world
            .as_ref()
            .and_then(|w| w.world_bounds())
            .map(|b| (b.max_y - b.min_y) as rhai::FLOAT)
            .unwrap_or(0.0)
    }

    /// Move the world-space camera so that world position `(x, y)` maps to the
    /// top-left corner of the visible viewport.
    ///
    /// Call each frame with `(ship_x - half_w, ship_y - half_h)` to keep the
    /// ship centered. UI layers (marked `ui: true` in scene YAML) are not affected.
    pub(crate) fn set_camera(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) {
        let Ok(mut queue) = self.ctx.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::SetCamera2d {
                x: x as f32,
                y: y as f32,
                zoom: None,
            },
        });
    }

    /// Set the 2D camera zoom factor.
    ///
    /// Values > 1.0 zoom in (fewer world pixels visible, objects appear larger).
    /// Values < 1.0 zoom out (more world pixels visible, objects appear smaller).
    /// Default is 1.0.
    pub(crate) fn set_camera_zoom(&mut self, zoom: rhai::FLOAT) {
        let Ok(mut queue) = self.ctx.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::SetCameraZoom { zoom: zoom as f32 });
    }

    pub(crate) fn set_camera_3d_look_at(
        &mut self,
        eye_x: rhai::FLOAT,
        eye_y: rhai::FLOAT,
        eye_z: rhai::FLOAT,
        target_x: rhai::FLOAT,
        target_y: rhai::FLOAT,
        target_z: rhai::FLOAT,
    ) {
        let Ok(mut queue) = self.ctx.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::LookAt {
                eye: [eye_x as f32, eye_y as f32, eye_z as f32],
                look_at: [target_x as f32, target_y as f32, target_z as f32],
            }),
        });
    }

    pub(crate) fn set_camera_3d_up(
        &mut self,
        up_x: rhai::FLOAT,
        up_y: rhai::FLOAT,
        up_z: rhai::FLOAT,
    ) {
        let Ok(mut queue) = self.ctx.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::SetCamera3d(Camera3dMutationRequest::Up {
                up: [up_x as f32, up_y as f32, up_z as f32],
            }),
        });
    }

    /// Attach an [`AngularBody`] to an entity.
    ///
    /// `config` map keys (all optional, snake_case field names):
    /// `accel`, `max`, `deadband`, `auto_brake`.
    pub(crate) fn angular_body_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let body = angular_body_from_rhai_map(&config);
        world.attach_angular_body(id as u64, body)
    }

    /// Set normalised turn input (−1.0…+1.0) for this frame.
    ///
    /// The `angular_body_system` reads this value and applies torque to the
    /// entity's angular velocity automatically — no manual tick needed.
    pub(crate) fn set_angular_input(&mut self, id: rhai::INT, input: rhai::FLOAT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.set_angular_input(id as u64, input as f32)
    }

    /// Read the current angular velocity (rad/s) of an entity's [`AngularBody`].
    ///
    /// Returns `0.0` if the entity has no `AngularBody`.
    pub(crate) fn angular_vel(&mut self, id: rhai::INT) -> rhai::FLOAT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0.0;
        };
        world.angular_vel(id as u64).unwrap_or(0.0) as rhai::FLOAT
    }

    /// Attach a [`LinearBrake`] component to an entity.
    ///
    /// Config map keys: `decel` (f32), `deadband` (f32), `auto_brake` (bool).
    pub(crate) fn linear_brake_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let brake = linear_brake_from_rhai_map(&config);
        world.attach_linear_brake(id as u64, brake)
    }

    /// Suppress auto-braking for this frame (call when entity is thrusting).
    pub(crate) fn set_linear_brake_active(&mut self, id: rhai::INT, active: bool) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.set_linear_brake_active(id as u64, active)
    }

    /// Attach a [`ThrusterRamp`] component to an entity.
    ///
    /// Config map keys (all optional):
    /// `thrust_delay_ms`, `thrust_ramp_ms`, `no_input_threshold_ms`,
    /// `rot_factor_max_vel`, `burst_speed_threshold`, `burst_wave_interval_ms`,
    /// `burst_wave_count`, `rot_deadband`, `move_deadband`.
    pub(crate) fn thruster_ramp_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let ramp = thruster_ramp_from_rhai_map(&config);
        world.attach_thruster_ramp(id as u64, ramp)
    }

    /// Read the per-frame factor outputs of a [`ThrusterRamp`] as a Rhai map.
    ///
    /// Returns an empty map if the entity has no `ThrusterRamp`.
    ///
    /// Map keys: `thrust_factor` (float), `rot_factor` (float),
    /// `brake_factor` (float), `brake_phase` (string),
    /// `final_burst_fired` (bool), `final_burst_wave` (int).
    pub(crate) fn thruster_ramp(&mut self, id: rhai::INT) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(ramp) = world.thruster_ramp(id as u64) else {
            return RhaiMap::new();
        };
        let mut map = RhaiMap::new();
        map.insert(
            "thrust_factor".into(),
            rhai::Dynamic::from_float(ramp.thrust_factor as rhai::FLOAT),
        );
        map.insert(
            "rot_factor".into(),
            rhai::Dynamic::from_float(ramp.rot_factor as rhai::FLOAT),
        );
        map.insert(
            "brake_factor".into(),
            rhai::Dynamic::from_float(ramp.brake_factor as rhai::FLOAT),
        );
        map.insert(
            "brake_phase".into(),
            rhai::Dynamic::from(ramp.brake_phase.as_str().to_string()),
        );
        map.insert(
            "final_burst_fired".into(),
            rhai::Dynamic::from_bool(ramp.final_burst_fired),
        );
        map.insert(
            "final_burst_wave".into(),
            rhai::Dynamic::from_int(ramp.final_burst_wave as rhai::INT),
        );
        map.insert(
            "thrust_ignition_ms".into(),
            rhai::Dynamic::from_float(ramp.thrust_ignition_ms as rhai::FLOAT),
        );
        map
    }

    /// Detach the [`ThrusterRamp`] component from an entity.
    pub(crate) fn thruster_ramp_detach(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.detach_thruster_ramp(id as u64)
    }

    pub(crate) fn rand_i(&mut self, min: rhai::INT, max: rhai::INT) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return min;
        };
        world.rand_i(min as i32, max as i32) as rhai::INT
    }

    pub(crate) fn rand_seed(&mut self, seed: rhai::INT) {
        let Some(world) = self.ctx.world.as_ref() else {
            return;
        };
        world.rand_seed(seed);
    }

    pub(crate) fn tag_add(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.tag_add(id as u64, tag)
    }

    pub(crate) fn tag_remove(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.tag_remove(id as u64, tag)
    }

    pub(crate) fn tag_has(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.tag_has(id as u64, tag)
    }

    pub(crate) fn after_ms(&mut self, label: &str, delay_ms: rhai::INT) {
        let Some(world) = self.ctx.world.as_ref() else {
            return;
        };
        world.after_ms(label, delay_ms);
    }

    pub(crate) fn timer_fired(&mut self, label: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.timer_fired(label)
    }

    pub(crate) fn cancel_timer(&mut self, label: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.cancel_timer(label)
    }

    /// Spawn multiple entities from an array of spec maps.
    /// Each map should have `kind: String` and optionally `data: Map`.
    /// Returns an array of spawned entity IDs.
    pub(crate) fn spawn_batch(&mut self, specs: rhai::Array) -> rhai::Array {
        let Some(world) = self.ctx.world.as_ref() else {
            return rhai::Array::new();
        };
        specs
            .into_iter()
            .filter_map(|spec| {
                let map = spec.try_cast::<RhaiMap>()?;
                let kind = map.get("kind")?.clone().try_cast::<String>()?;
                let data_dyn = map.get("data").cloned().unwrap_or_default();
                let data_json = rhai_dynamic_to_json(&data_dyn)
                    .unwrap_or(serde_json::Value::Object(Default::default()));
                let id = world.spawn(&kind, data_json)?;
                Some((id as rhai::INT).into())
            })
            .collect()
    }

    pub(crate) fn emit(
        &mut self,
        emitter_name: &str,
        owner_id: rhai::INT,
        args: RhaiMap,
    ) -> rhai::INT {
        let Some(world) = self.ctx.world.clone() else {
            return 0;
        };
        let Some(config) = self.catalogs.emitters.get(emitter_name).cloned() else {
            return 0;
        };
        let owner_uid = if owner_id > 0 { owner_id as u64 } else { 0 };
        if owner_uid == 0 || !world.exists(owner_uid) {
            return 0;
        }

        let owner = self.entity(owner_uid as rhai::INT);
        let xf = owner.clone().transform();
        let phys = owner.clone().physics();
        let heading = xf
            .get("heading")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);
        let x = xf
            .get("x")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);
        let y = xf
            .get("y")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);

        // Derive heading vector from Transform2D.heading (authoritative when AngularBody
        // is present) rather than ArcadeController.current_heading (which is not updated
        // by the AngularBody system and can be stale after rotation).
        let hx = heading.sin();
        let hy = -heading.cos();
        let base_vx = phys
            .get("vx")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);
        let base_vy = phys
            .get("vy")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);

        let cooldown_name = config
            .cooldown_name
            .clone()
            .unwrap_or_else(|| emitter_name.to_string());
        let cooldown_ms = config.cooldown_ms.unwrap_or(0).max(0) as f64;
        let min_cooldown_ms = config
            .min_cooldown_ms
            .unwrap_or(config.cooldown_ms.unwrap_or(0))
            .max(0) as f64;
        let ramp_ms = config.ramp_ms.unwrap_or(0).max(0) as f64;
        let thrust_ms = Self::map_int(&args, "thrust_ms", 0).max(0) as f64;
        let effective_cooldown = if ramp_ms > 0.0 && cooldown_ms > min_cooldown_ms {
            let t = (thrust_ms / ramp_ms).clamp(0.0, 1.0);
            cooldown_ms + (min_cooldown_ms - cooldown_ms) * t
        } else {
            cooldown_ms.max(min_cooldown_ms)
        };
        if effective_cooldown > 0.0 && !owner.clone().cooldown_ready(&cooldown_name) {
            return 0;
        }

        if let Some(max_count) = config.max_count.filter(|value| *value > 0) {
            if let Some(state) = &self.emitter_state {
                while state.active_count(emitter_name, Some(owner_uid)) >= max_count as usize {
                    let Some(oldest) = state.evict_oldest(emitter_name, Some(owner_uid)) else {
                        break;
                    };
                    // Queue visual despawn before removing the gameplay entity so
                    // the scene layer/sprite is cleaned up alongside the entity.
                    if let Some(binding) = world.visual(oldest) {
                        if let Ok(mut q) = self.ctx.queue.lock() {
                            for vid in binding.all_visual_ids() {
                                q.push(BehaviorCommand::ApplySceneMutation {
                                    request: SceneMutationRequest::DespawnObject {
                                        target: vid.to_string(),
                                    },
                                });
                            }
                        }
                    }
                    let _ = world.despawn(oldest);
                }
            }
        }

        let (spawn_offset, config_side_offset) = Self::resolve_emit_anchor_offsets(&config, &args);
        let backward_speed = config.backward_speed.unwrap_or(0.0);
        let velocity_scale = config.velocity_scale.unwrap_or(1.0);
        // side_offset: perpendicular right offset from heading direction.
        // Right-perp of (hx, hy) is (hy, -hx).  Negative values = left side.
        let side_offset = config_side_offset + Self::map_number(&args, "side_offset", 0.0);
        let resolved = self.resolve_emit(&config, &args, spawn_offset, hx, hy);

        // Base emission axis is provided in world-space by resolve_emit.
        // Additional spread is applied around that base.
        let dir_angle = resolved.base_dir_y.atan2(resolved.base_dir_x) + resolved.spread;
        let dir_x = dir_angle.cos();
        let dir_y = dir_angle.sin();

        let Some(id) = spawn_ephemeral_visual(
            &world,
            &self.ctx.queue,
            EphemeralSpawn {
                kind: resolved.kind.clone(),
                template: resolved.template.clone(),
                x: (x - hx * spawn_offset + hy * side_offset) as f32,
                y: (y - hy * spawn_offset - hx * side_offset) as f32,
                heading: heading as f32,
                vx: (base_vx * backward_speed + dir_x * resolved.speed * velocity_scale) as f32,
                vy: (base_vy * backward_speed + dir_y * resolved.speed * velocity_scale) as f32,
                drag: 0.0,
                max_speed: 0.0,
                ttl_ms: Some(resolved.ttl_ms),
                owner_id: resolved.lifecycle.is_owner_bound().then_some(owner_uid),
                lifecycle: resolved.lifecycle,
                follow_anchor: resolved.follow_anchor,
                extra_data: resolved.extra_data,
            },
        ) else {
            return 0;
        };

        if let Some(binding) = world.visual(id) {
            if let Some(visual_id) = binding.visual_id {
                if !resolved.fg.trim().is_empty() {
                    if let Ok(mut queue) = self.ctx.queue.lock() {
                        queue_set_property_or_mutation(
                            &mut queue,
                            visual_id.clone(),
                            "style.fg".to_string(),
                            JsonValue::from(resolved.fg.clone()),
                        );
                    }
                }
                if resolved.radius > 1 {
                    let points = vec![[0, 0], [resolved.radius as i32, 0]];
                    if let Ok(mut queue) = self.ctx.queue.lock() {
                        queue_set_property_or_mutation(
                            &mut queue,
                            visual_id,
                            "vector.points".to_string(),
                            JsonValue::Array(
                                points
                                    .into_iter()
                                    .map(|[px, py]| {
                                        JsonValue::Array(vec![
                                            JsonValue::from(px),
                                            JsonValue::from(py),
                                        ])
                                    })
                                    .collect(),
                            ),
                        );
                    }
                }
            }
        }

        // Attach particle physics configuration if emitter specifies thread_mode/collision/gravity
        if config.thread_mode.is_some()
            || config.collision.is_some()
            || config.gravity_scale.is_some()
            || config.gravity_mode.is_some()
        {
            let thread_mode = config
                .thread_mode
                .as_deref()
                .map(ParticleThreadMode::from_str)
                .unwrap_or_default();
            let collision = config.collision.unwrap_or(false);
            let gravity_scale = config.gravity_scale.unwrap_or(0.0) as f32;
            let bounce = config.bounce.unwrap_or(0.0) as f32;
            let mass = config.mass.unwrap_or(1.0) as f32;
            let collision_mask = config.collision_mask.clone().unwrap_or_default();
            let gravity_mode = match config.gravity_mode.as_deref() {
                Some(s) if s.eq_ignore_ascii_case("orbital") => {
                    engine_game::components::ParticleGravityMode::Orbital
                }
                _ => engine_game::components::ParticleGravityMode::Flat,
            };
            let gravity_center_x = args
                .get("gravity_center_x")
                .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
                .map(|v| v as f32)
                .unwrap_or_else(|| config.gravity_center_x.unwrap_or(0.0) as f32);
            let gravity_center_y = args
                .get("gravity_center_y")
                .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
                .map(|v| v as f32)
                .unwrap_or_else(|| config.gravity_center_y.unwrap_or(0.0) as f32);
            let gravity_center_z = args
                .get("gravity_center_z")
                .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
                .map(|v| v as f32)
                .unwrap_or_else(|| config.gravity_center_z.unwrap_or(0.0) as f32);
            let gravity_constant = config.gravity_constant.unwrap_or(0.0) as f32;

            let particle_physics = ParticlePhysics {
                thread_mode,
                collision,
                collision_mask,
                gravity_scale,
                bounce,
                mass,
                gravity_mode,
                gravity_center_x,
                gravity_center_y,
                gravity_center_z,
                gravity_constant,
            };
            let _ = world.attach_particle_physics(id, particle_physics);
        }

        // Resolve color ramp: args override > active palette[palette_ramp] > catalog static fallback
        let ramp_colors: Option<Vec<String>> = args
            .get("color_ramp")
            .and_then(|v| v.clone().try_cast::<rhai::Array>())
            .map(|arr| {
                arr.into_iter()
                    .filter_map(|c| c.try_cast::<String>())
                    .collect()
            })
            .or_else(|| {
                config
                    .palette_ramp
                    .as_deref()
                    .and_then(|name| self.resolve_palette_ramp(name))
            })
            .or_else(|| config.color_ramp.clone());
        if let Some(colors) = ramp_colors {
            if !colors.is_empty() {
                let radius_max = args
                    .get("radius_max")
                    .and_then(|v| v.clone().try_cast::<rhai::INT>())
                    .unwrap_or_else(|| config.radius_max.unwrap_or(resolved.radius))
                    as i32;
                let radius_min = args
                    .get("radius_min")
                    .and_then(|v| v.clone().try_cast::<rhai::INT>())
                    .unwrap_or_else(|| config.radius_min.unwrap_or(0))
                    as i32;
                let _ = world.attach_particle_ramp(
                    id,
                    ParticleColorRamp {
                        colors,
                        radius_max,
                        radius_min,
                    },
                );
            }
        }

        if effective_cooldown > 0.0 {
            let _ = self
                .entity(owner_uid as rhai::INT)
                .cooldown_start(&cooldown_name, effective_cooldown.round() as rhai::INT);
        }
        if let Some(state) = &self.emitter_state {
            state.track_spawn(emitter_name, Some(owner_uid), id);
        }
        id as rhai::INT
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_ephemeral_prefab(
        &self,
        prefab: &catalog::PrefabTemplate,
        args: &RhaiMap,
        lifecycle_str: &str,
        vx: rhai::FLOAT,
        vy: rhai::FLOAT,
        drag: f64,
        max_speed: f64,
    ) -> EphemeralPrefabResolved {
        let owner_id = Self::map_int(args, "owner_id", 0);
        let lifecycle = parse_lifecycle_policy(lifecycle_str, LifecyclePolicy::Ttl);
        let follow_anchor = lifecycle
            .follows_owner()
            .then(|| follow_anchor_from_args(args, 0.0, 0.0, true));

        let mut extra_data = BTreeMap::new();
        if let Some(components) = &prefab.components {
            if let Some(extra) = &components.extra_data {
                for (k, v) in extra {
                    extra_data.insert(k.clone(), v.clone());
                }
            }
        }
        extra_data = merge_json_map(
            extra_data,
            args,
            &[
                "x", "y", "heading", "vx", "vy", "radius", "ttl_ms", "owner_id", "fg", "kind",
                "template",
            ],
        );
        if let Some(radius) = args.get("radius") {
            if let Ok(r) = radius.as_int() {
                extra_data.insert("radius".to_string(), JsonValue::from(r));
            }
        }

        EphemeralPrefabResolved {
            ttl_ms: Self::map_int(args, "ttl_ms", 0) as i32,
            vx: vx as f32,
            vy: vy as f32,
            drag: drag as f32,
            max_speed: max_speed as f32,
            owner_id: (owner_id > 0).then_some(owner_id as u64),
            lifecycle,
            follow_anchor,
            extra_data,
        }
    }

    fn resolve_emit(
        &self,
        config: &catalog::EmitterConfig,
        args: &RhaiMap,
        spawn_offset: f64,
        heading_x: f64,
        heading_y: f64,
    ) -> EmitResolved {
        let owner_bound = args
            .get("owner_bound")
            .and_then(|v| v.clone().try_cast::<bool>())
            .unwrap_or(false);
        let lifecycle_name = args
            .get("lifecycle")
            .and_then(|v| v.clone().try_cast::<String>())
            .or_else(|| config.lifecycle.clone())
            .unwrap_or_else(|| {
                if owner_bound {
                    "TtlOwnerBound".to_string()
                } else {
                    "Ttl".to_string()
                }
            });
        let lifecycle = parse_lifecycle_policy(
            &lifecycle_name,
            if owner_bound {
                LifecyclePolicy::TtlOwnerBound
            } else {
                LifecyclePolicy::Ttl
            },
        );
        let follow_anchor = lifecycle.follows_owner().then(|| {
            follow_anchor_from_args(
                args,
                config.follow_local_x.unwrap_or(-spawn_offset),
                config.follow_local_y.unwrap_or(0.0),
                config.follow_inherit_heading.unwrap_or(true),
            )
        });

        let radius = Self::map_int(args, "radius", config.radius.unwrap_or(1)).max(1);
        let fg = map_string(args, "fg").unwrap_or_default();
        let mut extra_data = BTreeMap::new();
        extra_data.insert("radius".to_string(), JsonValue::from(radius));
        if !fg.trim().is_empty() {
            extra_data.insert("fg".to_string(), JsonValue::from(fg.clone()));
        }

        let (base_dir_x, base_dir_y) =
            Self::resolve_emit_base_dir(config, args, heading_x, heading_y);
        let base_emission_angle = config.emission_angle.unwrap_or(0.0);
        EmitResolved {
            speed: Self::map_number(args, "speed", 0.0),
            base_dir_x,
            base_dir_y,
            spread: base_emission_angle + Self::map_number(args, "spread", 0.0),
            ttl_ms: Self::map_int(args, "ttl_ms", config.ttl_ms.unwrap_or(250)).max(1) as i32,
            radius,
            template: map_string(args, "template").unwrap_or_else(|| "debris".to_string()),
            kind: map_string(args, "kind").unwrap_or_else(|| "fx".to_string()),
            fg,
            lifecycle,
            follow_anchor,
            extra_data,
        }
    }

    /// Resolve base emission axis in world-space.
    /// Priority:
    /// 1) args.emission_local_x/y
    /// 2) config.emission_local_x/y
    /// 3) default owner backward axis (-heading)
    fn resolve_emit_base_dir(
        config: &catalog::EmitterConfig,
        args: &RhaiMap,
        heading_x: f64,
        heading_y: f64,
    ) -> (f64, f64) {
        let arg_local_x = args
            .get("emission_local_x")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>());
        let arg_local_y = args
            .get("emission_local_y")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>());
        if let (Some(local_x), Some(local_y)) = (arg_local_x, arg_local_y) {
            if let Some(dir) = Self::local_vec_to_world_unit(local_x, local_y, heading_x, heading_y)
            {
                return dir;
            }
        }

        if let (Some(local_x), Some(local_y)) = (config.emission_local_x, config.emission_local_y) {
            if let Some(dir) = Self::local_vec_to_world_unit(local_x, local_y, heading_x, heading_y)
            {
                return dir;
            }
        }

        (-heading_x, -heading_y)
    }

    /// Convert an owner-local direction vector into normalized world-space direction.
    /// Local frame: +x right, +y down.
    fn local_vec_to_world_unit(
        local_x: f64,
        local_y: f64,
        heading_x: f64,
        heading_y: f64,
    ) -> Option<(f64, f64)> {
        let len = (local_x * local_x + local_y * local_y).sqrt();
        if len <= f64::EPSILON {
            return None;
        }
        let nx = local_x / len;
        let ny = local_y / len;
        // Build owner-local basis from heading:
        // forward=(hx,hy), right=(-hy,hx), down=backward=( -hx,-hy )
        // local(+x right,+y down): world = right*lx + down*ly
        let right_x = -heading_y;
        let right_y = heading_x;
        let down_x = -heading_x;
        let down_y = -heading_y;
        let wx = right_x * nx + down_x * ny;
        let wy = right_y * nx + down_y * ny;
        let wlen = (wx * wx + wy * wy).sqrt();
        if wlen <= f64::EPSILON {
            None
        } else {
            Some((wx / wlen, wy / wlen))
        }
    }

    /// Resolve emitter anchor as compatibility (spawn_offset/side_offset) from either:
    /// 1) args.local_x/local_y
    /// 2) config.local_x/local_y
    /// 3) config.edge_{from,to}_* + edge_t interpolation
    /// 4) compatibility config.spawn_offset/config.side_offset fallback
    fn resolve_emit_anchor_offsets(config: &catalog::EmitterConfig, args: &RhaiMap) -> (f64, f64) {
        let arg_local_x = args
            .get("local_x")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>());
        let arg_local_y = args
            .get("local_y")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>());
        if let (Some(local_x), Some(local_y)) = (arg_local_x, arg_local_y) {
            return (local_y, -local_x);
        }

        if let (Some(local_x), Some(local_y)) = (config.local_x, config.local_y) {
            return (local_y, -local_x);
        }

        if let (Some(from_x), Some(from_y), Some(to_x), Some(to_y)) = (
            config.edge_from_x,
            config.edge_from_y,
            config.edge_to_x,
            config.edge_to_y,
        ) {
            let t = config.edge_t.unwrap_or(0.5).clamp(0.0, 1.0);
            let local_x = from_x + (to_x - from_x) * t;
            let local_y = from_y + (to_y - from_y) * t;
            return (local_y, -local_x);
        }

        (
            config.spawn_offset.unwrap_or(0.0),
            config.side_offset.unwrap_or(0.0),
        )
    }

    pub(crate) fn poll_collision_events(&mut self) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        let collisions = world.poll_events("collision_enter");
        let mut array = RhaiArray::new();
        for (a, b) in collisions {
            let mut event = RhaiMap::new();
            event.insert("a".into(), (a as rhai::INT).into());
            event.insert("b".into(), (b as rhai::INT).into());
            array.push(RhaiDynamic::from(event));
        }
        array
    }

    pub(crate) fn clear_events(&mut self) {
        if let Some(world) = self.ctx.world.as_ref() {
            world.clear_events();
        }
    }

    pub(crate) fn distance(&mut self, a: rhai::INT, b: rhai::INT) -> rhai::FLOAT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0.0;
        };
        let ta = world.transform(a as u64);
        let tb = world.transform(b as u64);
        match (ta, tb) {
            (Some(a), Some(b)) => {
                let dx = a.x - b.x;
                let dy = a.y - b.y;
                ((dx * dx + dy * dy) as rhai::FLOAT).sqrt()
            }
            _ => 0.0,
        }
    }

    pub(crate) fn any_alive(&mut self, kind: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.count_kind(kind) > 0
    }
}

impl ScriptGameplayBodySnapshotApi {
    fn new(
        id: String,
        body: Option<catalog::BodyDef>,
        spatial_meters_per_world_unit: Option<f64>,
    ) -> Self {
        Self {
            id,
            body,
            spatial_meters_per_world_unit,
        }
    }

    fn body_ref(&self) -> Option<&catalog::BodyDef> {
        self.body.as_ref()
    }

    fn km_per_world_unit_value(&self) -> f64 {
        self.body_ref()
            .and_then(|body| body.km_per_world_unit(self.spatial_meters_per_world_unit))
            .unwrap_or(0.0)
    }

    fn radius_km_value(&self) -> f64 {
        self.body_ref()
            .and_then(|body| body.resolved_radius_km(self.spatial_meters_per_world_unit))
            .unwrap_or(0.0)
    }

    fn atmosphere_top_km_value(&self) -> f64 {
        self.body_ref()
            .and_then(|body| body.resolved_atmosphere_top_km(self.spatial_meters_per_world_unit))
            .unwrap_or(0.0)
    }

    fn atmosphere_dense_start_km_value(&self) -> f64 {
        self.body_ref()
            .and_then(|body| {
                body.resolved_atmosphere_dense_start_km(self.spatial_meters_per_world_unit)
            })
            .unwrap_or(0.0)
    }

    fn gravity_mu_value(&self) -> f64 {
        self.body_ref()
            .map(|body| body.resolved_gravity_mu_world_units(self.spatial_meters_per_world_unit))
            .unwrap_or(0.0)
    }

    fn km_per_px_value(&self) -> f64 {
        self.body_ref()
            .and_then(|body| {
                body.km_per_px.or_else(|| {
                    body.km_per_world_unit(self.spatial_meters_per_world_unit)
                        .or_else(|| {
                            body.resolved_radius_km(self.spatial_meters_per_world_unit)
                                .map(|radius_km| radius_km / body.surface_radius.max(0.0001))
                        })
                })
            })
            .unwrap_or(0.0)
    }

    fn compat_info_map(
        requested_id: &str,
        body: Option<&catalog::BodyDef>,
        spatial_meters_per_world_unit: Option<f64>,
    ) -> RhaiMap {
        let exists = body.is_some();
        let body = body.cloned().unwrap_or_default();
        let km_per_world_unit = body.km_per_world_unit(spatial_meters_per_world_unit);
        let radius_km = body.resolved_radius_km(spatial_meters_per_world_unit);
        let km_per_px = body.km_per_px.or_else(|| {
            km_per_world_unit.or_else(|| {
                radius_km
                    .map(|resolved_radius_km| resolved_radius_km / body.surface_radius.max(0.0001))
            })
        });
        let atmosphere_top_km = body.resolved_atmosphere_top_km(spatial_meters_per_world_unit);
        let atmosphere_dense_start_km =
            body.resolved_atmosphere_dense_start_km(spatial_meters_per_world_unit);
        let mut map = RhaiMap::new();
        if !requested_id.is_empty() {
            map.insert("id".into(), requested_id.to_string().into());
        }
        map.insert("exists".into(), exists.into());
        map.insert("center_x".into(), (body.center_x as rhai::FLOAT).into());
        map.insert("center_y".into(), (body.center_y as rhai::FLOAT).into());
        map.insert(
            "orbit_radius".into(),
            (body.orbit_radius as rhai::FLOAT).into(),
        );
        map.insert(
            "orbit_period_sec".into(),
            (body.orbit_period_sec as rhai::FLOAT).into(),
        );
        map.insert(
            "orbit_phase_deg".into(),
            (body.orbit_phase_deg as rhai::FLOAT).into(),
        );
        map.insert("radius_px".into(), (body.radius_px as rhai::FLOAT).into());
        map.insert(
            "surface_radius".into(),
            (body.surface_radius as rhai::FLOAT).into(),
        );
        map.insert("gravity_mu".into(), (body.gravity_mu as rhai::FLOAT).into());
        if let Some(v) = body.gravity_mu_km3_s2 {
            map.insert("gravity_mu_km3_s2".into(), v.into());
        }
        if let Some(v) = km_per_world_unit {
            map.insert("km_per_world_unit".into(), v.into());
        }
        if let Some(v) = radius_km {
            map.insert("radius_km".into(), v.into());
            map.insert("resolved_radius_km".into(), v.into());
        }
        map.insert(
            "resolved_gravity_mu".into(),
            body.resolved_gravity_mu_world_units(spatial_meters_per_world_unit)
                .into(),
        );
        if let Some(v) = km_per_px {
            map.insert("km_per_px".into(), v.into());
        }
        if let Some(v) = atmosphere_top_km {
            map.insert("atmosphere_top_km".into(), v.into());
            map.insert("resolved_atmosphere_top_km".into(), v.into());
        }
        if let Some(v) = atmosphere_dense_start_km {
            map.insert("atmosphere_dense_start_km".into(), v.into());
            map.insert("resolved_atmosphere_dense_start_km".into(), v.into());
        }
        if let Some(v) = body.atmosphere_top {
            map.insert("atmosphere_top".into(), v.into());
        }
        if let Some(v) = body.atmosphere_dense_start {
            map.insert("atmosphere_dense_start".into(), v.into());
        }
        if let Some(v) = body.atmosphere_drag_max {
            map.insert("atmosphere_drag_max".into(), v.into());
        }
        if let Some(v) = body.cloud_bottom_km {
            map.insert("cloud_bottom_km".into(), v.into());
        }
        if let Some(v) = body.cloud_top_km {
            map.insert("cloud_top_km".into(), v.into());
        }
        if let Some(s) = body.planet_type {
            map.insert("planet_type".into(), s.into());
        }
        if let Some(s) = body.parent {
            map.insert("parent".into(), s.into());
        }
        map
    }

    pub(crate) fn exists(&mut self) -> bool {
        self.body.is_some()
    }

    pub(crate) fn id(&mut self) -> String {
        self.id.clone()
    }

    pub(crate) fn center_x(&mut self) -> rhai::FLOAT {
        self.body_ref().map(|body| body.center_x).unwrap_or(0.0)
    }

    pub(crate) fn center_y(&mut self) -> rhai::FLOAT {
        self.body_ref().map(|body| body.center_y).unwrap_or(0.0)
    }

    pub(crate) fn orbit_radius(&mut self) -> rhai::FLOAT {
        self.body_ref().map(|body| body.orbit_radius).unwrap_or(0.0)
    }

    pub(crate) fn orbit_period_sec(&mut self) -> rhai::FLOAT {
        self.body_ref()
            .map(|body| body.orbit_period_sec)
            .unwrap_or(0.0)
    }

    pub(crate) fn orbit_phase_deg(&mut self) -> rhai::FLOAT {
        self.body_ref()
            .map(|body| body.orbit_phase_deg)
            .unwrap_or(0.0)
    }

    pub(crate) fn radius_px(&mut self) -> rhai::FLOAT {
        self.body_ref().map(|body| body.radius_px).unwrap_or(0.0)
    }

    pub(crate) fn surface_radius(&mut self) -> rhai::FLOAT {
        self.body_ref()
            .map(|body| body.surface_radius)
            .unwrap_or(0.0)
    }

    pub(crate) fn gravity_mu(&mut self) -> rhai::FLOAT {
        self.gravity_mu_value()
    }

    pub(crate) fn gravity_mu_km3_s2(&mut self) -> rhai::FLOAT {
        self.body_ref()
            .and_then(|body| body.gravity_mu_km3_s2)
            .unwrap_or(0.0)
    }

    pub(crate) fn km_per_px(&mut self) -> rhai::FLOAT {
        self.km_per_px_value()
    }

    pub(crate) fn km_per_world_unit(&mut self) -> rhai::FLOAT {
        self.km_per_world_unit_value()
    }

    pub(crate) fn radius_km(&mut self) -> rhai::FLOAT {
        self.radius_km_value()
    }

    pub(crate) fn resolved_radius_km(&mut self) -> rhai::FLOAT {
        self.radius_km_value()
    }

    pub(crate) fn resolved_gravity_mu(&mut self) -> rhai::FLOAT {
        self.gravity_mu_value()
    }

    pub(crate) fn atmosphere_top_km(&mut self) -> rhai::FLOAT {
        self.atmosphere_top_km_value()
    }

    pub(crate) fn atmosphere_dense_start_km(&mut self) -> rhai::FLOAT {
        self.atmosphere_dense_start_km_value()
    }

    pub(crate) fn resolved_atmosphere_top_km(&mut self) -> rhai::FLOAT {
        self.atmosphere_top_km_value()
    }

    pub(crate) fn resolved_atmosphere_dense_start_km(&mut self) -> rhai::FLOAT {
        self.atmosphere_dense_start_km_value()
    }

    pub(crate) fn atmosphere_drag_max(&mut self) -> rhai::FLOAT {
        self.body_ref()
            .and_then(|body| body.atmosphere_drag_max)
            .unwrap_or(0.0)
    }

    pub(crate) fn cloud_bottom_km(&mut self) -> rhai::FLOAT {
        self.body_ref()
            .and_then(|body| body.cloud_bottom_km)
            .unwrap_or(0.0)
    }

    pub(crate) fn cloud_top_km(&mut self) -> rhai::FLOAT {
        self.body_ref()
            .and_then(|body| body.cloud_top_km)
            .unwrap_or(0.0)
    }

    pub(crate) fn planet_type(&mut self) -> String {
        self.body_ref()
            .and_then(|body| body.planet_type.clone())
            .unwrap_or_default()
    }

    pub(crate) fn parent(&mut self) -> String {
        self.body_ref()
            .and_then(|body| body.parent.clone())
            .unwrap_or_default()
    }

    pub(crate) fn inspect(&mut self) -> RhaiMap {
        Self::compat_info_map(
            &self.id,
            self.body.as_ref(),
            self.spatial_meters_per_world_unit,
        )
    }
}

impl ScriptGameplayEntityApi {
    pub(crate) fn exists(&mut self) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.exists(self.ctx.id)
    }

    pub(crate) fn id(&mut self) -> rhai::INT {
        self.ctx.id as rhai::INT
    }

    pub(crate) fn flag(&mut self, name: &str) -> bool {
        let path = format!("/{}", name);
        self.get_bool(&path, false)
    }

    pub(crate) fn set_flag(&mut self, name: &str, value: bool) -> bool {
        let path = format!("/{}", name);
        self.set(&path, value.into())
    }

    pub(crate) fn despawn(&mut self) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let tree_ids = world.despawn_tree_ids(self.ctx.id);
        if let Ok(mut commands) = self.ctx.queue.lock() {
            for tree_id in &tree_ids {
                if let Some(binding) = world.visual(*tree_id) {
                    for vid in binding.all_visual_ids() {
                        commands.push(BehaviorCommand::ApplySceneMutation {
                            request: SceneMutationRequest::DespawnObject {
                                target: vid.to_string(),
                            },
                        });
                    }
                }
            }
        }
        world.despawn(self.ctx.id)
    }

    pub(crate) fn get(&mut self, path: &str) -> RhaiDynamic {
        let Some(world) = self.ctx.world.as_ref() else {
            return ().into();
        };
        world
            .get(self.ctx.id, path)
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    pub(crate) fn get_i(&mut self, path: &str, fallback: rhai::INT) -> rhai::INT {
        self.get(path).try_cast::<rhai::INT>().unwrap_or(fallback)
    }

    pub(crate) fn get_bool(&mut self, path: &str, fallback: bool) -> bool {
        self.get(path).try_cast::<bool>().unwrap_or(fallback)
    }

    pub(crate) fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.set(self.ctx.id, path, value)
    }

    pub(crate) fn kind(&mut self) -> String {
        let Some(world) = self.ctx.world.as_ref() else {
            return String::new();
        };
        world.kind_of(self.ctx.id).unwrap_or_default()
    }

    /// Returns the remaining lifetime as a fraction of the original TTL (1.0 = fresh, 0.0 = about to expire).
    pub(crate) fn lifetime_fraction(&mut self) -> rhai::FLOAT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0.0;
        };
        let Some(lt) = world.lifetime(self.ctx.id) else {
            return 0.0;
        };
        if lt.original_ttl_ms <= 0 {
            return 0.0;
        }
        (lt.ttl_ms as rhai::FLOAT / lt.original_ttl_ms as rhai::FLOAT).clamp(0.0, 1.0)
    }

    /// Queues a style.fg colour update on this entity's bound visual.
    pub(crate) fn set_fg(&mut self, color: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(binding) = world.visual(self.ctx.id) else {
            return false;
        };
        let Some(visual_id) = binding.visual_id else {
            return false;
        };
        let Ok(mut queue) = self.ctx.queue.lock() else {
            return false;
        };
        queue_set_property_or_mutation(
            &mut queue,
            visual_id,
            "style.fg".to_string(),
            JsonValue::from(color),
        );
        true
    }

    /// Queues a vector.points update to resize this entity's bound visual to [radius].
    /// r=1 leaves the template dot as-is; r=0 sets a zero-length point (invisible).
    pub(crate) fn set_radius(&mut self, r: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(binding) = world.visual(self.ctx.id) else {
            return false;
        };
        let Some(visual_id) = binding.visual_id else {
            return false;
        };
        let Ok(mut queue) = self.ctx.queue.lock() else {
            return false;
        };
        let r = r.max(0) as i32;
        queue_set_property_or_mutation(
            &mut queue,
            visual_id,
            "vector.points".to_string(),
            JsonValue::Array(vec![
                JsonValue::Array(vec![JsonValue::from(0), JsonValue::from(0)]),
                JsonValue::Array(vec![JsonValue::from(r), JsonValue::from(0)]),
            ]),
        );
        true
    }

    pub(crate) fn tags(&mut self) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .tags(self.ctx.id)
            .into_iter()
            .map(|tag| tag.into())
            .collect()
    }

    pub(crate) fn get_metadata(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(entity) = world.get_entity(self.ctx.id) else {
            return RhaiMap::new();
        };

        let mut metadata = RhaiMap::new();
        metadata.insert("id".into(), (self.ctx.id as rhai::INT).into());
        metadata.insert("kind".into(), entity.kind.into());

        let tags: RhaiArray = entity.tags.iter().map(|t| t.clone().into()).collect();
        metadata.insert("tags".into(), tags.into());

        // Include all components
        if let Some(transform) = world.transform(self.ctx.id) {
            let mut xf = RhaiMap::new();
            xf.insert("x".into(), (transform.x as rhai::FLOAT).into());
            xf.insert("y".into(), (transform.y as rhai::FLOAT).into());
            xf.insert("heading".into(), (transform.heading as rhai::FLOAT).into());
            metadata.insert("transform".into(), xf.into());
        }

        if let Some(physics) = world.physics(self.ctx.id) {
            let mut phys = RhaiMap::new();
            phys.insert("vx".into(), (physics.vx as rhai::FLOAT).into());
            phys.insert("vy".into(), (physics.vy as rhai::FLOAT).into());
            phys.insert("ax".into(), (physics.ax as rhai::FLOAT).into());
            phys.insert("ay".into(), (physics.ay as rhai::FLOAT).into());
            phys.insert("drag".into(), (physics.drag as rhai::FLOAT).into());
            phys.insert(
                "max_speed".into(),
                (physics.max_speed as rhai::FLOAT).into(),
            );
            phys.insert("mass".into(), (physics.mass as rhai::FLOAT).into());
            phys.insert(
                "restitution".into(),
                (physics.restitution as rhai::FLOAT).into(),
            );
            metadata.insert("physics".into(), phys.into());
        }

        if let Some(collider) = world.collider(self.ctx.id) {
            let mut coll = RhaiMap::new();
            match &collider.shape {
                ColliderShape::Circle { radius } => {
                    coll.insert("shape".into(), "circle".into());
                    coll.insert("radius".into(), (*radius as rhai::FLOAT).into());
                }
                ColliderShape::Polygon { points } => {
                    coll.insert("shape".into(), "polygon".into());
                    let pts: RhaiArray = points
                        .iter()
                        .map(|p| {
                            let mut point = RhaiMap::new();
                            point.insert("x".into(), (p[0] as rhai::FLOAT).into());
                            point.insert("y".into(), (p[1] as rhai::FLOAT).into());
                            point.into()
                        })
                        .collect();
                    coll.insert("points".into(), pts.into());
                }
            }
            coll.insert("layer".into(), (collider.layer as rhai::INT).into());
            coll.insert("mask".into(), (collider.mask as rhai::INT).into());
            metadata.insert("collider".into(), coll.into());
        }

        if let Some(lifetime) = world.lifetime(self.ctx.id) {
            let mut life = RhaiMap::new();
            life.insert("ttl_ms".into(), (lifetime.ttl_ms as rhai::INT).into());
            metadata.insert("lifetime".into(), life.into());
        }

        if let Some(visual) = world.visual(self.ctx.id) {
            if let Some(visual_id) = &visual.visual_id {
                metadata.insert("visual_id".into(), visual_id.clone().into());
            }
            if !visual.additional_visuals.is_empty() {
                let extras: RhaiArray = visual
                    .additional_visuals
                    .iter()
                    .map(|v| v.clone().into())
                    .collect();
                metadata.insert("additional_visuals".into(), extras.into());
            }
        }

        metadata
    }

    pub(crate) fn get_components(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };

        let mut components = RhaiMap::new();

        if let Some(transform) = world.transform(self.ctx.id) {
            let mut xf = RhaiMap::new();
            xf.insert("x".into(), (transform.x as rhai::FLOAT).into());
            xf.insert("y".into(), (transform.y as rhai::FLOAT).into());
            xf.insert("heading".into(), (transform.heading as rhai::FLOAT).into());
            components.insert("transform".into(), xf.into());
        }

        if let Some(physics) = world.physics(self.ctx.id) {
            let mut phys = RhaiMap::new();
            phys.insert("vx".into(), (physics.vx as rhai::FLOAT).into());
            phys.insert("vy".into(), (physics.vy as rhai::FLOAT).into());
            phys.insert("ax".into(), (physics.ax as rhai::FLOAT).into());
            phys.insert("ay".into(), (physics.ay as rhai::FLOAT).into());
            phys.insert("drag".into(), (physics.drag as rhai::FLOAT).into());
            phys.insert(
                "max_speed".into(),
                (physics.max_speed as rhai::FLOAT).into(),
            );
            phys.insert("mass".into(), (physics.mass as rhai::FLOAT).into());
            phys.insert(
                "restitution".into(),
                (physics.restitution as rhai::FLOAT).into(),
            );
            components.insert("physics".into(), phys.into());
        }

        if let Some(gravity) = world.gravity(self.ctx.id) {
            let mut grav = RhaiMap::new();
            grav.insert(
                "mode".into(),
                match gravity.mode {
                    GravityMode2D::Flat => "flat",
                    GravityMode2D::Point => "point",
                }
                .into(),
            );
            grav.insert(
                "gravity_scale".into(),
                (gravity.gravity_scale as rhai::FLOAT).into(),
            );
            grav.insert("flat_ax".into(), (gravity.flat_ax as rhai::FLOAT).into());
            grav.insert("flat_ay".into(), (gravity.flat_ay as rhai::FLOAT).into());
            if let Some(body_id) = gravity.body_id {
                grav.insert("body".into(), body_id.into());
            }
            components.insert("gravity".into(), grav.into());
        }

        if let Some(atmo) = world.atmosphere(self.ctx.id) {
            let mut map = RhaiMap::new();
            map.insert("drag_scale".into(), (atmo.drag_scale as rhai::FLOAT).into());
            map.insert("heat_scale".into(), (atmo.heat_scale as rhai::FLOAT).into());
            map.insert("cooling".into(), (atmo.cooling as rhai::FLOAT).into());
            map.insert("heat".into(), (atmo.heat as rhai::FLOAT).into());
            map.insert("density".into(), (atmo.density as rhai::FLOAT).into());
            map.insert(
                "dense_density".into(),
                (atmo.dense_density as rhai::FLOAT).into(),
            );
            map.insert(
                "altitude_km".into(),
                (atmo.altitude_km as rhai::FLOAT).into(),
            );
            if let Some(body_id) = atmo.body_id {
                map.insert("body".into(), body_id.into());
            }
            components.insert("atmosphere".into(), map.into());
        }

        if let Some(collider) = world.collider(self.ctx.id) {
            let mut coll = RhaiMap::new();
            match &collider.shape {
                ColliderShape::Circle { radius } => {
                    coll.insert("shape".into(), "circle".into());
                    coll.insert("radius".into(), (*radius as rhai::FLOAT).into());
                }
                ColliderShape::Polygon { points } => {
                    coll.insert("shape".into(), "polygon".into());
                    let pts: RhaiArray = points
                        .iter()
                        .map(|p| {
                            let mut point = RhaiMap::new();
                            point.insert("x".into(), (p[0] as rhai::FLOAT).into());
                            point.insert("y".into(), (p[1] as rhai::FLOAT).into());
                            point.into()
                        })
                        .collect();
                    coll.insert("points".into(), pts.into());
                }
            }
            coll.insert("layer".into(), (collider.layer as rhai::INT).into());
            coll.insert("mask".into(), (collider.mask as rhai::INT).into());
            components.insert("collider".into(), coll.into());
        }

        if let Some(lifetime) = world.lifetime(self.ctx.id) {
            let mut life = RhaiMap::new();
            life.insert("ttl_ms".into(), (lifetime.ttl_ms as rhai::INT).into());
            components.insert("lifetime".into(), life.into());
        }

        if let Some(visual) = world.visual(self.ctx.id) {
            if let Some(visual_id) = &visual.visual_id {
                components.insert("visual_id".into(), visual_id.clone().into());
            }
            if !visual.additional_visuals.is_empty() {
                let extras: RhaiArray = visual
                    .additional_visuals
                    .iter()
                    .map(|v| v.clone().into())
                    .collect();
                components.insert("additional_visuals".into(), extras.into());
            }
        }

        components
    }

    pub(crate) fn transform(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(xf) = world.transform(self.ctx.id) else {
            return RhaiMap::new();
        };

        let mut result = RhaiMap::new();
        result.insert("x".into(), (xf.x as rhai::FLOAT).into());
        result.insert("y".into(), (xf.y as rhai::FLOAT).into());
        result.insert("heading".into(), (xf.heading as rhai::FLOAT).into());
        result
    }

    pub(crate) fn set_position(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(mut xf) = world.transform(self.ctx.id) else {
            return false;
        };
        xf.x = x as f32;
        xf.y = y as f32;
        world.set_transform(self.ctx.id, xf)
    }

    pub(crate) fn set_heading(&mut self, heading: rhai::FLOAT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(mut xf) = world.transform(self.ctx.id) else {
            return false;
        };
        xf.heading = heading as f32;
        if !world.set_transform(self.ctx.id, xf) {
            return false;
        }
        let _ = world.with_controller(self.ctx.id, |ctrl| {
            ctrl.set_heading_radians(heading as f32);
        });
        true
    }

    #[allow(dead_code)]
    pub(crate) fn physics(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(phys) = world.physics(self.ctx.id) else {
            return RhaiMap::new();
        };

        let mut result = RhaiMap::new();
        result.insert("vx".into(), (phys.vx as rhai::FLOAT).into());
        result.insert("vy".into(), (phys.vy as rhai::FLOAT).into());
        result.insert("ax".into(), (phys.ax as rhai::FLOAT).into());
        result.insert("ay".into(), (phys.ay as rhai::FLOAT).into());
        result.insert("drag".into(), (phys.drag as rhai::FLOAT).into());
        result.insert("max_speed".into(), (phys.max_speed as rhai::FLOAT).into());
        result.insert("mass".into(), (phys.mass as rhai::FLOAT).into());
        result.insert(
            "restitution".into(),
            (phys.restitution as rhai::FLOAT).into(),
        );
        result
    }

    pub(crate) fn set_acceleration(&mut self, ax: rhai::FLOAT, ay: rhai::FLOAT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(mut phys) = world.physics(self.ctx.id) else {
            return false;
        };
        phys.ax = ax as f32;
        phys.ay = ay as f32;
        world.set_physics(self.ctx.id, phys)
    }

    pub(crate) fn apply_impulse(&mut self, vx: rhai::FLOAT, vy: rhai::FLOAT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.apply_impulse(self.ctx.id, vx as f32, vy as f32)
    }

    pub(crate) fn velocity_magnitude(&mut self) -> rhai::FLOAT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0.0;
        };
        world.velocity_magnitude(self.ctx.id) as rhai::FLOAT
    }

    pub(crate) fn velocity_angle(&mut self) -> rhai::FLOAT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0.0;
        };
        world.velocity_angle(self.ctx.id) as rhai::FLOAT
    }

    pub(crate) fn set_velocity_polar(&mut self, speed: rhai::FLOAT, angle: rhai::FLOAT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.set_velocity_polar(self.ctx.id, speed as f32, angle as f32)
    }

    pub(crate) fn collider(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(coll) = world.collider(self.ctx.id) else {
            return RhaiMap::new();
        };

        let mut result = RhaiMap::new();
        match &coll.shape {
            ColliderShape::Circle { radius } => {
                result.insert("shape".into(), "circle".into());
                result.insert("radius".into(), (*radius as rhai::FLOAT).into());
            }
            ColliderShape::Polygon { points } => {
                result.insert("shape".into(), "polygon".into());
                let pts: RhaiArray = points
                    .iter()
                    .map(|p| {
                        let mut point = RhaiMap::new();
                        point.insert("x".into(), (p[0] as rhai::FLOAT).into());
                        point.insert("y".into(), (p[1] as rhai::FLOAT).into());
                        point.into()
                    })
                    .collect();
                result.insert("points".into(), pts.into());
            }
        }
        result.insert("layer".into(), (coll.layer as rhai::INT).into());
        result.insert("mask".into(), (coll.mask as rhai::INT).into());
        result
    }

    pub(crate) fn lifetime_remaining(&mut self) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        let Some(lifetime) = world.lifetime(self.ctx.id) else {
            return 0;
        };
        lifetime.ttl_ms as rhai::INT
    }

    pub(crate) fn set_many(&mut self, map: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        for (key, value) in map {
            let Some(json_value) = rhai_dynamic_to_json(&value) else {
                return false;
            };
            if !world.set(self.ctx.id, &format!("/{}", key), json_value) {
                return false;
            }
        }
        true
    }

    pub(crate) fn data(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(entity) = world.get_entity(self.ctx.id) else {
            return RhaiMap::new();
        };
        json_to_rhai_dynamic(&entity.data)
            .try_cast::<RhaiMap>()
            .unwrap_or_default()
    }

    pub(crate) fn get_f(&mut self, path: &str, fallback: rhai::FLOAT) -> rhai::FLOAT {
        self.get(path).try_cast::<rhai::FLOAT>().unwrap_or(fallback)
    }

    pub(crate) fn get_s(&mut self, path: &str, fallback: &str) -> String {
        self.get(path)
            .try_cast::<String>()
            .unwrap_or_else(|| fallback.to_string())
    }

    // ── Cooldown API ──────────────────────────────────────────────────────

    pub(crate) fn cooldown_start(&mut self, name: &str, ms: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.cooldown_start(self.ctx.id, name, ms as i32)
    }

    pub(crate) fn cooldown_ready(&mut self, name: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return true;
        };
        world.cooldown_ready(self.ctx.id, name)
    }

    pub(crate) fn cooldown_remaining(&mut self, name: &str) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        world.cooldown_remaining(self.ctx.id, name) as rhai::INT
    }

    // ── Status API ────────────────────────────────────────────────────────

    pub(crate) fn status_add(&mut self, name: &str, ms: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.status_add(self.ctx.id, name, ms as i32)
    }

    pub(crate) fn status_has(&mut self, name: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.status_has(self.ctx.id, name)
    }

    pub(crate) fn status_remaining(&mut self, name: &str) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        world.status_remaining(self.ctx.id, name) as rhai::INT
    }

    // ── Arcade Controller API ─────────────────────────────────────────────

    pub(crate) fn attach_controller(&mut self, config: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        attach_vehicle_stack(world, self.ctx.id, config)
    }

    pub(crate) fn set_turn(&mut self, dir: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.with_controller(self.ctx.id, |ctrl| {
            ctrl.set_turn(dir.clamp(-1, 1) as i8);
        })
    }

    pub(crate) fn set_thrust(&mut self, on: bool) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.with_controller(self.ctx.id, |ctrl| {
            ctrl.set_thrust(on);
        })
    }

    pub(crate) fn heading(&mut self) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        world
            .controller(self.ctx.id)
            .map(|c| c.current_heading as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn heading_vector(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        if let Some(xf) = world.transform(self.ctx.id) {
            let mut map = RhaiMap::new();
            map.insert("x".into(), (xf.heading.sin() as rhai::FLOAT).into());
            map.insert("y".into(), ((-xf.heading.cos()) as rhai::FLOAT).into());
            return map;
        }
        match world.controller(self.ctx.id) {
            Some(ctrl) => {
                let (x, y) = ctrl.heading_vector();
                let mut map = RhaiMap::new();
                map.insert("x".into(), (x as rhai::FLOAT).into());
                map.insert("y".into(), (y as rhai::FLOAT).into());
                map
            }
            None => RhaiMap::new(),
        }
    }
}

impl ScriptGameplayObjectsApi {
    fn object_handle(&self, id: u64) -> ScriptGameplayObjectApi {
        ScriptGameplayObjectApi {
            world: self.world.clone(),
            id,
        }
    }

    fn handle_array<I>(&self, ids: I) -> RhaiArray
    where
        I: IntoIterator<Item = u64>,
    {
        ids.into_iter()
            .map(|id| RhaiDynamic::from(self.object_handle(id)))
            .collect()
    }

    fn object_name(world: &GameplayWorld, id: u64) -> Option<String> {
        world
            .get(id, "/name")
            .and_then(|value| value.as_str().map(|name| name.to_string()))
    }

    fn matches_visual_target(world: &GameplayWorld, id: u64, target: &str) -> bool {
        world
            .visual(id)
            .map(|binding| {
                binding
                    .all_visual_ids()
                    .into_iter()
                    .any(|visual_id| visual_id == target)
            })
            .unwrap_or(false)
    }

    pub(crate) fn find(&mut self, target: &str) -> ScriptGameplayObjectApi {
        let Some(world) = self.world.as_ref() else {
            return self.object_handle(0);
        };
        let target = target.trim();
        if target.is_empty() {
            return self.object_handle(0);
        }
        if let Ok(id) = target.parse::<u64>() {
            if world.exists(id) {
                return self.object_handle(id);
            }
        }
        let ids = world.ids();
        if let Some(id) = ids
            .iter()
            .copied()
            .find(|id| Self::matches_visual_target(world, *id, target))
        {
            return self.object_handle(id);
        }
        if let Some(id) = ids
            .iter()
            .copied()
            .find(|id| Self::object_name(world, *id).as_deref() == Some(target))
        {
            return self.object_handle(id);
        }
        self.object_handle(0)
    }

    pub(crate) fn find_id(&mut self, id: rhai::INT) -> ScriptGameplayObjectApi {
        let Some(world) = self.world.as_ref() else {
            return self.object_handle(0);
        };
        if id <= 0 {
            return self.object_handle(0);
        }
        let id = id as u64;
        if !world.exists(id) {
            return self.object_handle(0);
        }
        self.object_handle(id)
    }

    pub(crate) fn all(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        self.handle_array(world.ids())
    }

    pub(crate) fn by_tag(&mut self, tag: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        let mut ids = world.query_tag(tag);
        ids.sort_unstable();
        ids.dedup();
        self.handle_array(ids)
    }

    pub(crate) fn by_name(&mut self, name: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        let name = name.trim();
        if name.is_empty() {
            return RhaiArray::new();
        }
        self.handle_array(
            world
                .ids()
                .into_iter()
                .filter(|id| Self::object_name(world, *id).as_deref() == Some(name)),
        )
    }
}

impl ScriptGameplayObjectApi {
    // Live handles degrade to an inert empty view once the backing entity is gone.
    fn live_world(&self) -> Option<&GameplayWorld> {
        let world = self.world.as_ref()?;
        (self.id > 0 && world.exists(self.id)).then_some(world)
    }

    pub(crate) fn exists(&mut self) -> bool {
        self.live_world().is_some()
    }

    pub(crate) fn id(&mut self) -> rhai::INT {
        if self.exists() {
            self.id as rhai::INT
        } else {
            0
        }
    }

    pub(crate) fn kind(&mut self) -> String {
        let Some(world) = self.live_world() else {
            return String::new();
        };
        world.kind_of(self.id).unwrap_or_default()
    }

    pub(crate) fn tags(&mut self) -> RhaiArray {
        let Some(world) = self.live_world() else {
            return RhaiArray::new();
        };
        world.tags(self.id).into_iter().map(Into::into).collect()
    }

    pub(crate) fn inspect(&mut self) -> RhaiMap {
        let Some(world) = self.live_world() else {
            return RhaiMap::new();
        };
        let Some(entity) = world.get_entity(self.id) else {
            return RhaiMap::new();
        };

        let mut map = RhaiMap::new();
        map.insert("id".into(), (entity.id as rhai::INT).into());
        map.insert("kind".into(), entity.kind.into());
        if let Some(name) = ScriptGameplayObjectsApi::object_name(world, self.id) {
            map.insert("name".into(), name.into());
        }
        map.insert(
            "tags".into(),
            entity
                .tags
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<RhaiArray>()
                .into(),
        );
        map.insert("data".into(), json_to_rhai_dynamic(&entity.data));

        if let Some(binding) = world.visual(self.id) {
            if let Some(ref visual_id) = binding.visual_id {
                map.insert("visual_id".into(), visual_id.clone().into());
            }
            let visual_ids = binding
                .all_visual_ids()
                .into_iter()
                .map(|visual_id| visual_id.to_string().into())
                .collect::<RhaiArray>();
            if !visual_ids.is_empty() {
                map.insert("visual_ids".into(), visual_ids.into());
            }
        }

        map
    }

    pub(crate) fn get(&mut self, path: &str) -> RhaiDynamic {
        let Some(world) = self.live_world() else {
            return ().into();
        };
        world
            .get(self.id, path)
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    pub(crate) fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.live_world() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.set(self.id, path, value)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        prefab_assembly3d_from_components, prefab_bootstrap_assembly3d_from_components,
        prefab_components_from_runtime_object_families, ScriptGameplayApi,
    };
    use crate::catalog::{
        AngularMotor3DComponent, CharacterMotor3DComponent, FlightMotor3DComponent,
        FollowAnchor3DComponent, LinearMotor3DComponent, PrefabComponents, PrefabTemplate,
        PrefabTransform, ReferenceFrameComponent,
    };
    use crate::{catalog, palette::PaletteStore, BehaviorCommand};
    use engine_core::scene::model::RuntimeObjectDocument;
    use engine_game::components::{
        AngularMotorMode, CharacterUpMode, LifecyclePolicy, MotorSpace, ReferenceFrameMode,
    };
    use engine_game::GameplayWorld;
    use engine_terrain::{Biome, PlanetGenParams};
    use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Map as RhaiMap};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    fn insert(map: &mut RhaiMap, key: &str, value: impl Into<RhaiDynamic>) {
        map.insert(key.into(), value.into());
    }

    fn build_world_api(world: GameplayWorld, catalogs: catalog::ModCatalogs) -> ScriptGameplayApi {
        ScriptGameplayApi::new(
            Some(world),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            None,
            Arc::new(catalogs),
            None,
            Arc::new(Mutex::new(Vec::<BehaviorCommand>::new())),
            Arc::new(PaletteStore::default()),
            None,
            None,
        )
    }

    fn sample_params() -> PlanetGenParams {
        let mut config = RhaiMap::new();
        insert(&mut config, "seed", 1847);
        insert(&mut config, "has_ocean", true);
        insert(&mut config, "ocean", 0.55);
        insert(&mut config, "cscale", 2.5);
        insert(&mut config, "cwarp", 0.65);
        insert(&mut config, "coct", 5);
        insert(&mut config, "mscale", 6.0);
        insert(&mut config, "mstr", 0.45);
        insert(&mut config, "mroct", 5);
        insert(&mut config, "moisture", 3.0);
        insert(&mut config, "ice", 1.0);
        insert(&mut config, "lapse", 0.6);
        insert(&mut config, "rain", 0.4);
        ScriptGameplayApi::planet_gen_params_from_rhai_map(&config)
    }

    fn candidate_biomes() -> [Biome; 10] {
        [
            Biome::Ocean,
            Biome::ShallowWater,
            Biome::Beach,
            Biome::Desert,
            Biome::Grassland,
            Biome::Forest,
            Biome::Tundra,
            Biome::Snow,
            Biome::Mountain,
            Biome::Volcanic,
        ]
    }

    #[test]
    fn prefab_assembly3d_from_components_lowers_reference_frame_and_motor_stack() {
        let components = PrefabComponents {
            reference_frame: Some(ReferenceFrameComponent {
                mode: Some("LocalHorizon".to_string()),
                entity_id: Some(7),
                body_id: Some("earth".to_string()),
                inherit_linear_velocity: Some(true),
                inherit_angular_velocity: Some(false),
            }),
            follow_anchor_3d: Some(FollowAnchor3DComponent {
                local_offset: Some([1.0, 2.0, 3.0]),
                inherit_orientation: Some(false),
            }),
            linear_motor_3d: Some(LinearMotor3DComponent {
                space: Some("ReferenceFrame".to_string()),
                accel: Some(24.0),
                decel: Some(12.0),
                max_speed: Some(320.0),
                boost_scale: Some(1.5),
                air_control: Some(0.75),
            }),
            angular_motor_3d: Some(AngularMotor3DComponent {
                mode: Some("Torque".to_string()),
                yaw_rate: Some(90.0),
                pitch_rate: Some(20.0),
                roll_rate: Some(15.0),
                torque_scale: Some(2.0),
                look_sensitivity: Some(1.25),
            }),
            character_motor_3d: Some(CharacterMotor3DComponent {
                up_mode: Some("SurfaceNormal".to_string()),
                jump_speed: Some(8.5),
                stick_to_ground: Some(true),
                max_slope_deg: Some(55.0),
            }),
            flight_motor_3d: Some(FlightMotor3DComponent {
                translational_dofs: Some([true, false, true]),
                rotational_dofs: Some([true, true, false]),
                horizon_lock_strength: Some(0.35),
            }),
            ..PrefabComponents::default()
        };

        let assembly =
            prefab_assembly3d_from_components(&components).expect("prefab should lower to 3D");

        assert_eq!(
            assembly.control.reference_frame.as_ref().unwrap().mode,
            ReferenceFrameMode::LocalHorizon
        );
        assert_eq!(
            assembly.control.reference_frame.as_ref().unwrap().entity_id,
            Some(7)
        );
        assert_eq!(
            assembly
                .control
                .reference_frame
                .as_ref()
                .unwrap()
                .body_id
                .as_deref(),
            Some("earth")
        );
        assert_eq!(
            assembly
                .attachments
                .follow_anchor
                .as_ref()
                .unwrap()
                .local_offset,
            [1.0, 2.0, 3.0]
        );
        assert_eq!(
            assembly
                .attachments
                .follow_anchor
                .as_ref()
                .unwrap()
                .inherit_orientation,
            false
        );
        assert_eq!(
            assembly.motors.linear_motor.as_ref().unwrap().space,
            MotorSpace::ReferenceFrame
        );
        assert_eq!(assembly.motors.linear_motor.as_ref().unwrap().accel, 24.0);
        assert_eq!(
            assembly.motors.angular_motor.as_ref().unwrap().mode,
            AngularMotorMode::Torque
        );
        assert_eq!(
            assembly.motors.character_motor.as_ref().unwrap().up_mode,
            CharacterUpMode::SurfaceNormal
        );
        assert_eq!(
            assembly
                .motors
                .flight_motor
                .as_ref()
                .unwrap()
                .translational_dofs,
            [true, false, true]
        );
        assert!(assembly.spatial.is_empty());
    }

    #[test]
    fn runtime_object_component_families_lower_through_prefab_components_and_bootstrap() {
        let component_families = serde_json::json!({
            "reference-frame": {
                "mode": "LocalHorizon",
                "body_id": "earth",
                "inherit_linear_velocity": true
            },
            "linear-motor-3d": {
                "space": "ReferenceFrame",
                "accel": 24.0
            },
            "camera-rig": {
                "preset": "cockpit"
            },
            "celestial-binding": {
                "body_id": "earth",
                "frame_mode": "surface-local"
            },
            "lifecycle": "TtlFollowOwner"
        });

        let components = prefab_components_from_runtime_object_families(&component_families)
            .expect("runtime-object families should deserialize");
        let mut args = RhaiMap::new();
        insert(&mut args, "owner_id", 7_i64);
        let bootstrap = prefab_bootstrap_assembly3d_from_components(&components, &args)
            .expect("bootstrap lowering should exist");

        assert_eq!(bootstrap.owner_id, Some(7));
        assert_eq!(bootstrap.lifecycle, Some(LifecyclePolicy::TtlFollowOwner));
        assert_eq!(
            bootstrap
                .assembly
                .control
                .reference_frame
                .as_ref()
                .expect("reference frame")
                .mode,
            ReferenceFrameMode::LocalHorizon
        );
        assert_eq!(
            bootstrap
                .assembly
                .motors
                .linear_motor
                .as_ref()
                .expect("linear motor")
                .space,
            MotorSpace::ReferenceFrame
        );
        assert!(bootstrap.assembly.control.control_intent.is_none());
        assert!(bootstrap.assembly.attachments.follow_anchor.is_none());
    }

    #[test]
    fn runtime_object_prefab_tree_lowers_children_owner_links_and_overrides() {
        let world = GameplayWorld::new();
        let mut catalogs = catalog::ModCatalogs::test_catalogs();
        catalogs.prefabs.insert(
            "carrier".to_string(),
            PrefabTemplate {
                kind: "carrier".to_string(),
                sprite_template: Some("carrier-template".to_string()),
                transform: Some(PrefabTransform::default()),
                init_fields: HashMap::new(),
                components: Some(PrefabComponents {
                    lifecycle: Some("Persistent".to_string()),
                    ..PrefabComponents::default()
                }),
                fg_colour: None,
                default_tags: vec![],
            },
        );
        catalogs.prefabs.insert(
            "camera_mount".to_string(),
            PrefabTemplate {
                kind: "camera_mount".to_string(),
                sprite_template: Some("camera-template".to_string()),
                transform: Some(PrefabTransform::default()),
                init_fields: HashMap::new(),
                components: Some(PrefabComponents::default()),
                fg_colour: None,
                default_tags: vec![],
            },
        );
        catalogs.prefabs.insert(
            "escort_drone".to_string(),
            PrefabTemplate {
                kind: "escort_drone".to_string(),
                sprite_template: Some("escort-template".to_string()),
                transform: Some(PrefabTransform::default()),
                init_fields: HashMap::new(),
                components: Some(PrefabComponents::default()),
                fg_colour: None,
                default_tags: vec![],
            },
        );

        let runtime_object: RuntimeObjectDocument = serde_yaml::from_str(
            r#"
name: carrier-root
kind: runtime-object
prefab: carrier
transform:
  space: 3d
  translation: [10.0, 20.0, 30.0]
  rotation-deg: [0.0, 90.0, 0.0]
components:
  reference-frame:
    mode: LocalHorizon
    body_id: earth
children:
  - name: camera-mount
    prefab: camera_mount
    transform:
      space: 3d
      translation: [1.0, 2.0, -3.0]
    components:
      follow-anchor-3d:
        local-offset: [1.0, 2.0, -3.0]
    overrides:
      name: Camera Mount
      components:
        linear-motor-3d:
          space: ReferenceFrame
          accel: 8.0
  - name: escort
    prefab: escort_drone
    transform:
      space: 3d
      translation: [4.0, 0.0, 1.0]
    components:
      lifecycle: FollowOwner
      follow-anchor-3d:
        local-offset: [4.0, 0.0, 1.0]
"#,
        )
        .expect("runtime-object document");

        let mut api = build_world_api(world.clone(), catalogs);
        let root_id = api.spawn_runtime_object_document(&runtime_object, None);
        assert!(root_id > 0, "root runtime-object should spawn");
        let root_id = root_id as u64;

        let root_transform = world.transform3d(root_id).expect("root transform3d");
        assert_eq!(root_transform.position, [10.0, 20.0, 30.0]);
        assert!((root_transform.orientation[1] - 0.70710677).abs() < 0.001);
        assert!((root_transform.orientation[3] - 0.70710677).abs() < 0.001);
        assert_eq!(
            world.reference_frame3d(root_id).map(|binding| binding.mode),
            Some(ReferenceFrameMode::LocalHorizon)
        );
        assert_eq!(world.lifecycle(root_id), Some(LifecyclePolicy::Persistent));

        let mut child_ids = world
            .ids()
            .into_iter()
            .filter(|id| *id != root_id)
            .collect::<Vec<_>>();
        child_ids.sort_unstable();
        assert_eq!(child_ids.len(), 2, "two child runtime-objects should spawn");

        let camera_mount_id = child_ids
            .iter()
            .copied()
            .find(|id| world.get(*id, "/name") == Some(serde_json::json!("Camera Mount")))
            .expect("camera mount child");
        assert_eq!(
            world
                .ownership(camera_mount_id)
                .map(|ownership| ownership.owner_id),
            Some(root_id)
        );
        assert_eq!(
            world.lifecycle(camera_mount_id),
            Some(LifecyclePolicy::Persistent)
        );
        assert_eq!(
            world
                .follow_anchor3d(camera_mount_id)
                .expect("camera mount follow anchor")
                .local_offset,
            [1.0, 2.0, -3.0]
        );
        assert_eq!(
            world
                .linear_motor3d(camera_mount_id)
                .expect("camera mount linear motor")
                .space,
            MotorSpace::ReferenceFrame
        );
        assert_eq!(
            world
                .linear_motor3d(camera_mount_id)
                .expect("camera mount linear motor")
                .accel,
            8.0
        );

        let escort_id = child_ids
            .into_iter()
            .find(|id| *id != camera_mount_id)
            .expect("escort child");
        assert_eq!(
            world
                .ownership(escort_id)
                .map(|ownership| ownership.owner_id),
            Some(root_id)
        );
        assert_eq!(
            world.lifecycle(escort_id),
            Some(LifecyclePolicy::FollowOwner)
        );
        assert_eq!(
            world
                .follow_anchor3d(escort_id)
                .expect("escort follow anchor")
                .local_offset,
            [4.0, 0.0, 1.0]
        );

        assert!(world.despawn(root_id));
        assert!(!world.exists(root_id));
        assert!(!world.exists(camera_mount_id));
        assert!(!world.exists(escort_id));
    }

    #[test]
    fn planet_gen_params_from_rhai_map_parses_aliases_and_string_numbers() {
        let mut config = RhaiMap::new();
        insert(&mut config, "seed", "1847");
        insert(&mut config, "has-ocean", "yes");
        insert(&mut config, "ocean-fraction", "1.8");
        insert(&mut config, "continent-scale", "12.5");
        insert(&mut config, "continent-warp", "-1.0");
        insert(&mut config, "continent-octaves", "9");
        insert(&mut config, "mountain-scale", "0.5");
        insert(&mut config, "mountain-strength", "2.0");
        insert(&mut config, "mountain-ridge-octaves", "-3");
        insert(&mut config, "moisture-scale", "7.5");
        insert(&mut config, "ice-cap-strength", "4.5");
        insert(&mut config, "lapse-rate", "-0.5");
        insert(&mut config, "rain-shadow", "1.3");

        let params = ScriptGameplayApi::planet_gen_params_from_rhai_map(&config);

        assert_eq!(params.seed, 1847);
        assert!(params.has_ocean);
        assert_eq!(params.ocean_fraction, 1.0);
        assert_eq!(params.continent_scale, 10.0);
        assert_eq!(params.continent_warp, 0.0);
        assert_eq!(params.continent_octaves, 8);
        assert_eq!(params.mountain_scale, 1.0);
        assert_eq!(params.mountain_strength, 1.0);
        assert_eq!(params.mountain_ridge_octaves, 2);
        assert_eq!(params.moisture_scale, 7.5);
        assert_eq!(params.ice_cap_strength, 3.0);
        assert_eq!(params.lapse_rate, 0.0);
        assert_eq!(params.rain_shadow, 1.0);
    }

    #[test]
    fn planet_gen_params_from_rhai_map_ignores_invalid_strings_and_preserves_defaults() {
        let defaults = PlanetGenParams::default();
        let mut config = RhaiMap::new();
        insert(&mut config, "has-ocean", "maybe");
        insert(&mut config, "ocean", "not-a-number");
        insert(&mut config, "continent-scale", ());
        insert(&mut config, "mountain-strength", "bogus");

        let params = ScriptGameplayApi::planet_gen_params_from_rhai_map(&config);

        assert_eq!(params.has_ocean, defaults.has_ocean);
        assert_eq!(params.ocean_fraction, defaults.ocean_fraction);
        assert_eq!(params.continent_scale, defaults.continent_scale);
        assert_eq!(params.mountain_strength, defaults.mountain_strength);
    }

    #[test]
    fn preferred_biomes_from_array_defaults_for_empty_or_invalid_entries() {
        let empty = ScriptGameplayApi::preferred_biomes_from_array(RhaiArray::new());
        let invalid = ScriptGameplayApi::preferred_biomes_from_array(vec![
            RhaiDynamic::from("???"),
            RhaiDynamic::from(17_i64),
        ]);

        assert_eq!(empty, ScriptGameplayApi::default_spawn_biomes());
        assert_eq!(invalid, ScriptGameplayApi::default_spawn_biomes());
    }

    #[test]
    fn find_planet_spawn_angle_for_params_uses_default_order_for_empty_preferences() {
        let params = sample_params();
        let default_order = ScriptGameplayApi::default_spawn_biomes();
        let empty = ScriptGameplayApi::find_planet_spawn_angle_for_params(&params, &[]);
        let explicit =
            ScriptGameplayApi::find_planet_spawn_angle_for_params(&params, &default_order);

        assert_eq!(empty, explicit);
    }

    #[test]
    fn find_planet_spawn_angle_for_params_falls_back_deterministically_to_next_available_biome() {
        let params = sample_params();
        let mut missing = None;
        let mut present = None;

        for biome in candidate_biomes() {
            let angle = ScriptGameplayApi::find_planet_spawn_angle_for_params(&params, &[biome]);
            if angle == 0.0 && missing.is_none() {
                missing = Some(biome);
            } else if angle > 0.0 && present.is_none() {
                present = Some((biome, angle));
            }
            if missing.is_some() && present.is_some() {
                break;
            }
        }

        let missing = missing.expect("expected at least one absent biome");
        let (present, present_angle) = present.expect("expected at least one available biome");

        let fallback =
            ScriptGameplayApi::find_planet_spawn_angle_for_params(&params, &[missing, present]);
        let repeated =
            ScriptGameplayApi::find_planet_spawn_angle_for_params(&params, &[missing, present]);

        assert_eq!(fallback, present_angle);
        assert_eq!(repeated, present_angle);
    }
}

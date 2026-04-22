//! Mod-scoped gameplay catalogs for data-driven helpers.
//! Catalogs allow mods to define prefabs, weapons, emitters, input profiles, etc.
//! via YAML instead of hardcoding in Rust.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub use engine_celestial::{
    BodyDef, CelestialCatalogs, PlanetDef, RegionDef, RouteDef, SiteDef, SystemDef,
};

/// Complete set of catalogs for a mod.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModCatalogs {
    pub input_profiles: HashMap<String, InputProfile>,
    pub prefabs: HashMap<String, PrefabTemplate>,
    /// Named runtime policy presets referenced from scene `controller-defaults`.
    #[serde(default)]
    pub presets: CatalogPresets,
    /// Reusable non-instantiated data blobs referenced by prefabs/scenes.
    #[serde(default)]
    pub specs: HashMap<String, CatalogSpec>,
    pub weapons: HashMap<String, WeaponConfig>,
    pub emitters: HashMap<String, EmitterConfig>,
    pub groups: HashMap<String, GroupTemplate>,
    pub waves: HashMap<String, WaveTemplate>,
    /// Celestial world data: bodies, planet presets, regions, systems, sites, and routes.
    #[serde(default)]
    pub celestial: CelestialCatalogs,
}

/// Input action bindings: action_name -> list of key codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputProfile {
    pub bindings: HashMap<String, Vec<String>>,
}

fn merge_yaml_values(base: serde_yaml::Value, overlay: serde_yaml::Value) -> serde_yaml::Value {
    match (base, overlay) {
        (serde_yaml::Value::Mapping(mut base_map), serde_yaml::Value::Mapping(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                let merged_value = match base_map.get(&key).cloned() {
                    Some(base_value) => merge_yaml_values(base_value, overlay_value),
                    None => overlay_value,
                };
                base_map.insert(key, merged_value);
            }
            serde_yaml::Value::Mapping(base_map)
        }
        (_, overlay_value) => overlay_value,
    }
}

fn resolve_prefab_yaml(
    name: &str,
    raw_prefabs: &HashMap<String, serde_yaml::Value>,
    cache: &mut HashMap<String, serde_yaml::Value>,
    stack: &mut Vec<String>,
) -> Result<serde_yaml::Value, String> {
    if let Some(value) = cache.get(name) {
        return Ok(value.clone());
    }

    if stack.iter().any(|entry| entry == name) {
        let mut chain = stack.join(" -> ");
        if !chain.is_empty() {
            chain.push_str(" -> ");
        }
        chain.push_str(name);
        return Err(format!("Prefab inheritance cycle detected: {chain}"));
    }

    let Some(raw_value) = raw_prefabs.get(name) else {
        return Err(format!("Prefab '{name}' references missing prefab"));
    };

    stack.push(name.to_string());

    let resolved = if let Some(base_name) = raw_value.get("ref").and_then(|v| v.as_str()) {
        let base = resolve_prefab_yaml(base_name, raw_prefabs, cache, stack)?;
        let mut overlay = raw_value.clone();
        if let serde_yaml::Value::Mapping(ref mut map) = overlay {
            map.remove(&serde_yaml::Value::String("ref".to_string()));
        }
        merge_yaml_values(base, overlay)
    } else {
        raw_value.clone()
    };

    stack.pop();
    cache.insert(name.to_string(), resolved.clone());
    Ok(resolved)
}

/// Prefab template for entity spawning.
///
/// Responsibility split:
/// - prefab = instantiable object bundle
/// - preset = named runtime policy (camera/player/ui/spawn/etc.)
/// - spec = reusable data/config that is not instantiated by itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabTemplate {
    pub kind: String,
    #[serde(default)]
    pub sprite_template: Option<String>,
    #[serde(default)]
    pub transform: Option<PrefabTransform>,
    #[serde(default)]
    pub init_fields: HashMap<String, JsonValue>,
    #[serde(default)]
    pub components: Option<PrefabComponents>,
    /// Optional sprite foreground color. Supports `"@palette.<key>"` for live palette resolution,
    /// or a literal hex/named color. Applied automatically at spawn time.
    #[serde(default)]
    pub fg_colour: Option<String>,
    /// Tags automatically applied to every spawned entity of this prefab type.
    /// Merged with any `tags: [...]` provided at call site.
    #[serde(default)]
    pub default_tags: Vec<String>,
}

/// Reusable runtime presets referenced from scene `controller-defaults`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CatalogPresets {
    #[serde(default)]
    pub players: HashMap<String, PlayerPreset>,
    #[serde(default)]
    pub cameras: HashMap<String, CameraPreset>,
    #[serde(default)]
    pub ui: HashMap<String, UiPreset>,
    #[serde(default)]
    pub spawns: HashMap<String, SpawnPreset>,
    #[serde(default)]
    pub gravity: HashMap<String, GravityPreset>,
    #[serde(default)]
    pub surfaces: HashMap<String, SurfacePreset>,
}

impl CatalogPresets {
    pub fn player(&self, id: &str) -> Option<&PlayerPreset> {
        self.players.get(id)
    }

    pub fn camera(&self, id: &str) -> Option<&CameraPreset> {
        self.cameras.get(id)
    }

    pub fn ui(&self, id: &str) -> Option<&UiPreset> {
        self.ui.get(id)
    }

    pub fn spawn(&self, id: &str) -> Option<&SpawnPreset> {
        self.spawns.get(id)
    }

    pub fn gravity(&self, id: &str) -> Option<&GravityPreset> {
        self.gravity.get(id)
    }

    pub fn surface(&self, id: &str) -> Option<&SurfacePreset> {
        self.surfaces.get(id)
    }
}

/// Reusable non-instantiated config/spec data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogSpec {
    pub kind: String,
    #[serde(default)]
    pub data: HashMap<String, JsonValue>,
}

/// Author-time transform defaults for prefab placement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrefabTransform {
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub z: Option<f64>,
    #[serde(default)]
    pub heading: Option<f64>,
    #[serde(default)]
    pub pitch: Option<f64>,
    #[serde(default)]
    pub roll: Option<f64>,
    #[serde(default)]
    pub scale_x: Option<f64>,
    #[serde(default)]
    pub scale_y: Option<f64>,
    #[serde(default)]
    pub scale_z: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerPreset {
    #[serde(default)]
    pub input_profile: Option<String>,
    #[serde(default)]
    pub controller: Option<ControllerComponent>,
    #[serde(default)]
    pub components: Option<PrefabComponents>,
    #[serde(default)]
    pub default_tags: Vec<String>,
    #[serde(default)]
    pub config: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CameraPreset {
    #[serde(default, rename = "controller-kind", alias = "controller_kind")]
    pub controller_kind: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiPreset {
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpawnPreset {
    #[serde(default, rename = "spawn-type", alias = "spawn_type")]
    pub spawn_type: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GravityPreset {
    #[serde(default, rename = "gravity-type", alias = "gravity_type")]
    pub gravity_type: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SurfacePreset {
    #[serde(default, rename = "surface-type", alias = "surface_type")]
    pub surface_type: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, JsonValue>,
}

/// Component specifications for data-driven prefab spawning.
/// Allows mods to define physics, colliders, controllers, and lifecycle policies without Rust code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrefabComponents {
    #[serde(default)]
    pub render: Option<RenderComponent>,
    #[serde(default)]
    pub physics: Option<PhysicsComponent>,
    #[serde(default)]
    pub collider: Option<ColliderComponent>,
    #[serde(default)]
    pub controller: Option<ControllerComponent>,
    #[serde(default, rename = "reference-frame", alias = "reference_frame")]
    pub reference_frame: Option<ReferenceFrameComponent>,
    #[serde(default, rename = "follow-anchor-3d", alias = "follow_anchor_3d")]
    pub follow_anchor_3d: Option<FollowAnchor3DComponent>,
    #[serde(default, rename = "linear-motor-3d", alias = "linear_motor_3d")]
    pub linear_motor_3d: Option<LinearMotor3DComponent>,
    #[serde(default, rename = "angular-motor-3d", alias = "angular_motor_3d")]
    pub angular_motor_3d: Option<AngularMotor3DComponent>,
    #[serde(default, rename = "character-motor-3d", alias = "character_motor_3d")]
    pub character_motor_3d: Option<CharacterMotor3DComponent>,
    #[serde(default, rename = "flight-motor-3d", alias = "flight_motor_3d")]
    pub flight_motor_3d: Option<FlightMotor3DComponent>,
    #[serde(default, rename = "camera-rig", alias = "camera_rig")]
    pub camera_rig: Option<CameraRigComponent>,
    #[serde(default)]
    pub audio: Option<AudioComponent>,
    #[serde(default)]
    pub gameplay: Option<GameplayComponent>,
    #[serde(default, rename = "celestial-binding", alias = "celestial_binding")]
    pub celestial_binding: Option<CelestialBindingComponent>,
    #[serde(default)]
    pub lifecycle: Option<String>, // "Persistent", "Ttl", "OwnerBound", "TtlOwnerBound"
    #[serde(default)]
    pub wrappable: Option<bool>, // Enable wrap_bounds
    #[serde(default)]
    pub extra_data: Option<HashMap<String, JsonValue>>, // Additional entity fields
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderComponent {
    #[serde(default)]
    pub sprite_template: Option<String>,
    #[serde(default)]
    pub mesh: Option<String>,
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub camera_source: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, JsonValue>,
}

/// Physics component: velocity, drag, max_speed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhysicsComponent {
    #[serde(default)]
    pub vx: Option<f64>,
    #[serde(default)]
    pub vy: Option<f64>,
    #[serde(default)]
    pub vz: Option<f64>,
    #[serde(default)]
    pub ax: Option<f64>,
    #[serde(default)]
    pub ay: Option<f64>,
    #[serde(default)]
    pub az: Option<f64>,
    #[serde(default)]
    pub drag: Option<f64>,
    #[serde(default)]
    pub max_speed: Option<f64>,
    #[serde(default)]
    pub mass: Option<f64>,
    #[serde(default)]
    pub restitution: Option<f64>,
    #[serde(default)]
    pub gravity_scale: Option<f64>,
    #[serde(default)]
    pub gravity_mode: Option<String>,
    #[serde(default)]
    pub gravity_body: Option<String>,
    #[serde(default)]
    pub gravity_flat_x: Option<f64>,
    #[serde(default)]
    pub gravity_flat_y: Option<f64>,
    #[serde(default)]
    pub atmosphere_body: Option<String>,
    #[serde(default)]
    pub atmosphere_drag_scale: Option<f64>,
    #[serde(default)]
    pub atmosphere_heat_scale: Option<f64>,
    #[serde(default)]
    pub atmosphere_cooling: Option<f64>,
}

/// Collider component: shape and collision masks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColliderComponent {
    pub shape: String, // "circle", "rect" (future: "polygon")
    #[serde(default)]
    pub radius: Option<f64>, // for circles
    #[serde(default)]
    pub width: Option<f64>, // for rects
    #[serde(default)]
    pub height: Option<f64>, // for rects
    #[serde(default)]
    pub layer: Option<i64>, // collision layer (default 0xFFFF)
    #[serde(default)]
    pub mask: Option<i64>, // collision mask (default 0xFFFF)
}

/// Controller component: input/behavior driver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerComponent {
    pub controller_type: String, // "ArcadeController", "WaveSpawner", etc.
    #[serde(default)]
    pub config: Option<HashMap<String, JsonValue>>, // controller-specific config
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReferenceFrameComponent {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub entity_id: Option<u64>,
    #[serde(default)]
    pub body_id: Option<String>,
    #[serde(default)]
    pub inherit_linear_velocity: Option<bool>,
    #[serde(default)]
    pub inherit_angular_velocity: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FollowAnchor3DComponent {
    #[serde(default)]
    pub local_offset: Option<[f64; 3]>,
    #[serde(default)]
    pub inherit_orientation: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinearMotor3DComponent {
    #[serde(default)]
    pub space: Option<String>,
    #[serde(default)]
    pub accel: Option<f64>,
    #[serde(default)]
    pub decel: Option<f64>,
    #[serde(default)]
    pub max_speed: Option<f64>,
    #[serde(default)]
    pub boost_scale: Option<f64>,
    #[serde(default)]
    pub air_control: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AngularMotor3DComponent {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub yaw_rate: Option<f64>,
    #[serde(default)]
    pub pitch_rate: Option<f64>,
    #[serde(default)]
    pub roll_rate: Option<f64>,
    #[serde(default)]
    pub torque_scale: Option<f64>,
    #[serde(default)]
    pub look_sensitivity: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterMotor3DComponent {
    #[serde(default)]
    pub up_mode: Option<String>,
    #[serde(default)]
    pub jump_speed: Option<f64>,
    #[serde(default)]
    pub stick_to_ground: Option<bool>,
    #[serde(default)]
    pub max_slope_deg: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlightMotor3DComponent {
    #[serde(default)]
    pub translational_dofs: Option<[bool; 3]>,
    #[serde(default)]
    pub rotational_dofs: Option<[bool; 3]>,
    #[serde(default)]
    pub horizon_lock_strength: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CameraRigComponent {
    #[serde(default, rename = "rig-type", alias = "rig_type")]
    pub rig_type: Option<String>,
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioComponent {
    #[serde(default)]
    pub bus: Option<String>,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub config: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameplayComponent {
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CelestialBindingComponent {
    #[serde(default)]
    pub body_id: Option<String>,
    #[serde(default)]
    pub site_id: Option<String>,
    #[serde(default)]
    pub region_id: Option<String>,
    #[serde(default)]
    pub system_id: Option<String>,
    #[serde(default)]
    pub route_id: Option<String>,
    #[serde(default)]
    pub frame_mode: Option<String>,
}

/// Weapon configuration for firing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponConfig {
    pub max_projectiles: i64,
    #[serde(default)]
    pub projectile_kind: Option<String>,
    #[serde(default)]
    pub projectile_ttl_ms: Option<i64>,
    #[serde(default)]
    pub cooldown_ms: Option<i64>,
    #[serde(default)]
    pub cooldown_name: Option<String>,
    #[serde(default)]
    pub spawn_offset: Option<f64>,
    #[serde(default)]
    pub speed_scale: Option<f64>,
}

/// Emitter configuration for particle effects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmitterConfig {
    #[serde(default)]
    pub max_count: Option<i64>,
    #[serde(default)]
    pub cooldown_name: Option<String>,
    #[serde(default)]
    pub cooldown_ms: Option<i64>,
    #[serde(default)]
    pub min_cooldown_ms: Option<i64>,
    #[serde(default)]
    pub ramp_ms: Option<i64>,
    #[serde(default)]
    pub spawn_offset: Option<f64>,
    #[serde(default)]
    pub side_offset: Option<f64>,
    /// Emitter local anchor point in owner-local coordinates.
    /// If set, this overrides spawn_offset/side_offset.
    #[serde(default)]
    pub local_x: Option<f64>,
    #[serde(default)]
    pub local_y: Option<f64>,
    /// Z-axis position in owner-local space. 0.0 for pure 2D emitters.
    #[serde(default)]
    pub local_z: Option<f64>,
    /// Optional edge anchor in owner-local coordinates:
    /// anchor = from + (to - from) * edge_t.
    /// If set and local_x/local_y are not set, this overrides spawn_offset/side_offset.
    #[serde(default)]
    pub edge_from_x: Option<f64>,
    #[serde(default)]
    pub edge_from_y: Option<f64>,
    #[serde(default)]
    pub edge_to_x: Option<f64>,
    #[serde(default)]
    pub edge_to_y: Option<f64>,
    #[serde(default)]
    pub edge_t: Option<f64>,
    /// Base emission direction offset in radians, relative to the emitter's default backward axis.
    /// Applied before per-call `spread`.
    #[serde(default)]
    pub emission_angle: Option<f64>,
    /// Optional emission direction in owner-local coordinates.
    /// Local frame matches authored sprite space: +x right, +y down.
    /// If set, this becomes the base axis before emission_angle/spread rotation.
    #[serde(default)]
    pub emission_local_x: Option<f64>,
    #[serde(default)]
    pub emission_local_y: Option<f64>,
    /// Z-axis emission direction in owner-local coordinates. 0.0 for 2D emitters.
    #[serde(default)]
    pub emission_local_z: Option<f64>,
    #[serde(default)]
    pub backward_speed: Option<f64>,
    #[serde(default)]
    pub ttl_ms: Option<i64>,
    #[serde(default)]
    pub radius: Option<i64>,
    #[serde(default)]
    pub velocity_scale: Option<f64>,
    #[serde(default)]
    pub lifecycle: Option<String>,
    #[serde(default)]
    pub follow_local_x: Option<f64>,
    #[serde(default)]
    pub follow_local_y: Option<f64>,
    #[serde(default)]
    pub follow_local_z: Option<f64>,
    #[serde(default)]
    pub follow_inherit_heading: Option<bool>,

    // === PHYSICS FLAGS ===
    /// Thread mode for particle processing: "light" (main thread, default),
    /// "physics" (worker thread with full physics), "gravity" (worker with gravity only).
    #[serde(default)]
    pub thread_mode: Option<String>,
    /// Enable collision detection for particles from this emitter.
    #[serde(default)]
    pub collision: Option<bool>,
    /// Collision mask - which tags can this particle collide with.
    /// Example: ["enemy", "terrain"]
    #[serde(default)]
    pub collision_mask: Option<Vec<String>>,
    /// Gravity scale for particles (0.0 = no gravity, 1.0 = full gravity).
    #[serde(default)]
    pub gravity_scale: Option<f64>,
    /// Gravity mode: "flat" (constant downward, default) or "orbital" (centripetal toward a world point).
    #[serde(default)]
    pub gravity_mode: Option<String>,
    /// World X of the orbital gravity attractor (planet center). Used with gravity_mode: orbital.
    #[serde(default)]
    pub gravity_center_x: Option<f64>,
    /// World Y of the orbital gravity attractor (planet center). Used with gravity_mode: orbital.
    #[serde(default)]
    pub gravity_center_y: Option<f64>,
    /// World Z of the orbital gravity attractor. 0.0 for 2D orbital gravity.
    #[serde(default)]
    pub gravity_center_z: Option<f64>,
    /// Gravitational constant for orbital mode. Acceleration = gravity_constant / dist².
    #[serde(default)]
    pub gravity_constant: Option<f64>,
    /// Bounce coefficient when colliding (0.0 = absorb, 1.0 = elastic).
    #[serde(default)]
    pub bounce: Option<f64>,
    /// Particle mass for physics calculations.
    #[serde(default)]
    pub mass: Option<f64>,

    // === COLOR RAMP ===
    /// Named palette particle ramp to use as the default color ramp.
    /// Engine resolves this against the active palette's `particles` map at emit time.
    /// Resolution order: args `color_ramp` > active palette `particles[palette_ramp]` > `color_ramp`.
    #[serde(default)]
    pub palette_ramp: Option<String>,
    /// Per-particle color sequence: index 0 = freshest (life=1.0), last = oldest.
    /// Engine samples: idx = floor((1.0 - life_ratio) * N), clamped to N-1.
    /// Used as fallback when `palette_ramp` is unset or the palette has no matching entry.
    #[serde(default)]
    pub color_ramp: Option<Vec<String>>,
    /// Particle radius at full life (life=1.0). Defaults to `radius` field if unset.
    #[serde(default)]
    pub radius_max: Option<i64>,
    /// Particle radius at end of life (life→0). 0 = fade out, ≥1 = stays visible.
    #[serde(default)]
    pub radius_min: Option<i64>,
}

/// Group template: predefined batch spawn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTemplate {
    pub prefab: String,
    pub spawns: Vec<SpawnSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnSpec {
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub vx: f64,
    #[serde(default)]
    pub vy: f64,
    #[serde(default)]
    pub shape: i64,
    #[serde(default)]
    pub size: i64,
}

/// Wave template: dynamic spawn generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveTemplate {
    pub prefab: String,
    #[serde(default)]
    pub size_distribution: Vec<SizeDistribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeDistribution {
    #[serde(default)]
    pub min_idx: i64,
    #[serde(default)]
    pub max_idx: Option<i64>,
    pub size: i64,
}

impl ModCatalogs {
    /// Create an empty catalog set.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prefab(&self, id: &str) -> Option<&PrefabTemplate> {
        self.prefabs.get(id)
    }

    pub fn spec(&self, id: &str) -> Option<&CatalogSpec> {
        self.specs.get(id)
    }

    /// Load catalogs from a directory (mod_source/catalogs/).
    pub fn load_from_directory(catalogs_dir: &std::path::Path) -> Result<Self, String> {
        let mut catalogs = Self::new();

        // Load input profiles
        let input_path = catalogs_dir.join("input-profiles.yaml");
        if input_path.exists() {
            let content = std::fs::read_to_string(&input_path)
                .map_err(|e| format!("Failed to read input-profiles.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse input-profiles.yaml: {}", e))?;

            if let Some(profiles) = parsed.get("profiles").and_then(|v| v.as_mapping()) {
                for (key, value) in profiles.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(profile) = serde_yaml::from_value::<InputProfile>(value.clone()) {
                            catalogs.input_profiles.insert(key_str.to_string(), profile);
                        }
                    }
                }
            }
        }

        // Load prefabs
        let prefabs_path = catalogs_dir.join("prefabs.yaml");
        if prefabs_path.exists() {
            let content = std::fs::read_to_string(&prefabs_path)
                .map_err(|e| format!("Failed to read prefabs.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse prefabs.yaml: {}", e))?;

            if let Some(prefabs) = parsed.get("prefabs").and_then(|v| v.as_mapping()) {
                let mut raw_prefabs = HashMap::new();
                for (key, value) in prefabs.iter() {
                    if let Some(key_str) = key.as_str() {
                        raw_prefabs.insert(key_str.to_string(), value.clone());
                    }
                }

                let mut resolved_prefabs = HashMap::new();
                for key in raw_prefabs.keys().cloned().collect::<Vec<_>>() {
                    let resolved = resolve_prefab_yaml(
                        &key,
                        &raw_prefabs,
                        &mut resolved_prefabs,
                        &mut Vec::new(),
                    )?;
                    let prefab = serde_yaml::from_value::<PrefabTemplate>(resolved)
                        .map_err(|e| format!("Failed to decode prefab '{key}': {}", e))?;
                    catalogs.prefabs.insert(key, prefab);
                }
            }
        }

        // Load runtime presets
        let presets_path = catalogs_dir.join("presets.yaml");
        if presets_path.exists() {
            let content = std::fs::read_to_string(&presets_path)
                .map_err(|e| format!("Failed to read presets.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse presets.yaml: {}", e))?;

            if let Some(presets) = parsed.get("presets") {
                catalogs.presets = serde_yaml::from_value::<CatalogPresets>(presets.clone())
                    .map_err(|e| format!("Failed to decode presets.yaml: {}", e))?;
            }
        }

        // Load reusable specs
        let specs_path = catalogs_dir.join("specs.yaml");
        if specs_path.exists() {
            let content = std::fs::read_to_string(&specs_path)
                .map_err(|e| format!("Failed to read specs.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse specs.yaml: {}", e))?;

            if let Some(specs) = parsed.get("specs").and_then(|v| v.as_mapping()) {
                for (key, value) in specs.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(spec) = serde_yaml::from_value::<CatalogSpec>(value.clone()) {
                            catalogs.specs.insert(key_str.to_string(), spec);
                        }
                    }
                }
            }
        }

        // Load weapons
        let weapons_path = catalogs_dir.join("weapons.yaml");
        if weapons_path.exists() {
            let content = std::fs::read_to_string(&weapons_path)
                .map_err(|e| format!("Failed to read weapons.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse weapons.yaml: {}", e))?;

            if let Some(weapons) = parsed.get("weapons").and_then(|v| v.as_mapping()) {
                for (key, value) in weapons.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(weapon) = serde_yaml::from_value::<WeaponConfig>(value.clone()) {
                            catalogs.weapons.insert(key_str.to_string(), weapon);
                        }
                    }
                }
            }
        }

        // Load emitters
        let emitters_path = catalogs_dir.join("emitters.yaml");
        if emitters_path.exists() {
            let content = std::fs::read_to_string(&emitters_path)
                .map_err(|e| format!("Failed to read emitters.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse emitters.yaml: {}", e))?;

            if let Some(emitters) = parsed.get("emitters").and_then(|v| v.as_mapping()) {
                for (key, value) in emitters.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(emitter) = serde_yaml::from_value::<EmitterConfig>(value.clone())
                        {
                            catalogs.emitters.insert(key_str.to_string(), emitter);
                        }
                    }
                }
            }
        }

        // Load spawners (groups and waves)
        let spawners_path = catalogs_dir.join("spawners.yaml");
        if spawners_path.exists() {
            let content = std::fs::read_to_string(&spawners_path)
                .map_err(|e| format!("Failed to read spawners.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse spawners.yaml: {}", e))?;

            if let Some(groups) = parsed.get("groups").and_then(|v| v.as_mapping()) {
                for (key, value) in groups.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(group) = serde_yaml::from_value::<GroupTemplate>(value.clone()) {
                            catalogs.groups.insert(key_str.to_string(), group);
                        }
                    }
                }
            }

            if let Some(waves) = parsed.get("waves").and_then(|v| v.as_mapping()) {
                for (key, value) in waves.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(wave) = serde_yaml::from_value::<WaveTemplate>(value.clone()) {
                            catalogs.waves.insert(key_str.to_string(), wave);
                        }
                    }
                }
            }
        }

        catalogs.celestial = CelestialCatalogs::load_from_directory(catalogs_dir)?;

        Ok(catalogs)
    }

    /// Create test catalogs with generic prefabs, weapons, and emitters.
    /// Used by behavior tests when no mod catalogs are available.
    #[cfg(test)]
    pub fn test_catalogs() -> Self {
        let mut catalogs = ModCatalogs::default();

        // Add test prefabs (ship, entity, bullet, smoke) with components
        use serde_json::json;

        // Ship prefab
        let mut ship_components = HashMap::new();
        ship_components.insert("controller_type".to_string(), json!("ArcadeController"));
        ship_components.insert(
            "config".to_string(),
            json!({
                "turn_step_ms": 25,
                "thrust_power": 100.0,
                "max_speed": 200.0,
                "heading_bits": 8
            }),
        );

        catalogs.prefabs.insert(
            "vehicle".to_string(),
            PrefabTemplate {
                kind: "vehicle".to_string(),
                sprite_template: Some("vehicle".to_string()),
                transform: None,
                fg_colour: None,
                default_tags: vec![],
                init_fields: HashMap::new(),
                components: Some(PrefabComponents {
                    physics: Some(PhysicsComponent {
                        vx: Some(0.0),
                        vy: Some(0.0),
                        ax: Some(0.0),
                        ay: Some(0.0),
                        drag: Some(0.1),
                        max_speed: Some(200.0),
                        ..PhysicsComponent::default()
                    }),
                    collider: Some(ColliderComponent {
                        shape: "circle".to_string(),
                        radius: Some(10.0),
                        width: None,
                        height: None,
                        layer: Some(0xFFFF),
                        mask: Some(0xFFFF),
                    }),
                    controller: Some(ControllerComponent {
                        controller_type: "ArcadeController".to_string(),
                        config: Some(ship_components),
                    }),
                    lifecycle: None,
                    wrappable: Some(true),
                    extra_data: None,
                    ..PrefabComponents::default()
                }),
            },
        );

        // Entity prefab (generic non-player entity for tests)
        catalogs.prefabs.insert(
            "entity".to_string(),
            PrefabTemplate {
                kind: "entity".to_string(),
                sprite_template: Some("entity-template".to_string()),
                transform: None,
                fg_colour: None,
                default_tags: vec![],
                init_fields: HashMap::new(),
                components: Some(PrefabComponents {
                    physics: Some(PhysicsComponent {
                        vx: None,
                        vy: None,
                        ax: None,
                        ay: None,
                        drag: None,
                        max_speed: None,
                        ..PhysicsComponent::default()
                    }),
                    collider: Some(ColliderComponent {
                        shape: "circle".to_string(),
                        radius: Some(15.0),
                        width: None,
                        height: None,
                        layer: Some(0xFFFF),
                        mask: Some(0xFFFF),
                    }),
                    controller: None,
                    lifecycle: None,
                    wrappable: Some(true),
                    extra_data: Some(HashMap::from([(
                        "metadata".to_string(),
                        json!({
                            "family": "test-core",
                            "canary": {
                                "revision": 1,
                                "enabled": true
                            }
                        }),
                    )])),
                    ..PrefabComponents::default()
                }),
            },
        );

        // Projectile prefab
        catalogs.prefabs.insert(
            "projectile".to_string(),
            PrefabTemplate {
                kind: "projectile".to_string(),
                sprite_template: Some("projectile-template".to_string()),
                transform: None,
                fg_colour: None,
                default_tags: vec![],
                init_fields: HashMap::new(),
                components: Some(PrefabComponents {
                    physics: Some(PhysicsComponent {
                        vx: None,
                        vy: None,
                        ax: None,
                        ay: None,
                        drag: Some(0.0),
                        max_speed: None,
                        ..PhysicsComponent::default()
                    }),
                    collider: Some(ColliderComponent {
                        shape: "circle".to_string(),
                        radius: Some(3.0),
                        width: None,
                        height: None,
                        layer: Some(0xFFFF),
                        mask: Some(0xFFFF),
                    }),
                    controller: None,
                    lifecycle: Some("Ttl".to_string()),
                    wrappable: Some(true),
                    extra_data: None,
                    ..PrefabComponents::default()
                }),
            },
        );

        // Smoke prefab
        catalogs.prefabs.insert(
            "smoke".to_string(),
            PrefabTemplate {
                kind: "smoke".to_string(),
                sprite_template: Some("smoke-template".to_string()),
                transform: None,
                fg_colour: None,
                default_tags: vec![],
                init_fields: HashMap::new(),
                components: Some(PrefabComponents {
                    physics: Some(PhysicsComponent {
                        vx: None,
                        vy: None,
                        ax: None,
                        ay: None,
                        drag: Some(0.04),
                        max_speed: None,
                        ..PhysicsComponent::default()
                    }),
                    collider: None,
                    controller: None,
                    lifecycle: Some("Ttl".to_string()),
                    wrappable: None,
                    extra_data: None,
                    ..PrefabComponents::default()
                }),
            },
        );

        catalogs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_catalog_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "shell-quest-engine-behavior-{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("temp catalog dir should be created");
        dir
    }

    #[test]
    fn test_empty_catalogs() {
        let catalogs = ModCatalogs::new();
        assert!(catalogs.input_profiles.is_empty());
        assert!(catalogs.prefabs.is_empty());
        assert!(catalogs.presets.players.is_empty());
        assert!(catalogs.specs.is_empty());
        assert!(catalogs.weapons.is_empty());
        assert!(catalogs.emitters.is_empty());
        assert!(catalogs.groups.is_empty());
        assert!(catalogs.waves.is_empty());
        assert!(catalogs.celestial.planet_types.is_empty());
        assert!(catalogs.celestial.bodies.is_empty());
    }

    #[test]
    fn test_input_profile_parsing() {
        let yaml = r#"
profiles:
  default:
    bindings:
      turn_left: ["Left", "a", "A"]
      turn_right: ["Right", "d", "D"]
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let profile_val = parsed.get("profiles").unwrap().get("default").unwrap();
        let profile: InputProfile = serde_yaml::from_value(profile_val.clone()).unwrap();

        assert_eq!(profile.bindings.get("turn_left").unwrap().len(), 3);
        assert_eq!(profile.bindings.get("turn_right").unwrap().len(), 3);
    }

    #[test]
    fn test_prefab_parsing() {
        let yaml = r#"
prefabs:
  vehicle:
    kind: "vehicle"
    sprite_template: "vehicle"
    init_fields:
      x: 0
      y: 0
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let prefab_val = parsed.get("prefabs").unwrap().get("vehicle").unwrap();
        let prefab: PrefabTemplate = serde_yaml::from_value(prefab_val.clone()).unwrap();

        assert_eq!(prefab.kind, "vehicle");
        assert_eq!(prefab.sprite_template, Some("vehicle".to_string()));
        assert!(!prefab.init_fields.is_empty());
    }

    #[test]
    fn test_prefab_parsing_supports_extended_runtime_bundle_fields() {
        let yaml = r#"
prefabs:
  cockpit:
    kind: "cockpit"
    sprite_template: "cockpit-panel"
    transform:
      z: 1.5
      pitch: 4.0
      scale_z: 0.9
    components:
      render:
        mesh: "cockpit://sim"
        camera_source: "scene"
      reference-frame:
        mode: "LocalHorizon"
        body_id: "generated-planet"
        inherit_linear_velocity: true
      linear-motor-3d:
        space: "ReferenceFrame"
        accel: 24.0
        max_speed: 320.0
      angular-motor-3d:
        mode: "Rate"
        yaw_rate: 90.0
        look_sensitivity: 1.4
      character-motor-3d:
        up_mode: "SurfaceNormal"
        max_slope_deg: 50.0
      flight-motor-3d:
        translational_dofs: [true, true, true]
        rotational_dofs: [true, true, false]
        horizon_lock_strength: 0.2
      camera-rig:
        rig-type: "Cockpit"
        preset: "cockpit-default"
      audio:
        bus: "sfx"
        events: ["engine_hum"]
      gameplay:
        module: "player.ship"
      celestial-binding:
        body_id: "generated-planet"
        site_id: "surface-spawn"
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let prefab_val = parsed.get("prefabs").unwrap().get("cockpit").unwrap();
        let prefab: PrefabTemplate = serde_yaml::from_value(prefab_val.clone()).unwrap();

        assert_eq!(prefab.transform.as_ref().and_then(|t| t.z), Some(1.5));
        let components = prefab.components.expect("prefab should decode components");
        assert_eq!(
            components
                .render
                .as_ref()
                .and_then(|render| render.mesh.as_deref()),
            Some("cockpit://sim")
        );
        assert_eq!(
            components
                .reference_frame
                .as_ref()
                .and_then(|frame| frame.mode.as_deref()),
            Some("LocalHorizon")
        );
        assert_eq!(
            components
                .camera_rig
                .as_ref()
                .and_then(|rig| rig.rig_type.as_deref()),
            Some("Cockpit")
        );
        assert_eq!(
            components
                .celestial_binding
                .as_ref()
                .and_then(|binding| binding.site_id.as_deref()),
            Some("surface-spawn")
        );
    }

    #[test]
    fn test_prefab_ref_resolution_deep_merges_nested_component_fields() {
        let dir = temp_catalog_dir("prefab-ref");
        fs::write(
            dir.join("prefabs.yaml"),
            r#"
prefabs:
  canary-base:
    kind: "probe"
    sprite_template: "probe-template"
    components:
      extra_data:
        profile:
          family: "euclidean"
          revision: 1
          nested:
            enabled: true
  canary-nested:
    ref: canary-base
    components:
      extra_data:
        profile:
          revision: 2
          nested:
            note: "merged"
"#,
        )
        .unwrap();

        let catalogs = ModCatalogs::load_from_directory(&dir).unwrap();
        let prefab = catalogs.prefabs.get("canary-nested").unwrap();
        assert_eq!(prefab.kind, "probe");
        assert_eq!(prefab.sprite_template.as_deref(), Some("probe-template"));

        let profile = prefab
            .components
            .as_ref()
            .and_then(|components| components.extra_data.as_ref())
            .and_then(|extra| extra.get("profile"))
            .and_then(|value| value.as_object())
            .unwrap();
        assert_eq!(
            profile.get("family").and_then(|v| v.as_str()),
            Some("euclidean")
        );
        assert_eq!(profile.get("revision").and_then(|v| v.as_i64()), Some(2));
        let nested = profile
            .get("nested")
            .and_then(|value| value.as_object())
            .unwrap();
        assert_eq!(nested.get("enabled").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(nested.get("note").and_then(|v| v.as_str()), Some("merged"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_preset_catalog_parsing_separates_runtime_policy_roles() {
        let yaml = r#"
presets:
  players:
    eva-6dof:
      input_profile: "default-flight"
      controller:
        controller_type: "VehicleAssembly"
      config:
        controlled: true
  cameras:
    cockpit-default:
      controller-kind: "Cockpit"
      target: "controlled-entity"
      config:
        sway_lag_sec: 0.08
  ui:
    hud-standard:
      layout: "hud/cockpit"
  spawns:
    surface-spawn:
      spawn-type: "planet-surface"
      config:
        clearance_m: 6.0
  gravity:
    radial-body:
      gravity-type: "radial-body"
      config:
        body_id: "generated-planet"
  surfaces:
    local-horizon:
      surface-type: "local-horizon"
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let presets_val = parsed.get("presets").unwrap();
        let presets: CatalogPresets = serde_yaml::from_value(presets_val.clone()).unwrap();

        assert_eq!(
            presets
                .player("eva-6dof")
                .and_then(|preset| preset.input_profile.as_deref()),
            Some("default-flight")
        );
        assert_eq!(
            presets
                .camera("cockpit-default")
                .and_then(|preset| preset.controller_kind.as_deref()),
            Some("Cockpit")
        );
        assert_eq!(
            presets
                .spawn("surface-spawn")
                .and_then(|preset| preset.spawn_type.as_deref()),
            Some("planet-surface")
        );
        assert_eq!(
            presets
                .surface("local-horizon")
                .and_then(|preset| preset.surface_type.as_deref()),
            Some("local-horizon")
        );
    }

    #[test]
    fn test_spec_catalog_parsing_supports_reusable_non_instantiated_data() {
        let yaml = r#"
specs:
  cockpit-view:
    kind: "render"
    data:
      fov: 42
      glare: "thin"
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let spec_val = parsed.get("specs").unwrap().get("cockpit-view").unwrap();
        let spec: CatalogSpec = serde_yaml::from_value(spec_val.clone()).unwrap();

        assert_eq!(spec.kind, "render");
        assert_eq!(
            spec.data
                .get("glare")
                .and_then(|value| value.as_str())
                .unwrap(),
            "thin"
        );
    }

    #[test]
    fn test_load_from_directory_reads_presets_and_specs() {
        let dir = temp_catalog_dir("catalog-load");
        fs::write(
            dir.join("prefabs.yaml"),
            r#"
prefabs:
  probe:
    kind: "probe"
    sprite_template: "probe-template"
"#,
        )
        .unwrap();
        fs::write(
            dir.join("presets.yaml"),
            r#"
presets:
  cameras:
    orbit-inspector:
      controller-kind: "Orbit"
      config:
        distance: 240.0
"#,
        )
        .unwrap();
        fs::write(
            dir.join("specs.yaml"),
            r#"
specs:
  probe-view:
    kind: "render"
    data:
      mesh: "cockpit://probe"
"#,
        )
        .unwrap();

        let catalogs = ModCatalogs::load_from_directory(&dir).unwrap();

        assert!(catalogs.prefabs.contains_key("probe"));
        assert_eq!(
            catalogs
                .presets
                .camera("orbit-inspector")
                .and_then(|preset| preset.controller_kind.as_deref()),
            Some("Orbit")
        );
        assert_eq!(
            catalogs
                .specs
                .get("probe-view")
                .and_then(|spec| spec.data.get("mesh"))
                .and_then(|value| value.as_str()),
            Some("cockpit://probe")
        );

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_group_parsing() {
        let yaml = r#"
groups:
  game.initial:
    prefab: "entity"
    spawns:
      - {x: -300, y: -210, vx: 2.0, vy: 0.0, shape: 0, size: 2}
      - {x: 300, y: -210, vx: 0.0, vy: 2.0, shape: 1, size: 3}
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let group_val = parsed.get("groups").unwrap().get("game.initial").unwrap();
        let group: GroupTemplate = serde_yaml::from_value(group_val.clone()).unwrap();

        assert_eq!(group.prefab, "entity");
        assert_eq!(group.spawns.len(), 2);
        assert_eq!(group.spawns[0].x, -300.0);
    }

    #[test]
    fn test_wave_parsing() {
        let yaml = r#"
waves:
  game.dynamic:
    prefab: "entity"
    size_distribution:
      - {min_idx: 0, max_idx: 2, size: 3}
      - {min_idx: 2, max_idx: 5, size: 2}
      - {min_idx: 5, size: 1}
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let wave_val = parsed.get("waves").unwrap().get("game.dynamic").unwrap();
        let wave: WaveTemplate = serde_yaml::from_value(wave_val.clone()).unwrap();

        assert_eq!(wave.prefab, "entity");
        assert_eq!(wave.size_distribution.len(), 3);
    }
}

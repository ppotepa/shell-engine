//! Mod-scoped gameplay catalogs for data-driven helpers.
//! Catalogs allow mods to define prefabs, weapons, emitters, input profiles, etc.
//! via YAML instead of hardcoding in Rust.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Complete set of catalogs for a mod.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModCatalogs {
    pub input_profiles: HashMap<String, InputProfile>,
    pub prefabs: HashMap<String, PrefabTemplate>,
    pub weapons: HashMap<String, WeaponConfig>,
    pub emitters: HashMap<String, EmitterConfig>,
    pub groups: HashMap<String, GroupTemplate>,
    pub waves: HashMap<String, WaveTemplate>,
    /// Planet visual presets: archetype id → visual definition. Loaded from `catalogs/planets.yaml`.
    pub planet_types: HashMap<String, PlanetDef>,
    /// Orbital bodies: body id → orbital + physical definition. Loaded from `catalogs/bodies.yaml`.
    pub bodies: HashMap<String, BodyDef>,
}

/// Input action bindings: action_name -> list of key codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputProfile {
    pub bindings: HashMap<String, Vec<String>>,
}

/// Prefab template for entity spawning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabTemplate {
    pub kind: String,
    #[serde(default)]
    pub sprite_template: Option<String>,
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

/// Component specifications for data-driven prefab spawning.
/// Allows mods to define physics, colliders, controllers, and lifecycle policies without Rust code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabComponents {
    #[serde(default)]
    pub physics: Option<PhysicsComponent>,
    #[serde(default)]
    pub collider: Option<ColliderComponent>,
    #[serde(default)]
    pub controller: Option<ControllerComponent>,
    #[serde(default)]
    pub lifecycle: Option<String>, // "Persistent", "Ttl", "OwnerBound", "TtlOwnerBound"
    #[serde(default)]
    pub wrappable: Option<bool>, // Enable wrap_bounds
    #[serde(default)]
    pub extra_data: Option<HashMap<String, JsonValue>>, // Additional entity fields
}

/// Physics component: velocity, drag, max_speed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsComponent {
    #[serde(default)]
    pub vx: Option<f64>,
    #[serde(default)]
    pub vy: Option<f64>,
    #[serde(default)]
    pub ax: Option<f64>,
    #[serde(default)]
    pub ay: Option<f64>,
    #[serde(default)]
    pub drag: Option<f64>,
    #[serde(default)]
    pub max_speed: Option<f64>,
    #[serde(default)]
    pub mass: Option<f64>,
    #[serde(default)]
    pub restitution: Option<f64>,
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

/// Visual preset for a planet type (surface, clouds, atmosphere, biomes).
/// Defines all renderer-level parameters for one planet archetype.
/// Referenced by `BodyDef.planet_type`. Loaded from `catalogs/planets.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetDef {
    // ── Surface ──────────────────────────────────────────────────────────────
    #[serde(default = "PlanetDef::default_ocean_color")]
    pub ocean_color: String,
    #[serde(default = "PlanetDef::default_land_color")]
    pub land_color: String,
    #[serde(default = "PlanetDef::default_terrain_threshold")]
    pub terrain_threshold: f64,
    #[serde(default = "PlanetDef::default_terrain_noise_scale")]
    pub terrain_noise_scale: f64,
    #[serde(default = "PlanetDef::default_terrain_noise_octaves")]
    pub terrain_noise_octaves: u8,
    #[serde(default = "PlanetDef::default_marble_depth")]
    pub marble_depth: f64,
    #[serde(default = "PlanetDef::default_ambient")]
    pub ambient: f64,
    #[serde(default = "PlanetDef::default_latitude_bands")]
    pub latitude_bands: u8,
    #[serde(default = "PlanetDef::default_latitude_band_depth")]
    pub latitude_band_depth: f64,
    // ── Biomes ───────────────────────────────────────────────────────────────
    #[serde(default)]
    pub polar_ice_color: Option<String>,
    #[serde(default = "PlanetDef::default_polar_ice_start")]
    pub polar_ice_start: f64,
    #[serde(default = "PlanetDef::default_polar_ice_end")]
    pub polar_ice_end: f64,
    #[serde(default)]
    pub desert_color: Option<String>,
    #[serde(default)]
    pub desert_strength: f64,
    // ── Atmosphere ───────────────────────────────────────────────────────────
    #[serde(default)]
    pub atmo_color: Option<String>,
    #[serde(default)]
    pub atmo_strength: f64,
    #[serde(default = "PlanetDef::default_atmo_rim_power")]
    pub atmo_rim_power: f64,
    // ── Night lights ─────────────────────────────────────────────────────────
    #[serde(default)]
    pub night_light_color: Option<String>,
    #[serde(default = "PlanetDef::default_night_light_threshold")]
    pub night_light_threshold: f64,
    #[serde(default)]
    pub night_light_intensity: f64,
    // ── Light direction (sun) ─────────────────────────────────────────────────
    #[serde(default = "PlanetDef::default_sun_dir_x")]
    pub sun_dir_x: f64,
    #[serde(default = "PlanetDef::default_sun_dir_y")]
    pub sun_dir_y: f64,
    #[serde(default = "PlanetDef::default_sun_dir_z")]
    pub sun_dir_z: f64,
    // ── Spin rates (degrees per second) ──────────────────────────────────────
    #[serde(default = "PlanetDef::default_surface_spin_dps")]
    pub surface_spin_dps: f64,
    #[serde(default = "PlanetDef::default_cloud_spin_dps")]
    pub cloud_spin_dps: f64,
    #[serde(default = "PlanetDef::default_cloud_spin_2_dps")]
    pub cloud_spin_2_dps: f64,
    // ── Cloud visual ─────────────────────────────────────────────────────────
    #[serde(default)]
    pub cloud_color: Option<String>,
    #[serde(default = "PlanetDef::default_cloud_threshold")]
    pub cloud_threshold: f64,
    #[serde(default = "PlanetDef::default_cloud_noise_scale")]
    pub cloud_noise_scale: f64,
    #[serde(default = "PlanetDef::default_cloud_noise_octaves")]
    pub cloud_noise_octaves: u8,
    // ── Shading palette ──────────────────────────────────────────────────────
    #[serde(default)]
    pub shadow_color: Option<String>,
    #[serde(default)]
    pub midtone_color: Option<String>,
    #[serde(default)]
    pub highlight_color: Option<String>,
}

impl PlanetDef {
    fn default_ocean_color() -> String { "#0b2748".to_string() }
    fn default_land_color() -> String { "#4f6b3d".to_string() }
    fn default_terrain_threshold() -> f64 { 0.585 }
    fn default_terrain_noise_scale() -> f64 { 2.15 }
    fn default_terrain_noise_octaves() -> u8 { 4 }
    fn default_marble_depth() -> f64 { 0.018 }
    fn default_ambient() -> f64 { 0.055 }
    fn default_latitude_bands() -> u8 { 5 }
    fn default_latitude_band_depth() -> f64 { 0.08 }
    fn default_polar_ice_start() -> f64 { 0.78 }
    fn default_polar_ice_end() -> f64 { 0.93 }
    fn default_atmo_rim_power() -> f64 { 4.8 }
    fn default_night_light_threshold() -> f64 { 0.84 }
    fn default_sun_dir_x() -> f64 { 0.72 }
    fn default_sun_dir_y() -> f64 { -0.56 }
    fn default_sun_dir_z() -> f64 { 0.40 }
    fn default_surface_spin_dps() -> f64 { 0.06 }
    fn default_cloud_spin_dps() -> f64 { 0.10 }
    fn default_cloud_spin_2_dps() -> f64 { 0.08 }
    fn default_cloud_threshold() -> f64 { 0.68 }
    fn default_cloud_noise_scale() -> f64 { 3.8 }
    fn default_cloud_noise_octaves() -> u8 { 3 }
}

impl Default for PlanetDef {
    fn default() -> Self {
        Self {
            ocean_color: Self::default_ocean_color(),
            land_color: Self::default_land_color(),
            terrain_threshold: Self::default_terrain_threshold(),
            terrain_noise_scale: Self::default_terrain_noise_scale(),
            terrain_noise_octaves: Self::default_terrain_noise_octaves(),
            marble_depth: Self::default_marble_depth(),
            ambient: Self::default_ambient(),
            latitude_bands: Self::default_latitude_bands(),
            latitude_band_depth: Self::default_latitude_band_depth(),
            polar_ice_color: None,
            polar_ice_start: Self::default_polar_ice_start(),
            polar_ice_end: Self::default_polar_ice_end(),
            desert_color: None,
            desert_strength: 0.0,
            atmo_color: None,
            atmo_strength: 0.0,
            atmo_rim_power: Self::default_atmo_rim_power(),
            night_light_color: None,
            night_light_threshold: Self::default_night_light_threshold(),
            night_light_intensity: 0.0,
            sun_dir_x: Self::default_sun_dir_x(),
            sun_dir_y: Self::default_sun_dir_y(),
            sun_dir_z: Self::default_sun_dir_z(),
            surface_spin_dps: Self::default_surface_spin_dps(),
            cloud_spin_dps: Self::default_cloud_spin_dps(),
            cloud_spin_2_dps: Self::default_cloud_spin_2_dps(),
            cloud_color: None,
            cloud_threshold: Self::default_cloud_threshold(),
            cloud_noise_scale: Self::default_cloud_noise_scale(),
            cloud_noise_octaves: Self::default_cloud_noise_octaves(),
            shadow_color: None,
            midtone_color: None,
            highlight_color: None,
        }
    }
}

/// Orbital body definition — a specific planet, moon, or station in the scene.
/// References a `PlanetDef` type for visuals, and optionally orbits a parent body.
/// Loaded from `catalogs/bodies.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyDef {
    /// Visual preset type id (key in `planet_types`).
    #[serde(default)]
    pub planet_type: Option<String>,
    /// World-space center X when no parent (absolute position).
    #[serde(default)]
    pub center_x: f64,
    /// World-space center Y when no parent (absolute position).
    #[serde(default)]
    pub center_y: f64,
    /// Parent body id. `None` = fixed at center_x/y.
    #[serde(default)]
    pub parent: Option<String>,
    /// Orbit radius in world pixels (0 = stationary at center_x/y).
    #[serde(default)]
    pub orbit_radius: f64,
    /// Full orbit period in seconds (0 = stationary).
    #[serde(default)]
    pub orbit_period_sec: f64,
    /// Starting orbital phase in degrees (0 = 3-o'clock).
    #[serde(default)]
    pub orbit_phase_deg: f64,
    /// Visual sphere radius in world pixels (for display/camera reference).
    #[serde(default = "BodyDef::default_radius_px")]
    pub radius_px: f64,
    /// Gravitational mu constant (px³/s²) for orbital mechanics.
    #[serde(default)]
    pub gravity_mu: f64,
    /// Collision/gameplay surface radius in pixels.
    #[serde(default = "BodyDef::default_surface_radius")]
    pub surface_radius: f64,
    /// Scene sprite id for the surface layer.
    #[serde(default)]
    pub sprite_surface: Option<String>,
    /// Scene sprite id for cloud layer 1.
    #[serde(default)]
    pub sprite_clouds: Option<String>,
    /// Scene sprite id for cloud layer 2.
    #[serde(default)]
    pub sprite_clouds_2: Option<String>,
}

impl BodyDef {
    fn default_radius_px() -> f64 { 115.0 }
    fn default_surface_radius() -> f64 { 90.0 }
}

impl Default for BodyDef {
    fn default() -> Self {
        Self {
            planet_type: None,
            center_x: 0.0,
            center_y: 0.0,
            parent: None,
            orbit_radius: 0.0,
            orbit_period_sec: 0.0,
            orbit_phase_deg: 0.0,
            radius_px: Self::default_radius_px(),
            gravity_mu: 0.0,
            surface_radius: Self::default_surface_radius(),
            sprite_surface: None,
            sprite_clouds: None,
            sprite_clouds_2: None,
        }
    }
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
                for (key, value) in prefabs.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(prefab) = serde_yaml::from_value::<PrefabTemplate>(value.clone())
                        {
                            catalogs.prefabs.insert(key_str.to_string(), prefab);
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

        // Load planet visual presets
        let planets_path = catalogs_dir.join("planets.yaml");
        if planets_path.exists() {
            let content = std::fs::read_to_string(&planets_path)
                .map_err(|e| format!("Failed to read planets.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse planets.yaml: {}", e))?;

            if let Some(types) = parsed.get("planet_types").and_then(|v| v.as_mapping()) {
                for (key, value) in types.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(pdef) = serde_yaml::from_value::<PlanetDef>(value.clone()) {
                            catalogs.planet_types.insert(key_str.to_string(), pdef);
                        }
                    }
                }
            }
        }

        // Load orbital bodies
        let bodies_path = catalogs_dir.join("bodies.yaml");
        if bodies_path.exists() {
            let content = std::fs::read_to_string(&bodies_path)
                .map_err(|e| format!("Failed to read bodies.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse bodies.yaml: {}", e))?;

            if let Some(bodies) = parsed.get("bodies").and_then(|v| v.as_mapping()) {
                for (key, value) in bodies.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(body) = serde_yaml::from_value::<BodyDef>(value.clone()) {
                            catalogs.bodies.insert(key_str.to_string(), body);
                        }
                    }
                }
            }
        }

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
        ship_components.insert("config".to_string(), json!({
            "turn_step_ms": 25,
            "thrust_power": 100.0,
            "max_speed": 200.0,
            "heading_bits": 8
        }));
        
        catalogs.prefabs.insert(
            "vehicle".to_string(),
            PrefabTemplate {
                kind: "vehicle".to_string(),
                sprite_template: Some("vehicle".to_string()),
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
                }),
            },
        );

        // Entity prefab (generic non-player entity for tests)
        catalogs.prefabs.insert(
            "entity".to_string(),
            PrefabTemplate {
                kind: "entity".to_string(),
                sprite_template: Some("entity-template".to_string()),
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
                    extra_data: None,
                }),
            },
        );

        // Projectile prefab
        catalogs.prefabs.insert(
            "projectile".to_string(),
            PrefabTemplate {
                kind: "projectile".to_string(),
                sprite_template: Some("projectile-template".to_string()),
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
                }),
            },
        );

        // Smoke prefab
        catalogs.prefabs.insert(
            "smoke".to_string(),
            PrefabTemplate {
                kind: "smoke".to_string(),
                sprite_template: Some("smoke-template".to_string()),
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
                    }),
                    collider: None,
                    controller: None,
                    lifecycle: Some("Ttl".to_string()),
                    wrappable: None,
                    extra_data: None,
                }),
            },
        );

        catalogs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_catalogs() {
        let catalogs = ModCatalogs::new();
        assert!(catalogs.input_profiles.is_empty());
        assert!(catalogs.prefabs.is_empty());
        assert!(catalogs.weapons.is_empty());
        assert!(catalogs.emitters.is_empty());
        assert!(catalogs.groups.is_empty());
        assert!(catalogs.waves.is_empty());
        assert!(catalogs.planet_types.is_empty());
        assert!(catalogs.bodies.is_empty());
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
        let group_val = parsed
            .get("groups")
            .unwrap()
            .get("game.initial")
            .unwrap();
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
        let wave_val = parsed
            .get("waves")
            .unwrap()
            .get("game.dynamic")
            .unwrap();
        let wave: WaveTemplate = serde_yaml::from_value(wave_val.clone()).unwrap();

        assert_eq!(wave.prefab, "entity");
        assert_eq!(wave.size_distribution.len(), 3);
    }
}

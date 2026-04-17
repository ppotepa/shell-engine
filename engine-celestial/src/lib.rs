//! Celestial domain catalogs shared by rendering, gameplay, and scene binding.
//!
//! This crate owns the data model for authored celestial structures such as
//! orbital bodies, planet presets, regions, and systems. It intentionally stays
//! lightweight so both engine systems and higher-level behavior/runtime crates
//! can depend on it without pulling in scripting or gameplay layers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub mod derive;
pub use engine_terrain::PlanetGenParams;

/// Complete celestial catalog set for a mod.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CelestialCatalogs {
    /// Hierarchical region/sector/cluster definitions.
    #[serde(default)]
    pub regions: HashMap<String, RegionDef>,
    /// Star system definitions and map metadata.
    #[serde(default)]
    pub systems: HashMap<String, SystemDef>,
    /// Planet visual presets keyed by archetype id.
    #[serde(default)]
    pub planet_types: HashMap<String, PlanetDef>,
    /// Orbital + physical body definitions keyed by body id.
    #[serde(default)]
    pub bodies: HashMap<String, BodyDef>,
    /// Authored points of interest attached to systems/bodies.
    #[serde(default)]
    pub sites: HashMap<String, SiteDef>,
    /// High-level travel graph connections.
    #[serde(default)]
    pub routes: HashMap<String, RouteDef>,
}

/// Logical map position used by region/system navigation views.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct MapPosition {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
}

/// Galaxy / cluster / sector grouping.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RegionDef {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default, rename = "display-name")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default, rename = "map-position")]
    pub map_position: Option<MapPosition>,
}

/// One star system plus its bodies and map placement.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SystemDef {
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default, rename = "display-name")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub star: Option<String>,
    #[serde(default)]
    pub bodies: Vec<String>,
    #[serde(default, rename = "map-position")]
    pub map_position: Option<MapPosition>,
}

/// Optional authored site/POI bound to a body or system.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SiteDef {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default, rename = "display-name")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default, rename = "orbit-altitude-km")]
    pub orbit_altitude_km: Option<f64>,
    #[serde(default, rename = "lat-deg")]
    pub lat_deg: Option<f64>,
    #[serde(default, rename = "lon-deg")]
    pub lon_deg: Option<f64>,
}

/// High-level travel connection between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteDef {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub bidirectional: bool,
}

/// Visual preset for a planet type (surface, clouds, atmosphere, biomes).
/// Defines all renderer-level parameters for one planet archetype.
/// Loaded from `catalogs/celestial/planets.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Elevation-based shade relief for land pixels. 0.0 = off, ~0.35 = strong.
    /// Brightens highlands, darkens valleys — gives terrain height perception.
    #[serde(default)]
    pub terrain_relief: f64,
    /// Seed offset for terrain noise. Different seeds produce different continent shapes.
    #[serde(default)]
    pub noise_seed: f64,
    /// Domain warp strength. > 0 creates organic, twisted coastlines (0.0–2.0).
    #[serde(default)]
    pub warp_strength: f64,
    /// Octaves used for the domain warp field (1–4). Default 2.
    #[serde(default = "PlanetDef::default_warp_octaves")]
    pub warp_octaves: u8,
    /// FBM lacunarity: frequency multiplier per octave (1.5–3.0). Default 2.0.
    #[serde(default = "PlanetDef::default_lacunarity")]
    pub noise_lacunarity: f64,
    /// FBM persistence: amplitude decay per octave (0.3–0.7). Default 0.5.
    #[serde(default = "PlanetDef::default_persistence")]
    pub noise_persistence: f64,
    /// Per-pixel normal perturbation strength. Fakes bumps that respond to light (0.0–1.0).
    #[serde(default)]
    pub normal_perturb_strength: f64,
    /// Ocean specular highlight strength. Simulates sunglint on water (0.0–1.0).
    #[serde(default)]
    pub ocean_specular: f64,
    /// Scale factor for ocean surface noise. Higher = finer waves. Default 4.0.
    #[serde(default = "PlanetDef::default_ocean_noise_scale")]
    pub ocean_noise_scale: f64,
    /// Crater density scale for rocky bodies. 0.0 = no craters.
    #[serde(default)]
    pub crater_density: f64,
    /// Crater rim brightness boost (0.0–1.0). Default 0.35.
    #[serde(default = "PlanetDef::default_crater_rim")]
    pub crater_rim_height: f64,
    /// Altitude (normalized 0–1 above terrain_threshold) where snow starts. 0.0 = disabled.
    #[serde(default)]
    pub snow_line_altitude: f64,
    /// Vertex displacement strength (fraction of sphere radius).
    /// 0.0 = flat sphere; 0.12–0.22 = mountains visible at the silhouette.
    #[serde(default)]
    pub terrain_displacement: f64,
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
    /// Haze falloff power for atmospheric scattering. Lower = broader haze. Default 1.7.
    #[serde(default = "PlanetDef::default_atmo_haze_power")]
    pub atmo_haze_power: f64,
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
    /// Ambient light for cloud layer (0.0–0.1). Default 0.012.
    #[serde(default = "PlanetDef::default_cloud_ambient")]
    pub cloud_ambient: f64,
    // ── Shading palette ──────────────────────────────────────────────────────
    #[serde(default)]
    pub shadow_color: Option<String>,
    #[serde(default)]
    pub midtone_color: Option<String>,
    #[serde(default)]
    pub highlight_color: Option<String>,
    /// Blend factor between raw Lambertian shading and the shadow/midtone/highlight palette.
    /// 0.0 = no palette (ocean renders as pale grey), 1.0 = fully palette-driven.
    /// Recommended 0.7–0.85 for stylized terminal planets.
    #[serde(default)]
    pub tone_mix: f64,
    /// Cel-shading quantization levels for surface shading. 0 = smooth (no quantization).
    /// 3–5 gives a stylized look; 0 is photorealistic gradients.
    #[serde(default)]
    pub cel_levels: u8,
    // ── Tectonic heightmap (runtime only, not YAML) ──────────────────────────
    /// Tectonic heightmap elevation grid (row-major, 0=south pole, 0.5=sea level).
    /// Populated at catalog load time for generated planets; `None` for hand-authored planets.
    #[serde(skip)]
    pub generated_heightmap: Option<std::sync::Arc<Vec<f32>>>,
    /// Width of the tectonic heightmap grid.
    #[serde(skip)]
    pub generated_heightmap_w: u32,
    /// Height of the tectonic heightmap grid.
    #[serde(skip)]
    pub generated_heightmap_h: u32,
    /// Blend factor: 0.0 = pure fBm noise, 1.0 = pure tectonic heightmap.
    /// Intermediate values blend both for organic continent edges with fine detail.
    #[serde(default = "PlanetDef::default_heightmap_blend")]
    pub heightmap_blend: f64,
}

impl PlanetDef {
    fn default_ocean_color() -> String {
        "#0b2748".to_string()
    }
    fn default_land_color() -> String {
        "#4f6b3d".to_string()
    }
    fn default_terrain_threshold() -> f64 {
        0.585
    }
    fn default_terrain_noise_scale() -> f64 {
        2.15
    }
    fn default_terrain_noise_octaves() -> u8 {
        4
    }
    fn default_marble_depth() -> f64 {
        0.018
    }
    fn default_ambient() -> f64 {
        0.055
    }
    fn default_latitude_bands() -> u8 {
        5
    }
    fn default_latitude_band_depth() -> f64 {
        0.08
    }
    fn default_polar_ice_start() -> f64 {
        0.78
    }
    fn default_polar_ice_end() -> f64 {
        0.93
    }
    fn default_atmo_rim_power() -> f64 {
        4.8
    }
    fn default_night_light_threshold() -> f64 {
        0.84
    }
    fn default_sun_dir_x() -> f64 {
        0.72
    }
    fn default_sun_dir_y() -> f64 {
        0.22
    }
    fn default_sun_dir_z() -> f64 {
        0.66
    }
    fn default_surface_spin_dps() -> f64 {
        0.45
    }
    fn default_cloud_spin_dps() -> f64 {
        0.7
    }
    fn default_cloud_spin_2_dps() -> f64 {
        0.18
    }
    fn default_cloud_threshold() -> f64 {
        0.52
    }
    fn default_cloud_noise_scale() -> f64 {
        3.6
    }
    fn default_cloud_noise_octaves() -> u8 {
        4
    }
    fn default_warp_octaves() -> u8 {
        2
    }
    fn default_lacunarity() -> f64 {
        2.0
    }
    fn default_persistence() -> f64 {
        0.5
    }
    fn default_crater_rim() -> f64 {
        0.35
    }
    fn default_atmo_haze_power() -> f64 {
        1.7
    }
    fn default_cloud_ambient() -> f64 {
        0.012
    }
    fn default_ocean_noise_scale() -> f64 {
        4.0
    }
    fn default_heightmap_blend() -> f64 {
        0.0
    }

    /// A `PlanetDef` suitable as a base for generated planets.
    /// Same as `Default` but with moderate spin rates pre-set.
    pub fn default_generated() -> Self {
        let mut d = Self::default();
        d.surface_spin_dps = 0.06;
        d.cloud_spin_dps = 0.10;
        d.cloud_spin_2_dps = 0.08;
        d.ambient = 0.06;
        d.marble_depth = 0.03;
        d.noise_lacunarity = 2.1;
        d.noise_persistence = 0.48;
        d.atmo_rim_power = 3.8;
        d
    }
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
            terrain_relief: 0.0,
            noise_seed: 0.0,
            warp_strength: 0.0,
            warp_octaves: 2,
            noise_lacunarity: 2.0,
            noise_persistence: 0.5,
            normal_perturb_strength: 0.0,
            ocean_specular: 0.0,
            ocean_noise_scale: Self::default_ocean_noise_scale(),
            crater_density: 0.0,
            crater_rim_height: 0.35,
            snow_line_altitude: 0.0,
            terrain_displacement: 0.0,
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
            atmo_haze_power: Self::default_atmo_haze_power(),
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
            cloud_ambient: Self::default_cloud_ambient(),
            shadow_color: None,
            midtone_color: None,
            highlight_color: None,
            tone_mix: 0.0,
            cel_levels: 0,
            generated_heightmap: None,
            generated_heightmap_w: 0,
            generated_heightmap_h: 0,
            heightmap_blend: 0.0,
        }
    }
}

/// Orbital body definition used by gameplay, HUD, and planet rendering.
/// Loaded from `catalogs/celestial/bodies.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Orbit radius in world units (historical field name kept as authored contract).
    #[serde(default)]
    pub orbit_radius: f64,
    /// Full orbit period in seconds (0 = stationary).
    #[serde(default)]
    pub orbit_period_sec: f64,
    /// Starting orbital phase in degrees (0 = 3-o'clock).
    #[serde(default)]
    pub orbit_phase_deg: f64,
    /// Visual sphere radius in world units (historical `_px` name kept for compatibility).
    #[serde(default = "BodyDef::default_radius_px")]
    pub radius_px: f64,
    /// Physical radius in kilometers, when authored explicitly.
    #[serde(default)]
    pub radius_km: Option<f64>,
    /// Kilometers represented by one world unit (historical field name kept as authored contract).
    #[serde(default)]
    pub km_per_px: Option<f64>,
    /// Gravitational mu constant in world-unit^3/s^2 for orbital mechanics.
    ///
    /// With default scene spatial policy (`1 wu = 1 m`) this remains equivalent to
    /// previous behavior; authored values are interpreted in simulation world units.
    #[serde(default)]
    pub gravity_mu: f64,
    /// Optional physical gravitational parameter in km^3/s^2.
    ///
    /// When provided, runtime may convert it into world-unit^3/s^2 using the active
    /// scene spatial scale and body km mapping.
    #[serde(default, rename = "gravity-mu-km3-s2")]
    pub gravity_mu_km3_s2: Option<f64>,
    /// Collision/gameplay surface radius in world units.
    #[serde(default = "BodyDef::default_surface_radius")]
    pub surface_radius: f64,
    /// Atmosphere top in world units above the surface.
    #[serde(default)]
    pub atmosphere_top: Option<f64>,
    /// Dense atmosphere start in world units above the surface.
    #[serde(default)]
    pub atmosphere_dense_start: Option<f64>,
    /// Max drag coefficient applied in dense atmosphere.
    #[serde(default)]
    pub atmosphere_drag_max: Option<f64>,
    /// Atmosphere top in kilometers above the surface.
    #[serde(default)]
    pub atmosphere_top_km: Option<f64>,
    /// Dense atmosphere start in kilometers above the surface.
    #[serde(default)]
    pub atmosphere_dense_start_km: Option<f64>,
    /// Cloud deck bottom in kilometers above the surface.
    #[serde(default)]
    pub cloud_bottom_km: Option<f64>,
    /// Cloud deck top in kilometers above the surface.
    #[serde(default)]
    pub cloud_top_km: Option<f64>,
}

impl BodyDef {
    fn default_radius_px() -> f64 {
        115.0
    }
    fn default_surface_radius() -> f64 {
        90.0
    }

    /// Resolve kilometers represented by one world-unit for this body.
    ///
    /// Resolution order:
    /// 1. explicit `km_per_px` on the body
    /// 2. derived from `radius_km / radius_px`
    /// 3. scene-level `meters_per_world_unit` (when provided and non-default)
    pub fn km_per_world_unit(&self, scene_meters_per_world_unit: Option<f64>) -> Option<f64> {
        if let Some(km_per_px) = self.km_per_px {
            return Some(km_per_px.max(0.0001));
        }
        if let Some(radius_km) = self.radius_km {
            return Some(radius_km / self.radius_px.max(1.0));
        }
        let meters = scene_meters_per_world_unit?;
        if (meters - 1.0).abs() <= f64::EPSILON {
            return None;
        }
        Some((meters / 1000.0).max(f64::MIN_POSITIVE))
    }

    /// Resolve kilometers per world-unit with Earth-radius fallback.
    ///
    /// Falls back to `6371 / radius_px` when no explicit/derived/scene mapping exists.
    pub fn km_per_world_unit_or_earth(&self, scene_meters_per_world_unit: Option<f64>) -> f64 {
        self.km_per_world_unit(scene_meters_per_world_unit)
            .unwrap_or_else(|| 6371.0 / self.radius_px.max(1.0))
    }

    /// Resolve physical body radius in kilometers.
    pub fn resolved_radius_km(&self, scene_meters_per_world_unit: Option<f64>) -> Option<f64> {
        self.radius_km.or_else(|| {
            self.km_per_world_unit(scene_meters_per_world_unit)
                .map(|v| self.radius_px * v)
        })
    }

    /// Resolve atmosphere top altitude in kilometers above the surface.
    pub fn resolved_atmosphere_top_km(
        &self,
        scene_meters_per_world_unit: Option<f64>,
    ) -> Option<f64> {
        self.atmosphere_top_km.or_else(|| {
            self.atmosphere_top.and_then(|top| {
                self.km_per_world_unit(scene_meters_per_world_unit)
                    .map(|v| top * v)
            })
        })
    }

    /// Resolve dense atmosphere start altitude in kilometers above the surface.
    pub fn resolved_atmosphere_dense_start_km(
        &self,
        scene_meters_per_world_unit: Option<f64>,
    ) -> Option<f64> {
        self.atmosphere_dense_start_km.or_else(|| {
            self.atmosphere_dense_start.and_then(|top| {
                self.km_per_world_unit(scene_meters_per_world_unit)
                    .map(|v| top * v)
            })
        })
    }

    /// Resolve gravitational parameter in world-unit^3/s^2.
    ///
    /// If physical `gravity-mu-km3-s2` is authored and km/world-unit can be resolved,
    /// it takes precedence. Otherwise falls back to `gravity_mu`.
    pub fn resolved_gravity_mu_world_units(&self, scene_meters_per_world_unit: Option<f64>) -> f64 {
        if let Some(mu_km3_s2) = self.gravity_mu_km3_s2 {
            if let Some(km_per_world_unit) = self.km_per_world_unit(scene_meters_per_world_unit) {
                let denom = km_per_world_unit.max(f64::MIN_POSITIVE).powi(3);
                return mu_km3_s2 / denom;
            }
        }
        self.gravity_mu
    }

    /// Orbit angle in radians for `elapsed_sec` in this body's local orbit frame.
    ///
    /// `0 rad` points to +X direction.
    pub fn orbit_angle_rad(&self, elapsed_sec: f64) -> f64 {
        let base = self.orbit_phase_deg.to_radians();
        if self.orbit_period_sec.abs() <= f64::EPSILON {
            return base;
        }
        let omega = std::f64::consts::TAU / self.orbit_period_sec;
        base + elapsed_sec * omega
    }

    /// Local orbital offset from parent center at `elapsed_sec`.
    pub fn orbit_offset(&self, elapsed_sec: f64) -> (f64, f64) {
        if self.orbit_radius.abs() <= f64::EPSILON {
            return (0.0, 0.0);
        }
        let a = self.orbit_angle_rad(elapsed_sec);
        (self.orbit_radius * a.cos(), self.orbit_radius * a.sin())
    }
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
            radius_km: None,
            km_per_px: None,
            gravity_mu: 0.0,
            gravity_mu_km3_s2: None,
            surface_radius: Self::default_surface_radius(),
            atmosphere_top: None,
            atmosphere_dense_start: None,
            atmosphere_drag_max: None,
            atmosphere_top_km: None,
            atmosphere_dense_start_km: None,
            cloud_bottom_km: None,
            cloud_top_km: None,
        }
    }
}

impl CelestialCatalogs {
    /// Create an empty celestial catalog set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load celestial catalogs from the mod `/catalogs/celestial` directory.
    pub fn load_from_directory(catalogs_dir: &Path) -> Result<Self, String> {
        let mut catalogs = Self::new();
        let celestial_dir = catalogs_dir.join("celestial");
        load_named_catalog(
            &celestial_dir.join("regions.yaml"),
            "regions",
            "region",
            &mut catalogs.regions,
        )?;
        load_named_catalog(
            &celestial_dir.join("systems.yaml"),
            "systems",
            "system",
            &mut catalogs.systems,
        )?;
        load_named_catalog(
            &celestial_dir.join("sites.yaml"),
            "sites",
            "site",
            &mut catalogs.sites,
        )?;
        load_named_catalog(
            &celestial_dir.join("routes.yaml"),
            "routes",
            "route",
            &mut catalogs.routes,
        )?;
        load_named_catalog(
            &celestial_dir.join("planets.yaml"),
            "planet_types",
            "planet preset",
            &mut catalogs.planet_types,
        )?;
        // Re-process any planet_type that has a `generate:` block.
        resolve_generated_planets(
            &celestial_dir.join("planets.yaml"),
            &mut catalogs.planet_types,
        )?;
        load_named_catalog(
            &celestial_dir.join("bodies.yaml"),
            "bodies",
            "body",
            &mut catalogs.bodies,
        )?;

        Ok(catalogs)
    }

    /// Resolve world-space body center at `elapsed_sec`, including parent orbits.
    ///
    /// Returns `None` when body id is not found or parent chain is cyclic/invalid.
    pub fn body_world_position(&self, body_id: &str, elapsed_sec: f64) -> Option<(f64, f64)> {
        fn resolve(
            catalogs: &CelestialCatalogs,
            body_id: &str,
            elapsed_sec: f64,
            visited: &mut std::collections::HashSet<String>,
        ) -> Option<(f64, f64)> {
            if !visited.insert(body_id.to_string()) {
                return None;
            }
            let body = catalogs.bodies.get(body_id)?;
            let local_center = (body.center_x, body.center_y);
            let local_offset = body.orbit_offset(elapsed_sec);
            let world_center = if let Some(parent_id) = body.parent.as_deref() {
                let parent_center = resolve(catalogs, parent_id, elapsed_sec, visited)?;
                (
                    parent_center.0 + local_offset.0,
                    parent_center.1 + local_offset.1,
                )
            } else {
                (
                    local_center.0 + local_offset.0,
                    local_center.1 + local_offset.1,
                )
            };
            visited.remove(body_id);
            Some(world_center)
        }

        resolve(
            self,
            body_id,
            elapsed_sec,
            &mut std::collections::HashSet::new(),
        )
    }
}

/// Post-process planet_types: for any entry that has a `generate:` YAML key,
/// run the tectonic generator and merge authored overrides on top.
fn resolve_generated_planets(
    path: &Path,
    target: &mut HashMap<String, PlanetDef>,
) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("resolve_generated_planets: read {}: {}", path.display(), e))?;
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("resolve_generated_planets: parse {}: {}", path.display(), e))?;

    let Some(entries) = parsed.get("planet_types").and_then(|v| v.as_mapping()) else {
        return Ok(());
    };

    for (key, value) in entries {
        let Some(key_str) = key.as_str() else {
            continue;
        };
        let Some(map) = value.as_mapping() else {
            continue;
        };

        // Only process entries that contain a `generate:` key
        let gen_key = serde_yaml::Value::String("generate".to_string());
        let Some(gen_value) = map.get(&gen_key) else {
            continue;
        };

        // Parse generation params
        let params: PlanetGenParams = serde_yaml::from_value(gen_value.clone())
            .map_err(|e| format!("planet_type '{}' generate: block: {}", key_str, e))?;

        // Run the tectonic generator
        let generated = engine_terrain::generate(&params);

        // Derive base PlanetDef from generated data
        let base = derive::planet_def_from_generated(&generated);

        // Serialize base to YAML Value, then overlay authored fields (excluding `generate:`)
        let mut base_value = serde_yaml::to_value(&base)
            .map_err(|e| format!("planet_type '{}' serialize base: {}", key_str, e))?;

        if let (Some(base_map), Some(overlay_map)) =
            (base_value.as_mapping_mut(), value.as_mapping())
        {
            for (k, v) in overlay_map {
                if k.as_str() != Some("generate") {
                    base_map.insert(k.clone(), v.clone());
                }
            }
        }

        // Deserialize merged value as final PlanetDef
        let final_def: PlanetDef = serde_yaml::from_value(base_value)
            .map_err(|e| format!("planet_type '{}' merge: {}", key_str, e))?;

        target.insert(key_str.to_string(), final_def);
    }

    Ok(())
}

fn load_named_catalog<T>(
    path: &Path,
    root_key: &str,
    kind_label: &str,
    target: &mut HashMap<String, T>,
) -> Result<(), String>
where
    T: for<'de> Deserialize<'de>,
{
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
    if let Some(entries) = parsed.get(root_key).and_then(|value| value.as_mapping()) {
        for (key, value) in entries {
            let Some(key_str) = key.as_str() else {
                continue;
            };
            let parsed_entry = serde_yaml::from_value::<T>(value.clone()).map_err(|e| {
                format!(
                    "Failed to parse {} '{}' in {}: {}",
                    kind_label,
                    key_str,
                    path.display(),
                    e
                )
            })?;
            target.insert(key_str.to_string(), parsed_entry);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{BodyDef, CelestialCatalogs};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn load_from_directory_reads_only_celestial_subdirectory_catalogs() {
        let temp = tempdir().expect("tempdir");
        let catalogs_dir = temp.path().join("catalogs");
        let celestial_dir = catalogs_dir.join("celestial");
        fs::create_dir_all(&celestial_dir).expect("create catalogs");
        fs::write(
            catalogs_dir.join("planets.yaml"),
            r##"
planet_types:
  root_style:
    land_color: "#101010"
"##,
        )
        .expect("write root planets");
        fs::write(
            catalogs_dir.join("bodies.yaml"),
            r#"
bodies:
  root-earth:
    planet_type: root_style
    radius_km: 1111.0
"#,
        )
        .expect("write root bodies");
        fs::write(
            celestial_dir.join("planets.yaml"),
            r##"
planet_types:
  earth_like:
    land_color: "#4f6b3d"
"##,
        )
        .expect("write celestial planets");
        fs::write(
            celestial_dir.join("bodies.yaml"),
            r#"
bodies:
  earth:
    planet_type: earth_like
    radius_km: 6371.0
"#,
        )
        .expect("write celestial bodies");

        let catalogs =
            CelestialCatalogs::load_from_directory(&catalogs_dir).expect("load celestial catalogs");

        assert!(catalogs.regions.is_empty());
        assert!(catalogs.systems.is_empty());
        assert_eq!(catalogs.planet_types.len(), 1);
        assert_eq!(catalogs.bodies.len(), 1);
        assert!(!catalogs.planet_types.contains_key("root_style"));
        assert!(!catalogs.bodies.contains_key("root-earth"));
        assert_eq!(
            catalogs
                .bodies
                .get("earth")
                .and_then(|body| body.radius_km)
                .expect("earth radius"),
            6371.0
        );
    }

    #[test]
    fn load_from_directory_reads_full_celestial_hierarchy() {
        let temp = tempdir().expect("tempdir");
        let catalogs_dir = temp.path().join("catalogs");
        let celestial_dir = catalogs_dir.join("celestial");
        fs::create_dir_all(&celestial_dir).expect("create celestial catalogs");
        fs::write(
            celestial_dir.join("bodies.yaml"),
            r#"
bodies:
  earth:
    radius_km: 6371.0
    parent: sun
  moon:
    parent: earth
"#,
        )
        .expect("write celestial bodies");
        fs::write(
            celestial_dir.join("regions.yaml"),
            r#"
regions:
  local-cluster:
    kind: cluster
"#,
        )
        .expect("write regions");
        fs::write(
            celestial_dir.join("systems.yaml"),
            r#"
systems:
  sol:
    region: local-cluster
    star: sun
    bodies: [sun, earth, moon]
"#,
        )
        .expect("write systems");
        fs::write(
            celestial_dir.join("sites.yaml"),
            r#"
sites:
  leo:
    body: earth
    orbit-altitude-km: 400.0
"#,
        )
        .expect("write sites");
        fs::write(
            celestial_dir.join("routes.yaml"),
            r#"
routes:
  sol-to-alpha:
    from: sol
    to: alpha-centauri
    bidirectional: true
"#,
        )
        .expect("write routes");

        let catalogs =
            CelestialCatalogs::load_from_directory(&catalogs_dir).expect("load celestial catalogs");

        assert_eq!(catalogs.regions.len(), 1);
        assert_eq!(catalogs.systems.len(), 1);
        assert_eq!(catalogs.sites.len(), 1);
        assert_eq!(catalogs.routes.len(), 1);
        assert_eq!(
            catalogs.bodies.get("earth"),
            Some(&BodyDef {
                radius_km: Some(6371.0),
                parent: Some("sun".into()),
                ..BodyDef::default()
            })
        );
    }

    #[test]
    fn body_resolvers_use_expected_priority_order() {
        let body = BodyDef {
            km_per_px: Some(42.0),
            radius_km: Some(1111.0),
            radius_px: 100.0,
            atmosphere_top: Some(8.0),
            atmosphere_dense_start: Some(3.0),
            ..BodyDef::default()
        };
        assert_eq!(body.km_per_world_unit(Some(2000.0)), Some(42.0));
        assert_eq!(body.resolved_radius_km(Some(2000.0)), Some(1111.0));
        assert_eq!(body.resolved_atmosphere_top_km(Some(2000.0)), Some(336.0));
        assert_eq!(
            body.resolved_atmosphere_dense_start_km(Some(2000.0)),
            Some(126.0)
        );
    }

    #[test]
    fn scene_spatial_scale_is_used_when_body_km_values_are_missing() {
        let body = BodyDef {
            radius_px: 100.0,
            atmosphere_top: Some(10.0),
            ..BodyDef::default()
        };
        assert_eq!(body.km_per_world_unit(None), None);
        assert_eq!(body.km_per_world_unit(Some(1.0)), None);
        assert_eq!(body.km_per_world_unit(Some(2000.0)), Some(2.0));
        assert_eq!(body.resolved_radius_km(Some(2000.0)), Some(200.0));
        assert_eq!(body.resolved_atmosphere_top_km(Some(2000.0)), Some(20.0));
    }

    #[test]
    fn resolved_gravity_mu_prefers_physical_value_when_scale_is_known() {
        let body = BodyDef {
            gravity_mu: 123.0,
            gravity_mu_km3_s2: Some(1000.0),
            km_per_px: Some(2.0),
            ..BodyDef::default()
        };
        // 1 wu = 2 km => mu_wu = 1000 / (2^3) = 125
        assert!((body.resolved_gravity_mu_world_units(None) - 125.0).abs() < f64::EPSILON);
    }

    #[test]
    fn resolved_gravity_mu_uses_scene_scale_when_body_km_mapping_is_missing() {
        let body = BodyDef {
            gravity_mu: 11.0,
            gravity_mu_km3_s2: Some(1000.0),
            ..BodyDef::default()
        };
        // 2000 m / wu => 2 km / wu => mu_wu = 1000 / 8 = 125
        assert!((body.resolved_gravity_mu_world_units(Some(2000.0)) - 125.0).abs() < f64::EPSILON);
    }

    #[test]
    fn km_per_world_unit_or_earth_uses_earth_radius_fallback() {
        let body = BodyDef {
            radius_px: 100.0,
            ..BodyDef::default()
        };
        assert!((body.km_per_world_unit_or_earth(None) - 63.71).abs() < 0.00001);
    }

    #[test]
    fn body_world_position_resolves_parent_chain() {
        let mut catalogs = CelestialCatalogs::default();
        catalogs.bodies.insert(
            "star".into(),
            BodyDef {
                center_x: 100.0,
                center_y: 200.0,
                ..BodyDef::default()
            },
        );
        catalogs.bodies.insert(
            "planet".into(),
            BodyDef {
                parent: Some("star".into()),
                orbit_radius: 10.0,
                orbit_period_sec: 100.0,
                orbit_phase_deg: 0.0,
                ..BodyDef::default()
            },
        );
        catalogs.bodies.insert(
            "moon".into(),
            BodyDef {
                parent: Some("planet".into()),
                orbit_radius: 2.0,
                orbit_period_sec: 50.0,
                orbit_phase_deg: 90.0,
                ..BodyDef::default()
            },
        );

        let (px, py) = catalogs
            .body_world_position("planet", 0.0)
            .expect("planet position");
        assert!((px - 110.0).abs() < 0.0001);
        assert!((py - 200.0).abs() < 0.0001);

        let (mx, my) = catalogs
            .body_world_position("moon", 0.0)
            .expect("moon position");
        assert!((mx - 110.0).abs() < 0.0001);
        assert!((my - 202.0).abs() < 0.0001);
    }

    #[test]
    fn body_world_position_detects_parent_cycles() {
        let mut catalogs = CelestialCatalogs::default();
        catalogs.bodies.insert(
            "a".into(),
            BodyDef {
                parent: Some("b".into()),
                ..BodyDef::default()
            },
        );
        catalogs.bodies.insert(
            "b".into(),
            BodyDef {
                parent: Some("a".into()),
                ..BodyDef::default()
            },
        );

        assert_eq!(catalogs.body_world_position("a", 0.0), None);
    }
}

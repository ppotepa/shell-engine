use crate::{BodyDef, CelestialCatalogs, SiteDef, SystemDef};
use engine_core::{game_state::GameState, scene::CelestialClockSource};
use engine_terrain::{Biome, GeneratedPlanet, PlanetGenParams};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct WorldPoint3 {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub z: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct WorldVec3 {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub z: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BodyPatch {
    #[serde(default)]
    pub planet_type: Option<Option<String>>,
    #[serde(default)]
    pub center_x: Option<f64>,
    #[serde(default)]
    pub center_y: Option<f64>,
    #[serde(default)]
    pub parent: Option<Option<String>>,
    #[serde(default)]
    pub orbit_radius: Option<f64>,
    #[serde(default)]
    pub orbit_period_sec: Option<f64>,
    #[serde(default)]
    pub orbit_phase_deg: Option<f64>,
    #[serde(default)]
    pub radius_px: Option<f64>,
    #[serde(default)]
    pub radius_km: Option<Option<f64>>,
    #[serde(default)]
    pub km_per_px: Option<Option<f64>>,
    #[serde(default)]
    pub gravity_mu: Option<f64>,
    #[serde(default)]
    pub gravity_mu_km3_s2: Option<Option<f64>>,
    #[serde(default)]
    pub surface_radius: Option<f64>,
    #[serde(default)]
    pub atmosphere_top: Option<Option<f64>>,
    #[serde(default)]
    pub atmosphere_dense_start: Option<Option<f64>>,
    #[serde(default)]
    pub atmosphere_drag_max: Option<Option<f64>>,
    #[serde(default)]
    pub atmosphere_top_km: Option<Option<f64>>,
    #[serde(default)]
    pub atmosphere_dense_start_km: Option<Option<f64>>,
    #[serde(default)]
    pub cloud_bottom_km: Option<Option<f64>>,
    #[serde(default)]
    pub cloud_top_km: Option<Option<f64>>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct SurfaceAnchor3 {
    pub center: WorldPoint3,
    #[serde(default)]
    pub render_radius_world: f64,
    #[serde(default)]
    pub surface_radius_world: f64,
    #[serde(default)]
    pub radius_km: Option<f64>,
    #[serde(default)]
    pub km_per_world_unit: Option<f64>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct BodyPose3 {
    pub center: WorldPoint3,
    #[serde(default)]
    pub parent_center: Option<WorldPoint3>,
    #[serde(default)]
    pub orbit_angle_rad: f64,
    #[serde(default)]
    pub render_radius_world: f64,
    #[serde(default)]
    pub surface_radius_world: f64,
    #[serde(default)]
    pub radius_km: Option<f64>,
    #[serde(default)]
    pub km_per_world_unit: Option<f64>,
    #[serde(default)]
    pub gravity_mu_world_units: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct GravitySample3 {
    pub accel: WorldVec3,
    pub radial_up: WorldVec3,
    #[serde(default)]
    pub distance_world: f64,
    #[serde(default)]
    pub altitude_world: f64,
    #[serde(default)]
    pub altitude_km: f64,
    #[serde(default)]
    pub gravity_mu_world_units: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct AtmosphereSample {
    #[serde(default)]
    pub altitude_world: f64,
    #[serde(default)]
    pub altitude_km: f64,
    #[serde(default)]
    pub density: f64,
    #[serde(default)]
    pub dense_density: f64,
    #[serde(default)]
    pub drag: f64,
    #[serde(default)]
    pub heat_band: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct LocalFrame3 {
    pub origin: WorldPoint3,
    pub up: WorldVec3,
    pub east: WorldVec3,
    pub north: WorldVec3,
    pub tangent_forward: WorldVec3,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct SurfacePoint3 {
    pub point: WorldPoint3,
    pub normal: WorldVec3,
    #[serde(default)]
    pub radius_world: f64,
    #[serde(default)]
    pub altitude_world: f64,
    #[serde(default)]
    pub altitude_km: f64,
    #[serde(default)]
    pub longitude_deg: f64,
    #[serde(default)]
    pub latitude_deg: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SitePose3 {
    #[serde(default)]
    pub body_id: Option<String>,
    pub position: WorldPoint3,
    #[serde(default)]
    pub body_center: Option<WorldPoint3>,
    pub up: WorldVec3,
    #[serde(default)]
    pub altitude_world: f64,
    #[serde(default)]
    pub altitude_km: f64,
    #[serde(default)]
    pub longitude_deg: Option<f64>,
    #[serde(default)]
    pub latitude_deg: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SystemQuery3 {
    pub id: String,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub star_body_id: Option<String>,
    #[serde(default)]
    pub star_center: Option<WorldPoint3>,
    #[serde(default)]
    pub map_position: Option<WorldPoint3>,
    #[serde(default)]
    pub bodies: Vec<String>,
    #[serde(default)]
    pub sites: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct PlanetSpawnSample {
    #[serde(default)]
    pub row: usize,
    #[serde(default)]
    pub col: usize,
    #[serde(default)]
    pub longitude_deg: f64,
    #[serde(default)]
    pub latitude_deg: f64,
    pub normal: WorldVec3,
    #[serde(default)]
    pub surface_radius_scale: f64,
    #[serde(default)]
    pub surface_offset: f64,
    #[serde(default)]
    pub elevation: f32,
    #[serde(default)]
    pub moisture: f32,
    #[serde(default)]
    pub temperature: f32,
    #[serde(default)]
    pub biome: Option<Biome>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct CelestialQueryContext {
    #[serde(default)]
    pub elapsed_sec: f64,
    #[serde(default)]
    pub scene_meters_per_world_unit: Option<f64>,
}

pub const CAMPAIGN_CLOCK_MS_PATH: &str = "/runtime/celestial/campaign_clock_ms";
pub const CAMPAIGN_CLOCK_SEC_PATH: &str = "/runtime/celestial/campaign_clock_sec";
pub const FIXED_CLOCK_MS_PATH: &str = "/runtime/celestial/fixed_clock_ms";
pub const FIXED_CLOCK_SEC_PATH: &str = "/runtime/celestial/fixed_clock_sec";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OfficialClockResolution {
    pub elapsed_sec: f64,
    pub used_path: &'static str,
}

impl CelestialQueryContext {
    pub fn from_elapsed_sec(elapsed_sec: f64) -> Self {
        Self {
            elapsed_sec,
            scene_meters_per_world_unit: None,
        }
    }

    pub fn from_elapsed_ms(elapsed_ms: u64) -> Self {
        Self::from_elapsed_sec(elapsed_ms as f64 / 1000.0)
    }

    pub fn with_scene_meters_per_world_unit(mut self, meters_per_world_unit: Option<f64>) -> Self {
        self.scene_meters_per_world_unit = meters_per_world_unit;
        self
    }
}

fn json_number(value: serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(number) => number.as_f64(),
        serde_json::Value::String(text) => text.trim().parse::<f64>().ok(),
        serde_json::Value::Bool(flag) => Some(if flag { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn read_clock_seconds(
    state: &GameState,
    sec_path: &'static str,
    ms_path: &'static str,
) -> Option<OfficialClockResolution> {
    state
        .get(sec_path)
        .and_then(json_number)
        .map(|sec| OfficialClockResolution {
            elapsed_sec: sec.max(0.0),
            used_path: sec_path,
        })
        .or_else(|| {
            state
                .get(ms_path)
                .and_then(json_number)
                .map(|ms| OfficialClockResolution {
                    elapsed_sec: ms.max(0.0) / 1000.0,
                    used_path: ms_path,
                })
        })
}

fn official_clock_paths(source: CelestialClockSource) -> Option<(&'static str, &'static str)> {
    match source {
        CelestialClockSource::Campaign => Some((CAMPAIGN_CLOCK_SEC_PATH, CAMPAIGN_CLOCK_MS_PATH)),
        CelestialClockSource::Fixed => Some((FIXED_CLOCK_SEC_PATH, FIXED_CLOCK_MS_PATH)),
        CelestialClockSource::Scene => None,
    }
}

pub fn resolve_official_clock_seconds(
    state: Option<&GameState>,
    source: CelestialClockSource,
) -> Option<OfficialClockResolution> {
    let state = state?;
    let (sec_path, ms_path) = official_clock_paths(source)?;
    read_clock_seconds(state, sec_path, ms_path)
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn normalize_or(value: WorldVec3, fallback: WorldVec3) -> WorldVec3 {
    let len_sq = value.x * value.x + value.y * value.y + value.z * value.z;
    if len_sq <= f64::EPSILON {
        return fallback;
    }
    let inv = len_sq.sqrt().recip();
    WorldVec3 {
        x: value.x * inv,
        y: value.y * inv,
        z: value.z * inv,
    }
}

fn cross(a: WorldVec3, b: WorldVec3) -> WorldVec3 {
    WorldVec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

fn longitude_deg_from_col(col: usize, width: usize) -> f64 {
    ((col as f64 + 0.5) / width.max(1) as f64) * 360.0
}

fn latitude_deg_from_row(row: usize, height: usize) -> f64 {
    90.0 - ((row as f64 + 0.5) / height.max(1) as f64) * 180.0
}

fn normal_from_lat_lon(latitude_deg: f64, longitude_deg: f64) -> WorldVec3 {
    let lat_rad = latitude_deg.to_radians();
    let lon_rad = longitude_deg.to_radians();
    let cos_lat = lat_rad.cos();
    normalize_or(
        WorldVec3 {
            x: cos_lat * lon_rad.cos(),
            y: lat_rad.sin(),
            z: cos_lat * lon_rad.sin(),
        },
        WorldVec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    )
}

impl BodyDef {
    pub fn apply_patch(&mut self, patch: &BodyPatch) {
        if let Some(value) = &patch.planet_type {
            self.planet_type = value.clone();
        }
        if let Some(value) = patch.center_x {
            self.center_x = value;
        }
        if let Some(value) = patch.center_y {
            self.center_y = value;
        }
        if let Some(value) = &patch.parent {
            self.parent = value.clone();
        }
        if let Some(value) = patch.orbit_radius {
            self.orbit_radius = value;
        }
        if let Some(value) = patch.orbit_period_sec {
            self.orbit_period_sec = value;
        }
        if let Some(value) = patch.orbit_phase_deg {
            self.orbit_phase_deg = value;
        }
        if let Some(value) = patch.radius_px {
            self.radius_px = value;
        }
        if let Some(value) = patch.radius_km {
            self.radius_km = value;
        }
        if let Some(value) = patch.km_per_px {
            self.km_per_px = value;
        }
        if let Some(value) = patch.gravity_mu {
            self.gravity_mu = value;
        }
        if let Some(value) = patch.gravity_mu_km3_s2 {
            self.gravity_mu_km3_s2 = value;
        }
        if let Some(value) = patch.surface_radius {
            self.surface_radius = value;
        }
        if let Some(value) = patch.atmosphere_top {
            self.atmosphere_top = value;
        }
        if let Some(value) = patch.atmosphere_dense_start {
            self.atmosphere_dense_start = value;
        }
        if let Some(value) = patch.atmosphere_drag_max {
            self.atmosphere_drag_max = value;
        }
        if let Some(value) = patch.atmosphere_top_km {
            self.atmosphere_top_km = value;
        }
        if let Some(value) = patch.atmosphere_dense_start_km {
            self.atmosphere_dense_start_km = value;
        }
        if let Some(value) = patch.cloud_bottom_km {
            self.cloud_bottom_km = value;
        }
        if let Some(value) = patch.cloud_top_km {
            self.cloud_top_km = value;
        }
    }

    pub fn surface_anchor(
        &self,
        center: WorldPoint3,
        scene_meters_per_world_unit: Option<f64>,
    ) -> SurfaceAnchor3 {
        SurfaceAnchor3 {
            center,
            render_radius_world: self.radius_px,
            surface_radius_world: self.surface_radius,
            radius_km: self.resolved_radius_km(scene_meters_per_world_unit),
            km_per_world_unit: self.km_per_world_unit(scene_meters_per_world_unit),
        }
    }

    pub fn gravity_sample_at(
        &self,
        center: WorldPoint3,
        point: WorldPoint3,
        scene_meters_per_world_unit: Option<f64>,
    ) -> GravitySample3 {
        let dx = center.x - point.x;
        let dy = center.y - point.y;
        let dz = center.z - point.z;
        let dist_sq = dx * dx + dy * dy + dz * dz;
        let distance_world = dist_sq.sqrt();
        let radial_up = normalize_or(
            WorldVec3 {
                x: point.x - center.x,
                y: point.y - center.y,
                z: point.z - center.z,
            },
            WorldVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        );
        let altitude_world = (distance_world - self.surface_radius).max(0.0);
        let altitude_km =
            altitude_world * self.km_per_world_unit_or_earth(scene_meters_per_world_unit);
        let gravity_mu_world_units =
            self.resolved_gravity_mu_world_units(scene_meters_per_world_unit);
        let accel =
            if distance_world <= f64::EPSILON || gravity_mu_world_units.abs() <= f64::EPSILON {
                WorldVec3::default()
            } else {
                let inv_dist = distance_world.recip();
                let inv_dist_sq = inv_dist * inv_dist;
                let scalar = gravity_mu_world_units * inv_dist_sq * inv_dist;
                WorldVec3 {
                    x: dx * scalar,
                    y: dy * scalar,
                    z: dz * scalar,
                }
            };
        GravitySample3 {
            accel,
            radial_up,
            distance_world,
            altitude_world,
            altitude_km,
            gravity_mu_world_units,
        }
    }

    pub fn atmosphere_sample_at(
        &self,
        center: WorldPoint3,
        point: WorldPoint3,
        scene_meters_per_world_unit: Option<f64>,
    ) -> AtmosphereSample {
        let gravity = self.gravity_sample_at(center, point, scene_meters_per_world_unit);
        let atmo_top_km = self
            .resolved_atmosphere_top_km(scene_meters_per_world_unit)
            .unwrap_or(0.0);
        let atmo_dense_km = self
            .resolved_atmosphere_dense_start_km(scene_meters_per_world_unit)
            .unwrap_or(0.0);
        let drag_max = self.atmosphere_drag_max.unwrap_or(0.0);
        let density = if atmo_top_km > 0.0 {
            clamp01((atmo_top_km - gravity.altitude_km) / atmo_top_km.max(0.001))
        } else {
            0.0
        };
        let dense_density = if atmo_dense_km > 0.0 {
            clamp01((atmo_dense_km - gravity.altitude_km) / atmo_dense_km.max(0.001))
        } else {
            0.0
        };
        let drag = density * density * drag_max.max(0.0);
        let heat_band = clamp01(density * 0.45 + dense_density * 0.55);
        AtmosphereSample {
            altitude_world: gravity.altitude_world,
            altitude_km: gravity.altitude_km,
            density,
            dense_density,
            drag,
            heat_band,
        }
    }

    pub fn local_frame_at(&self, center: WorldPoint3, point: WorldPoint3) -> LocalFrame3 {
        let up = normalize_or(
            WorldVec3 {
                x: point.x - center.x,
                y: point.y - center.y,
                z: point.z - center.z,
            },
            WorldVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        );
        let east = normalize_or(
            cross(
                WorldVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
                up,
            ),
            normalize_or(
                cross(
                    WorldVec3 {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                    },
                    up,
                ),
                WorldVec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
        );
        let north = normalize_or(
            cross(up, east),
            WorldVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        );
        LocalFrame3 {
            origin: center,
            up,
            east,
            north,
            tangent_forward: north,
        }
    }

    pub fn surface_point_at_lat_lon(
        &self,
        center: WorldPoint3,
        latitude_deg: f64,
        longitude_deg: f64,
        altitude_world: f64,
        scene_meters_per_world_unit: Option<f64>,
    ) -> SurfacePoint3 {
        let normal = normal_from_lat_lon(latitude_deg, longitude_deg);
        let radius_world = (self.surface_radius + altitude_world.max(0.0)).max(0.0);
        let point = WorldPoint3 {
            x: center.x + normal.x * radius_world,
            y: center.y + normal.y * radius_world,
            z: center.z + normal.z * radius_world,
        };
        let altitude_km =
            altitude_world.max(0.0) * self.km_per_world_unit_or_earth(scene_meters_per_world_unit);
        SurfacePoint3 {
            point,
            normal,
            radius_world,
            altitude_world: altitude_world.max(0.0),
            altitude_km,
            longitude_deg,
            latitude_deg,
        }
    }

    pub fn local_frame_from_lat_lon(
        &self,
        center: WorldPoint3,
        latitude_deg: f64,
        longitude_deg: f64,
        altitude_world: f64,
        scene_meters_per_world_unit: Option<f64>,
    ) -> LocalFrame3 {
        let surface = self.surface_point_at_lat_lon(
            center,
            latitude_deg,
            longitude_deg,
            altitude_world,
            scene_meters_per_world_unit,
        );
        self.local_frame_at(center, surface.point)
    }
}

impl CelestialCatalogs {
    pub fn query_context(
        &self,
        elapsed_sec: f64,
        scene_meters_per_world_unit: Option<f64>,
    ) -> CelestialQueryContext {
        CelestialQueryContext::from_elapsed_sec(elapsed_sec)
            .with_scene_meters_per_world_unit(scene_meters_per_world_unit)
    }

    pub fn body_world_position3(&self, body_id: &str, elapsed_sec: f64) -> Option<WorldPoint3> {
        self.body_world_position(body_id, elapsed_sec)
            .map(|(x, y)| WorldPoint3 { x, y, z: 0.0 })
    }

    pub fn system_query(&self, system_id: &str, elapsed_sec: f64) -> Option<SystemQuery3> {
        let system: &SystemDef = self.systems.get(system_id)?;
        let star_center = system
            .star
            .as_deref()
            .and_then(|star| self.body_world_position3(star, elapsed_sec));
        let map_position = system.map_position.map(|p| WorldPoint3 {
            x: p.x,
            y: p.y,
            z: 0.0,
        });
        let mut sites: Vec<String> = self
            .sites
            .iter()
            .filter_map(|(site_id, site)| {
                (site.system.as_deref() == Some(system_id)).then_some(site_id.clone())
            })
            .collect();
        sites.sort();
        Some(SystemQuery3 {
            id: system_id.to_string(),
            region: system.region.clone(),
            star_body_id: system.star.clone(),
            star_center,
            map_position,
            bodies: system.bodies.clone(),
            sites,
        })
    }

    pub fn body_pose(
        &self,
        body_id: &str,
        elapsed_sec: f64,
        scene_meters_per_world_unit: Option<f64>,
    ) -> Option<BodyPose3> {
        let body = self.bodies.get(body_id)?;
        let center = self.body_world_position3(body_id, elapsed_sec)?;
        let parent_center = body
            .parent
            .as_deref()
            .and_then(|parent| self.body_world_position3(parent, elapsed_sec));
        Some(BodyPose3 {
            center,
            parent_center,
            orbit_angle_rad: body.orbit_angle_rad(elapsed_sec),
            render_radius_world: body.radius_px,
            surface_radius_world: body.surface_radius,
            radius_km: body.resolved_radius_km(scene_meters_per_world_unit),
            km_per_world_unit: body.km_per_world_unit(scene_meters_per_world_unit),
            gravity_mu_world_units: body
                .resolved_gravity_mu_world_units(scene_meters_per_world_unit),
        })
    }

    pub fn surface_anchor(
        &self,
        body_id: &str,
        elapsed_sec: f64,
        scene_meters_per_world_unit: Option<f64>,
    ) -> Option<SurfaceAnchor3> {
        let body = self.bodies.get(body_id)?;
        let center = self.body_world_position3(body_id, elapsed_sec)?;
        Some(body.surface_anchor(center, scene_meters_per_world_unit))
    }

    pub fn gravity_sample(
        &self,
        body_id: &str,
        point: WorldPoint3,
        elapsed_sec: f64,
        scene_meters_per_world_unit: Option<f64>,
    ) -> Option<GravitySample3> {
        let body = self.bodies.get(body_id)?;
        let center = self.body_world_position3(body_id, elapsed_sec)?;
        Some(body.gravity_sample_at(center, point, scene_meters_per_world_unit))
    }

    pub fn atmosphere_sample(
        &self,
        body_id: &str,
        point: WorldPoint3,
        elapsed_sec: f64,
        scene_meters_per_world_unit: Option<f64>,
    ) -> Option<AtmosphereSample> {
        let body = self.bodies.get(body_id)?;
        let center = self.body_world_position3(body_id, elapsed_sec)?;
        Some(body.atmosphere_sample_at(center, point, scene_meters_per_world_unit))
    }

    pub fn local_frame(
        &self,
        body_id: &str,
        point: WorldPoint3,
        elapsed_sec: f64,
    ) -> Option<LocalFrame3> {
        let body = self.bodies.get(body_id)?;
        let center = self.body_world_position3(body_id, elapsed_sec)?;
        Some(body.local_frame_at(center, point))
    }

    pub fn surface_point(
        &self,
        body_id: &str,
        latitude_deg: f64,
        longitude_deg: f64,
        altitude_world: f64,
        elapsed_sec: f64,
        scene_meters_per_world_unit: Option<f64>,
    ) -> Option<SurfacePoint3> {
        let body = self.bodies.get(body_id)?;
        let center = self.body_world_position3(body_id, elapsed_sec)?;
        Some(body.surface_point_at_lat_lon(
            center,
            latitude_deg,
            longitude_deg,
            altitude_world,
            scene_meters_per_world_unit,
        ))
    }

    pub fn local_frame_from_lat_lon(
        &self,
        body_id: &str,
        latitude_deg: f64,
        longitude_deg: f64,
        altitude_world: f64,
        elapsed_sec: f64,
        scene_meters_per_world_unit: Option<f64>,
    ) -> Option<LocalFrame3> {
        let body = self.bodies.get(body_id)?;
        let center = self.body_world_position3(body_id, elapsed_sec)?;
        Some(body.local_frame_from_lat_lon(
            center,
            latitude_deg,
            longitude_deg,
            altitude_world,
            scene_meters_per_world_unit,
        ))
    }

    pub fn site_pose(
        &self,
        site_id: &str,
        elapsed_sec: f64,
        scene_meters_per_world_unit: Option<f64>,
    ) -> Option<SitePose3> {
        let site: &SiteDef = self.sites.get(site_id)?;
        let body_id = site.body.as_ref()?;
        let body = self.bodies.get(body_id)?;
        let center = self.body_world_position3(body_id, elapsed_sec)?;
        if let (Some(lat), Some(lon)) = (site.lat_deg, site.lon_deg) {
            let altitude_world = site
                .orbit_altitude_km
                .map(|km| km / body.km_per_world_unit_or_earth(scene_meters_per_world_unit))
                .unwrap_or(0.0);
            let surface = body.surface_point_at_lat_lon(
                center,
                lat,
                lon,
                altitude_world,
                scene_meters_per_world_unit,
            );
            return Some(SitePose3 {
                body_id: Some(body_id.clone()),
                position: surface.point,
                body_center: Some(center),
                up: surface.normal,
                altitude_world: surface.altitude_world,
                altitude_km: surface.altitude_km,
                longitude_deg: Some(lon),
                latitude_deg: Some(lat),
            });
        }
        if let Some(orbit_altitude_km) = site.orbit_altitude_km {
            let altitude_world =
                orbit_altitude_km / body.km_per_world_unit_or_earth(scene_meters_per_world_unit);
            let surface = body.surface_point_at_lat_lon(
                center,
                0.0,
                0.0,
                altitude_world,
                scene_meters_per_world_unit,
            );
            return Some(SitePose3 {
                body_id: Some(body_id.clone()),
                position: surface.point,
                body_center: Some(center),
                up: surface.normal,
                altitude_world: surface.altitude_world,
                altitude_km: surface.altitude_km,
                longitude_deg: Some(0.0),
                latitude_deg: Some(0.0),
            });
        }
        None
    }

    pub fn body_pose_in_context(
        &self,
        body_id: &str,
        ctx: CelestialQueryContext,
    ) -> Option<BodyPose3> {
        self.body_pose(body_id, ctx.elapsed_sec, ctx.scene_meters_per_world_unit)
    }

    pub fn surface_anchor_in_context(
        &self,
        body_id: &str,
        ctx: CelestialQueryContext,
    ) -> Option<SurfaceAnchor3> {
        self.surface_anchor(body_id, ctx.elapsed_sec, ctx.scene_meters_per_world_unit)
    }

    pub fn gravity_sample_in_context(
        &self,
        body_id: &str,
        point: WorldPoint3,
        ctx: CelestialQueryContext,
    ) -> Option<GravitySample3> {
        self.gravity_sample(
            body_id,
            point,
            ctx.elapsed_sec,
            ctx.scene_meters_per_world_unit,
        )
    }

    pub fn atmosphere_sample_in_context(
        &self,
        body_id: &str,
        point: WorldPoint3,
        ctx: CelestialQueryContext,
    ) -> Option<AtmosphereSample> {
        self.atmosphere_sample(
            body_id,
            point,
            ctx.elapsed_sec,
            ctx.scene_meters_per_world_unit,
        )
    }

    pub fn surface_point_in_context(
        &self,
        body_id: &str,
        latitude_deg: f64,
        longitude_deg: f64,
        altitude_world: f64,
        ctx: CelestialQueryContext,
    ) -> Option<SurfacePoint3> {
        self.surface_point(
            body_id,
            latitude_deg,
            longitude_deg,
            altitude_world,
            ctx.elapsed_sec,
            ctx.scene_meters_per_world_unit,
        )
    }

    pub fn local_frame_in_context(
        &self,
        body_id: &str,
        point: WorldPoint3,
        ctx: CelestialQueryContext,
    ) -> Option<LocalFrame3> {
        self.local_frame(body_id, point, ctx.elapsed_sec)
    }

    pub fn local_frame_from_lat_lon_in_context(
        &self,
        body_id: &str,
        latitude_deg: f64,
        longitude_deg: f64,
        altitude_world: f64,
        ctx: CelestialQueryContext,
    ) -> Option<LocalFrame3> {
        self.local_frame_from_lat_lon(
            body_id,
            latitude_deg,
            longitude_deg,
            altitude_world,
            ctx.elapsed_sec,
            ctx.scene_meters_per_world_unit,
        )
    }

    pub fn site_pose_in_context(
        &self,
        site_id: &str,
        ctx: CelestialQueryContext,
    ) -> Option<SitePose3> {
        self.site_pose(site_id, ctx.elapsed_sec, ctx.scene_meters_per_world_unit)
    }

    pub fn body_world_position3_in_context(
        &self,
        body_id: &str,
        ctx: CelestialQueryContext,
    ) -> Option<WorldPoint3> {
        self.body_world_position3(body_id, ctx.elapsed_sec)
    }

    pub fn system_query_in_context(
        &self,
        system_id: &str,
        ctx: CelestialQueryContext,
    ) -> Option<SystemQuery3> {
        self.system_query(system_id, ctx.elapsed_sec)
    }
}

pub fn default_spawn_biomes() -> Vec<Biome> {
    vec![Biome::Beach, Biome::Desert, Biome::Grassland]
}

fn spawn_search_rows(height: usize, band_only: bool) -> Vec<usize> {
    let height = height.max(1);
    let mid_row = height / 2;
    let band = ((height as f32) * 0.12).round() as usize;
    let mut rows = Vec::new();
    if band_only {
        rows.push(mid_row);
        for offset in 1..=band {
            let down = mid_row.saturating_add(offset);
            if down < height {
                rows.push(down);
            }
            if let Some(up) = mid_row.checked_sub(offset) {
                rows.push(up);
            }
        }
    } else {
        rows.extend(0..height);
    }
    rows
}

pub fn find_planet_spawn_selection(
    planet: &GeneratedPlanet,
    preferred_biomes: &[Biome],
) -> Option<(usize, usize)> {
    let preferred = if preferred_biomes.is_empty() {
        default_spawn_biomes()
    } else {
        preferred_biomes.to_vec()
    };
    let width = planet.width.max(1);
    let height = planet.height.max(1);
    let start_col = (planet.params.seed as usize) % width;

    for &target_biome in &preferred {
        for rows in [
            spawn_search_rows(height, true),
            spawn_search_rows(height, false),
        ] {
            for row in rows {
                for offset in 0..width {
                    let col = (start_col + offset) % width;
                    let cell = planet.cell(col, row);
                    if cell.biome == target_biome {
                        return Some((row, col));
                    }
                }
            }
        }
    }

    None
}

pub fn sample_planet_spawn(
    planet: &GeneratedPlanet,
    row: usize,
    col: usize,
    displacement_scale: f32,
) -> PlanetSpawnSample {
    let width = planet.width.max(1);
    let height = planet.height.max(1);
    let cell = *planet.cell(col, row);
    let (normal_x, normal_y, normal_z) = engine_terrain::grid::cell_to_xyz(col, row, width, height);
    let surface_offset = ((cell.elevation as f64 - 0.5) * 2.0) * displacement_scale as f64;
    let surface_radius_scale = (1.0 + surface_offset).max(0.0001);
    PlanetSpawnSample {
        row,
        col,
        longitude_deg: longitude_deg_from_col(col, width),
        latitude_deg: latitude_deg_from_row(row, height),
        normal: WorldVec3 {
            x: normal_x,
            y: normal_y,
            z: normal_z,
        },
        surface_radius_scale,
        surface_offset,
        elevation: cell.elevation,
        moisture: cell.moisture,
        temperature: cell.temperature,
        biome: Some(cell.biome),
    }
}

pub fn find_planet_spawn(
    planet: &GeneratedPlanet,
    displacement_scale: f32,
    preferred_biomes: &[Biome],
) -> PlanetSpawnSample {
    let width = planet.width.max(1);
    let height = planet.height.max(1);
    let (row, col) = find_planet_spawn_selection(planet, preferred_biomes).unwrap_or_else(|| {
        let start_col = (planet.params.seed as usize) % width;
        (height / 2, start_col % width)
    });
    sample_planet_spawn(planet, row, col, displacement_scale)
}

pub fn find_planet_spawn_from_params(
    params: &PlanetGenParams,
    displacement_scale: f32,
    preferred_biomes: &[Biome],
) -> PlanetSpawnSample {
    let planet = engine_terrain::generate(params);
    find_planet_spawn(&planet, displacement_scale, preferred_biomes)
}

#[cfg(test)]
mod tests {
    use super::{
        default_spawn_biomes, find_planet_spawn_from_params, resolve_official_clock_seconds,
        AtmosphereSample, BodyPatch, BodyPose3, CelestialCatalogs, CelestialQueryContext,
        GravitySample3, OfficialClockResolution, SitePose3, SurfaceAnchor3, SystemQuery3,
        WorldPoint3, CAMPAIGN_CLOCK_MS_PATH, CAMPAIGN_CLOCK_SEC_PATH, FIXED_CLOCK_MS_PATH,
        FIXED_CLOCK_SEC_PATH,
    };
    use crate::{BodyDef, SiteDef, SystemDef};
    use engine_core::game_state::GameState;
    use engine_core::scene::CelestialClockSource;
    use engine_terrain::{Biome, PlanetGenParams};

    #[test]
    fn body_apply_patch_updates_optional_and_scalar_fields() {
        let mut body = BodyDef::default();
        body.apply_patch(&BodyPatch {
            planet_type: Some(Some("earth".into())),
            radius_km: Some(Some(6371.0)),
            gravity_mu: Some(12.5),
            atmosphere_drag_max: Some(Some(0.4)),
            ..BodyPatch::default()
        });
        assert_eq!(body.planet_type.as_deref(), Some("earth"));
        assert_eq!(body.radius_km, Some(6371.0));
        assert_eq!(body.gravity_mu, 12.5);
        assert_eq!(body.atmosphere_drag_max, Some(0.4));
    }

    #[test]
    fn catalogs_resolve_pose_anchor_gravity_and_atmosphere_samples() {
        let mut catalogs = CelestialCatalogs::default();
        catalogs.bodies.insert(
            "planet".into(),
            BodyDef {
                center_x: 10.0,
                center_y: -5.0,
                radius_px: 120.0,
                surface_radius: 90.0,
                radius_km: Some(5000.0),
                gravity_mu: 1000.0,
                atmosphere_top_km: Some(50.0),
                atmosphere_dense_start_km: Some(10.0),
                atmosphere_drag_max: Some(0.8),
                ..BodyDef::default()
            },
        );

        let pose: BodyPose3 = catalogs.body_pose("planet", 0.0, None).expect("pose");
        let anchor: SurfaceAnchor3 = catalogs
            .surface_anchor("planet", 0.0, None)
            .expect("anchor");
        let gravity: GravitySample3 = catalogs
            .gravity_sample(
                "planet",
                WorldPoint3 {
                    x: 110.0,
                    y: -5.0,
                    z: 0.0,
                },
                0.0,
                None,
            )
            .expect("gravity");
        let atmo: AtmosphereSample = catalogs
            .atmosphere_sample(
                "planet",
                WorldPoint3 {
                    x: 95.0,
                    y: -5.0,
                    z: 0.0,
                },
                0.0,
                None,
            )
            .expect("atmo");

        assert_eq!(pose.center.x, 10.0);
        assert_eq!(anchor.surface_radius_world, 90.0);
        assert!(gravity.accel.x < 0.0);
        assert!(gravity.altitude_world >= 0.0);
        assert!(atmo.density > 0.0);
        assert!(atmo.drag > 0.0);
    }

    #[test]
    fn site_pose_resolves_surface_bound_sites() {
        let mut catalogs = CelestialCatalogs::default();
        catalogs.bodies.insert(
            "planet".into(),
            BodyDef {
                surface_radius: 100.0,
                km_per_px: Some(10.0),
                ..BodyDef::default()
            },
        );
        catalogs.sites.insert(
            "base".into(),
            SiteDef {
                body: Some("planet".into()),
                lat_deg: Some(20.0),
                lon_deg: Some(30.0),
                orbit_altitude_km: Some(25.0),
                ..SiteDef::default()
            },
        );

        let pose: SitePose3 = catalogs.site_pose("base", 0.0, None).expect("site pose");
        assert_eq!(pose.body_id.as_deref(), Some("planet"));
        assert!(pose.altitude_km > 0.0);
        assert!(pose.position.y > 0.0);
    }

    #[test]
    fn find_planet_spawn_from_params_returns_surface_metadata() {
        let sample = find_planet_spawn_from_params(
            &PlanetGenParams::default(),
            0.22,
            &default_spawn_biomes(),
        );
        assert!(sample.surface_radius_scale > 0.0);
        assert!(sample.longitude_deg >= 0.0);
        assert!(sample.latitude_deg >= -90.0);
        assert!(sample.biome.is_some());
    }

    #[test]
    fn find_planet_spawn_prefers_requested_biomes_when_available() {
        let params = PlanetGenParams::default();
        let sample = find_planet_spawn_from_params(&params, 0.22, &[Biome::Beach, Biome::Desert]);
        assert!(matches!(sample.biome, Some(Biome::Beach | Biome::Desert)));
    }

    #[test]
    fn query_context_routes_to_pose_and_site_helpers() {
        let mut catalogs = CelestialCatalogs::default();
        catalogs.bodies.insert(
            "planet".into(),
            BodyDef {
                surface_radius: 100.0,
                km_per_px: Some(10.0),
                ..BodyDef::default()
            },
        );
        catalogs.sites.insert(
            "base".into(),
            SiteDef {
                body: Some("planet".into()),
                lat_deg: Some(0.0),
                lon_deg: Some(90.0),
                ..SiteDef::default()
            },
        );
        let ctx = CelestialQueryContext {
            elapsed_sec: 0.0,
            scene_meters_per_world_unit: None,
        };
        assert!(catalogs.body_pose_in_context("planet", ctx).is_some());
        assert!(catalogs.site_pose_in_context("base", ctx).is_some());
    }

    #[test]
    fn query_context_uses_canonical_elapsed_ms_conversion() {
        let ctx = CelestialQueryContext::from_elapsed_ms(12_500)
            .with_scene_meters_per_world_unit(Some(3.0));
        assert!((ctx.elapsed_sec - 12.5).abs() < 0.0001);
        assert_eq!(ctx.scene_meters_per_world_unit, Some(3.0));
    }

    #[test]
    fn resolve_official_clock_prefers_seconds_path_over_milliseconds() {
        let state = GameState::new();
        assert!(state.set(CAMPAIGN_CLOCK_MS_PATH, serde_json::json!(4200)));
        assert!(state.set(CAMPAIGN_CLOCK_SEC_PATH, serde_json::json!(4.5)));

        let clock = resolve_official_clock_seconds(Some(&state), CelestialClockSource::Campaign)
            .expect("official clock");

        assert_eq!(
            clock,
            OfficialClockResolution {
                elapsed_sec: 4.5,
                used_path: CAMPAIGN_CLOCK_SEC_PATH,
            }
        );
    }

    #[test]
    fn resolve_official_clock_uses_milliseconds_path_when_seconds_missing() {
        let state = GameState::new();
        assert!(state.set(FIXED_CLOCK_MS_PATH, serde_json::json!(12500)));

        let clock = resolve_official_clock_seconds(Some(&state), CelestialClockSource::Fixed)
            .expect("official clock");

        assert_eq!(
            clock,
            OfficialClockResolution {
                elapsed_sec: 12.5,
                used_path: FIXED_CLOCK_MS_PATH,
            }
        );
    }

    #[test]
    fn resolve_official_clock_uses_seconds_path_for_fixed_source() {
        let state = GameState::new();
        assert!(state.set(FIXED_CLOCK_SEC_PATH, serde_json::json!(12.5)));

        let clock = resolve_official_clock_seconds(Some(&state), CelestialClockSource::Fixed)
            .expect("official clock");

        assert_eq!(
            clock,
            OfficialClockResolution {
                elapsed_sec: 12.5,
                used_path: FIXED_CLOCK_SEC_PATH,
            }
        );
    }

    #[test]
    fn resolve_official_clock_returns_none_without_source_specific_runtime_data() {
        let state = GameState::new();
        assert!(state.set(CAMPAIGN_CLOCK_SEC_PATH, serde_json::json!(7.0)));

        assert_eq!(
            resolve_official_clock_seconds(Some(&state), CelestialClockSource::Scene),
            None
        );
        assert_eq!(
            resolve_official_clock_seconds(None, CelestialClockSource::Campaign),
            None
        );
    }

    #[test]
    fn system_query_collects_star_center_and_sites() {
        let mut catalogs = CelestialCatalogs::default();
        catalogs.bodies.insert(
            "sun".into(),
            BodyDef {
                center_x: 5.0,
                center_y: 7.0,
                ..BodyDef::default()
            },
        );
        catalogs.systems.insert(
            "sol".into(),
            SystemDef {
                region: Some("local".into()),
                star: Some("sun".into()),
                bodies: vec!["sun".into()],
                ..SystemDef::default()
            },
        );
        catalogs.sites.insert(
            "station".into(),
            SiteDef {
                system: Some("sol".into()),
                ..SiteDef::default()
            },
        );
        let query: SystemQuery3 = catalogs.system_query("sol", 0.0).expect("system query");
        assert_eq!(query.star_body_id.as_deref(), Some("sun"));
        assert_eq!(query.star_center.expect("star center").x, 5.0);
        assert_eq!(query.sites, vec!["station".to_string()]);
    }
}

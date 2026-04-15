use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use engine_core::assets::AssetRoot;

use crate::obj_loader::{load_obj_mesh, ObjMesh};

// Global OBJ mesh cache — parse once, reuse via Arc.
static OBJ_MESH_CACHE: OnceLock<Mutex<HashMap<String, Arc<ObjMesh>>>> = OnceLock::new();

/// Parse a terrain-plane URI query string into `TerrainParams`.
///
/// Query format: `amp=A&freq=F&oct=O&rough=R&sx=S&sz=Z`
/// All parameters are optional — missing ones fall back to `TerrainParams::default()`.
fn parse_terrain_params(query: &str) -> engine_mesh::primitives::TerrainParams {
    let mut p = engine_mesh::primitives::TerrainParams::default();
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "amp" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.amplitude = f.clamp(0.01, 10.0);
                    }
                }
                "freq" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.frequency = f.clamp(0.01, 16.0);
                    }
                }
                "oct" => {
                    if let Ok(n) = v.parse::<u8>() {
                        p.octaves = n.clamp(1, 3);
                    }
                }
                "rough" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.roughness = f.clamp(0.0, 1.0);
                    }
                }
                "sx" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.seed_x = f;
                    }
                }
                "sz" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.seed_z = f;
                    }
                }
                "lac" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.lacunarity = f.clamp(1.0, 4.0);
                    }
                }
                "ridge" => {
                    p.ridge = v == "1";
                }
                "plat" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.plateau = f.clamp(0.0, 1.0);
                    }
                }
                "sea" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.sea_level = f.clamp(0.0, 1.0);
                    }
                }
                "scx" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.scale_x = f.clamp(0.25, 4.0);
                    }
                }
                "scz" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.scale_z = f.clamp(0.25, 4.0);
                    }
                }
                _ => {}
            }
        }
    }
    p
}

/// Parse terrain params from a `terrain-plane://N[?params]` or `terrain-sphere://N[?params]` URI.
///
/// Returns default params if the URI has no query string or cannot be parsed.
pub(crate) fn parse_terrain_params_from_uri(uri: &str) -> engine_mesh::primitives::TerrainParams {
    let query = uri.splitn(2, '?').nth(1).unwrap_or("");
    parse_terrain_params(query)
}

/// Get or load an OBJ mesh from cache.
/// Supports the `cube-sphere://N` and `terrain-plane://N[?params]` URI schemes for procedurally generated meshes.
pub(super) fn get_or_load_obj_mesh(asset_root: &AssetRoot, path: &str) -> Option<Arc<ObjMesh>> {
    let cache = OBJ_MESH_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Try to get from cache first.
    {
        let cache_lock = cache.lock().ok()?;
        if let Some(mesh) = cache_lock.get(path) {
            return Some(Arc::clone(mesh));
        }
    }

    // Handle procedural cube-sphere URI: cube-sphere://N
    if let Some(rest) = path.strip_prefix("cube-sphere://") {
        let subdivisions: u32 = rest.trim().parse().unwrap_or(64);
        let mesh = engine_mesh::primitives::cube_sphere(subdivisions);
        let mesh_arc = crate::obj_loader::mesh_to_obj_mesh(&mesh);
        if let Ok(mut cache_lock) = cache.lock() {
            cache_lock.insert(path.to_string(), Arc::clone(&mesh_arc));
        }
        return Some(mesh_arc);
    }

    // Handle procedural terrain-plane URI: terrain-plane://N  or  terrain-plane://N?amp=A&freq=F&oct=O&rough=R&sx=S&sz=Z
    if let Some(rest) = path.strip_prefix("terrain-plane://") {
        let (subdiv_str, query) = rest.split_once('?').unwrap_or((rest, ""));
        let subdivisions: u32 = subdiv_str.trim().parse().unwrap_or(64);
        let params = parse_terrain_params(query);
        let mesh = engine_mesh::primitives::terrain_plane(subdivisions, params);
        let mesh_arc = crate::obj_loader::mesh_to_obj_mesh(&mesh);
        if let Ok(mut cache_lock) = cache.lock() {
            cache_lock.insert(path.to_string(), Arc::clone(&mesh_arc));
        }
        return Some(mesh_arc);
    }

    // Handle procedural terrain-sphere URI: terrain-sphere://N  or  terrain-sphere://N?params
    if let Some(rest) = path.strip_prefix("terrain-sphere://") {
        let (subdiv_str, query) = rest.split_once('?').unwrap_or((rest, ""));
        let subdivisions: u32 = subdiv_str.trim().parse().unwrap_or(32);
        let params = parse_terrain_params(query);
        let mesh = engine_mesh::primitives::terrain_sphere(subdivisions, params);
        let mesh_arc = crate::obj_loader::mesh_to_obj_mesh(&mesh);
        if let Ok(mut cache_lock) = cache.lock() {
            cache_lock.insert(path.to_string(), Arc::clone(&mesh_arc));
        }
        return Some(mesh_arc);
    }

    // Handle procedural earth-sphere URI: earth-sphere://N  or  earth-sphere://N?params
    // Generates terrain-sphere geometry with altitude-based Earth-palette face colors baked in.
    if let Some(rest) = path.strip_prefix("earth-sphere://") {
        let (subdiv_str, query) = rest.split_once('?').unwrap_or((rest, ""));
        let subdivisions: u32 = subdiv_str.trim().parse().unwrap_or(32);
        let params = parse_terrain_params(query);
        let (mesh, colors) = engine_mesh::primitives::earth_terrain_sphere(subdivisions, params);
        let mesh_arc = crate::obj_loader::colored_mesh_to_obj_mesh(&mesh, &colors);
        if let Ok(mut cache_lock) = cache.lock() {
            cache_lock.insert(path.to_string(), Arc::clone(&mesh_arc));
        }
        return Some(mesh_arc);
    }

    // Handle world:// URI — full biome/climate pipeline.
    // Format: world://N?shape=sphere&base=cube&coloring=biome&seed=0&ocean=0.55&...
    if path.starts_with("world://") {
        let params = engine_worldgen::parse_world_params_from_uri(path);
        let world = engine_worldgen::build_world_mesh(&params);
        let mesh_arc = crate::obj_loader::colored_mesh_to_obj_mesh(&world.mesh, &world.face_colors);
        if let Ok(mut cache_lock) = cache.lock() {
            cache_lock.insert(path.to_string(), Arc::clone(&mesh_arc));
        }
        return Some(mesh_arc);
    }

    // Not in cache, load from asset file.
    let mesh_arc = load_obj_mesh(asset_root, path)?;

    // Store in cache.
    if let Ok(mut cache_lock) = cache.lock() {
        cache_lock.insert(path.to_string(), Arc::clone(&mesh_arc));
    }

    Some(mesh_arc)
}

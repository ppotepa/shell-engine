//! Shared 3D mesh loading and generation for render pipelines.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use crate::ModAssetSourceLoader;
use engine_core::asset_cache::AssetCache;
use engine_core::asset_source::{load_decoded_source, SourceAdapter, SourceLoader, SourceRef};
use engine_core::assets::AssetRoot;
use engine_error::EngineError;

#[derive(Debug, Clone)]
pub struct ObjMesh {
    pub vertices: Vec<[f32; 3]>,
    /// Area-weighted smooth vertex normals, computed at parse time by averaging adjacent face cross-products.
    pub smooth_normals: Vec<[f32; 3]>,
    pub edges: Vec<(usize, usize)>,
    pub faces: Vec<ObjFace>,
    pub center: [f32; 3],
    pub radius: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjFace {
    pub indices: [usize; 3],
    /// Diffuse color (sRGB, gamma-corrected from MTL Kd).
    pub color: [u8; 3],
    /// Per-material ambient reflectance (linear, from MTL Ka).
    pub ka: [f32; 3],
    /// Specular strength (average of MTL Ks, linear).
    pub ks: f32,
    /// Shininess exponent from MTL Ns.
    pub ns: f32,
}

static OBJ_CACHE: AssetCache<ObjMesh> = AssetCache::new();
const RENDER_MESH_CACHE_MAX_ENTRIES: usize = 64;

#[derive(Default)]
struct RenderMeshCache {
    entries: HashMap<String, (Arc<ObjMesh>, u64)>,
    access_tick: u64,
}

static RENDER_MESH_CACHE: OnceLock<Mutex<RenderMeshCache>> = OnceLock::new();

struct ObjMeshAdapter;

impl SourceAdapter<ObjMesh> for ObjMeshAdapter {
    fn decode(
        &self,
        source: &SourceRef,
        bytes: &[u8],
        loader: &dyn SourceLoader,
    ) -> Result<ObjMesh, Box<dyn std::error::Error + Send + Sync>> {
        let text = std::str::from_utf8(bytes).map_err(
            |_| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(EngineError::StartupCheckFailed {
                    check: "obj-decode".to_string(),
                    details: format!("OBJ source is not valid UTF-8: {}", source.value()),
                })
            },
        )?;
        let materials = load_material_palette(text, source, loader);
        parse_obj_mesh_from_text(text, &materials).ok_or_else(
            || -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(EngineError::StartupCheckFailed {
                    check: "obj-decode".to_string(),
                    details: format!("failed to parse OBJ mesh: {}", source.value()),
                })
            },
        )
    }
}

#[derive(Debug, Clone)]
struct MaterialProps {
    /// Diffuse color as sRGB u8 (gamma-corrected from MTL linear Kd).
    kd_srgb: [u8; 3],
    /// Ambient reflectance (linear, from Ka).
    ka: [f32; 3],
    /// Specular strength (average of Ks components).
    ks: f32,
    /// Shininess exponent (Ns).
    ns: f32,
}

impl Default for MaterialProps {
    fn default() -> Self {
        Self {
            kd_srgb: [220, 220, 220],
            ka: [0.18, 0.18, 0.18],
            ks: 0.05,
            ns: 10.0,
        }
    }
}

/// Loads an OBJ mesh from `mod_source` + `source` through the shared decode cache.
pub fn load_obj_mesh(mod_source: &Path, source: &str) -> Option<Arc<ObjMesh>> {
    let loader = ModAssetSourceLoader::new(mod_source).ok()?;
    let source = SourceRef::mod_asset(source);
    load_decoded_source(&OBJ_CACHE, &loader, &source, &ObjMeshAdapter)
}

/// Loads an OBJ mesh from an [`AssetRoot`] through the shared decode cache.
pub fn load_obj_mesh_from_root(asset_root: &AssetRoot, source: &str) -> Option<Arc<ObjMesh>> {
    load_obj_mesh(asset_root.mod_source(), source)
}

/// Loads any render mesh source (asset path or procedural URI) with shared caching.
///
/// Supported procedural sources:
/// - `cube-sphere://N`
/// - `terrain-plane://N?amp=A&freq=F&oct=O&rough=R&...`
/// - `terrain-sphere://N?params`
/// - `earth-sphere://N?params`
/// - `world://N?...`
pub fn load_render_mesh(asset_root: &AssetRoot, source: &str) -> Option<Arc<ObjMesh>> {
    let cache = RENDER_MESH_CACHE.get_or_init(|| Mutex::new(RenderMeshCache::default()));
    let cache_key = render_mesh_cache_key(asset_root.mod_source(), source);

    {
        let mut cache_lock = cache.lock().ok()?;
        cache_lock.access_tick = cache_lock.access_tick.saturating_add(1);
        let hit_tick = cache_lock.access_tick;
        if let Some((mesh, last_used)) = cache_lock.entries.get_mut(&cache_key) {
            *last_used = hit_tick;
            return Some(Arc::clone(mesh));
        }
    }

    let mesh = if let Some(rest) = source.strip_prefix("cube-sphere://") {
        let subdivisions: u32 = rest.trim().parse().unwrap_or(64);
        let mesh = engine_mesh::primitives::cube_sphere(subdivisions);
        mesh_to_obj_mesh(&mesh)
    } else if let Some(rest) = source.strip_prefix("terrain-plane://") {
        let (subdiv_str, query) = rest.split_once('?').unwrap_or((rest, ""));
        let subdivisions: u32 = subdiv_str.trim().parse().unwrap_or(64);
        let params = parse_terrain_params(query);
        let mesh = engine_mesh::primitives::terrain_plane(subdivisions, params);
        mesh_to_obj_mesh(&mesh)
    } else if let Some(rest) = source.strip_prefix("terrain-sphere://") {
        let (subdiv_str, query) = rest.split_once('?').unwrap_or((rest, ""));
        let subdivisions: u32 = subdiv_str.trim().parse().unwrap_or(32);
        let params = parse_terrain_params(query);
        let mesh = engine_mesh::primitives::terrain_sphere(subdivisions, params);
        mesh_to_obj_mesh(&mesh)
    } else if let Some(rest) = source.strip_prefix("earth-sphere://") {
        let (subdiv_str, query) = rest.split_once('?').unwrap_or((rest, ""));
        let subdivisions: u32 = subdiv_str.trim().parse().unwrap_or(32);
        let params = parse_terrain_params(query);
        let (mesh, colors) = engine_mesh::primitives::earth_terrain_sphere(subdivisions, params);
        colored_mesh_to_obj_mesh(&mesh, &colors)
    } else if source.starts_with("world://") {
        let params = engine_worldgen::parse_world_params_from_uri(source);
        let world = engine_worldgen::build_world_mesh(&params);
        colored_mesh_to_obj_mesh(&world.mesh, &world.face_colors)
    } else {
        load_obj_mesh_from_root(asset_root, source)?
    };

    if let Ok(mut cache_lock) = cache.lock() {
        cache_lock.access_tick = cache_lock.access_tick.saturating_add(1);
        let insert_tick = cache_lock.access_tick;
        cache_lock
            .entries
            .insert(cache_key, (Arc::clone(&mesh), insert_tick));

        while cache_lock.entries.len() > RENDER_MESH_CACHE_MAX_ENTRIES {
            let lru_key = cache_lock
                .entries
                .iter()
                .min_by_key(|(_, (_, last_used))| *last_used)
                .map(|(key, _)| key.clone());
            let Some(lru_key) = lru_key else {
                break;
            };
            cache_lock.entries.remove(&lru_key);
        }
    }
    Some(mesh)
}

fn render_mesh_cache_key(mod_source: &Path, source: &str) -> String {
    if source.contains("://") {
        return format!("generated::{source}");
    }
    let normalized = source.trim_start_matches('/').replace('\\', "/");
    format!("{}::{normalized}", mod_source.display())
}

fn load_material_palette(
    obj_text: &str,
    obj_source: &SourceRef,
    loader: &dyn SourceLoader,
) -> HashMap<String, MaterialProps> {
    let mut out = HashMap::new();
    for raw in obj_text.lines() {
        let line = raw.trim();
        let Some(rest) = line.strip_prefix("mtllib ") else {
            continue;
        };
        for rel in rest.split_whitespace() {
            let mtl_path = resolve_relative_asset_path(obj_source.value(), rel);
            let mtl_source = SourceRef::mod_asset(mtl_path.as_str());
            let Ok(bytes) = loader.read_bytes(&mtl_source) else {
                continue;
            };
            let parsed = parse_mtl_palette(&bytes);
            for (name, props) in parsed {
                out.entry(name).or_insert(props);
            }
        }
    }
    out
}

fn parse_mtl_palette(bytes: &[u8]) -> HashMap<String, MaterialProps> {
    let text = match std::str::from_utf8(bytes) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };
    let mut out: HashMap<String, MaterialProps> = HashMap::new();
    let mut current_name: Option<String> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix("newmtl ") {
            let n = name.trim();
            if !n.is_empty() {
                current_name = Some(n.to_string());
                out.entry(n.to_string()).or_default();
            }
            continue;
        }
        let Some(name) = current_name.clone() else {
            continue;
        };
        if let Some(rest) = line.strip_prefix("Kd ") {
            if let Some([r, g, b]) = parse_3f(rest) {
                let entry = out.entry(name).or_default();
                entry.kd_srgb = [linear_to_srgb(r), linear_to_srgb(g), linear_to_srgb(b)];
            }
        } else if let Some(rest) = line.strip_prefix("Ka ") {
            if let Some([r, g, b]) = parse_3f(rest) {
                out.entry(name).or_default().ka = [r, g, b];
            }
        } else if let Some(rest) = line.strip_prefix("Ks ") {
            if let Some([r, g, b]) = parse_3f(rest) {
                let entry = out.entry(name).or_default();
                entry.ks = (r + g + b) / 3.0;
            }
        } else if let Some(rest) = line.strip_prefix("Ns ") {
            if let Ok(ns) = rest.trim().parse::<f32>() {
                out.entry(name).or_default().ns = ns;
            }
        }
    }
    out
}

fn parse_3f(s: &str) -> Option<[f32; 3]> {
    let mut parts = s.split_whitespace();
    let r = parts.next()?.parse::<f32>().ok()?;
    let g = parts.next()?.parse::<f32>().ok()?;
    let b = parts.next()?.parse::<f32>().ok()?;
    Some([r, g, b])
}

fn linear_to_srgb(v: f32) -> u8 {
    let s = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn resolve_relative_asset_path(obj_source: &str, relative: &str) -> String {
    if relative.starts_with('/') {
        return relative.to_string();
    }
    let normalized_obj = obj_source.trim_start_matches('/');
    let base = Path::new(normalized_obj)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let joined = base.join(relative);
    format!("/{}", joined.to_string_lossy().replace('\\', "/"))
}

fn parse_obj_mesh_from_text(
    text: &str,
    materials: &HashMap<String, MaterialProps>,
) -> Option<ObjMesh> {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut edges: HashSet<(usize, usize)> = HashSet::new();
    let mut faces: Vec<ObjFace> = Vec::new();
    let default_mat = MaterialProps::default();
    let mut active_mat: &MaterialProps = &default_mat;

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("v ") {
            let mut parts = rest.split_whitespace();
            let (Some(xs), Some(ys), Some(zs)) = (parts.next(), parts.next(), parts.next()) else {
                continue;
            };
            let (Ok(x), Ok(y), Ok(z)) = (xs.parse::<f32>(), ys.parse::<f32>(), zs.parse::<f32>())
            else {
                continue;
            };
            vertices.push([x, y, z]);
            continue;
        }
        if let Some(rest) = line.strip_prefix("usemtl ") {
            let mtl_name = rest.trim();
            active_mat = materials.get(mtl_name).unwrap_or(&default_mat);
            continue;
        }
        if let Some(rest) = line.strip_prefix("f ") {
            let mut face: Vec<usize> = Vec::new();
            for token in rest.split_whitespace() {
                if let Some(idx) = parse_obj_vertex_index(token, vertices.len()) {
                    face.push(idx);
                }
            }
            if face.len() >= 2 {
                for idx in 0..face.len() {
                    let a = face[idx];
                    let b = face[(idx + 1) % face.len()];
                    if a != b {
                        edges.insert((a.min(b), a.max(b)));
                    }
                }
            }
            if face.len() >= 3 {
                for tri in 1..(face.len() - 1) {
                    faces.push(ObjFace {
                        indices: [face[0], face[tri], face[tri + 1]],
                        color: active_mat.kd_srgb,
                        ka: active_mat.ka,
                        ks: active_mat.ks,
                        ns: active_mat.ns,
                    });
                }
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("l ") {
            let mut polyline: Vec<usize> = Vec::new();
            for token in rest.split_whitespace() {
                if let Some(idx) = parse_obj_vertex_index(token, vertices.len()) {
                    polyline.push(idx);
                }
            }
            for pair in polyline.windows(2) {
                let a = pair[0];
                let b = pair[1];
                if a != b {
                    edges.insert((a.min(b), a.max(b)));
                }
            }
        }
    }

    if vertices.is_empty() || (edges.is_empty() && faces.is_empty()) {
        return None;
    }

    // Compute area-weighted smooth vertex normals by accumulating face cross-products.
    let mut normal_accum: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; vertices.len()];
    for face in &faces {
        let v0 = vertices[face.indices[0]];
        let v1 = vertices[face.indices[1]];
        let v2 = vertices[face.indices[2]];
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &i in &face.indices {
            normal_accum[i][0] += n[0];
            normal_accum[i][1] += n[1];
            normal_accum[i][2] += n[2];
        }
    }
    let smooth_normals: Vec<[f32; 3]> = normal_accum
        .iter()
        .map(|n| {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len <= 1e-6 {
                [0.0, 0.0, 1.0]
            } else {
                [n[0] / len, n[1] / len, n[2] / len]
            }
        })
        .collect();

    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for v in &vertices {
        for axis in 0..3 {
            min[axis] = min[axis].min(v[axis]);
            max[axis] = max[axis].max(v[axis]);
        }
    }
    let center = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];
    let mut radius = 0.0f32;
    for v in &vertices {
        let dx = v[0] - center[0];
        let dy = v[1] - center[1];
        let dz = v[2] - center[2];
        radius = radius.max((dx * dx + dy * dy + dz * dz).sqrt());
    }

    Some(ObjMesh {
        vertices,
        smooth_normals,
        edges: edges.into_iter().collect(),
        faces,
        center,
        radius: radius.max(0.001),
    })
}

/// Convert an `engine_mesh::Mesh` into an `ObjMesh` for use by renderers.
pub fn mesh_to_obj_mesh(mesh: &engine_mesh::Mesh) -> Arc<ObjMesh> {
    let mut edges: HashSet<(usize, usize)> = HashSet::new();
    let mut faces: Vec<ObjFace> = Vec::new();

    for &[a, b, c] in &mesh.faces {
        faces.push(ObjFace {
            indices: [a, b, c],
            color: [200, 200, 200],
            ka: [0.18, 0.18, 0.18],
            ks: 0.05,
            ns: 10.0,
        });
        for (x, y) in [(a, b), (b, c), (a, c)] {
            edges.insert((x.min(y), x.max(y)));
        }
    }

    Arc::new(ObjMesh {
        smooth_normals: mesh.normals.clone(),
        vertices: mesh.vertices.clone(),
        edges: edges.into_iter().collect(),
        faces,
        center: [0.0, 0.0, 0.0],
        radius: 1.0,
    })
}

/// Like [`mesh_to_obj_mesh`] but applies a pre-computed per-face color palette.
pub fn colored_mesh_to_obj_mesh(mesh: &engine_mesh::Mesh, colors: &[[u8; 3]]) -> Arc<ObjMesh> {
    let mut edges: HashSet<(usize, usize)> = HashSet::new();
    let mut faces: Vec<ObjFace> = Vec::new();

    for (fi, &[a, b, c]) in mesh.faces.iter().enumerate() {
        let color = colors.get(fi).copied().unwrap_or([200, 200, 200]);
        faces.push(ObjFace {
            indices: [a, b, c],
            color,
            ka: [0.18, 0.18, 0.18],
            ks: 0.05,
            ns: 10.0,
        });
        for (x, y) in [(a, b), (b, c), (a, c)] {
            edges.insert((x.min(y), x.max(y)));
        }
    }

    Arc::new(ObjMesh {
        smooth_normals: mesh.normals.clone(),
        vertices: mesh.vertices.clone(),
        edges: edges.into_iter().collect(),
        faces,
        center: [0.0, 0.0, 0.0],
        radius: 1.0,
    })
}

fn parse_obj_vertex_index(token: &str, vertex_count: usize) -> Option<usize> {
    let raw = token.split('/').next()?.trim();
    let idx = raw.parse::<i64>().ok()?;
    if idx > 0 {
        let zero_based = (idx - 1) as usize;
        return (zero_based < vertex_count).then_some(zero_based);
    }
    if idx < 0 {
        let abs = idx.unsigned_abs() as usize;
        if abs <= vertex_count {
            return Some(vertex_count - abs);
        }
    }
    None
}

/// Parse a terrain-plane URI query string into `TerrainParams`.
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

#[cfg(test)]
mod tests {
    use super::{load_obj_mesh, load_render_mesh, parse_mtl_palette, parse_obj_mesh_from_text};
    use engine_core::assets::AssetRoot;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    #[test]
    fn parses_vertices_faces_and_edges() {
        let raw = r#"
v 0 0 0
v 1 0 0
v 1 1 0
f 1 2 3
"#;
        let mesh = parse_obj_mesh_from_text(raw, &HashMap::new()).expect("mesh");
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.edges.len(), 3);
        assert_eq!(mesh.faces.len(), 1);
    }

    #[test]
    fn parses_mtl_diffuse_palette() {
        let raw = br#"
newmtl Wall
Ka 0.1 0.1 0.1
Kd 0.5 0.25 0.0
Ks 0.05 0.05 0.05
Ns 10
"#;
        let palette = parse_mtl_palette(raw);
        let mat = palette.get("Wall").expect("Wall material");
        assert!(
            mat.kd_srgb[0] > mat.kd_srgb[1],
            "R should be > G for orange"
        );
        assert!(mat.kd_srgb[1] > mat.kd_srgb[2], "G should be > B (B is 0)");
        assert_eq!(mat.kd_srgb[2], 0, "B channel should be 0");
        assert!((mat.ka[0] - 0.1).abs() < 1e-3);
        assert!((mat.ns - 10.0).abs() < 1e-3);
    }

    #[test]
    fn loads_obj_mesh_from_directory_source() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("assets/models")).expect("models dir");
        fs::write(
            mod_dir.join("assets/models/test.obj"),
            "mtllib test.mtl\nv 0 0 0\nv 1 0 0\nv 1 1 0\nusemtl Wall\nf 1 2 3\n",
        )
        .expect("obj");
        fs::write(
            mod_dir.join("assets/models/test.mtl"),
            "newmtl Wall\nKd 0.5 0.25 0.0\n",
        )
        .expect("mtl");

        let mesh = load_obj_mesh(&mod_dir, "/assets/models/test.obj").expect("mesh should load");
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.faces.len(), 1);
    }

    #[test]
    fn loads_obj_mesh_from_zip_source() {
        let temp = tempdir().expect("temp dir");
        let zip_path = temp.path().join("mod.zip");
        let file = fs::File::create(&zip_path).expect("zip file");
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("assets/models/test.obj", SimpleFileOptions::default())
            .expect("obj entry");
        std::io::Write::write_all(
            &mut writer,
            b"mtllib test.mtl\nv 0 0 0\nv 1 0 0\nv 1 1 0\nusemtl Wall\nf 1 2 3\n",
        )
        .expect("obj bytes");
        writer
            .start_file("assets/models/test.mtl", SimpleFileOptions::default())
            .expect("mtl entry");
        std::io::Write::write_all(&mut writer, b"newmtl Wall\nKd 0.5 0.25 0.0\n")
            .expect("mtl bytes");
        writer.finish().expect("finish zip");

        let mesh = load_obj_mesh(&zip_path, "/assets/models/test.obj").expect("mesh should load");
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.faces.len(), 1);
    }

    #[test]
    fn loads_generated_cube_sphere_mesh() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(&mod_dir).expect("mod dir");
        let root = AssetRoot::new(mod_dir);

        let mesh = load_render_mesh(&root, "cube-sphere://8").expect("generated mesh");
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.faces.is_empty());
    }
}

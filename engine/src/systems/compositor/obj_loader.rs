use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::asset_cache::AssetCache;
use crate::asset_source::{
    load_decoded_source, ModAssetSourceLoader, SourceAdapter, SourceLoader, SourceRef,
};
use crate::assets::AssetRoot;
use crate::EngineError;

#[derive(Debug, Clone)]
pub(crate) struct ObjMesh {
    pub vertices: Vec<[f32; 3]>,
    pub edges: Vec<(usize, usize)>,
    pub faces: Vec<ObjFace>,
    pub center: [f32; 3],
    pub radius: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ObjFace {
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

struct ObjMeshAdapter;

impl SourceAdapter<ObjMesh> for ObjMeshAdapter {
    fn decode(
        &self,
        source: &SourceRef,
        bytes: &[u8],
        loader: &dyn SourceLoader,
    ) -> Result<ObjMesh, Box<dyn std::error::Error + Send + Sync>> {
        let text = std::str::from_utf8(bytes).map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { Box::new(EngineError::StartupCheckFailed {
            check: "obj-decode".to_string(),
            details: format!("OBJ source is not valid UTF-8: {}", source.value()),
        })
        })?;
        let materials = load_material_palette(text, source, loader);
        parse_obj_mesh_from_text(text, &materials).ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> { Box::new(EngineError::StartupCheckFailed {
            check: "obj-decode".to_string(),
            details: format!("failed to parse OBJ mesh: {}", source.value()),
        })
        })
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

pub(crate) fn load_obj_mesh(asset_root: &AssetRoot, source: &str) -> Option<std::sync::Arc<ObjMesh>> {
    let loader = ModAssetSourceLoader::new(asset_root.mod_source()).ok()?;
    let source = SourceRef::mod_asset(source);
    load_decoded_source(&OBJ_CACHE, &loader, &source, &ObjMeshAdapter)
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
            let mtl_source = SourceRef::mod_asset(mtl_path);
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
        edges: edges.into_iter().collect(),
        faces,
        center,
        radius: radius.max(0.001),
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use super::{load_obj_mesh, parse_mtl_palette, parse_obj_mesh_from_text};
    use crate::assets::AssetRoot;
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

        let mesh = load_obj_mesh(&AssetRoot::new(mod_dir), "/assets/models/test.obj")
            .expect("mesh should load");
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

        let mesh = load_obj_mesh(&AssetRoot::new(zip_path), "/assets/models/test.obj")
            .expect("mesh should load");
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.faces.len(), 1);
    }
}

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use crossterm::style::Color;

use crate::assets::AssetRoot;
use crate::buffer::Buffer;
use crate::repositories::{create_asset_repository, AssetRepository};

#[derive(Debug, Clone)]
struct ObjMesh {
    vertices: Vec<[f32; 3]>,
    edges: Vec<(usize, usize)>,
    center: [f32; 3],
    radius: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ObjRenderParams {
    pub scale: f32,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub roll_deg: f32,
    pub rotate_y_deg_per_sec: f32,
    pub camera_distance: f32,
    pub fov_degrees: f32,
    pub scene_elapsed_ms: u64,
}

static OBJ_CACHE: OnceLock<Mutex<HashMap<String, Option<ObjMesh>>>> = OnceLock::new();

pub(super) fn obj_sprite_dimensions(width: Option<u16>, height: Option<u16>) -> (u16, u16) {
    (width.unwrap_or(64).max(1), height.unwrap_or(24).max(1))
}

pub(super) fn render_obj_content(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    params: ObjRenderParams,
    draw_char: char,
    fg: Color,
    bg: Color,
    asset_root: Option<&AssetRoot>,
    x: u16,
    y: u16,
    buf: &mut Buffer,
) {
    let Some(root) = asset_root else {
        return;
    };
    let Some(mesh) = load_obj_mesh(root, source) else {
        return;
    };
    let (target_w, target_h) = obj_sprite_dimensions(width, height);
    if target_w < 2 || target_h < 2 {
        return;
    }

    let elapsed_s = params.scene_elapsed_ms as f32 / 1000.0;
    let yaw = (params.yaw_deg + params.rotate_y_deg_per_sec * elapsed_s).to_radians();
    let pitch = params.pitch_deg.to_radians();
    let roll = params.roll_deg.to_radians();
    let fov = params.fov_degrees.clamp(10.0, 170.0).to_radians();
    let inv_tan = 1.0 / (fov * 0.5).tan().max(0.0001);
    let camera_distance = params.camera_distance.max(0.1);
    let model_scale = params.scale.max(0.0001) / mesh.radius.max(0.0001);
    let aspect = target_w as f32 / target_h as f32;

    let viewport = Viewport {
        min_x: x as i32,
        min_y: y as i32,
        max_x: x as i32 + target_w as i32 - 1,
        max_y: y as i32 + target_h as i32 - 1,
    };

    let projected: Vec<Option<(i32, i32)>> = mesh
        .vertices
        .iter()
        .map(|v| {
            let centered = [
                (v[0] - mesh.center[0]) * model_scale,
                (v[1] - mesh.center[1]) * model_scale,
                (v[2] - mesh.center[2]) * model_scale,
            ];
            let rotated = rotate_xyz(centered, pitch, yaw, roll);
            let view_z = rotated[2] + camera_distance;
            if view_z <= 0.01 {
                return None;
            }
            let ndc_x = (rotated[0] / aspect) * inv_tan / view_z;
            let ndc_y = rotated[1] * inv_tan / view_z;
            if !ndc_x.is_finite() || !ndc_y.is_finite() {
                return None;
            }
            let screen_x = x as i32 + ((ndc_x + 1.0) * 0.5 * (target_w as f32 - 1.0)).round() as i32;
            let screen_y =
                y as i32 + ((1.0 - (ndc_y + 1.0) * 0.5) * (target_h as f32 - 1.0)).round() as i32;
            Some((screen_x, screen_y))
        })
        .collect();

    let mut drawn_edges = 0usize;
    for (a, b) in &mesh.edges {
        if drawn_edges > 12_000 {
            break;
        }
        let Some((x0, y0)) = projected.get(*a).and_then(|p| *p) else {
            continue;
        };
        let Some((x1, y1)) = projected.get(*b).and_then(|p| *p) else {
            continue;
        };
        if let Some((cx0, cy0, cx1, cy1)) = clip_line_to_viewport(x0, y0, x1, y1, viewport) {
            draw_line(buf, cx0, cy0, cx1, cy1, draw_char, fg, bg);
            drawn_edges += 1;
        }
    }
}

#[derive(Clone, Copy)]
struct Viewport {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

fn draw_line(buf: &mut Buffer, mut x0: i32, mut y0: i32, x1: i32, y1: i32, ch: char, fg: Color, bg: Color) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && y0 >= 0 {
            buf.set(x0 as u16, y0 as u16, ch, fg, bg);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = err.saturating_mul(2);
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn clip_line_to_viewport(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    vp: Viewport,
) -> Option<(i32, i32, i32, i32)> {
    let mut out0 = out_code(x0, y0, vp);
    let mut out1 = out_code(x1, y1, vp);

    loop {
        if (out0 | out1) == 0 {
            return Some((x0, y0, x1, y1));
        }
        if (out0 & out1) != 0 {
            return None;
        }
        let out = if out0 != 0 { out0 } else { out1 };

        let (nx, ny) = if (out & OUT_TOP) != 0 {
            intersect_horizontal(x0, y0, x1, y1, vp.min_y)?
        } else if (out & OUT_BOTTOM) != 0 {
            intersect_horizontal(x0, y0, x1, y1, vp.max_y)?
        } else if (out & OUT_RIGHT) != 0 {
            intersect_vertical(x0, y0, x1, y1, vp.max_x)?
        } else {
            intersect_vertical(x0, y0, x1, y1, vp.min_x)?
        };

        if out == out0 {
            x0 = nx;
            y0 = ny;
            out0 = out_code(x0, y0, vp);
        } else {
            x1 = nx;
            y1 = ny;
            out1 = out_code(x1, y1, vp);
        }
    }
}

const OUT_LEFT: u8 = 1;
const OUT_RIGHT: u8 = 2;
const OUT_BOTTOM: u8 = 4;
const OUT_TOP: u8 = 8;

fn out_code(x: i32, y: i32, vp: Viewport) -> u8 {
    let mut code = 0u8;
    if x < vp.min_x {
        code |= OUT_LEFT;
    } else if x > vp.max_x {
        code |= OUT_RIGHT;
    }
    if y > vp.max_y {
        code |= OUT_BOTTOM;
    } else if y < vp.min_y {
        code |= OUT_TOP;
    }
    code
}

fn intersect_vertical(x0: i32, y0: i32, x1: i32, y1: i32, x: i32) -> Option<(i32, i32)> {
    let dx = x1 - x0;
    if dx == 0 {
        return None;
    }
    let t = (x - x0) as f32 / dx as f32;
    let y = y0 as f32 + t * (y1 - y0) as f32;
    Some((x, y.round() as i32))
}

fn intersect_horizontal(x0: i32, y0: i32, x1: i32, y1: i32, y: i32) -> Option<(i32, i32)> {
    let dy = y1 - y0;
    if dy == 0 {
        return None;
    }
    let t = (y - y0) as f32 / dy as f32;
    let x = x0 as f32 + t * (x1 - x0) as f32;
    Some((x.round() as i32, y))
}

fn rotate_xyz(v: [f32; 3], pitch: f32, yaw: f32, roll: f32) -> [f32; 3] {
    let (sp, cp) = pitch.sin_cos();
    let (sy, cy) = yaw.sin_cos();
    let (sr, cr) = roll.sin_cos();

    let x1 = v[0];
    let y1 = v[1] * cp - v[2] * sp;
    let z1 = v[1] * sp + v[2] * cp;

    let x2 = x1 * cy + z1 * sy;
    let y2 = y1;
    let z2 = -x1 * sy + z1 * cy;

    let x3 = x2 * cr - y2 * sr;
    let y3 = x2 * sr + y2 * cr;
    [x3, y3, z2]
}

fn load_obj_mesh(asset_root: &AssetRoot, source: &str) -> Option<ObjMesh> {
    let normalized = source.trim_start_matches('/');
    let key = format!("{}::{normalized}", asset_root.mod_source().display());
    let cache = OBJ_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(&key) {
            return cached.clone();
        }
    }

    let loaded = load_obj_mesh_uncached(asset_root, source);
    if let Ok(mut guard) = cache.lock() {
        guard.insert(key, loaded.clone());
    }
    loaded
}

fn load_obj_mesh_uncached(asset_root: &AssetRoot, source: &str) -> Option<ObjMesh> {
    let repo = create_asset_repository(asset_root.mod_source()).ok()?;
    let bytes = repo.read_asset_bytes(source).ok()?;
    parse_obj_mesh(&bytes)
}

fn parse_obj_mesh(bytes: &[u8]) -> Option<ObjMesh> {
    let text = std::str::from_utf8(bytes).ok()?;
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut edges: HashSet<(usize, usize)> = HashSet::new();

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

    if vertices.is_empty() || edges.is_empty() {
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
    use super::parse_obj_mesh;

    #[test]
    fn parses_vertices_and_face_edges() {
        let raw = br#"
v 0 0 0
v 1 0 0
v 1 1 0
f 1 2 3
"#;
        let mesh = parse_obj_mesh(raw).expect("mesh");
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.edges.len(), 3);
    }
}

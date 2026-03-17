use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crossterm::style::Color;

use crate::assets::AssetRoot;
use crate::buffer::Buffer;
use crate::repositories::{create_asset_repository, AssetRepository};
use crate::scene::SceneRenderedMode;

#[derive(Debug, Clone)]
struct ObjMesh {
    vertices: Vec<[f32; 3]>,
    edges: Vec<(usize, usize)>,
    faces: Vec<ObjFace>,
    center: [f32; 3],
    radius: f32,
}

#[derive(Debug, Clone, Copy)]
struct ObjFace {
    indices: [usize; 3],
    color: [u8; 3],
}

#[derive(Debug, Clone, Copy)]
struct ProjectedVertex {
    x: f32,
    y: f32,
    depth: f32,
    view: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ObjRenderParams {
    pub scale: f32,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub roll_deg: f32,
    /// Static initial rotation offsets (x=pitch, y=yaw, z=roll) from `rotation-x/y/z` YAML.
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub rotate_y_deg_per_sec: f32,
    pub camera_distance: f32,
    pub fov_degrees: f32,
    pub near_clip: f32,
    pub scene_elapsed_ms: u64,
    /// Camera pan offset in view-space units (applied before projection).
    pub camera_pan_x: f32,
    pub camera_pan_y: f32,
    /// Additional camera look rotation (accumulated from mouse). Yaw = horizontal, pitch = vertical.
    pub camera_look_yaw: f32,
    pub camera_look_pitch: f32,
}

static OBJ_CACHE: OnceLock<Mutex<HashMap<String, Option<ObjMesh>>>> = OnceLock::new();

pub(super) fn obj_sprite_dimensions(width: Option<u16>, height: Option<u16>) -> (u16, u16) {
    (width.unwrap_or(64).max(1), height.unwrap_or(24).max(1))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_obj_content(
    source: &str,
    width: Option<u16>,
    height: Option<u16>,
    mode: SceneRenderedMode,
    params: ObjRenderParams,
    wireframe: bool,
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
    let (virtual_w, virtual_h) = virtual_dimensions(mode, target_w, target_h);
    if virtual_w < 2 || virtual_h < 2 {
        return;
    }

    let elapsed_s = params.scene_elapsed_ms as f32 / 1000.0;
    // Combine static rotation-y/x/z offsets with yaw-deg/pitch-deg/roll-deg + orbit + camera look.
    let yaw = (params.yaw_deg + params.rotation_y + params.rotate_y_deg_per_sec * elapsed_s
        + params.camera_look_yaw)
        .to_radians();
    let pitch = (params.pitch_deg + params.rotation_x + params.camera_look_pitch).to_radians();
    let roll = (params.roll_deg + params.rotation_z).to_radians();
    let fov = params.fov_degrees.clamp(10.0, 170.0).to_radians();
    let inv_tan = 1.0 / (fov * 0.5).tan().max(0.0001);
    let camera_distance = params.camera_distance.max(0.1);
    let near_clip = params.near_clip.max(0.000001);
    let model_scale = params.scale.max(0.0001) / mesh.radius.max(0.0001);
    let aspect = virtual_w as f32 / virtual_h as f32;

    let viewport = Viewport {
        min_x: 0,
        min_y: 0,
        max_x: virtual_w as i32 - 1,
        max_y: virtual_h as i32 - 1,
    };
    let projected: Vec<Option<ProjectedVertex>> = mesh
        .vertices
        .iter()
        .map(|v| {
            let centered = [
                (v[0] - mesh.center[0]) * model_scale,
                (v[1] - mesh.center[1]) * model_scale,
                (v[2] - mesh.center[2]) * model_scale,
            ];
            let rotated = rotate_xyz(centered, pitch, yaw, roll);
            // Apply camera pan: shift the scene in view-space (equivalent to moving camera).
            let panned = [
                rotated[0] - params.camera_pan_x,
                rotated[1] - params.camera_pan_y,
                rotated[2],
            ];
            let view_z = panned[2] + camera_distance;
            if view_z <= near_clip {
                return None;
            }
            let ndc_x = (panned[0] / aspect) * inv_tan / view_z;
            let ndc_y = panned[1] * inv_tan / view_z;
            if !ndc_x.is_finite() || !ndc_y.is_finite() {
                return None;
            }
            Some(ProjectedVertex {
                x: (ndc_x + 1.0) * 0.5 * (virtual_w as f32 - 1.0),
                y: (1.0 - (ndc_y + 1.0) * 0.5) * (virtual_h as f32 - 1.0),
                depth: view_z,
                view: panned,
            })
        })
        .collect();

    let mut canvas: Vec<Option<[u8; 3]>> = vec![None; virtual_w as usize * virtual_h as usize];
    if wireframe {
        let line_color = color_to_rgb(fg);
        let mut drawn_edges = 0usize;
        for (a, b) in &mesh.edges {
            if drawn_edges > 12_000 {
                break;
            }
            let Some(pa) = projected.get(*a).and_then(|p| *p) else {
                continue;
            };
            let Some(pb) = projected.get(*b).and_then(|p| *p) else {
                continue;
            };
            let x0 = pa.x.round() as i32;
            let y0 = pa.y.round() as i32;
            let x1 = pb.x.round() as i32;
            let y1 = pb.y.round() as i32;
            if let Some((cx0, cy0, cx1, cy1)) = clip_line_to_viewport(x0, y0, x1, y1, viewport) {
                draw_line_color(&mut canvas, virtual_w, virtual_h, cx0, cy0, cx1, cy1, line_color);
                drawn_edges += 1;
            }
        }
    } else {
        let mut depth = vec![f32::INFINITY; canvas.len()];
        let mut drawn_faces = 0usize;
        // Sort faces back-to-front for correct painter's-algorithm blending when
        // depth-buffering alone isn't enough (avoids most z-fighting glitches).
        let mut sorted_faces: Vec<&ObjFace> = mesh.faces.iter().collect();
        sorted_faces.sort_unstable_by(|a, b| {
            let za = face_avg_depth(&projected, a);
            let zb = face_avg_depth(&projected, b);
            zb.partial_cmp(&za).unwrap_or(std::cmp::Ordering::Equal)
        });

        for face in &sorted_faces {
            if drawn_faces > 20_000 {
                break;
            }
            let Some(v0) = projected.get(face.indices[0]).and_then(|p| *p) else {
                continue;
            };
            let Some(v1) = projected.get(face.indices[1]).and_then(|p| *p) else {
                continue;
            };
            let Some(v2) = projected.get(face.indices[2]).and_then(|p| *p) else {
                continue;
            };
            // Back-face culling: skip faces whose screen-space winding is CW
            // (negative signed area means back-facing in standard left-handed screen coords).
            let signed_area = (v1.x - v0.x) * (v2.y - v0.y) - (v2.x - v0.x) * (v1.y - v0.y);
            if signed_area >= 0.0 {
                continue;
            }
            let shading = face_shading_with_specular(v0.view, v1.view, v2.view);
            let shaded_color = apply_shading(face.color, shading);
            rasterize_triangle(
                &mut canvas,
                &mut depth,
                virtual_w,
                virtual_h,
                v0,
                v1,
                v2,
                shaded_color,
            );
            drawn_faces += 1;
        }

        // Fallback if model has no valid faces/materials.
        if drawn_faces == 0 {
            let line_color = color_to_rgb(fg);
            for (a, b) in &mesh.edges {
                let Some(pa) = projected.get(*a).and_then(|p| *p) else {
                    continue;
                };
                let Some(pb) = projected.get(*b).and_then(|p| *p) else {
                    continue;
                };
                let x0 = pa.x.round() as i32;
                let y0 = pa.y.round() as i32;
                let x1 = pb.x.round() as i32;
                let y1 = pb.y.round() as i32;
                if let Some((cx0, cy0, cx1, cy1)) = clip_line_to_viewport(x0, y0, x1, y1, viewport) {
                    draw_line_color(&mut canvas, virtual_w, virtual_h, cx0, cy0, cx1, cy1, line_color);
                }
            }
        }
    }

    blit_color_canvas(
        buf, mode, &canvas, virtual_w, virtual_h, target_w, target_h, x, y, wireframe, draw_char,
        fg, bg,
    );
}

#[derive(Clone, Copy)]
struct Viewport {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

fn virtual_dimensions(mode: SceneRenderedMode, target_w: u16, target_h: u16) -> (u16, u16) {
    match mode {
        SceneRenderedMode::Cell => (target_w, target_h),
        SceneRenderedMode::HalfBlock => (target_w, target_h.saturating_mul(2)),
        SceneRenderedMode::QuadBlock => (target_w.saturating_mul(2), target_h.saturating_mul(2)),
        SceneRenderedMode::Braille => (target_w.saturating_mul(2), target_h.saturating_mul(4)),
    }
}

fn draw_line_color(
    canvas: &mut [Option<[u8; 3]>],
    w: u16,
    h: u16,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: [u8; 3],
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as u16) < w && (y0 as u16) < h {
            let idx = y0 as usize * w as usize + x0 as usize;
            if let Some(px) = canvas.get_mut(idx) {
                *px = Some(color);
            }
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

#[allow(clippy::too_many_arguments)]
fn rasterize_triangle(
    canvas: &mut [Option<[u8; 3]>],
    depth: &mut [f32],
    w: u16,
    h: u16,
    v0: ProjectedVertex,
    v1: ProjectedVertex,
    v2: ProjectedVertex,
    color: [u8; 3],
) {
    let area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
    if area.abs() < 1e-5 {
        return;
    }

    let min_x = v0.x.min(v1.x).min(v2.x).floor().max(0.0) as i32;
    let max_x = v0.x.max(v1.x).max(v2.x).ceil().min((w - 1) as f32) as i32;
    let min_y = v0.y.min(v1.y).min(v2.y).floor().max(0.0) as i32;
    let max_y = v0.y.max(v1.y).max(v2.y).ceil().min((h - 1) as f32) as i32;

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let x = px as f32 + 0.5;
            let y = py as f32 + 0.5;
            let w0 = edge(v1.x, v1.y, v2.x, v2.y, x, y) / area;
            let w1 = edge(v2.x, v2.y, v0.x, v0.y, x, y) / area;
            let w2 = edge(v0.x, v0.y, v1.x, v1.y, x, y) / area;
            if w0 < -1e-5 || w1 < -1e-5 || w2 < -1e-5 {
                continue;
            }
            let z = w0 * v0.depth + w1 * v1.depth + w2 * v2.depth;
            let idx = py as usize * w as usize + px as usize;
            if z < depth[idx] {
                depth[idx] = z;
                canvas[idx] = Some(color);
            }
        }
    }
}

fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

fn face_avg_depth(projected: &[Option<ProjectedVertex>], face: &ObjFace) -> f32 {
    let depths: Vec<f32> = face
        .indices
        .iter()
        .filter_map(|&i| projected.get(i).and_then(|p| p.map(|v| v.depth)))
        .collect();
    if depths.is_empty() {
        f32::INFINITY
    } else {
        depths.iter().sum::<f32>() / depths.len() as f32
    }
}

fn face_shading_with_specular(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let normal = normalize3(cross3(e1, e2));
    let light_dir = normalize3([-0.45, 0.70, -0.85]);
    // One-sided lambert (with culling in effect, back faces are skipped).
    let lambert = dot3(normal, light_dir).max(0.0);
    // Simple Blinn-Phong specular: half-vector between light and view (0,0,-1 in view space).
    let view_dir = normalize3([0.0, 0.0, -1.0]);
    let half_dir = normalize3([
        light_dir[0] + view_dir[0],
        light_dir[1] + view_dir[1],
        light_dir[2] + view_dir[2],
    ]);
    let spec = dot3(normal, half_dir).max(0.0).powi(16) * 0.18;
    (0.20 + 0.72 * lambert + spec).clamp(0.0, 1.0)
}

fn apply_shading(rgb: [u8; 3], shade: f32) -> [u8; 3] {
    [
        ((rgb[0] as f32 * shade).round().clamp(0.0, 255.0)) as u8,
        ((rgb[1] as f32 * shade).round().clamp(0.0, 255.0)) as u8,
        ((rgb[2] as f32 * shade).round().clamp(0.0, 255.0)) as u8,
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len <= 1e-6 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[allow(clippy::too_many_arguments)]
fn blit_color_canvas(
    buf: &mut Buffer,
    mode: SceneRenderedMode,
    canvas: &[Option<[u8; 3]>],
    virtual_w: u16,
    virtual_h: u16,
    target_w: u16,
    target_h: u16,
    x: u16,
    y: u16,
    wireframe: bool,
    draw_char: char,
    fg: Color,
    bg: Color,
) {
    let px = |vx: u16, vy: u16| -> Option<[u8; 3]> {
        if vx >= virtual_w || vy >= virtual_h {
            return None;
        }
        canvas
            .get(vy as usize * virtual_w as usize + vx as usize)
            .copied()
            .unwrap_or(None)
    };
    let bg_rgb = color_to_rgb(bg);
    let bg_color = rgb_to_color(bg_rgb);

    match mode {
        SceneRenderedMode::Cell => {
            for oy in 0..target_h {
                for ox in 0..target_w {
                    let Some(rgb) = px(ox, oy) else {
                        continue;
                    };
                    let symbol = if wireframe { draw_char } else { '█' };
                    let fg_out = if wireframe { fg } else { rgb_to_color(rgb) };
                    buf.set(x + ox, y + oy, symbol, fg_out, bg_color);
                }
            }
        }
        SceneRenderedMode::HalfBlock => {
            for oy in 0..target_h {
                for ox in 0..target_w {
                    let top = px(ox, oy * 2);
                    let bottom = px(ox, oy * 2 + 1);
                    let (symbol, fg_out, bg_out) = match (top, bottom) {
                        (None, None) => continue,
                        (Some(t), None) => ('▀', rgb_to_color(t), bg_color),
                        (None, Some(b)) => ('▄', rgb_to_color(b), bg_color),
                        (Some(t), Some(b)) => ('▀', rgb_to_color(t), rgb_to_color(b)),
                    };
                    buf.set(x + ox, y + oy, symbol, fg_out, bg_out);
                }
            }
        }
        SceneRenderedMode::QuadBlock => {
            for oy in 0..target_h {
                for ox in 0..target_w {
                    let mut mask = 0u8;
                    let mut cols = Vec::new();
                    if let Some(c) = px(ox * 2, oy * 2) {
                        mask |= 0b0001;
                        cols.push(c);
                    }
                    if let Some(c) = px(ox * 2 + 1, oy * 2) {
                        mask |= 0b0010;
                        cols.push(c);
                    }
                    if let Some(c) = px(ox * 2, oy * 2 + 1) {
                        mask |= 0b0100;
                        cols.push(c);
                    }
                    if let Some(c) = px(ox * 2 + 1, oy * 2 + 1) {
                        mask |= 0b1000;
                        cols.push(c);
                    }
                    let Some(symbol) = quadrant_char(mask) else {
                        continue;
                    };
                    let fg_out = if cols.is_empty() {
                        fg
                    } else {
                        rgb_to_color(average_rgb(&cols))
                    };
                    buf.set(x + ox, y + oy, symbol, fg_out, bg_color);
                }
            }
        }
        SceneRenderedMode::Braille => {
            for oy in 0..target_h {
                for ox in 0..target_w {
                    let sx = ox * 2;
                    let sy = oy * 4;
                    let mut mask = 0u8;
                    let mut cols = Vec::new();
                    if let Some(c) = px(sx, sy) {
                        mask |= 1 << 0;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx, sy + 1) {
                        mask |= 1 << 1;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx, sy + 2) {
                        mask |= 1 << 2;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx + 1, sy) {
                        mask |= 1 << 3;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx + 1, sy + 1) {
                        mask |= 1 << 4;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx + 1, sy + 2) {
                        mask |= 1 << 5;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx, sy + 3) {
                        mask |= 1 << 6;
                        cols.push(c);
                    }
                    if let Some(c) = px(sx + 1, sy + 3) {
                        mask |= 1 << 7;
                        cols.push(c);
                    }
                    let Some(symbol) = braille_char(mask) else {
                        continue;
                    };
                    let fg_out = if cols.is_empty() {
                        fg
                    } else {
                        rgb_to_color(average_rgb(&cols))
                    };
                    buf.set(x + ox, y + oy, symbol, fg_out, bg_color);
                }
            }
        }
    }
}

fn average_rgb(colours: &[[u8; 3]]) -> [u8; 3] {
    if colours.is_empty() {
        return [255, 255, 255];
    }
    let mut rs = 0u32;
    let mut gs = 0u32;
    let mut bs = 0u32;
    for c in colours {
        rs += c[0] as u32;
        gs += c[1] as u32;
        bs += c[2] as u32;
    }
    let len = colours.len() as u32;
    [(rs / len) as u8, (gs / len) as u8, (bs / len) as u8]
}

fn color_to_rgb(color: Color) -> [u8; 3] {
    match color {
        Color::Rgb { r, g, b } => [r, g, b],
        Color::Black => [0, 0, 0],
        Color::DarkGrey => [80, 80, 80],
        Color::Grey => [160, 160, 160],
        Color::White => [255, 255, 255],
        Color::Red | Color::DarkRed => [220, 64, 64],
        Color::Green | Color::DarkGreen => [64, 220, 64],
        Color::Blue | Color::DarkBlue => [64, 64, 220],
        Color::Yellow | Color::DarkYellow => [220, 220, 64],
        Color::Magenta | Color::DarkMagenta => [220, 64, 220],
        Color::Cyan | Color::DarkCyan => [64, 220, 220],
        _ => [255, 255, 255],
    }
}

fn rgb_to_color(rgb: [u8; 3]) -> Color {
    Color::Rgb {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
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

fn quadrant_char(mask: u8) -> Option<char> {
    match mask {
        0 => None,
        1 => Some('▘'),
        2 => Some('▝'),
        3 => Some('▀'),
        4 => Some('▖'),
        5 => Some('▌'),
        6 => Some('▞'),
        7 => Some('▛'),
        8 => Some('▗'),
        9 => Some('▚'),
        10 => Some('▐'),
        11 => Some('▜'),
        12 => Some('▄'),
        13 => Some('▙'),
        14 => Some('▟'),
        15 => Some('█'),
        _ => None,
    }
}

fn braille_char(mask: u8) -> Option<char> {
    if mask == 0 {
        None
    } else {
        char::from_u32(0x2800 + mask as u32)
    }
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
    let text = std::str::from_utf8(&bytes).ok()?;
    let materials = load_material_palette(text, source, &repo);
    parse_obj_mesh_from_text(text, &materials)
}

fn load_material_palette(
    obj_text: &str,
    obj_source: &str,
    repo: &dyn AssetRepository,
) -> HashMap<String, [u8; 3]> {
    let mut out = HashMap::new();
    for raw in obj_text.lines() {
        let line = raw.trim();
        let Some(rest) = line.strip_prefix("mtllib ") else {
            continue;
        };
        for rel in rest.split_whitespace() {
            let mtl_path = resolve_relative_asset_path(obj_source, rel);
            let Ok(bytes) = repo.read_asset_bytes(&mtl_path) else {
                continue;
            };
            let parsed = parse_mtl_palette(&bytes);
            for (name, color) in parsed {
                out.entry(name).or_insert(color);
            }
        }
    }
    out
}

fn parse_mtl_palette(bytes: &[u8]) -> HashMap<String, [u8; 3]> {
    let text = match std::str::from_utf8(bytes) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };
    let mut out = HashMap::new();
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
            }
            continue;
        }
        let Some(rest) = line.strip_prefix("Kd ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let (Some(rs), Some(gs), Some(bs)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        let (Ok(r), Ok(g), Ok(b)) = (rs.parse::<f32>(), gs.parse::<f32>(), bs.parse::<f32>())
        else {
            continue;
        };
        if let Some(name) = current_name.clone() {
            out.insert(name, [to_u8_color(r), to_u8_color(g), to_u8_color(b)]);
        }
    }
    out
}

fn to_u8_color(value: f32) -> u8 {
    if value <= 1.0 {
        (value.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        value.clamp(0.0, 255.0).round() as u8
    }
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
    materials: &HashMap<String, [u8; 3]>,
) -> Option<ObjMesh> {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut edges: HashSet<(usize, usize)> = HashSet::new();
    let mut faces: Vec<ObjFace> = Vec::new();
    let mut active_color = [220, 220, 220];

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
            if let Some(color) = materials.get(mtl_name) {
                active_color = *color;
            }
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
                        color: active_color,
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

    use super::{parse_mtl_palette, parse_obj_mesh_from_text};

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
Kd 0.5 0.25 0.0
"#;
        let palette = parse_mtl_palette(raw);
        assert_eq!(palette.get("Wall"), Some(&[128, 64, 0]));
    }
}

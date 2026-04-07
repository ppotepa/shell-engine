use engine_core::buffer::Buffer;
use engine_core::color::Color;
use engine_core::scene::SceneRenderedMode;

use super::obj_loader::ObjFace;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProjectedVertex {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) depth: f32,
    pub(crate) view: [f32; 3],
}

#[derive(Clone, Copy)]
pub(crate) struct Viewport {
    pub(crate) min_x: i32,
    pub(crate) min_y: i32,
    pub(crate) max_x: i32,
    pub(crate) max_y: i32,
}

#[inline]
pub fn virtual_dimensions(mode: SceneRenderedMode, target_w: u16, target_h: u16) -> (u16, u16) {
    match mode {
        SceneRenderedMode::Cell => (target_w, target_h),
        SceneRenderedMode::HalfBlock => (target_w, target_h.saturating_mul(2)),
        SceneRenderedMode::QuadBlock => (target_w.saturating_mul(2), target_h.saturating_mul(2)),
        SceneRenderedMode::Braille => (target_w.saturating_mul(2), target_h.saturating_mul(4)),
    }
}

/// Interpolate depths at clipped line endpoints using parametric projection.
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn clipped_depths(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    cx0: i32,
    cy0: i32,
    cx1: i32,
    cy1: i32,
    z0: f32,
    z1: f32,
) -> (f32, f32) {
    let ldx = (x1 - x0) as f32;
    let ldy = (y1 - y0) as f32;
    let len_sq = ldx * ldx + ldy * ldy;
    if len_sq < 1.0 {
        return (z0, z1);
    }
    let t0 = ((cx0 - x0) as f32 * ldx + (cy0 - y0) as f32 * ldy) / len_sq;
    let t1 = ((cx1 - x0) as f32 * ldx + (cy1 - y0) as f32 * ldy) / len_sq;
    (
        z0 + (z1 - z0) * t0.clamp(0.0, 1.0),
        z0 + (z1 - z0) * t1.clamp(0.0, 1.0),
    )
}

/// Simple Bresenham line — flat color, no depth test (fallback for face-less models).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_line_flat(
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

/// Bresenham line with z-buffer and depth-based brightness falloff.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_line_depth(
    canvas: &mut [Option<[u8; 3]>],
    depth_buf: &mut [f32],
    w: u16,
    h: u16,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    base_color: [u8; 3],
    z0: f32,
    z1: f32,
    depth_near: f32,
    depth_far: f32,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let total_steps = dx.max(-dy) as f32;
    let depth_range = depth_far - depth_near;
    let mut step = 0f32;

    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as u16) < w && (y0 as u16) < h {
            let idx = y0 as usize * w as usize + x0 as usize;
            let t = if total_steps > 0.0 {
                step / total_steps
            } else {
                0.0
            };
            let z = z0 + (z1 - z0) * t;
            if z < depth_buf[idx] {
                depth_buf[idx] = z;
                let norm = if depth_range > f32::EPSILON {
                    ((z - depth_near) / depth_range).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                // Brightness: 1.0 at nearest, fades to 0.15 at farthest.
                let brightness = 1.0 - 0.85 * norm;
                let r = (base_color[0] as f32 * brightness) as u8;
                let g = (base_color[1] as f32 * brightness) as u8;
                let b = (base_color[2] as f32 * brightness) as u8;
                canvas[idx] = Some([r, g, b]);
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
        step += 1.0;
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn rasterize_triangle(
    canvas: &mut [Option<[u8; 3]>],
    depth: &mut [f32],
    w: u16,
    h: u16,
    v0: ProjectedVertex,
    v1: ProjectedVertex,
    v2: ProjectedVertex,
    color: [u8; 3],
    clip_min_y: i32,
    clip_max_y: i32,
) {
    let area = edge(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
    if area.abs() < 1e-5 {
        return;
    }
    let inv_area = 1.0 / area;

    let min_x = v0.x.min(v1.x).min(v2.x).floor().max(0.0) as i32;
    let max_x = v0.x.max(v1.x).max(v2.x).ceil().min((w - 1) as f32) as i32;
    let min_y = v0.y.min(v1.y).min(v2.y).floor().max(0.0) as i32;
    let max_y = v0.y.max(v1.y).max(v2.y).ceil().min((h - 1) as f32) as i32;
    let min_y = min_y.max(clip_min_y);
    let max_y = max_y.min(clip_max_y);

    // Bounding box culling: skip if triangle is completely off-screen.
    if min_x > max_x || min_y > max_y {
        return;
    }
    for py in min_y..=max_y {
        let y = py as f32 + 0.5;
        let row_start = py as usize * w as usize;
        for px in min_x..=max_x {
            let x = px as f32 + 0.5;
            let w0 = edge(v1.x, v1.y, v2.x, v2.y, x, y) * inv_area;
            let w1 = edge(v2.x, v2.y, v0.x, v0.y, x, y) * inv_area;
            let w2 = edge(v0.x, v0.y, v1.x, v1.y, x, y) * inv_area;
            if w0 < -1e-5 || w1 < -1e-5 || w2 < -1e-5 {
                continue;
            }
            let z = w0 * v0.depth + w1 * v1.depth + w2 * v2.depth;
            let idx = row_start + px as usize;
            if z < depth[idx] {
                depth[idx] = z;
                canvas[idx] = Some(color);
            }
        }
    }
}

#[inline]
pub(crate) fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

#[inline(always)]
pub(crate) fn face_avg_depth(projected: &[Option<ProjectedVertex>], face: &ObjFace) -> f32 {
    let mut sum = 0.0f32;
    let mut count = 0u32;
    for &i in &face.indices {
        if let Some(Some(v)) = projected.get(i) {
            sum += v.depth;
            count += 1;
        }
    }
    if count == 0 {
        f32::INFINITY
    } else {
        sum / count as f32
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn face_shading_with_specular(
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    ka: [f32; 3],
    ks: f32,
    ns: f32,
    light_dir: [f32; 3],
    light_2_dir: [f32; 3],
    half_dir_1: [f32; 3],
    half_dir_2: [f32; 3],
    light_2_intensity: f32,
    light_point: [f32; 3],
    light_point_intensity: f32,
    light_point_2: [f32; 3],
    light_point_2_intensity: f32,
    cel_levels: u8,
    tone_mix: f32,
) -> (f32, f32, f32, f32) {
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let normal = normalize3(cross3(e1, e2));
    // light_dir and light_2_dir arrive pre-normalized from the caller.
    let light_2_strength = light_2_intensity.clamp(0.0, 2.0);
    let point_strength = light_point_intensity.clamp(0.0, 4.0);
    let point_2_strength = light_point_2_intensity.clamp(0.0, 4.0);
    let centroid = [
        (v0[0] + v1[0] + v2[0]) / 3.0,
        (v0[1] + v1[1] + v2[1]) / 3.0,
        (v0[2] + v1[2] + v2[2]) / 3.0,
    ];
    let to_point = sub3(light_point, centroid);
    let point_dir = normalize3(to_point);
    let point_dist =
        (to_point[0] * to_point[0] + to_point[1] * to_point[1] + to_point[2] * to_point[2])
            .sqrt()
            .max(0.0001);
    let point_atten = 1.0 / (1.0 + 0.7 * point_dist * point_dist);
    let to_point_2 = sub3(light_point_2, centroid);
    let point_2_dir = normalize3(to_point_2);
    let point_2_dist = (to_point_2[0] * to_point_2[0]
        + to_point_2[1] * to_point_2[1]
        + to_point_2[2] * to_point_2[2])
        .sqrt()
        .max(0.0001);
    let point_2_atten = 1.0 / (1.0 + 0.7 * point_2_dist * point_2_dist);
    // Two-sided Lambert: abs() keeps shading stable for OBJ files with inconsistent winding.
    let lambert_1 = dot3(normal, light_dir).abs();
    let lambert_2 = dot3(normal, light_2_dir).abs() * light_2_strength;
    let lambert_point = dot3(normal, point_dir).abs() * point_strength * point_atten;
    let lambert_point_2 = dot3(normal, point_2_dir).abs() * point_2_strength * point_2_atten;
    let lambert = (lambert_1 + lambert_2 + lambert_point + lambert_point_2).clamp(0.0, 1.0);
    // When tone_mix is high we intentionally reduce material influence so different OBJ
    // material packs still produce consistent silhouette lighting.
    let material_influence = (1.0 - tone_mix.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let ka_lum_material = (ka[0] * 0.299 + ka[1] * 0.587 + ka[2] * 0.114).clamp(0.03, 0.25);
    let ka_lum = 0.06 + (ka_lum_material - 0.06) * material_influence;
    // half_dir_1 and half_dir_2 are pre-computed by the caller (constant per render).
    // point half-vectors depend on per-face centroid, so they remain here.
    // VIEW_DIR = [0, 0, -1]; add directly without allocating a constant.
    let half_dir_point = normalize3([point_dir[0], point_dir[1], point_dir[2] - 1.0]);
    let half_dir_point_2 = normalize3([point_2_dir[0], point_2_dir[1], point_2_dir[2] - 1.0]);
    let shininess = 24.0 + (ns.clamp(2.0, 200.0) - 24.0) * material_influence;
    let spec_1 = dot3(normal, half_dir_1).abs().powf(shininess);
    let spec_2 = dot3(normal, half_dir_2).abs().powf(shininess) * light_2_strength * 0.7;
    let spec_point =
        dot3(normal, half_dir_point).abs().powf(shininess) * point_strength * point_atten * 0.9;
    let spec_point_2 = dot3(normal, half_dir_point_2).abs().powf(shininess)
        * point_2_strength
        * point_2_atten
        * 0.9;
    let ks_strength = 0.08 + (ks.clamp(0.0, 0.6) - 0.08) * material_influence;
    let spec = (spec_1 + spec_2 + spec_point + spec_point_2) * ks_strength;
    let diffuse = ka_lum + (1.0 - ka_lum) * lambert * 0.9;
    let cel_diffuse = quantize_shade(diffuse, cel_levels);
    (
        (cel_diffuse + spec).clamp(0.0, 1.0),
        cel_diffuse,
        lambert_point.clamp(0.0, 1.0),
        lambert_point_2.clamp(0.0, 1.0),
    )
}

pub(crate) fn quantize_shade(value: f32, levels: u8) -> f32 {
    if levels <= 1 {
        return value.clamp(0.0, 1.0);
    }
    let levels = levels.clamp(2, 8) as f32;
    let steps = levels - 1.0;
    let v = value.clamp(0.0, 1.0);
    (v * steps).round() / steps
}

#[inline(always)]
pub(crate) fn apply_shading(rgb: [u8; 3], shade: f32) -> [u8; 3] {
    // Apply shading in linear space then convert back.
    let lin = [
        srgb_to_linear(rgb[0]),
        srgb_to_linear(rgb[1]),
        srgb_to_linear(rgb[2]),
    ];
    // Boost saturation slightly (1.25) before shading — compensates for terminal display.
    let sat_lin = saturate(lin, 1.25);
    [
        linear_to_srgb((sat_lin[0] * shade).clamp(0.0, 1.0)),
        linear_to_srgb((sat_lin[1] * shade).clamp(0.0, 1.0)),
        linear_to_srgb((sat_lin[2] * shade).clamp(0.0, 1.0)),
    ]
}

#[inline(always)]
pub(crate) fn apply_tone_palette(
    base_rgb: [u8; 3],
    tone: f32,
    shadow: Option<Color>,
    midtone: Option<Color>,
    highlight: Option<Color>,
    tone_mix: f32,
) -> [u8; 3] {
    let mix = tone_mix.clamp(0.0, 1.0);
    if mix <= 0.0 {
        return base_rgb;
    }
    let shadow_rgb = shadow.map(color_to_rgb).unwrap_or([0, 0, 0]);
    let midtone_rgb = midtone
        .map(color_to_rgb)
        .unwrap_or(mix_rgb(shadow_rgb, base_rgb, 0.45));
    let highlight_rgb = highlight.map(color_to_rgb).unwrap_or(base_rgb);
    let t = tone.clamp(0.0, 1.0);
    let toon_rgb = if t <= 0.5 {
        mix_rgb(shadow_rgb, midtone_rgb, t * 2.0)
    } else {
        mix_rgb(midtone_rgb, highlight_rgb, (t - 0.5) * 2.0)
    };
    mix_rgb(base_rgb, toon_rgb, mix)
}

#[inline(always)]
pub(crate) fn apply_point_light_tint(
    base_rgb: [u8; 3],
    light_1_colour: Option<Color>,
    light_1_strength: f32,
    light_2_colour: Option<Color>,
    light_2_strength: f32,
) -> [u8; 3] {
    let mut out = base_rgb;
    if let Some(colour) = light_1_colour {
        let tint = color_to_rgb(colour);
        let blend = (light_1_strength * 0.45).clamp(0.0, 0.65);
        out = mix_rgb(out, tint, blend);
    }
    if let Some(colour) = light_2_colour {
        let tint = color_to_rgb(colour);
        let blend = (light_2_strength * 0.45).clamp(0.0, 0.65);
        out = mix_rgb(out, tint, blend);
    }
    out
}

#[inline(always)]
pub(crate) fn flicker_multiplier(elapsed_s: f32, hz: f32, depth: f32, phase: f32) -> f32 {
    let d = depth.clamp(0.0, 1.0);
    if d <= f32::EPSILON {
        return 1.0;
    }
    let rate = hz.clamp(0.1, 40.0);
    let base = ((elapsed_s * std::f32::consts::TAU * rate + phase).sin() * 0.5 + 0.5).powf(1.5);
    let chatter = ((elapsed_s * std::f32::consts::TAU * (rate * 2.31) + phase * 1.7)
        .sin()
        .abs())
    .powf(2.3);
    let pulse = (base * 0.65 + chatter * 0.35).clamp(0.0, 1.0);
    ((1.0 - d) + d * pulse).clamp(0.0, 1.0)
}

#[inline(always)]
pub(crate) fn mix_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t).round() as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t).round() as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t).round() as u8,
    ]
}

/// Convert sRGB u8 → linear f32.
#[inline(always)]
pub(crate) fn srgb_to_linear(c: u8) -> f32 {
    let v = c as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear f32 → sRGB u8.
#[inline(always)]
pub(crate) fn linear_to_srgb(v: f32) -> u8 {
    let s = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Boost saturation of a linear-space RGB triplet by `factor`.
#[inline(always)]
pub(crate) fn saturate(lin: [f32; 3], factor: f32) -> [f32; 3] {
    let lum = lin[0] * 0.299 + lin[1] * 0.587 + lin[2] * 0.114;
    [
        (lum + (lin[0] - lum) * factor).clamp(0.0, 1.0),
        (lum + (lin[1] - lum) * factor).clamp(0.0, 1.0),
        (lum + (lin[2] - lum) * factor).clamp(0.0, 1.0),
    ]
}

#[inline(always)]
pub(crate) fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline(always)]
pub(crate) fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline(always)]
pub(crate) fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline(always)]
pub(crate) fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len <= 1e-6 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[allow(clippy::too_many_arguments)]
pub fn blit_color_canvas(
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
    clip_row_min: usize,
    clip_row_max: usize,
) {
    let px = |vx: u16, vy: u16| -> Option<[u8; 3]> {
        if vx >= virtual_w || vy >= virtual_h {
            return None;
        }
        let vy_usize = vy as usize;
        if vy_usize < clip_row_min || vy_usize >= clip_row_max {
            return None;
        }
        canvas
            .get(vy_usize * virtual_w as usize + vx as usize)
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
                    let fg_out = rgb_to_color(rgb);
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
                    let mut cols: [[u8; 3]; 4] = [[0, 0, 0]; 4];
                    let mut col_count = 0usize;
                    if let Some(c) = px(ox * 2, oy * 2) {
                        mask |= 0b0001;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    if let Some(c) = px(ox * 2 + 1, oy * 2) {
                        mask |= 0b0010;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    if let Some(c) = px(ox * 2, oy * 2 + 1) {
                        mask |= 0b0100;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    if let Some(c) = px(ox * 2 + 1, oy * 2 + 1) {
                        mask |= 0b1000;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    let Some(symbol) = quadrant_char(mask) else {
                        continue;
                    };
                    let fg_out = if col_count == 0 {
                        fg
                    } else {
                        rgb_to_color(average_rgb(&cols[..col_count]))
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
                    let mut cols: [[u8; 3]; 8] = [[0, 0, 0]; 8];
                    let mut col_count = 0usize;
                    if let Some(c) = px(sx, sy) {
                        mask |= 1 << 0;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    if let Some(c) = px(sx, sy + 1) {
                        mask |= 1 << 1;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    if let Some(c) = px(sx, sy + 2) {
                        mask |= 1 << 2;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    if let Some(c) = px(sx + 1, sy) {
                        mask |= 1 << 3;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    if let Some(c) = px(sx + 1, sy + 1) {
                        mask |= 1 << 4;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    if let Some(c) = px(sx + 1, sy + 2) {
                        mask |= 1 << 5;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    if let Some(c) = px(sx, sy + 3) {
                        mask |= 1 << 6;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    if let Some(c) = px(sx + 1, sy + 3) {
                        mask |= 1 << 7;
                        cols[col_count] = c;
                        col_count += 1;
                    }
                    let Some(symbol) = braille_char(mask) else {
                        continue;
                    };
                    let fg_out = if col_count == 0 {
                        fg
                    } else {
                        rgb_to_color(average_rgb(&cols[..col_count]))
                    };
                    buf.set(x + ox, y + oy, symbol, fg_out, bg_color);
                }
            }
        }
    }
}

#[inline]
pub(crate) fn average_rgb(colours: &[[u8; 3]]) -> [u8; 3] {
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

#[inline(always)]
pub(crate) fn color_to_rgb(color: Color) -> [u8; 3] {
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

#[inline]
pub(crate) fn rgb_to_color(rgb: [u8; 3]) -> Color {
    Color::Rgb {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
    }
}

pub(crate) fn clip_line_to_viewport(
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

#[inline]
pub(crate) fn out_code(x: i32, y: i32, vp: Viewport) -> u8 {
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

#[inline]
pub(crate) fn intersect_vertical(x0: i32, y0: i32, x1: i32, y1: i32, x: i32) -> Option<(i32, i32)> {
    let dx = x1 - x0;
    if dx == 0 {
        return None;
    }
    let t = (x - x0) as f32 / dx as f32;
    let y = y0 as f32 + t * (y1 - y0) as f32;
    Some((x, y.round() as i32))
}

#[inline]
pub(crate) fn intersect_horizontal(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    y: i32,
) -> Option<(i32, i32)> {
    let dy = y1 - y0;
    if dy == 0 {
        return None;
    }
    let t = (y - y0) as f32 / dy as f32;
    let x = x0 as f32 + t * (x1 - x0) as f32;
    Some((x.round() as i32, y))
}

#[inline]
pub(crate) fn quadrant_char(mask: u8) -> Option<char> {
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

#[inline]
pub(crate) fn braille_char(mask: u8) -> Option<char> {
    if mask == 0 {
        None
    } else {
        char::from_u32(0x2800 + mask as u32)
    }
}

#[inline(always)]
pub(crate) fn rotate_xyz(v: [f32; 3], pitch: f32, yaw: f32, roll: f32) -> [f32; 3] {
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

#[cfg(test)]
mod tests {
    use super::obj_sprite_dimensions;
    use engine_core::scene::SpriteSizePreset;

    #[test]
    fn obj_size_preset_uses_type_defaults() {
        assert_eq!(
            obj_sprite_dimensions(None, None, Some(SpriteSizePreset::Small)),
            (32, 12)
        );
        assert_eq!(
            obj_sprite_dimensions(None, None, Some(SpriteSizePreset::Medium)),
            (64, 24)
        );
        assert_eq!(
            obj_sprite_dimensions(None, None, Some(SpriteSizePreset::Large)),
            (96, 36)
        );
    }
}

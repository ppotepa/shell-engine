use super::noise::{fbm_3d_full, value_noise_3d};
use crate::ObjRenderParams;

#[derive(Debug, Clone, Copy)]
pub struct CraterParams {
    pub density: f32,
    pub rim_height: f32,
}

/// Evaluate terrain noise at a model-space sphere position.
/// Returns [0,1]. Used for both vertex displacement and surface coloring so they are always in sync.
#[inline]
pub fn compute_terrain_noise_at(centered: [f32; 3], params: &ObjRenderParams) -> f32 {
    let seed = params.noise_seed;
    let scale = params.terrain_noise_scale;
    let lac = params.noise_lacunarity;
    let per = params.noise_persistence;
    let sx = centered[0] * scale + seed;
    let sy = centered[1] * scale + seed * 1.7;
    let sz = centered[2] * scale + seed * 0.3;
    let (wx, wy, wz) = if params.warp_strength > 0.0 {
        let ws = scale * 0.6;
        let w0 = fbm_3d_full(
            centered[0] * ws + seed + 5.1,
            centered[1] * ws + seed * 1.7 + 2.3,
            centered[2] * ws + seed * 0.3 + 1.1,
            params.warp_octaves.max(1),
            lac,
            per,
        ) - 0.5;
        let w1 = fbm_3d_full(
            centered[0] * ws + seed + 1.9,
            centered[1] * ws + seed * 1.7 + 4.7,
            centered[2] * ws + seed * 0.3 + 3.3,
            params.warp_octaves.max(1),
            lac,
            per,
        ) - 0.5;
        (
            sx + w0 * params.warp_strength,
            sy + w1 * params.warp_strength,
            sz + (w0 * 0.5 + w1 * 0.5) * params.warp_strength,
        )
    } else {
        (sx, sy, sz)
    };
    let fbm_val = fbm_3d_full(wx, wy, wz, params.terrain_noise_octaves, lac, per);
    if params.heightmap_blend > 0.0 {
        if let Some(hmap) = &params.heightmap {
            let hv = sample_heightmap(
                hmap,
                params.heightmap_w,
                params.heightmap_h,
                centered[0],
                centered[1],
                centered[2],
            );
            fbm_val * (1.0 - params.heightmap_blend) + hv * params.heightmap_blend
        } else {
            fbm_val
        }
    } else {
        fbm_val
    }
}

/// Displace a model-space sphere vertex outward along its normal by `noise * strength`.
/// noise in [0,1]; positive values raise terrain, negative sink it.
#[inline]
pub fn displace_sphere_vertex(c: [f32; 3], noise: f32, strength: f32) -> [f32; 3] {
    let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt().max(1e-5);
    let d = noise * strength;
    [
        c[0] + c[0] / len * d,
        c[1] + c[1] / len * d,
        c[2] + c[2] / len * d,
    ]
}

/// Elevation relief modulation for land shading.
#[inline]
pub fn land_elevation_relief(
    base_shade: f32,
    noise: f32,
    terrain_threshold: f32,
    terrain_relief: f32,
) -> f32 {
    if terrain_relief <= 0.0 {
        return base_shade.clamp(0.0, 1.0);
    }
    let elev = (noise - terrain_threshold) / (1.0 - terrain_threshold).max(0.01);
    (base_shade + (elev - 0.5) * terrain_relief).clamp(0.0, 1.0)
}

/// Normal-perturbation shading term based on local noise gradient projected to sphere tangent.
pub fn normal_perturb_shade(
    base_shade: f32,
    local_pos: [f32; 3],
    sphere_pos: [f32; 3],
    sun_dir: [f32; 3],
    noise_scale: f32,
    perturb_strength: f32,
) -> f32 {
    if perturb_strength <= 0.0 || noise_scale <= 0.0 {
        return base_shade.clamp(0.0, 1.0);
    }
    let eps = 0.04 / noise_scale.max(0.1);
    let s = noise_scale;
    let n0 = value_noise_3d(local_pos[0] * s, local_pos[1] * s, local_pos[2] * s);
    let nx = value_noise_3d((local_pos[0] + eps) * s, local_pos[1] * s, local_pos[2] * s) - n0;
    let ny = value_noise_3d(local_pos[0] * s, (local_pos[1] + eps) * s, local_pos[2] * s) - n0;
    let nz = value_noise_3d(local_pos[0] * s, local_pos[1] * s, (local_pos[2] + eps) * s) - n0;

    // Project gradient tangent to sphere (remove radial component).
    let rn = normalize3(sphere_pos);
    let rdot = nx * rn[0] + ny * rn[1] + nz * rn[2];
    let tx = nx - rdot * rn[0];
    let ty = ny - rdot * rn[1];
    let tz = nz - rdot * rn[2];

    // Perturb shade using tangent gradient dot sun.
    let g_sun = tx * sun_dir[0] + ty * sun_dir[1] + tz * sun_dir[2];
    (base_shade + g_sun * perturb_strength * 1.5).clamp(0.0, 1.0)
}

/// Smooth snow-line blend mask from normalized elevation [0,1].
#[inline]
pub fn snow_line_mask(snow_line: f32, elevation: f32) -> f32 {
    smoothstep(snow_line, (snow_line + 0.2).min(1.0), elevation)
}

/// Marble-like ocean shading modulation.
#[inline]
pub fn ocean_marble_shade(base_shade: f32, noise: f32, marble_depth: f32) -> f32 {
    (base_shade + (noise - 0.5) * marble_depth).clamp(0.0, 1.0)
}

/// Ocean shade from local-space noise sampling and marble modulation.
#[inline]
pub fn ocean_shade_from_local(
    base_shade: f32,
    local_pos: [f32; 3],
    noise_scale: f32,
    marble_depth: f32,
) -> f32 {
    let ns = noise_scale.max(0.01);
    let noise = value_noise_3d(local_pos[0] * ns, local_pos[1] * ns, local_pos[2] * ns);
    ocean_marble_shade(base_shade, noise, marble_depth)
}

/// Ocean sunglint additive highlight in [0,1].
pub fn ocean_specular_add(
    normal: [f32; 3],
    sun_dir: [f32; 3],
    view_dir: [f32; 3],
    strength: f32,
    shininess: f32,
) -> f32 {
    if strength <= 0.0 {
        return 0.0;
    }
    let n = normalize3(normal);
    let sun_dot = dot3(n, sun_dir);
    if sun_dot <= 0.0 {
        return 0.0;
    }
    let rx = 2.0 * sun_dot * n[0] - sun_dir[0];
    let ry = 2.0 * sun_dot * n[1] - sun_dir[1];
    let rz = 2.0 * sun_dot * n[2] - sun_dir[2];
    let spec_dot = (rx * view_dir[0] + ry * view_dir[1] + rz * view_dir[2]).max(0.0);
    spec_dot.powf(shininess.max(1.0)) * strength * sun_dot
}

/// Apply Voronoi-style crater rim/bowl modulation to an RGB pixel.
pub fn apply_crater_overlay_rgb(
    pixel: [u8; 3],
    local_pos: [f32; 3],
    params: CraterParams,
) -> [u8; 3] {
    if params.density <= 0.0 {
        return pixel;
    }
    let v_dist = voronoi_3d(
        local_pos[0] * params.density,
        local_pos[1] * params.density,
        local_pos[2] * params.density,
    );
    let rim_w = smoothstep(0.25, 0.42, v_dist) * (1.0 - smoothstep(0.42, 0.58, v_dist));
    let bowl_w = (1.0 - smoothstep(0.0, 0.28, v_dist)) * 0.55;
    if rim_w <= 0.01 && bowl_w <= 0.01 {
        return pixel;
    }
    let (pr, pg, pb) = (pixel[0] as f32, pixel[1] as f32, pixel[2] as f32);
    let rim_boost = rim_w * params.rim_height * 80.0;
    let bowl_dark = bowl_w * 60.0;
    [
        (pr + rim_boost - bowl_dark).clamp(0.0, 255.0) as u8,
        (pg + rim_boost - bowl_dark).clamp(0.0, 255.0) as u8,
        (pb + rim_boost - bowl_dark).clamp(0.0, 255.0) as u8,
    ]
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn terrain_hash(xi: i32, yi: i32, zi: i32) -> f32 {
    let mut h = xi as u32;
    h = h
        .wrapping_mul(374_761_393)
        .wrapping_add((yi as u32).wrapping_mul(668_265_263));
    h ^= (zi as u32).wrapping_mul(2_147_483_647);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) & 0x00FF_FFFF) as f32 / 0x00FF_FFFF as f32
}

#[inline]
fn voronoi_3d(px: f32, py: f32, pz: f32) -> f32 {
    let xi = px.floor() as i32;
    let yi = py.floor() as i32;
    let zi = pz.floor() as i32;
    let mut min_d = f32::INFINITY;
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cx = xi + dx;
                let cy = yi + dy;
                let cz = zi + dz;
                let fx = terrain_hash(cx.wrapping_mul(3), cy.wrapping_mul(5), cz.wrapping_mul(7));
                let fy = terrain_hash(cx.wrapping_mul(7), cy.wrapping_mul(3), cz.wrapping_mul(11));
                let fz = terrain_hash(cx.wrapping_mul(11), cy.wrapping_mul(7), cz.wrapping_mul(3));
                let sx = cx as f32 + fx;
                let sy = cy as f32 + fy;
                let sz = cz as f32 + fz;
                let dx = sx - px;
                let dy = sy - py;
                let dz = sz - pz;
                let d = (dx * dx + dy * dy + dz * dz).sqrt();
                if d < min_d {
                    min_d = d;
                }
            }
        }
    }
    min_d
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Bilinear-sample the tectonic elevation heightmap at a 3D sphere point.
/// `cx/cy/cz` is the pre-rotation local sphere position (unit sphere scale).
/// Grid: row 0 = south pole, row h-1 = north pole. Column wraps (longitude).
fn sample_heightmap(data: &[f32], w: u32, h: u32, cx: f32, cy: f32, cz: f32) -> f32 {
    let len = (cx * cx + cy * cy + cz * cz).sqrt().max(1e-6);
    let yn = cy / len;
    let lat_t = ((yn + 1.0) * 0.5).clamp(0.0, 1.0);
    let lon_t = (cz.atan2(cx) / std::f32::consts::TAU + 0.5).rem_euclid(1.0);
    let w = w as usize;
    let h = h as usize;
    let fx = lon_t * (w as f32 - 1.0);
    let fy = lat_t * (h as f32 - 1.0);
    let ix = fx as usize;
    let iy = fy as usize;
    let tx = fx - ix as f32;
    let ty = fy - iy as f32;
    let ix1 = (ix + 1) % w;
    let iy1 = (iy + 1).min(h - 1);
    let v00 = data[iy * w + ix];
    let v10 = data[iy * w + ix1];
    let v01 = data[iy1 * w + ix];
    let v11 = data[iy1 * w + ix1];
    v00 * (1.0 - tx) * (1.0 - ty) + v10 * tx * (1.0 - ty) + v01 * (1.0 - tx) * ty + v11 * tx * ty
}

use super::ObjRenderParams;
use crate::obj_render_helpers::fbm_3d_full;

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
    v00 * (1.0 - tx) * (1.0 - ty)
        + v10 * tx * (1.0 - ty)
        + v01 * (1.0 - tx) * ty
        + v11 * tx * ty
}

/// Evaluate terrain noise at a model-space sphere position.
/// Returns [0,1]. Used for both vertex displacement and surface coloring so they are always in sync.
#[inline]
pub(super) fn compute_terrain_noise_at(centered: [f32; 3], params: &ObjRenderParams) -> f32 {
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
pub(super) fn displace_sphere_vertex(c: [f32; 3], noise: f32, strength: f32) -> [f32; 3] {
    let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt().max(1e-5);
    let d = noise * strength;
    [c[0] + c[0] / len * d, c[1] + c[1] / len * d, c[2] + c[2] / len * d]
}

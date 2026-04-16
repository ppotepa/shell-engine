use crate::effects::params::PlanetBiomeParams;
use crate::geom::types::ProjectedVertex;

#[derive(Debug, Clone, Copy)]
pub struct AtmosphereParams {
    pub color: [u8; 3],
    pub strength: f32,
    pub rim_power: f32,
    pub haze_strength: f32,
    pub haze_power: f32,
}

/// Apply atmosphere rim + haze to an RGB pixel.
pub fn apply_atmosphere_overlay_rgb(
    pixel: [u8; 3],
    params: AtmosphereParams,
    normal: [f32; 3],
    sun_dir: [f32; 3],
    view_dir: [f32; 3],
) -> [u8; 3] {
    if params.strength <= 0.0 && params.haze_strength <= 0.0 {
        return pixel;
    }
    let n = normalize3(normal);
    let nd = dot3(n, view_dir).abs().clamp(0.0, 1.0);
    let rim = (1.0 - nd).powf(params.rim_power.max(0.1));
    let haze = (1.0 - nd).powf(params.haze_power.max(0.1));
    if rim <= 0.01 && haze <= 0.01 {
        return pixel;
    }
    let day = smoothstep(-0.1, 0.3, dot3(n, sun_dir));
    let rim_alpha = rim * (0.55 + 0.90 * day) * params.strength.max(0.0);
    let haze_alpha = haze * (0.32 + 0.38 * day) * params.haze_strength.max(0.0);
    let a = (rim_alpha + haze_alpha).clamp(0.0, 0.92);
    mix_rgb(pixel, params.color, a)
}

/// Apply atmosphere overlay using barycentric interpolation of per-vertex normals.
#[allow(clippy::too_many_arguments)]
pub fn apply_atmosphere_overlay_barycentric(
    pixel: [u8; 3],
    biome: &PlanetBiomeParams,
    v0: &ProjectedVertex,
    v1: &ProjectedVertex,
    v2: &ProjectedVertex,
    w0: f32,
    w1: f32,
    w2: f32,
) -> [u8; 3] {
    let Some(ac) = biome.atmo_color else {
        return pixel;
    };
    let normal = [
        w0 * v0.normal[0] + w1 * v1.normal[0] + w2 * v2.normal[0],
        w0 * v0.normal[1] + w1 * v1.normal[1] + w2 * v2.normal[1],
        w0 * v0.normal[2] + w1 * v1.normal[2] + w2 * v2.normal[2],
    ];
    apply_atmosphere_overlay_rgb(
        pixel,
        AtmosphereParams {
            color: ac,
            strength: biome.atmo_strength,
            rim_power: biome.atmo_rim_power,
            haze_strength: biome.atmo_haze_strength,
            haze_power: biome.atmo_haze_power,
        },
        normal,
        biome.sun_dir,
        biome.view_dir,
    )
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

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn mix_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
    ]
}

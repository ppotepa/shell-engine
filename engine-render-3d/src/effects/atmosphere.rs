use crate::effects::params::PlanetBiomeParams;
use crate::geom::types::ProjectedVertex;

#[derive(Debug, Clone, Copy)]
pub struct AtmosphereParams {
    pub color: [u8; 3],
    pub strength: f32,
    pub rim_power: f32,
    pub haze_strength: f32,
    pub haze_power: f32,
    pub veil_strength: f32,
    pub veil_power: f32,
}

/// Apply atmosphere rim + haze to an RGB pixel.
pub fn apply_atmosphere_overlay_rgb(
    pixel: [u8; 3],
    params: AtmosphereParams,
    normal: [f32; 3],
    sun_dir: [f32; 3],
    view_dir: [f32; 3],
) -> [u8; 3] {
    if params.strength <= 0.0 && params.haze_strength <= 0.0 && params.veil_strength <= 0.0 {
        return pixel;
    }
    let n = normalize3(normal);
    let nd = dot3(n, view_dir).abs().clamp(0.0, 1.0);
    let rim = (1.0 - nd).powf(params.rim_power.max(0.1));
    let haze = (1.0 - nd).powf(params.haze_power.max(0.1));
    let veil = (1.0 - nd * 0.88).clamp(0.0, 1.0).powf(params.veil_power.max(0.1));
    if rim <= 0.01 && haze <= 0.01 && veil <= 0.01 {
        return pixel;
    }
    let day = smoothstep(-0.1, 0.3, dot3(n, sun_dir));
    let rim_alpha = rim * (0.55 + 0.90 * day) * params.strength.max(0.0);
    let haze_alpha = haze * (0.32 + 0.38 * day) * params.haze_strength.max(0.0);
    let veil_alpha = veil * (0.24 + 0.46 * day) * params.veil_strength.max(0.0);
    let a = (rim_alpha + haze_alpha + veil_alpha).clamp(0.0, 0.92);
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
    let world_pos = [
        w0 * v0.view[0] + w1 * v1.view[0] + w2 * v2.view[0],
        w0 * v0.view[1] + w1 * v1.view[1] + w2 * v2.view[1],
        w0 * v0.view[2] + w1 * v1.view[2] + w2 * v2.view[2],
    ];
    let view_dir = normalize3([
        biome.camera_pos[0] - world_pos[0],
        biome.camera_pos[1] - world_pos[1],
        biome.camera_pos[2] - world_pos[2],
    ]);
    apply_atmosphere_overlay_rgb(
        pixel,
        AtmosphereParams {
            color: ac,
            strength: biome.atmo_strength,
            rim_power: biome.atmo_rim_power,
            haze_strength: biome.atmo_haze_strength,
            haze_power: biome.atmo_haze_power,
            veil_strength: biome.atmo_veil_strength,
            veil_power: biome.atmo_veil_power,
        },
        normal,
        biome.sun_dir,
        view_dir,
    )
}

#[cfg(test)]
mod tests {
    use super::apply_atmosphere_overlay_barycentric;
    use crate::effects::params::PlanetBiomeParams;
    use crate::geom::types::ProjectedVertex;

    #[test]
    fn atmosphere_uses_camera_position_per_pixel() {
        let biome = PlanetBiomeParams {
            polar_ice_color: None,
            polar_ice_start: 0.0,
            polar_ice_end: 0.0,
            desert_color: None,
            desert_strength: 0.0,
            atmo_color: Some([124, 200, 255]),
            atmo_strength: 0.8,
            atmo_rim_power: 2.5,
            atmo_haze_strength: 0.4,
            atmo_haze_power: 1.4,
            atmo_veil_strength: 0.25,
            atmo_veil_power: 1.6,
            night_light_color: None,
            night_light_threshold: 0.0,
            night_light_intensity: 0.0,
            sun_dir: [0.0, 0.0, -1.0],
            view_dir: [0.0, 0.0, -1.0],
            camera_pos: [0.0, 0.0, -3.0],
        };
        let v0 = ProjectedVertex {
            x: 0.0,
            y: 0.0,
            depth: 1.0,
            view: [1.0, 0.0, 0.0],
            normal: [1.0, 0.0, 0.0],
            local: [1.0, 0.0, 0.0],
            terrain_noise: 0.0,
        };
        let v1 = v0;
        let v2 = v0;

        let pixel = apply_atmosphere_overlay_barycentric([10, 20, 30], &biome, &v0, &v1, &v2, 1.0, 0.0, 0.0);
        assert_ne!(pixel, [10, 20, 30]);
    }
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

use crate::effects::params::PlanetBiomeParams;
use crate::geom::types::ProjectedVertex;

#[derive(Debug, Clone, Copy)]
pub struct AtmosphereParams {
    pub height: f32,
    pub density: f32,
    pub rayleigh_amount: f32,
    pub rayleigh_color: [u8; 3],
    pub rayleigh_falloff: f32,
    pub haze_amount: f32,
    pub haze_color: [u8; 3],
    pub haze_falloff: f32,
    pub absorption_amount: f32,
    pub absorption_color: [u8; 3],
    pub absorption_height: f32,
    pub absorption_width: f32,
    pub forward_scatter: f32,
    pub limb_boost: f32,
    pub terminator_softness: f32,
    pub night_glow: f32,
    pub night_glow_color: [u8; 3],
}

/// Apply atmosphere rim + haze to an RGB pixel.
pub fn apply_atmosphere_overlay_rgb(
    pixel: [u8; 3],
    params: AtmosphereParams,
    normal: [f32; 3],
    sun_dir: [f32; 3],
    view_dir: [f32; 3],
) -> [u8; 3] {
    let profile_active = params.rayleigh_amount > 0.0
        || params.haze_amount > 0.0
        || params.absorption_amount > 0.0
        || params.night_glow > 0.0;
    if !profile_active {
        return pixel;
    }
    let n = normalize3(normal);
    let nd = dot3(n, view_dir).abs().clamp(0.0, 1.0);
    let edge = 1.0 - nd;
    let rayleigh_exp = (1.0 / (params.rayleigh_falloff.max(0.01) + 0.12)).clamp(0.6, 8.0);
    let haze_exp = (1.0 / (params.haze_falloff.max(0.01) + 0.18)).clamp(0.35, 6.0);
    let rayleigh_limb = edge.powf(rayleigh_exp);
    let haze_limb = edge.powf(haze_exp);
    let profile_height_boost = (0.45 + params.height.clamp(0.0, 1.0) * 1.55).clamp(0.25, 2.0);
    let density = params.density.clamp(0.0, 1.0).powf(0.72);
    let soft = params.terminator_softness.max(0.05);
    let day = smoothstep(-0.12 * soft, 0.28 * soft, dot3(n, sun_dir));
    let night = (1.0 - day).clamp(0.0, 1.0);
    let forward = smoothstep(0.15, 1.0, dot3(view_dir, sun_dir)).powf(2.0) * params.forward_scatter.clamp(0.0, 1.0);
    let absorption_profile = gaussian(
        edge,
        params.absorption_height.clamp(0.0, 1.0),
        params.absorption_width.max(0.01),
    );
    if rayleigh_limb <= 0.01
        && haze_limb <= 0.01
        && absorption_profile <= 0.01
    {
        return pixel;
    }

    let ray_alpha = (density
        * params.rayleigh_amount.clamp(0.0, 1.0)
        * rayleigh_limb
        * (0.40 + 1.30 * day + 0.65 * forward)
        * profile_height_boost
        * params.limb_boost.max(0.0))
        .clamp(0.0, 0.98);
    let haze_alpha2 = (density
        * params.haze_amount.clamp(0.0, 1.0)
        * haze_limb
        * (0.24 + 0.85 * day + 0.55 * forward)
        * profile_height_boost)
        .clamp(0.0, 0.96);
    let absorb_alpha = (density * params.absorption_amount.clamp(0.0, 1.0) * absorption_profile)
        .clamp(0.0, 0.88);
    let night_glow_alpha = (params.night_glow.clamp(0.0, 1.0)
        * density
        * (0.20 + 0.80 * edge)
        * night)
        .clamp(0.0, 0.55);

    let mut out = pixel;
    if ray_alpha > 0.0 {
        out = mix_rgb(out, params.rayleigh_color, ray_alpha);
    }
    if haze_alpha2 > 0.0 {
        out = mix_rgb(out, params.haze_color, haze_alpha2);
    }
    if absorb_alpha > 0.0 {
        out = mix_rgb(out, params.absorption_color, absorb_alpha);
    }
    if night_glow_alpha > 0.0 {
        out = mix_rgb(out, params.night_glow_color, night_glow_alpha);
    }
    out
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
    if biome.atmo_rayleigh_amount <= 0.0
        && biome.atmo_haze_amount <= 0.0
        && biome.atmo_absorption_amount <= 0.0
        && biome.atmo_night_glow <= 0.0
    {
        return pixel;
    }
    let ac = biome.atmo_color.unwrap_or([124, 200, 255]);
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
            height: biome.atmo_height,
            density: biome.atmo_density,
            rayleigh_amount: biome.atmo_rayleigh_amount,
            rayleigh_color: biome.atmo_rayleigh_color.unwrap_or(ac),
            rayleigh_falloff: biome.atmo_rayleigh_falloff,
            haze_amount: biome.atmo_haze_amount,
            haze_color: biome.atmo_haze_color.unwrap_or(ac),
            haze_falloff: biome.atmo_haze_falloff,
            absorption_amount: biome.atmo_absorption_amount,
            absorption_color: biome.atmo_absorption_color.unwrap_or([255, 170, 110]),
            absorption_height: biome.atmo_absorption_height,
            absorption_width: biome.atmo_absorption_width,
            forward_scatter: biome.atmo_forward_scatter,
            limb_boost: biome.atmo_limb_boost,
            terminator_softness: biome.atmo_terminator_softness,
            night_glow: biome.atmo_night_glow,
            night_glow_color: biome.atmo_night_glow_color.unwrap_or([90, 130, 255]),
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
            atmo_height: 0.12,
            atmo_density: 0.8,
            atmo_strength: 0.8,
            atmo_rayleigh_amount: 0.8,
            atmo_rayleigh_color: Some([124, 200, 255]),
            atmo_rayleigh_falloff: 0.32,
            atmo_haze_amount: 0.25,
            atmo_haze_color: Some([180, 210, 245]),
            atmo_haze_falloff: 0.18,
            atmo_absorption_amount: 0.0,
            atmo_absorption_color: None,
            atmo_absorption_height: 0.55,
            atmo_absorption_width: 0.18,
            atmo_forward_scatter: 0.72,
            atmo_limb_boost: 1.0,
            atmo_terminator_softness: 1.0,
            atmo_night_glow: 0.0,
            atmo_night_glow_color: None,
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

#[inline]
fn gaussian(x: f32, center: f32, width: f32) -> f32 {
    let w = width.max(0.001);
    let z = (x - center) / w;
    (-0.5 * z * z).exp()
}

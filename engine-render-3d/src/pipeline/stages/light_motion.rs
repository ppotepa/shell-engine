use crate::shading::flicker_multiplier;
use crate::ObjRenderParams;

#[derive(Debug, Clone, Copy)]
pub(crate) struct AnimatedPointLights {
    pub point_1_x: f32,
    pub point_1_z: f32,
    pub point_2_x: f32,
    pub point_2_z: f32,
    pub point_1_flicker: f32,
    pub point_2_flicker: f32,
}

#[inline]
fn snap_angle(elapsed_s: f32, snap_hz: f32, seed: u32) -> f32 {
    let snap_index = (elapsed_s * snap_hz) as u32;
    let h = snap_index.wrapping_mul(2654435761u32).wrapping_add(seed);
    (h as f32 / u32::MAX as f32) * std::f32::consts::TAU
}

pub(crate) fn animate_point_lights(params: &ObjRenderParams) -> AnimatedPointLights {
    let elapsed_s = params.scene_elapsed_ms as f32 / 1000.0;
    let point_1_flicker = flicker_multiplier(
        elapsed_s,
        params.light_point_flicker_hz,
        params.light_point_flicker_depth,
        0.37,
    );
    let point_2_flicker = flicker_multiplier(
        elapsed_s,
        params.light_point_2_flicker_hz,
        params.light_point_2_flicker_depth,
        1.91,
    );

    let orbit_radius_1 = (params.light_point_x.powi(2) + params.light_point_z.powi(2))
        .sqrt()
        .max(0.0001);
    let orbit_radius_2 = (params.light_point_2_x.powi(2) + params.light_point_2_z.powi(2))
        .sqrt()
        .max(0.0001);

    let (point_1_x, point_1_z) = if params.light_point_snap_hz > f32::EPSILON {
        let angle = snap_angle(elapsed_s, params.light_point_snap_hz, 0x9e37_79b9);
        (orbit_radius_1 * angle.sin(), orbit_radius_1 * angle.cos())
    } else if params.light_point_orbit_hz > f32::EPSILON {
        let angle = elapsed_s * params.light_point_orbit_hz * std::f32::consts::TAU;
        (orbit_radius_1 * angle.sin(), orbit_radius_1 * angle.cos())
    } else {
        (params.light_point_x, params.light_point_z)
    };

    let (point_2_x, point_2_z) = if params.light_point_2_snap_hz > f32::EPSILON {
        let angle = snap_angle(elapsed_s, params.light_point_2_snap_hz, 0x6c62_272d);
        (orbit_radius_2 * angle.sin(), orbit_radius_2 * angle.cos())
    } else if params.light_point_2_orbit_hz > f32::EPSILON {
        let angle = elapsed_s * params.light_point_2_orbit_hz * std::f32::consts::TAU;
        (orbit_radius_2 * angle.sin(), orbit_radius_2 * angle.cos())
    } else {
        (params.light_point_2_x, params.light_point_2_z)
    };

    AnimatedPointLights {
        point_1_x,
        point_1_z,
        point_2_x,
        point_2_z,
        point_1_flicker,
        point_2_flicker,
    }
}

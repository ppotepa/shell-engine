use std::collections::HashMap;

use engine_3d::scene3d_format::{CameraDef, TweenDef};
use engine_core::scene_runtime_types::SceneCamera3D;

use super::look_at_basis;

#[derive(Debug, Clone)]
pub struct CameraFrameState {
    pub camera_position: [f32; 3],
    pub camera_distance: f32,
    pub view_right: [f32; 3],
    pub view_up: [f32; 3],
    pub view_forward: [f32; 3],
}

pub type TweenValues = HashMap<String, HashMap<String, f32>>;

pub fn evaluate_tween_values(tweens: &[TweenDef], t: f32) -> TweenValues {
    let mut tween_values: TweenValues = HashMap::new();
    for tw in tweens {
        let et = tw.easing.apply(t);
        let value = tw.from + (tw.to - tw.from) * et;
        tween_values
            .entry(tw.object.clone())
            .or_default()
            .insert(tw.property.clone(), value);
    }
    tween_values
}

pub fn resolve_camera_frame_state(
    camera: &CameraDef,
    camera_override: Option<&SceneCamera3D>,
    tween_values: &TweenValues,
) -> CameraFrameState {
    let base_cam_pos = camera_override
        .map(|camera| camera.eye)
        .unwrap_or_else(|| camera.position.unwrap_or([0.0, 0.0, camera.distance]));
    let look_at = camera_override
        .map(|camera| camera.look_at)
        .or(camera.look_at);
    let effective_cam_pos =
        if let (Some(cam_tw), Some(look_at)) = (tween_values.get("camera"), look_at) {
            if let Some(&orbit_angle_deg) = cam_tw.get("orbit_angle_deg") {
                let dx = base_cam_pos[0] - look_at[0];
                let dy = base_cam_pos[1] - look_at[1];
                let dz = base_cam_pos[2] - look_at[2];
                let horiz_r = (dx * dx + dz * dz).sqrt();
                let elevation = dy.atan2(horiz_r);
                let base_phase = dz.atan2(dx);
                let theta = base_phase + orbit_angle_deg.to_radians();
                let total_r = (dx * dx + dy * dy + dz * dz).sqrt();
                [
                    look_at[0] + total_r * elevation.cos() * theta.cos(),
                    look_at[1] + total_r * elevation.sin(),
                    look_at[2] + total_r * elevation.cos() * theta.sin(),
                ]
            } else {
                base_cam_pos
            }
        } else {
            base_cam_pos
        };

    let camera_distance = if camera_override.is_some() {
        let look_at = look_at.unwrap_or([0.0, 0.0, 0.0]);
        ((effective_cam_pos[0] - look_at[0]).powi(2)
            + (effective_cam_pos[1] - look_at[1]).powi(2)
            + (effective_cam_pos[2] - look_at[2]).powi(2))
        .sqrt()
        .max(0.001)
    } else {
        (effective_cam_pos[0].powi(2) + effective_cam_pos[1].powi(2) + effective_cam_pos[2].powi(2))
            .sqrt()
            .max(camera.distance.abs())
    };

    let (view_right, view_up, view_forward) = if let Some(look_at) = look_at {
        let up = camera_override
            .map(|camera| camera.up)
            .unwrap_or([0.0, 1.0, 0.0]);
        look_at_basis(effective_cam_pos, look_at, up)
    } else {
        ([1.0f32, 0.0, 0.0], [0.0f32, 1.0, 0.0], [0.0f32, 0.0, 1.0])
    };

    CameraFrameState {
        camera_position: effective_cam_pos,
        camera_distance,
        view_right,
        view_up,
        view_forward,
    }
}

#[cfg(test)]
mod tests {
    use engine_3d::scene3d_format::{CameraDef, Easing, TweenDef};

    use super::{evaluate_tween_values, resolve_camera_frame_state};

    #[test]
    fn camera_orbit_tween_keeps_distance() {
        let camera = CameraDef {
            position: Some([0.0, 0.0, 5.0]),
            look_at: Some([0.0, 0.0, 0.0]),
            distance: 5.0,
            fov_degrees: 60.0,
            near_clip: 0.1,
        };
        let tweens = vec![TweenDef {
            object: "camera".to_string(),
            property: "orbit_angle_deg".to_string(),
            from: 0.0,
            to: 90.0,
            easing: Easing::Linear,
        }];
        let tween_values = evaluate_tween_values(&tweens, 1.0);

        let state = resolve_camera_frame_state(&camera, None, &tween_values);
        let distance = (state.camera_position[0].powi(2)
            + state.camera_position[1].powi(2)
            + state.camera_position[2].powi(2))
        .sqrt();

        assert!((distance - 5.0).abs() < 0.001);
    }
}

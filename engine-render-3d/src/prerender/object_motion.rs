use std::collections::HashMap;

use engine_3d::scene3d_format::ObjectDef;

#[derive(Debug, Clone, Copy)]
pub struct ObjectFrameMotion {
    pub translation_x: f32,
    pub translation_y: f32,
    pub translation_z: f32,
    pub yaw_offset: f32,
    pub clip_y_min: f32,
    pub clip_y_max: f32,
}

pub fn resolve_object_frame_motion(
    obj: &ObjectDef,
    obj_tweens: Option<&HashMap<String, f32>>,
    clip_orbit_origin: Option<[f32; 3]>,
) -> ObjectFrameMotion {
    let yaw_offset = obj_tweens
        .and_then(|m| m.get("yaw_offset"))
        .copied()
        .unwrap_or(0.0);
    let clip_y_min = obj_tweens
        .and_then(|m| m.get("clip_y_min"))
        .copied()
        .unwrap_or(0.0);
    let clip_y_max = obj_tweens
        .and_then(|m| m.get("clip_y_max"))
        .copied()
        .unwrap_or(1.0);
    let base_translation = obj.transform.translation.unwrap_or([0.0, 0.0, 0.0]);
    let translation_x = obj_tweens
        .and_then(|m| m.get("translation_x"))
        .copied()
        .unwrap_or(base_translation[0]);
    let translation_y = obj_tweens
        .and_then(|m| m.get("translation_y"))
        .copied()
        .unwrap_or(base_translation[1]);
    let translation_z = obj_tweens
        .and_then(|m| m.get("translation_z"))
        .copied()
        .unwrap_or(base_translation[2]);
    let orbit_angle_deg = obj_tweens.and_then(|m| m.get("orbit_angle_deg")).copied();

    let (translation_x, translation_y, translation_z) = if let Some(orbit_angle_deg) = orbit_angle_deg {
        let origin = clip_orbit_origin.unwrap_or([0.0, 0.0, 0.0]);
        let orbit_center_x = obj_tweens
            .and_then(|m| m.get("orbit_center_x"))
            .copied()
            .unwrap_or(origin[0]);
        let orbit_center_y = obj_tweens
            .and_then(|m| m.get("orbit_center_y"))
            .copied()
            .unwrap_or(origin[1]);
        let orbit_center_z = obj_tweens
            .and_then(|m| m.get("orbit_center_z"))
            .copied()
            .unwrap_or(origin[2]);

        let dx0 = translation_x - orbit_center_x;
        let dz0 = translation_z - orbit_center_z;
        let derived_radius = (dx0 * dx0 + dz0 * dz0).sqrt();
        let orbit_radius = obj_tweens
            .and_then(|m| m.get("orbit_radius"))
            .copied()
            .unwrap_or(derived_radius);
        let orbit_phase_deg = obj_tweens
            .and_then(|m| m.get("orbit_phase_deg"))
            .copied()
            .unwrap_or_else(|| dz0.atan2(dx0).to_degrees());

        let theta = (orbit_phase_deg + orbit_angle_deg).to_radians();
        (
            orbit_center_x + orbit_radius * theta.cos(),
            translation_y + (orbit_center_y - origin[1]),
            orbit_center_z + orbit_radius * theta.sin(),
        )
    } else {
        (translation_x, translation_y, translation_z)
    };

    ObjectFrameMotion {
        translation_x,
        translation_y,
        translation_z,
        yaw_offset,
        clip_y_min,
        clip_y_max,
    }
}

#[cfg(test)]
mod tests {
    use engine_3d::scene3d_format::{ObjectDef, TransformDef};

    use super::resolve_object_frame_motion;

    #[test]
    fn object_orbit_updates_position_from_angle() {
        let obj = ObjectDef {
            id: "planet".to_string(),
            mesh: "planet.obj".to_string(),
            material: "m".to_string(),
            transform: TransformDef {
                translation: Some([2.0, 0.0, 0.0]),
                ..Default::default()
            },
        };
        let mut tweens = std::collections::HashMap::new();
        tweens.insert("orbit_angle_deg".to_string(), 90.0);
        tweens.insert("orbit_center_x".to_string(), 0.0);
        tweens.insert("orbit_center_z".to_string(), 0.0);

        let motion = resolve_object_frame_motion(&obj, Some(&tweens), Some([0.0, 0.0, 0.0]));
        assert!(motion.translation_x.abs() < 0.001);
        assert!((motion.translation_z - 2.0).abs() < 0.001);
    }
}

use engine_game::components::Transform3D;
use engine_game::GameplayWorld;

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn quat_normalize(q: [f32; 4]) -> [f32; 4] {
    let len_sq = q.iter().map(|v| v * v).sum::<f32>();
    if len_sq <= f32::EPSILON {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let inv = len_sq.sqrt().recip();
    [q[0] * inv, q[1] * inv, q[2] * inv, q[3] * inv]
}

fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

fn quat_from_axis_angle(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let mag = vec3_len(axis);
    if mag <= f32::EPSILON || angle.abs() <= f32::EPSILON {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let half = angle * 0.5;
    let s = half.sin() / mag;
    [axis[0] * s, axis[1] * s, axis[2] * s, half.cos()]
}

fn quat_rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let qn = quat_normalize(q);
    let conj = [-qn[0], -qn[1], -qn[2], qn[3]];
    let vec_q = [v[0], v[1], v[2], 0.0];
    let rotated = quat_mul(quat_mul(qn, vec_q), conj);
    [rotated[0], rotated[1], rotated[2]]
}

pub fn physics_3d_system(gameplay_world: &GameplayWorld, dt_ms: u64) {
    if dt_ms == 0 {
        return;
    }
    let dt_sec = dt_ms as f32 / 1000.0;
    let work_items = gameplay_world.batch_read_all_physics3d();
    if work_items.is_empty() {
        return;
    }

    let mut results = Vec::with_capacity(work_items.len());
    for (id, mut xf, mut body) in work_items {
        body.linear_velocity =
            vec3_add(body.linear_velocity, vec3_scale(body.linear_accel, dt_sec));

        if body.linear_drag > 0.0 {
            let damp = 1.0 - body.linear_drag.clamp(0.0, 1.0) * dt_sec;
            body.linear_velocity = vec3_scale(body.linear_velocity, damp.max(0.0));
        }

        if body.max_linear_speed > 0.0 {
            let speed = vec3_len(body.linear_velocity);
            if speed > body.max_linear_speed {
                body.linear_velocity = vec3_scale(
                    body.linear_velocity,
                    body.max_linear_speed / speed.max(f32::EPSILON),
                );
            }
        }

        xf.position = vec3_add(xf.position, vec3_scale(body.linear_velocity, dt_sec));

        body.angular_velocity = vec3_add(
            body.angular_velocity,
            vec3_scale(body.angular_accel, dt_sec),
        );

        if body.angular_drag > 0.0 {
            let damp = 1.0 - body.angular_drag.clamp(0.0, 1.0) * dt_sec;
            body.angular_velocity = vec3_scale(body.angular_velocity, damp.max(0.0));
        }

        if body.max_angular_speed > 0.0 {
            let speed = vec3_len(body.angular_velocity);
            if speed > body.max_angular_speed {
                body.angular_velocity = vec3_scale(
                    body.angular_velocity,
                    body.max_angular_speed / speed.max(f32::EPSILON),
                );
            }
        }

        let angular_speed = vec3_len(body.angular_velocity);
        if angular_speed > f32::EPSILON {
            let delta_q = quat_from_axis_angle(body.angular_velocity, angular_speed * dt_sec);
            xf.orientation = quat_normalize(quat_mul(delta_q, xf.orientation));
        }

        results.push((id, xf, body));
    }

    gameplay_world.batch_write_physics3d(&results);
}

pub fn apply_wrap_3d_system(gameplay_world: &GameplayWorld) {
    for id in gameplay_world.ids_with_transform3d() {
        let Some(bounds) = gameplay_world.wrap_bounds_for(id) else {
            continue;
        };
        let Some(mut xf) = gameplay_world.transform3d(id) else {
            continue;
        };
        let next = [
            bounds.wrap_x(xf.position[0]),
            bounds.wrap_y(xf.position[1]),
            bounds.wrap_z(xf.position[2]),
        ];
        if next != xf.position {
            xf.position = next;
            let _ = gameplay_world.set_transform3d(id, xf);
        }
    }
}

pub fn apply_follow_anchors_3d_system(gameplay_world: &GameplayWorld) {
    for id in gameplay_world.ids_with_follow_anchor3d() {
        let Some(follow) = gameplay_world.follow_anchor3d(id) else {
            continue;
        };
        let Some(ownership) = gameplay_world.ownership(id) else {
            continue;
        };
        let Some(owner_xf) = gameplay_world.transform3d(ownership.owner_id) else {
            continue;
        };
        let current_orientation = gameplay_world
            .transform3d(id)
            .map(|xf| xf.orientation)
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);
        let offset = if follow.inherit_orientation {
            quat_rotate(owner_xf.orientation, follow.local_offset)
        } else {
            follow.local_offset
        };
        let xf = Transform3D {
            position: vec3_add(owner_xf.position, offset),
            orientation: if follow.inherit_orientation {
                owner_xf.orientation
            } else {
                current_orientation
            },
        };
        let _ = gameplay_world.set_transform3d(id, xf);
    }
}

#[cfg(test)]
mod tests {
    use super::physics_3d_system;
    use engine_game::components::{PhysicsBody3D, Transform3D};
    use engine_game::GameplayWorld;
    use serde_json::json;

    #[test]
    fn physics_3d_integrates_linear_and_angular_motion() {
        let gameplay = GameplayWorld::new();
        let id = gameplay.spawn("probe", json!({})).expect("spawn");
        assert!(gameplay.set_transform3d(id, Transform3D::default()));
        assert!(gameplay.set_physics3d(
            id,
            PhysicsBody3D {
                linear_accel: [2.0, 0.0, 0.0],
                angular_velocity: [0.0, 0.0, 1.0],
                ..PhysicsBody3D::default()
            }
        ));

        physics_3d_system(&gameplay, 1000);

        let xf = gameplay.transform3d(id).expect("transform");
        let body = gameplay.physics3d(id).expect("physics");
        assert!(xf.position[0] > 1.9, "expected linear integration on x");
        assert!(
            body.linear_velocity[0] > 1.9,
            "expected accumulated linear velocity on x"
        );
        assert_ne!(
            xf.orientation,
            Transform3D::default().orientation,
            "expected orientation to change from angular velocity"
        );
    }
}

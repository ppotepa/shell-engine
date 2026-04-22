use engine_game::components::{AngularMotorMode, CharacterUpMode, MotorSpace, Transform3D};
use engine_game::GameplayWorld;
use std::collections::BTreeSet;

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(v);
    if len <= f32::EPSILON {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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

fn quat_conjugate(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

fn quat_rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let qn = quat_normalize(q);
    let vec_q = [v[0], v[1], v[2], 0.0];
    let rotated = quat_mul(quat_mul(qn, vec_q), quat_conjugate(qn));
    [rotated[0], rotated[1], rotated[2]]
}

fn basis_from_orientation(q: [f32; 4]) -> ([f32; 3], [f32; 3], [f32; 3]) {
    (
        quat_rotate(q, [1.0, 0.0, 0.0]),
        quat_rotate(q, [0.0, 1.0, 0.0]),
        quat_rotate(q, [0.0, 0.0, 1.0]),
    )
}

fn combine_basis(right: [f32; 3], up: [f32; 3], forward: [f32; 3], local: [f32; 3]) -> [f32; 3] {
    [
        right[0] * local[0] + up[0] * local[1] + forward[0] * local[2],
        right[1] * local[0] + up[1] * local[1] + forward[1] * local[2],
        right[2] * local[0] + up[2] * local[1] + forward[2] * local[2],
    ]
}

fn up_for_character(
    up_mode: CharacterUpMode,
    frame_up: [f32; 3],
    surface_normal: [f32; 3],
) -> [f32; 3] {
    match up_mode {
        CharacterUpMode::WorldUp => [0.0, 1.0, 0.0],
        CharacterUpMode::SurfaceNormal => vec3_normalize(surface_normal),
        CharacterUpMode::ReferenceFrameUp => vec3_normalize(frame_up),
    }
}

fn reference_basis(
    gameplay_world: &GameplayWorld,
    id: u64,
    xf: Transform3D,
) -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3]) {
    if let Some(frame) = gameplay_world.reference_frame_state3d(id) {
        return (
            frame.basis_right,
            frame.basis_up,
            frame.basis_forward,
            frame.surface_normal,
        );
    }
    let (right, up, forward) = basis_from_orientation(xf.orientation);
    (right, up, forward, up)
}

pub fn motor_3d_system(gameplay_world: &GameplayWorld, _dt_ms: u64) {
    let mut ids = BTreeSet::new();
    ids.extend(gameplay_world.ids_with_linear_motor3d());
    ids.extend(gameplay_world.ids_with_angular_motor3d());
    ids.extend(gameplay_world.ids_with_character_motor3d());
    ids.extend(gameplay_world.ids_with_flight_motor3d());

    for id in ids {
        let Some(mut body) = gameplay_world.physics3d(id) else {
            continue;
        };
        let Some(xf) = gameplay_world.transform3d(id) else {
            continue;
        };
        let intent = gameplay_world.control_intent3d(id).unwrap_or_default();
        let flight_motor = gameplay_world.flight_motor3d(id);
        let angular_motor = gameplay_world.angular_motor3d(id);
        let linear_motor = gameplay_world.linear_motor3d(id);
        let character_motor = gameplay_world.character_motor3d(id);

        let (local_right, local_up, local_forward) = basis_from_orientation(xf.orientation);
        let (frame_right, frame_up, frame_forward, surface_normal) =
            reference_basis(gameplay_world, id, xf);

        // Motors are authoritative for commanded acceleration each frame. Start
        // from a clean command state so stale thrust/torque does not leak across
        // frames when input drops or a different motor mode takes over.
        body.linear_accel = [0.0, 0.0, 0.0];
        body.angular_accel = [0.0, 0.0, 0.0];

        if let Some(motor) = linear_motor {
            let mut move_local = intent.move_local;
            move_local[2] += intent.throttle;

            if let Some(flight) = flight_motor {
                for (axis, enabled) in flight.translational_dofs.iter().enumerate() {
                    if !enabled {
                        move_local[axis] = 0.0;
                    }
                }
            }

            let input_mag = vec3_len(move_local).min(1.0);
            let move_dir = if input_mag > 0.0 {
                vec3_scale(vec3_normalize(move_local), input_mag)
            } else {
                [0.0, 0.0, 0.0]
            };
            let basis_vec = match motor.space {
                MotorSpace::World => move_dir,
                MotorSpace::Local => combine_basis(local_right, local_up, local_forward, move_dir),
                MotorSpace::ReferenceFrame => {
                    combine_basis(frame_right, frame_up, frame_forward, move_dir)
                }
            };

            let mut linear_accel = if input_mag > 0.0 {
                let boost = if intent.boost {
                    motor.boost_scale.max(1.0)
                } else {
                    1.0
                };
                vec3_scale(
                    vec3_normalize(basis_vec),
                    motor.accel.max(0.0) * input_mag * boost,
                )
            } else {
                [0.0, 0.0, 0.0]
            };

            if intent.brake {
                let speed = vec3_len(body.linear_velocity);
                if speed > f32::EPSILON {
                    linear_accel = vec3_add(
                        linear_accel,
                        vec3_scale(
                            vec3_normalize(body.linear_velocity),
                            -motor.decel.max(motor.accel).max(0.0),
                        ),
                    );
                }
            }

            body.linear_accel = linear_accel;
            if motor.max_speed > 0.0 {
                body.max_linear_speed = if body.max_linear_speed > 0.0 {
                    body.max_linear_speed.min(motor.max_speed)
                } else {
                    motor.max_speed
                };
            }
        }

        if let Some(character) = character_motor {
            if intent.jump && character.jump_speed > 0.0 {
                let jump_up = up_for_character(character.up_mode, frame_up, surface_normal);
                body.linear_velocity = vec3_add(
                    body.linear_velocity,
                    vec3_scale(vec3_normalize(jump_up), character.jump_speed),
                );
            }
        }

        if let Some(motor) = angular_motor {
            let look = [
                intent.look_local[0] * motor.look_sensitivity,
                intent.look_local[1] * motor.look_sensitivity,
                intent.look_local[2] * motor.look_sensitivity,
            ];
            let yaw_axis = vec3_normalize(frame_up);
            let pitch_axis = vec3_normalize(local_right);
            let roll_axis = vec3_normalize(local_forward);

            let mut yaw_cmd = look[0] * motor.yaw_rate;
            let mut pitch_cmd = look[1] * motor.pitch_rate;
            let mut roll_cmd = look[2] * motor.roll_rate;

            if let Some(flight) = flight_motor {
                if !flight.rotational_dofs[0] {
                    pitch_cmd = 0.0;
                }
                if !flight.rotational_dofs[1] {
                    yaw_cmd = 0.0;
                }
                if !flight.rotational_dofs[2] {
                    roll_cmd = 0.0;
                }
            }

            let commanded = vec3_add(
                vec3_add(
                    vec3_scale(yaw_axis, yaw_cmd),
                    vec3_scale(pitch_axis, pitch_cmd),
                ),
                vec3_scale(roll_axis, roll_cmd),
            );

            match motor.mode {
                AngularMotorMode::Rate => {
                    body.angular_velocity = commanded;
                }
                AngularMotorMode::Torque => {
                    body.angular_accel = vec3_scale(commanded, motor.torque_scale.max(0.0));
                }
            }
        }

        if let Some(flight) = flight_motor {
            if flight.horizon_lock_strength > 0.0 {
                let current_up = quat_rotate(xf.orientation, [0.0, 1.0, 0.0]);
                let correction_axis = vec3_cross(current_up, vec3_normalize(frame_up));
                body.angular_accel = vec3_add(
                    body.angular_accel,
                    vec3_scale(correction_axis, flight.horizon_lock_strength.max(0.0)),
                );
            }
        }

        let _ = gameplay_world.set_physics3d(id, body);
    }
}

#[cfg(test)]
mod tests {
    use super::motor_3d_system;
    use engine_game::components::{AngularMotor3D, LinearMotor3D, PhysicsBody3D, Transform3D};
    use engine_game::GameplayWorld;
    use serde_json::json;

    #[test]
    fn motor_3d_clears_stale_command_acceleration_without_input() {
        let gameplay = GameplayWorld::new();
        let id = gameplay.spawn("probe", json!({})).expect("spawn");
        assert!(gameplay.set_transform3d(id, Transform3D::default()));
        assert!(gameplay.set_physics3d(
            id,
            PhysicsBody3D {
                linear_accel: [3.0, 1.0, -2.0],
                angular_accel: [0.5, -0.25, 0.75],
                ..PhysicsBody3D::default()
            }
        ));
        assert!(gameplay.attach_linear_motor3d(
            id,
            LinearMotor3D {
                accel: 10.0,
                ..LinearMotor3D::default()
            }
        ));
        assert!(gameplay.attach_angular_motor3d(id, AngularMotor3D::default()));

        motor_3d_system(&gameplay, 16);

        let body = gameplay.physics3d(id).expect("physics");
        assert_eq!(body.linear_accel, [0.0, 0.0, 0.0]);
        assert_eq!(body.angular_accel, [0.0, 0.0, 0.0]);
    }
}

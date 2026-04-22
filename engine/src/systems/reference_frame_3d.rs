use engine_behavior::catalog::ModCatalogs;
use engine_celestial::WorldPoint3;
use engine_game::components::{ReferenceFrameMode, ReferenceFrameState3D, Transform3D};
use engine_game::GameplayWorld;

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

fn world_point_from_transform(xf: Transform3D) -> WorldPoint3 {
    WorldPoint3 {
        x: xf.position[0] as f64,
        y: xf.position[1] as f64,
        z: xf.position[2] as f64,
    }
}

pub fn reference_frame_3d_system(
    gameplay_world: &GameplayWorld,
    catalogs: Option<&ModCatalogs>,
    world: &engine_core::world::World,
) {
    for id in gameplay_world.ids_with_reference_frame3d() {
        let Some(binding) = gameplay_world.reference_frame3d(id) else {
            continue;
        };

        let mut state = ReferenceFrameState3D::default();

        match binding.mode {
            ReferenceFrameMode::World => {}
            ReferenceFrameMode::ParentEntity => {
                let Some(parent_id) = binding.entity_id else {
                    let _ = gameplay_world.set_reference_frame_state3d(id, state);
                    continue;
                };
                let Some(parent_xf) = gameplay_world.transform3d(parent_id) else {
                    let _ = gameplay_world.set_reference_frame_state3d(id, state);
                    continue;
                };
                let (right, up, forward) = basis_from_orientation(parent_xf.orientation);
                state.origin = parent_xf.position;
                state.basis_right = right;
                state.basis_up = up;
                state.basis_forward = forward;
                state.surface_normal = up;
                if let Some(parent_body) = gameplay_world.physics3d(parent_id) {
                    if binding.inherit_linear_velocity {
                        state.carrier_linear_velocity = parent_body.linear_velocity;
                    }
                    if binding.inherit_angular_velocity {
                        state.carrier_angular_velocity = parent_body.angular_velocity;
                    }
                }
            }
            ReferenceFrameMode::CelestialBody
            | ReferenceFrameMode::LocalHorizon
            | ReferenceFrameMode::Orbital => {
                let Some(body_id) = binding.body_id.as_deref() else {
                    let _ = gameplay_world.set_reference_frame_state3d(id, state);
                    continue;
                };
                let Some(catalogs) = catalogs else {
                    let _ = gameplay_world.set_reference_frame_state3d(id, state);
                    continue;
                };
                let Some(xf) = gameplay_world.transform3d(id) else {
                    let _ = gameplay_world.set_reference_frame_state3d(id, state);
                    continue;
                };
                let point = world_point_from_transform(xf);
                let Some(frame) =
                    super::celestial_runtime::local_frame(catalogs, Some(body_id), point, world)
                else {
                    let _ = gameplay_world.set_reference_frame_state3d(id, state);
                    continue;
                };
                let gravity =
                    super::celestial_runtime::gravity_sample(catalogs, Some(body_id), point, world);
                let pose = super::celestial_runtime::body_pose(catalogs, Some(body_id), world);

                state.origin = [
                    frame.origin.x as f32,
                    frame.origin.y as f32,
                    frame.origin.z as f32,
                ];
                state.basis_right = [
                    frame.east.x as f32,
                    frame.east.y as f32,
                    frame.east.z as f32,
                ];
                state.basis_up = [frame.up.x as f32, frame.up.y as f32, frame.up.z as f32];
                state.basis_forward = [
                    frame.tangent_forward.x as f32,
                    frame.tangent_forward.y as f32,
                    frame.tangent_forward.z as f32,
                ];
                state.surface_normal = state.basis_up;
                if let Some(sample) = gravity {
                    state.altitude_km = sample.altitude_km as f32;
                }
                if let Some(body_pose) = pose {
                    state.origin = [
                        body_pose.center.x as f32,
                        body_pose.center.y as f32,
                        body_pose.center.z as f32,
                    ];
                }
            }
        }

        let _ = gameplay_world.set_reference_frame_state3d(id, state);
    }
}

#[cfg(test)]
mod tests {
    use super::reference_frame_3d_system;
    use engine_behavior::catalog::{BodyDef, ModCatalogs};
    use engine_core::world::World;
    use engine_game::components::{ReferenceFrameBinding3D, ReferenceFrameMode, Transform3D};
    use engine_game::GameplayWorld;
    use serde_json::json;

    #[test]
    fn local_horizon_frame_uses_celestial_surface_normal() {
        let world = World::default();
        let gameplay = GameplayWorld::new();
        let mut catalogs = ModCatalogs::default();
        catalogs.celestial.bodies.insert(
            "planet".into(),
            BodyDef {
                center_x: 0.0,
                center_y: 0.0,
                surface_radius: 90.0,
                gravity_mu: 1000.0,
                ..BodyDef::default()
            },
        );

        let id = gameplay.spawn("pilot", json!({})).expect("spawn");
        assert!(gameplay.set_transform3d(
            id,
            Transform3D {
                position: [0.0, 100.0, 0.0],
                ..Transform3D::default()
            }
        ));
        assert!(gameplay.attach_reference_frame3d(
            id,
            ReferenceFrameBinding3D {
                mode: ReferenceFrameMode::LocalHorizon,
                body_id: Some("planet".into()),
                ..ReferenceFrameBinding3D::default()
            }
        ));

        reference_frame_3d_system(&gameplay, Some(&catalogs), &world);

        let state = gameplay
            .reference_frame_state3d(id)
            .expect("reference frame state");
        assert!(
            state.basis_up[1] > 0.99,
            "expected up vector from planet normal"
        );
        assert!(state.altitude_km >= 0.0, "expected non-negative altitude");
    }
}

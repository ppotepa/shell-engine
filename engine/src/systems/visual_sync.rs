//! Visual-sync system — pushes gameplay transforms into scene object properties
//! so that Rhai scripts do not need to manually call `scene.set(...)` every frame.

use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_api::scene::{Render3dMutationRequest, SceneMutationRequest};
use engine_behavior::BehaviorCommand;
use engine_core::game_object::GameObjectKind;
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

fn heading_from_orientation(q: [f32; 4]) -> f32 {
    let sprite_forward = quat_rotate(q, [0.0, -1.0, 0.0]);
    sprite_forward[0].atan2(-sprite_forward[1])
}

fn quat_to_euler_deg_xyz(q: [f32; 4]) -> [f32; 3] {
    let q = quat_normalize(q);
    let x = q[0];
    let y = q[1];
    let z = q[2];
    let w = q[3];

    let sinr_cosp = 2.0 * (w * x + y * z);
    let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
    let roll_x = sinr_cosp.atan2(cosr_cosp);

    let sinp = 2.0 * (w * y - z * x);
    let pitch_y = if sinp.abs() >= 1.0 {
        sinp.signum() * (std::f32::consts::PI * 0.5)
    } else {
        sinp.asin()
    };

    let siny_cosp = 2.0 * (w * z + x * y);
    let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
    let yaw_z = siny_cosp.atan2(cosy_cosp);

    [
        roll_x.to_degrees(),
        pitch_y.to_degrees(),
        yaw_z.to_degrees(),
    ]
}

fn is_typed_3d_visual(runtime: &engine_scene_runtime::SceneRuntime, visual_id: &str) -> bool {
    let resolver = runtime.target_resolver();
    let Some(object_id) = resolver.resolve_alias(visual_id) else {
        return false;
    };
    let Some(object) = runtime.object(object_id) else {
        return false;
    };
    matches!(object.kind, GameObjectKind::ObjSprite)
}

fn apply_2d_visual_sync(
    runtime: &mut engine_scene_runtime::SceneRuntime,
    sync_data: &[(String, f32, f32, f32, f32)],
) {
    // Transitional visual sync hook: the current gameplay store still emits
    // `Transform2D { x, y, z, heading }`, and scene-runtime already supports
    // position.z on the same fast path. Keep the seam explicit so the upcoming
    // `Transform3D` store can plug in here without reshaping the render sync
    // call sites.
    runtime.apply_particle_visual_sync(sync_data);
}

fn apply_3d_visual_sync(
    runtime: &mut engine_scene_runtime::SceneRuntime,
    sync_data: &[(String, [f32; 3], [f32; 4])],
) {
    if sync_data.is_empty() {
        return;
    }

    let resolver = runtime.target_resolver();
    let commands: Vec<BehaviorCommand> = sync_data
        .iter()
        .map(
            |(visual_id, position, orientation)| BehaviorCommand::ApplySceneMutation {
                request: SceneMutationRequest::SetRender3d(
                    Render3dMutationRequest::SetNodeTransform {
                        target: visual_id.clone(),
                        translation: Some(*position),
                        rotation_deg: Some(quat_to_euler_deg_xyz(*orientation)),
                        scale: None,
                    },
                ),
            },
        )
        .collect();
    runtime.apply_behavior_commands(&resolver, &commands);
}

fn push_legacy_visual_sync(
    sync_data: &mut Vec<(String, f32, f32, f32, f32)>,
    visual_ids: &[&str],
    x: f32,
    y: f32,
    z: f32,
    heading: f32,
) {
    for visual_id in visual_ids {
        sync_data.push(((*visual_id).to_string(), x, y, z, heading));
    }
}

/// Iterates all entities with a `VisualBinding`, preferring `Transform3D`
/// when available and falling back to `Transform2D` for legacy visuals.
pub fn visual_sync_system(world: &mut World) {
    let Some(gameplay_world) = world.get::<GameplayWorld>().cloned() else {
        return;
    };
    let Some(runtime) = world.scene_runtime_mut() else {
        return;
    };
    let mut sync_data_2d = Vec::new();
    let mut sync_data_3d = Vec::new();

    for id in gameplay_world.ids_with_visual_binding() {
        let Some(binding) = gameplay_world.visual(id) else {
            continue;
        };
        let visual_ids = binding.all_visual_ids();
        let Some(xf) = gameplay_world.transform3d(id) else {
            let Some(transform) = gameplay_world.transform(id) else {
                continue;
            };
            push_legacy_visual_sync(
                &mut sync_data_2d,
                &visual_ids,
                transform.x,
                transform.y,
                transform.z,
                transform.heading,
            );
            continue;
        };

        let legacy_heading = heading_from_orientation(xf.orientation);

        for visual_id in visual_ids {
            if is_typed_3d_visual(runtime, visual_id) {
                sync_data_3d.push((visual_id.to_string(), xf.position, xf.orientation));
                continue;
            }

            sync_data_2d.push((
                visual_id.to_string(),
                xf.position[0],
                xf.position[1],
                xf.position[2],
                legacy_heading,
            ));
        }
    }
    if sync_data_2d.is_empty() && sync_data_3d.is_empty() {
        return;
    }
    if !sync_data_2d.is_empty() {
        // Legacy compatibility path. This remains authoritative for authored 2D
        // visuals and for 3D gameplay entities that are still bound to non-3D
        // scene nodes such as text/image/panel sprites.
        apply_2d_visual_sync(runtime, &sync_data_2d);
    }
    if !sync_data_3d.is_empty() {
        // Typed 3D path is now authoritative for gameplay entities bound to
        // render3d-capable visuals. Heading mirroring is intentionally not
        // applied to those nodes anymore.
        apply_3d_visual_sync(runtime, &sync_data_3d);
    }
}

#[cfg(test)]
mod tests {
    use super::visual_sync_system;
    use crate::services::EngineWorldAccess;
    use crate::world::World;
    use engine_api::scene::{Camera3dMutationRequest, SceneMutationRequest};
    use engine_behavior::BehaviorCommand;
    use engine_core::render_types::DirtyMask3D;
    use engine_core::scene::{Scene, Sprite};
    use engine_game::components::{
        FollowAnchor2D, LifecyclePolicy, Transform2D, Transform3D, VisualBinding,
    };
    use engine_game::GameplayWorld;
    use engine_scene_runtime::SceneRuntime;
    use serde_json::json;

    fn find_obj_sprite<'a>(scene: &'a Scene, alias: &str) -> &'a Sprite {
        scene
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find(|sprite| matches!(sprite, Sprite::Obj { id: Some(id), .. } if id == alias))
            .expect("object sprite")
    }

    #[test]
    fn visual_sync_prefers_typed_transform3d_over_legacy_transform2d_for_3d_visuals() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: visual-sync-3d
title: Visual Sync 3D
render-space: 3d
layers:
  - name: world
    sprites:
      - type: obj
        id: ship
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
"#,
        )
        .expect("scene parse");
        let mut world = World::new();
        let gameplay = GameplayWorld::new();
        world.register(gameplay.clone());
        world.register_scoped(SceneRuntime::new(scene));

        let id = gameplay.spawn("ship", json!({})).expect("spawn ship");
        assert!(gameplay.set_visual(
            id,
            VisualBinding {
                visual_id: Some("ship".into()),
                ..VisualBinding::default()
            }
        ));
        assert!(gameplay.set_transform(
            id,
            Transform2D {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                heading: 0.25,
            }
        ));
        assert!(gameplay.set_transform3d(
            id,
            Transform3D {
                position: [12.0, -4.0, 7.0],
                orientation: [0.0, 0.70710677, 0.0, 0.70710677],
            }
        ));

        visual_sync_system(&mut world);

        let runtime = world.scene_runtime_mut().expect("runtime");
        let dirty = runtime.take_render3d_dirty_mask();
        assert!(dirty.contains(DirtyMask3D::TRANSFORM));
        let resolver = runtime.target_resolver();
        let ship_id = resolver.resolve_alias("ship").expect("ship alias");
        let ship_state = runtime.object_state(ship_id).expect("ship state");
        assert_eq!(ship_state.offset_x, 12);
        assert_eq!(ship_state.offset_y, -4);
        assert_eq!(ship_state.offset_z, 0);
        assert_eq!(ship_state.heading, 0.0);
    }

    #[test]
    fn visual_sync_keeps_legacy_mirror_for_3d_entities_bound_to_non_render3d_visuals() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: visual-sync-legacy-bridge
title: Visual Sync Legacy Bridge
render-space: 3d
layers:
  - name: hud
    sprites:
      - type: text
        id: marker
        content: MARKER
"#,
        )
        .expect("scene parse");
        let mut world = World::new();
        let gameplay = GameplayWorld::new();
        world.register(gameplay.clone());
        world.register_scoped(SceneRuntime::new(scene));

        let id = gameplay.spawn("marker", json!({})).expect("spawn marker");
        assert!(gameplay.set_visual(
            id,
            VisualBinding {
                visual_id: Some("marker".into()),
                ..VisualBinding::default()
            }
        ));
        assert!(gameplay.set_transform3d(
            id,
            Transform3D {
                position: [5.0, 6.0, 7.0],
                orientation: [0.0, 0.0, 0.70710677, 0.70710677],
            }
        ));

        visual_sync_system(&mut world);

        let runtime = world.scene_runtime_mut().expect("runtime");
        assert_eq!(runtime.take_render3d_dirty_mask(), DirtyMask3D::empty());
        let resolver = runtime.target_resolver();
        let marker_id = resolver.resolve_alias("marker").expect("marker alias");
        let marker_state = runtime.object_state(marker_id).expect("marker state");
        assert_eq!(marker_state.offset_x, 5);
        assert_eq!(marker_state.offset_y, 6);
        assert_eq!(marker_state.offset_z, 7);
        assert!(
            (marker_state.heading - std::f32::consts::FRAC_PI_2).abs() < 0.0001,
            "expected compatibility heading mirror for non-render3d visual"
        );
    }

    #[test]
    fn visual_sync_updates_additional_legacy_visuals_for_same_entity() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: visual-sync-additional-visuals
title: Visual Sync Additional Visuals
layers:
  - name: hud
    sprites:
      - type: text
        id: primary
        content: PRIMARY
      - type: text
        id: ghost
        content: GHOST
"#,
        )
        .expect("scene parse");
        let mut world = World::new();
        let gameplay = GameplayWorld::new();
        world.register(gameplay.clone());
        world.register_scoped(SceneRuntime::new(scene));

        let id = gameplay.spawn("ghosting", json!({})).expect("spawn entity");
        assert!(gameplay.set_visual(
            id,
            VisualBinding {
                visual_id: Some("primary".into()),
                additional_visuals: vec!["ghost".into()],
            }
        ));
        assert!(gameplay.set_transform(
            id,
            Transform2D {
                x: 9.0,
                y: -3.0,
                z: 2.0,
                heading: 0.75,
            }
        ));

        visual_sync_system(&mut world);

        let runtime = world.scene_runtime_mut().expect("runtime");
        let resolver = runtime.target_resolver();
        for alias in ["primary", "ghost"] {
            let object_id = resolver.resolve_alias(alias).expect("alias");
            let state = runtime.object_state(object_id).expect("state");
            assert_eq!(state.offset_x, 9);
            assert_eq!(state.offset_y, -3);
            assert_eq!(state.offset_z, 2);
            assert!((state.heading - 0.75).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn visual_sync_applies_follow_anchor_motion_before_render_node_sync() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: visual-sync-follow-anchor
title: Visual Sync Follow Anchor
layers:
  - name: hud
    sprites:
      - type: text
        id: follower-node
        content: FOLLOWER
"#,
        )
        .expect("scene parse");
        let mut world = World::new();
        let gameplay = GameplayWorld::new();
        world.register(gameplay.clone());
        world.register_scoped(SceneRuntime::new(scene));

        let owner = gameplay.spawn("owner", json!({})).expect("spawn owner");
        let follower = gameplay
            .spawn("follower", json!({}))
            .expect("spawn follower");
        assert!(gameplay.register_child(owner, follower));
        assert!(gameplay.set_lifecycle(follower, LifecyclePolicy::FollowOwner));
        assert!(gameplay.set_transform(
            owner,
            Transform2D {
                x: 10.0,
                y: 20.0,
                z: 0.0,
                heading: std::f32::consts::FRAC_PI_2,
            }
        ));
        assert!(gameplay.set_transform(
            follower,
            Transform2D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                heading: 0.0,
            }
        ));
        assert!(gameplay.set_follow_anchor(
            follower,
            FollowAnchor2D {
                local_x: -4.0,
                local_y: 2.0,
                inherit_heading: true,
            }
        ));
        assert!(gameplay.set_visual(
            follower,
            VisualBinding {
                visual_id: Some("follower-node".into()),
                ..VisualBinding::default()
            }
        ));

        gameplay.apply_follow_anchors();
        visual_sync_system(&mut world);

        let runtime = world.scene_runtime_mut().expect("runtime");
        let resolver = runtime.target_resolver();
        let node_id = resolver.resolve_alias("follower-node").expect("node alias");
        let state = runtime.object_state(node_id).expect("node state");
        assert_eq!(state.offset_x, 8);
        assert_eq!(state.offset_y, 16);
        assert_eq!(state.heading, std::f32::consts::FRAC_PI_2);
    }

    #[test]
    fn scene_runtime_maps_cockpit_and_chase_object_camera_requests_to_render_nodes() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: visual-sync-camera-nodes
title: Visual Sync Camera Nodes
render-space: 3d
layers:
  - name: world
    sprites:
      - type: obj
        id: cockpit-camera
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
      - type: obj
        id: chase-camera
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
"#,
        )
        .expect("scene parse");
        let mut runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::ApplySceneMutation {
                    request: SceneMutationRequest::SetCamera3d(
                        Camera3dMutationRequest::ObjectLookAt {
                            target: "cockpit-camera".to_string(),
                            eye: [1.0, 2.0, 3.0],
                            look_at: [1.0, 2.0, 4.0],
                            up: Some([0.0, 1.0, 0.0]),
                        },
                    ),
                },
                BehaviorCommand::ApplySceneMutation {
                    request: SceneMutationRequest::SetCamera3d(
                        Camera3dMutationRequest::ObjectBasis {
                            target: "chase-camera".to_string(),
                            eye: [5.0, 6.0, 7.0],
                            right: [1.0, 0.0, 0.0],
                            up: [0.0, 1.0, 0.0],
                            forward: [0.0, 0.0, 1.0],
                        },
                    ),
                },
            ],
        );

        let cockpit = find_obj_sprite(runtime.scene(), "cockpit-camera");
        let chase = find_obj_sprite(runtime.scene(), "chase-camera");

        match cockpit {
            Sprite::Obj {
                cam_world_x,
                cam_world_y,
                cam_world_z,
                view_right_x,
                view_right_y,
                view_right_z,
                view_up_x,
                view_up_y,
                view_up_z,
                view_fwd_x,
                view_fwd_y,
                view_fwd_z,
                ..
            } => {
                assert_eq!(
                    (*cam_world_x, *cam_world_y, *cam_world_z),
                    (Some(1.0), Some(2.0), Some(3.0))
                );
                assert_eq!(
                    (*view_right_x, *view_right_y, *view_right_z),
                    (Some(-1.0), Some(0.0), Some(0.0))
                );
                assert_eq!(
                    (*view_up_x, *view_up_y, *view_up_z),
                    (Some(0.0), Some(1.0), Some(0.0))
                );
                assert_eq!(
                    (*view_fwd_x, *view_fwd_y, *view_fwd_z),
                    (Some(0.0), Some(0.0), Some(1.0))
                );
            }
            _ => unreachable!(),
        }

        match chase {
            Sprite::Obj {
                cam_world_x,
                cam_world_y,
                cam_world_z,
                view_right_x,
                view_right_y,
                view_right_z,
                view_up_x,
                view_up_y,
                view_up_z,
                view_fwd_x,
                view_fwd_y,
                view_fwd_z,
                ..
            } => {
                assert_eq!(
                    (*cam_world_x, *cam_world_y, *cam_world_z),
                    (Some(5.0), Some(6.0), Some(7.0))
                );
                assert_eq!(
                    (*view_right_x, *view_right_y, *view_right_z),
                    (Some(1.0), Some(0.0), Some(0.0))
                );
                assert_eq!(
                    (*view_up_x, *view_up_y, *view_up_z),
                    (Some(0.0), Some(1.0), Some(0.0))
                );
                assert_eq!(
                    (*view_fwd_x, *view_fwd_y, *view_fwd_z),
                    (Some(0.0), Some(0.0), Some(1.0))
                );
            }
            _ => unreachable!(),
        }
    }
}

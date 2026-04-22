use crate::world::World;

pub fn orbit_camera_system(_world: &mut World, _dt_ms: u64) {
    // Legacy wrapper kept for call-site compatibility; free-look owns the director step.
}

#[cfg(test)]
mod tests {
    use super::orbit_camera_system;
    use crate::services::EngineWorldAccess;
    use crate::systems::free_look_camera::free_look_camera_system;
    use crate::world::World;
    use engine_core::scene::Scene;
    use engine_scene_runtime::SceneRuntime;

    #[test]
    fn orbit_and_free_look_wrappers_share_one_camera_director_step() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: orbit-wrapper-compat
title: Orbit Wrapper Compat
input:
  orbit-camera:
    target: helsinki-uni-wireframe
    yaw: 90
    pitch: 20
    distance: 1.6
layers:
  - name: obj
    sprites:
      - type: obj
        id: helsinki-uni-wireframe
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
"#,
        )
        .expect("scene parse");

        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world
            .scene_runtime_mut()
            .expect("scene runtime")
            .begin_camera_director_frame();

        free_look_camera_system(&mut world, 16);
        let after_first = world
            .scene_runtime()
            .expect("scene runtime")
            .scene_camera_3d()
            .eye;

        orbit_camera_system(&mut world, 16);
        let after_second = world
            .scene_runtime()
            .expect("scene runtime")
            .scene_camera_3d()
            .eye;

        assert_eq!(
            after_first, after_second,
            "compat camera wrappers should share one director step per frame"
        );
    }
}

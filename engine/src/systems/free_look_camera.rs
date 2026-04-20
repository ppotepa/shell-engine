use engine_behavior::catalog::ModCatalogs;

use crate::services::EngineWorldAccess;
use crate::world::World;

pub fn free_look_camera_system(world: &mut World, dt_ms: u64) {
    sync_free_look_surface_shell_from_focus_body(world);

    if let Some(runtime) = world.scene_runtime_mut() {
        let _ = runtime.step_free_look_camera(dt_ms);
    }
}

fn sync_free_look_surface_shell_from_focus_body(world: &mut World) {
    let Some(focus_body_id) = world.scene_runtime().and_then(|runtime| {
        runtime
            .free_look_surface_mode_enabled()
            .then(|| runtime.scene().celestial.focus_body.clone())
            .flatten()
    }) else {
        return;
    };

    let Some((center_x, center_y, render_radius)) = world
        .get::<ModCatalogs>()
        .and_then(|catalogs| catalogs.celestial.bodies.get(focus_body_id.as_str()))
        .map(|body| {
            (
                body.center_x as f32,
                body.center_y as f32,
                body.radius_px as f32,
            )
        })
    else {
        return;
    };

    if let Some(runtime) = world.scene_runtime_mut() {
        let _ = runtime.sync_free_look_surface_shell_2d(center_x, center_y, render_radius);
    }
}

#[cfg(test)]
mod tests {
    use super::free_look_camera_system;
    use crate::services::EngineWorldAccess;
    use crate::world::World;
    use engine_behavior::catalog::{BodyDef, ModCatalogs};
    use engine_core::scene::Scene;
    use engine_events::{KeyCode, KeyEvent, KeyModifiers};
    use engine_scene_runtime::SceneRuntime;

    #[test]
    fn free_look_surface_shell_uses_focus_body_render_radius() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: free-look-focus-body
title: Free Look Focus Body
celestial:
  focus-body: generated-planet
input:
  free-look-camera:
    surface-mode: true
    surface-radius: 1.0
    surface-altitude: 0.05
layers: []
"#,
        )
        .expect("scene parse");

        let mut world = World::new();
        let mut catalogs = ModCatalogs::default();
        catalogs.celestial.bodies.insert(
            "generated-planet".into(),
            BodyDef {
                center_x: 5.0,
                center_y: 0.0,
                radius_px: 2.5,
                ..BodyDef::default()
            },
        );

        world.register(catalogs);
        world.register_scoped(SceneRuntime::new(scene));

        {
            let runtime = world.scene_runtime_mut().expect("scene runtime");
            let toggled = runtime.apply_free_look_key_events(
                &[KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)],
                &[],
            );
            assert!(toggled, "expected ctrl+f to engage free-look");
        }

        free_look_camera_system(&mut world, 16);

        let runtime = world.scene_runtime().expect("scene runtime");
        let eye = runtime.scene_camera_3d().eye;
        let dx = eye[0] - 5.0;
        let dy = eye[1];
        let dz = eye[2];
        let shell_radius = (dx * dx + dy * dy + dz * dz).sqrt();
        assert!(
            (shell_radius - 2.55).abs() < 0.03,
            "expected free-look shell radius to follow focus-body render radius, got {shell_radius}"
        );
    }
}

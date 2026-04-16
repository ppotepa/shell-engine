use crate::scene::{
    GeneratedWorldInstance, MeshInstance, Node3DInstance, Renderable3D, SceneClip3DInstance,
};
use engine_core::render_types::Transform3D;
use engine_core::scene::{CameraSource, Sprite};

pub fn map_sprite_to_node3d(sprite: &Sprite) -> Option<Node3DInstance> {
    match sprite {
        Sprite::Obj {
            id,
            source,
            x,
            y,
            scale,
            pitch_deg,
            yaw_deg,
            roll_deg,
            visible,
            ..
        } => Some(Node3DInstance {
            id: id.clone().unwrap_or_else(|| "obj-node".to_string()),
            transform: Transform3D {
                translation: [*x as f32, *y as f32, 0.0],
                rotation_deg: [
                    pitch_deg.unwrap_or(0.0),
                    yaw_deg.unwrap_or(0.0),
                    roll_deg.unwrap_or(0.0),
                ],
                scale: [scale.unwrap_or(1.0), scale.unwrap_or(1.0), scale.unwrap_or(1.0)],
            },
            visible: *visible,
            renderable: Renderable3D::Mesh(MeshInstance {
                source: source.clone(),
                material: None,
            }),
        }),
        Sprite::Planet {
            id,
            body_id,
            preset,
            mesh_source,
            x,
            y,
            scale,
            pitch_deg,
            yaw_deg,
            roll_deg,
            visible,
            ..
        } => Some(Node3DInstance {
            id: id.clone().unwrap_or_else(|| format!("planet-{body_id}")),
            transform: Transform3D {
                translation: [*x as f32, *y as f32, 0.0],
                rotation_deg: [
                    pitch_deg.unwrap_or(0.0),
                    yaw_deg.unwrap_or(0.0),
                    roll_deg.unwrap_or(0.0),
                ],
                scale: [scale.unwrap_or(1.0), scale.unwrap_or(1.0), scale.unwrap_or(1.0)],
            },
            visible: *visible,
            renderable: Renderable3D::GeneratedWorld(GeneratedWorldInstance {
                body_id: body_id.clone(),
                preset_id: preset.clone(),
                mesh_source: mesh_source.clone(),
                params_uri: None,
                material: None,
            }),
        }),
        Sprite::Scene3D {
            id,
            src,
            frame,
            x,
            y,
            camera_source,
            visible,
            ..
        } => Some(Node3DInstance {
            id: id.clone().unwrap_or_else(|| "scene3d-node".to_string()),
            transform: Transform3D {
                translation: [*x as f32, *y as f32, 0.0],
                rotation_deg: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
            visible: *visible,
            renderable: Renderable3D::SceneClip(SceneClip3DInstance {
                source: src.clone(),
                frame: frame.clone(),
                use_scene_camera: *camera_source == CameraSource::Scene,
            }),
        }),
        Sprite::Text { .. }
        | Sprite::Image { .. }
        | Sprite::Vector { .. }
        | Sprite::Panel { .. }
        | Sprite::Grid { .. }
        | Sprite::Flex { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::map_sprite_to_node3d;
    use crate::scene::Renderable3D;
    use engine_core::scene::Sprite;

    #[test]
    fn maps_obj_sprite_to_mesh_node() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: obj
id: test-obj
source: /assets/3d/sphere.obj
scale: 2.0
yaw-deg: 15
"#,
        )
        .expect("obj sprite should parse");

        let node = map_sprite_to_node3d(&sprite).expect("node");
        assert_eq!(node.id, "test-obj");
        match node.renderable {
            Renderable3D::Mesh(mesh) => assert_eq!(mesh.source, "/assets/3d/sphere.obj"),
            _ => panic!("expected mesh renderable"),
        }
    }

    #[test]
    fn maps_planet_sprite_to_generated_world_node() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: planet
id: earth-view
body-id: earth
preset: earth-like
"#,
        )
        .expect("planet sprite should parse");

        let node = map_sprite_to_node3d(&sprite).expect("node");
        assert_eq!(node.id, "earth-view");
        match node.renderable {
            Renderable3D::GeneratedWorld(world) => {
                assert_eq!(world.body_id, "earth");
                assert_eq!(world.preset_id.as_deref(), Some("earth-like"));
            }
            _ => panic!("expected generated world renderable"),
        }
    }

    #[test]
    fn maps_scene3d_sprite_to_scene_clip_node() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: scene3_d
id: clip-view
src: /assets/3d/sample.scene3d.yml
frame: idle
"#,
        )
        .expect("scene3d sprite should parse");

        let node = map_sprite_to_node3d(&sprite).expect("node");
        assert_eq!(node.id, "clip-view");
        match node.renderable {
            Renderable3D::SceneClip(clip) => {
                assert_eq!(clip.source, "/assets/3d/sample.scene3d.yml");
                assert_eq!(clip.frame, "idle");
                assert!(!clip.use_scene_camera);
            }
            _ => panic!("expected scene clip renderable"),
        }
    }
}

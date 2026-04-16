use super::generated_world_sprite_spec::extract_generated_world_sprite_spec;
use super::obj_sprite_spec::extract_obj_sprite_spec;
use super::scene_clip_sprite_spec::extract_scene_clip_sprite_spec;
use crate::scene::Node3DInstance;
use engine_core::scene::Sprite;

pub fn map_sprite_to_node3d(sprite: &Sprite) -> Option<Node3DInstance> {
    if let Some(spec) = extract_obj_sprite_spec(sprite) {
        return Some(spec.node);
    }
    if let Some(spec) = extract_scene_clip_sprite_spec(sprite) {
        return Some(spec.node);
    }
    if let Some(spec) = extract_generated_world_sprite_spec(sprite) {
        return Some(spec.node);
    }

    match sprite {
        Sprite::Obj { .. } => None,
        Sprite::Planet { .. } => None,
        Sprite::Scene3D { .. } => None,
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

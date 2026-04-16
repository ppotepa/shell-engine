use super::generated_world_sprite_spec::{
    extract_generated_world_sprite_spec, GeneratedWorldSpriteSpec,
};
use super::obj_sprite_spec::{extract_obj_sprite_spec, ObjSpriteSpec};
use super::scene_clip_sprite_spec::{extract_scene_clip_sprite_spec, SceneClipSpriteSpec};
use engine_core::scene::Sprite;

pub enum Render3dSpriteSpec<'a> {
    Obj(ObjSpriteSpec<'a>),
    GeneratedWorld(GeneratedWorldSpriteSpec<'a>),
    SceneClip(SceneClipSpriteSpec<'a>),
}

pub fn extract_render3d_sprite_spec(sprite: &Sprite) -> Option<Render3dSpriteSpec<'_>> {
    if let Some(spec) = extract_obj_sprite_spec(sprite) {
        return Some(Render3dSpriteSpec::Obj(spec));
    }
    if let Some(spec) = extract_generated_world_sprite_spec(sprite) {
        return Some(Render3dSpriteSpec::GeneratedWorld(spec));
    }
    if let Some(spec) = extract_scene_clip_sprite_spec(sprite) {
        return Some(Render3dSpriteSpec::SceneClip(spec));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{extract_render3d_sprite_spec, Render3dSpriteSpec};
    use engine_core::scene::Sprite;

    #[test]
    fn extracts_obj_variant() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: obj
source: /assets/3d/sphere.obj
"#,
        )
        .expect("obj sprite should parse");

        match extract_render3d_sprite_spec(&sprite) {
            Some(Render3dSpriteSpec::Obj(_)) => {}
            _ => panic!("expected obj spec"),
        }
    }

    #[test]
    fn extracts_generated_world_variant() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: planet
body-id: earth
"#,
        )
        .expect("planet sprite should parse");

        match extract_render3d_sprite_spec(&sprite) {
            Some(Render3dSpriteSpec::GeneratedWorld(_)) => {}
            _ => panic!("expected generated world spec"),
        }
    }

    #[test]
    fn extracts_scene_clip_variant() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: scene3_d
src: /assets/3d/sample.scene3d.yml
"#,
        )
        .expect("scene3d sprite should parse");

        match extract_render3d_sprite_spec(&sprite) {
            Some(Render3dSpriteSpec::SceneClip(_)) => {}
            _ => panic!("expected scene clip spec"),
        }
    }
}

use crate::pipeline::{
    extract_generated_world_sprite_spec, extract_obj_sprite_spec, extract_scene_clip_sprite_spec,
    GeneratedWorldSpriteSpec, ObjSpriteSpec, SceneClipSpriteSpec,
};
use engine_core::scene::Sprite;

pub mod generated_world;
pub mod mesh;
pub mod scene_clip;

pub enum PreparedRender3dSource<'a> {
    Mesh(ObjSpriteSpec<'a>),
    GeneratedWorld(GeneratedWorldSpriteSpec<'a>),
    SceneClip(SceneClipSpriteSpec<'a>),
}

pub struct PreparedRender3dItem<'a> {
    pub source: PreparedRender3dSource<'a>,
}

pub trait Render3dProducer {
    fn produce<'a>(&self, sprite: &'a Sprite) -> Option<PreparedRender3dItem<'a>>;
}

pub struct MeshFrameProducer;
pub struct GeneratedWorldFrameProducer;
pub struct SceneClipFrameProducer;

impl Render3dProducer for MeshFrameProducer {
    fn produce<'a>(&self, sprite: &'a Sprite) -> Option<PreparedRender3dItem<'a>> {
        extract_obj_sprite_spec(sprite).map(|spec| PreparedRender3dItem {
            source: PreparedRender3dSource::Mesh(spec),
        })
    }
}

impl Render3dProducer for GeneratedWorldFrameProducer {
    fn produce<'a>(&self, sprite: &'a Sprite) -> Option<PreparedRender3dItem<'a>> {
        extract_generated_world_sprite_spec(sprite).map(|spec| PreparedRender3dItem {
            source: PreparedRender3dSource::GeneratedWorld(spec),
        })
    }
}

impl Render3dProducer for SceneClipFrameProducer {
    fn produce<'a>(&self, sprite: &'a Sprite) -> Option<PreparedRender3dItem<'a>> {
        extract_scene_clip_sprite_spec(sprite).map(|spec| PreparedRender3dItem {
            source: PreparedRender3dSource::SceneClip(spec),
        })
    }
}

pub fn prepare_render3d_item(sprite: &Sprite) -> Option<PreparedRender3dItem<'_>> {
    MeshFrameProducer
        .produce(sprite)
        .or_else(|| GeneratedWorldFrameProducer.produce(sprite))
        .or_else(|| SceneClipFrameProducer.produce(sprite))
}

#[cfg(test)]
mod tests {
    use super::{prepare_render3d_item, PreparedRender3dSource};
    use engine_core::scene::Sprite;

    #[test]
    fn prepares_mesh_item() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: obj
source: /assets/3d/sphere.obj
"#,
        )
        .expect("obj sprite should parse");

        match prepare_render3d_item(&sprite).map(|item| item.source) {
            Some(PreparedRender3dSource::Mesh(_)) => {}
            Some(_) => panic!("expected mesh prepared item"),
            None => panic!("expected prepared item"),
        }
    }

    #[test]
    fn prepares_generated_world_item() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: planet
body-id: earth
"#,
        )
        .expect("planet sprite should parse");

        match prepare_render3d_item(&sprite).map(|item| item.source) {
            Some(PreparedRender3dSource::GeneratedWorld(_)) => {}
            Some(_) => panic!("expected generated world prepared item"),
            None => panic!("expected prepared item"),
        }
    }

    #[test]
    fn prepares_scene_clip_item() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: scene3_d
src: /assets/3d/sample.scene3d.yml
"#,
        )
        .expect("scene3d sprite should parse");

        match prepare_render3d_item(&sprite).map(|item| item.source) {
            Some(PreparedRender3dSource::SceneClip(_)) => {}
            Some(_) => panic!("expected scene clip prepared item"),
            None => panic!("expected prepared item"),
        }
    }
}

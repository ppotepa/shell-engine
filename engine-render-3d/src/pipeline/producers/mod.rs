use crate::pipeline::{
    extract_generated_world_sprite_spec, extract_obj_sprite_spec, extract_scene_clip_sprite_spec,
    GeneratedWorldSpriteSpec, ObjSpriteSpec, SceneClipSpriteSpec,
};
use engine_core::scene::Sprite;

pub mod generated_world;
pub mod mesh;
pub mod scene_clip;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedRender3dKind {
    Mesh,
    GeneratedWorld,
    SceneClip,
}

pub enum PreparedRender3dSource<'a> {
    Mesh(ObjSpriteSpec<'a>),
    GeneratedWorld(GeneratedWorldSpriteSpec<'a>),
    SceneClip(SceneClipSpriteSpec<'a>),
}

impl<'a> PreparedRender3dSource<'a> {
    pub fn kind(&self) -> PreparedRender3dKind {
        match self {
            Self::Mesh(_) => PreparedRender3dKind::Mesh,
            Self::GeneratedWorld(_) => PreparedRender3dKind::GeneratedWorld,
            Self::SceneClip(_) => PreparedRender3dKind::SceneClip,
        }
    }

    pub fn sprite(&self) -> &'a Sprite {
        match self {
            Self::Mesh(spec) => spec.sprite,
            Self::GeneratedWorld(spec) => spec.sprite,
            Self::SceneClip(spec) => spec.sprite,
        }
    }

    pub fn id(&self) -> Option<&'a str> {
        match self {
            Self::Mesh(spec) => spec.id,
            Self::GeneratedWorld(_) => None,
            Self::SceneClip(spec) => spec.id,
        }
    }
}

pub struct PreparedRender3dItem<'a> {
    pub source: PreparedRender3dSource<'a>,
}

/// Buffer-agnostic packet wrapper for prepared 3D sprite items.
///
/// This intentionally carries only prepared source identity and references,
/// leaving render-target selection to a higher layer.
pub struct PreparedRender3dPacket<'a> {
    pub item: PreparedRender3dItem<'a>,
    pub kind: PreparedRender3dKind,
    pub sprite_id: Option<&'a str>,
}

impl<'a> PreparedRender3dPacket<'a> {
    pub fn builder(item: PreparedRender3dItem<'a>) -> PreparedRender3dPacketBuilder<'a> {
        PreparedRender3dPacketBuilder { item }
    }
}

pub struct PreparedRender3dPacketBuilder<'a> {
    item: PreparedRender3dItem<'a>,
}

impl<'a> PreparedRender3dPacketBuilder<'a> {
    pub fn build(self) -> PreparedRender3dPacket<'a> {
        let kind = self.item.source.kind();
        let sprite_id = self.item.source.id();
        PreparedRender3dPacket {
            item: self.item,
            kind,
            sprite_id,
        }
    }
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

pub fn prepare_render3d_packet(sprite: &Sprite) -> Option<PreparedRender3dPacket<'_>> {
    prepare_render3d_item(sprite).map(|item| PreparedRender3dPacket::builder(item).build())
}

#[cfg(test)]
mod tests {
    use super::{
        prepare_render3d_item, prepare_render3d_packet, PreparedRender3dKind, PreparedRender3dSource,
    };
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

    #[test]
    fn prepares_mesh_packet_kind_and_id() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: obj
id: asteroid
source: /assets/3d/sphere.obj
"#,
        )
        .expect("obj sprite should parse");

        let packet = prepare_render3d_packet(&sprite).expect("expected prepared packet");
        assert_eq!(packet.kind, PreparedRender3dKind::Mesh);
        assert_eq!(packet.sprite_id, Some("asteroid"));
        assert_eq!(packet.item.source.sprite().id(), Some("asteroid"));
    }
}

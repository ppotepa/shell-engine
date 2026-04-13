use rayon::prelude::*;

use engine_core::assets::AssetRoot;
use engine_core::logging;
use engine_core::scene::{Layer, Scene, Sprite};

use crate::obj_loader::load_obj_mesh;

pub fn warmup_scene_meshes(scene: &Scene, asset_root: &AssetRoot) {
    let sources = collect_obj_sources(&scene.layers);
    if sources.is_empty() {
        return;
    }

    logging::info(
        "engine.warmup",
        format!(
            "scene={}: pre-loading {} OBJ mesh(es) in parallel",
            scene.id,
            sources.len()
        ),
    );

    let loaded = sources
        .par_iter()
        .filter(|source| load_obj_mesh(asset_root, source).is_some())
        .count();

    logging::info(
        "engine.warmup",
        format!(
            "scene={}: warmed up {}/{} meshes",
            scene.id,
            loaded,
            sources.len()
        ),
    );
}

fn collect_obj_sources(layers: &[Layer]) -> Vec<String> {
    let mut sources = Vec::new();
    for layer in layers {
        collect_from_sprites(&layer.sprites, &mut sources);
    }
    sources.sort();
    sources.dedup();
    sources
}

fn collect_from_sprites(sprites: &[Sprite], out: &mut Vec<String>) {
    for sprite in sprites {
        match sprite {
            Sprite::Obj { source, .. } => out.push(source.clone()),
            Sprite::Planet { mesh_source, .. } => out.push(
                mesh_source
                    .clone()
                    .unwrap_or_else(|| "/assets/3d/sphere.obj".to_string()),
            ),
            Sprite::Panel { children, .. }
            | Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. } => collect_from_sprites(children, out),
            _ => {}
        }
    }
}

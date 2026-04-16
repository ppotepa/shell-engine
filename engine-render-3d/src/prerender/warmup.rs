use rayon::prelude::*;

use crate::pipeline::{extract_generated_world_sprite_spec, extract_obj_sprite_spec};
use engine_asset::load_render_mesh;
use engine_core::assets::AssetRoot;
use engine_core::logging;
use engine_core::scene::{Layer, Scene};

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
        .filter(|source| load_render_mesh(asset_root, source).is_some())
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
        for root in &layer.sprites {
            root.walk_recursive(&mut |sprite| {
                if let Some(spec) = extract_obj_sprite_spec(sprite) {
                    sources.push(spec.source.to_string());
                }
                if let Some(spec) = extract_generated_world_sprite_spec(sprite) {
                    if let crate::scene::Renderable3D::GeneratedWorld(world) = spec.node.renderable
                    {
                        sources.push(world.mesh_key.as_str().to_string());
                    }
                }
            });
        }
    }
    sources.sort();
    sources.dedup();
    sources
}

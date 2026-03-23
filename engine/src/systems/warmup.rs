//! Streaming asset warmup — pre-loads OBJ meshes into the cache before rendering.
//!
//! Loads all OBJ sprites (static and dynamic) in parallel during scene preparation,
//! so the compositor finds them already cached on first render instead of loading
//! synchronously mid-frame.
//!
//! This eliminates the "first frame stutter" caused by on-demand mesh parsing.

use rayon::prelude::*;

use crate::scene::{Layer, Scene, Sprite};
use crate::scene_pipeline::ScenePreparationStep;
use crate::services::EngineWorldAccess;
use crate::systems::compositor::obj_loader::load_obj_mesh;
use crate::world::World;
use engine_core::logging;

/// Scene preparation step: pre-loads all OBJ meshes in parallel before rendering.
///
/// Runs for every scene (not gated on `scene.prerender`). Warm-up is a no-op for
/// scenes with no `type: obj` sprites.
pub struct MeshWarmupStep;

impl ScenePreparationStep for MeshWarmupStep {
    fn name(&self) -> &'static str {
        "mesh-warmup"
    }

    fn run(&self, scene: &Scene, world: &mut World) {
        let Some(asset_root) = world.asset_root().cloned() else {
            return;
        };

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

        // Load all meshes in parallel. Each call populates OBJ_CACHE on miss.
        let loaded: usize = sources
            .par_iter()
            .filter(|source| load_obj_mesh(&asset_root, source).is_some())
            .count();

        logging::info(
            "engine.warmup",
            format!("scene={}: warmed up {}/{} meshes", scene.id, loaded, sources.len()),
        );
    }
}

/// Collect unique OBJ sprite sources from all layers (static and dynamic).
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
            Sprite::Obj { source, .. } => {
                out.push(source.clone());
            }
            Sprite::Panel { children, .. }
            | Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. } => {
                collect_from_sprites(children, out);
            }
            _ => {}
        }
    }
}

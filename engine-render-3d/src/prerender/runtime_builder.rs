use std::collections::HashSet;

use engine_3d::scene3d_format::load_scene3d;
use engine_3d::scene3d_resolve::{resolve_scene3d_refs, Scene3DAssetResolver};
use engine_core::assets::AssetRoot;
use engine_core::logging;
use engine_core::scene::{Layer, Scene, Sprite};

use super::{Scene3DRuntimeEntry, Scene3DRuntimeStore};

struct AssetRootResolver<'a> {
    asset_root: &'a AssetRoot,
}

impl Scene3DAssetResolver for AssetRootResolver<'_> {
    fn resolve_and_load_asset(
        &self,
        asset_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let full = self.asset_root.resolve(asset_path);
        Ok(std::fs::read_to_string(full)?)
    }
}

fn collect_scene3d_sources(layers: &[Layer]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for layer in layers {
        collect_sources_from_sprites(&layer.sprites, &mut seen, &mut out);
    }
    out
}

fn collect_sources_from_sprites(
    sprites: &[Sprite],
    seen: &mut HashSet<String>,
    out: &mut Vec<String>,
) {
    for sprite in sprites {
        match sprite {
            Sprite::Scene3D { src, .. } => {
                if seen.insert(src.clone()) {
                    out.push(src.clone());
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                collect_sources_from_sprites(children, seen, out);
            }
            _ => {}
        }
    }
}

/// Build a [`Scene3DRuntimeStore`] holding parsed `Scene3DDefinition` entries for every
/// `.scene3d.yml` referenced by `scene`.
pub fn build_scene3d_runtime_store(
    scene: &Scene,
    asset_root: &AssetRoot,
) -> Option<Scene3DRuntimeStore> {
    let sources = collect_scene3d_sources(&scene.layers);
    if sources.is_empty() {
        return None;
    }

    let resolver = AssetRootResolver { asset_root };
    let mut store = Scene3DRuntimeStore::new();

    for src in &sources {
        let path = asset_root.resolve(src);
        let path_str = path.to_string_lossy();
        let mut def = match load_scene3d(&path_str) {
            Ok(d) => d,
            Err(error) => {
                logging::warn(
                    "engine.scene3d",
                    format!("runtime-store: failed to load {src}: {error}"),
                );
                continue;
            }
        };
        resolve_scene3d_refs(&mut def, src, &resolver);
        store.insert(src.clone(), Scene3DRuntimeEntry { def });
    }

    if store.is_empty() {
        None
    } else {
        Some(store)
    }
}

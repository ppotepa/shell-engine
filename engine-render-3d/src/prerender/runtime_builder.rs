use engine_core::assets::AssetRoot;
use engine_core::logging;
use engine_core::scene::Scene;

use super::{collect_scene3d_sources, load_and_resolve_scene3d, Scene3DRuntimeEntry, Scene3DRuntimeStore};

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

    let mut store = Scene3DRuntimeStore::new();

    for src in &sources {
        let def = match load_and_resolve_scene3d(asset_root, src) {
            Ok(def) => def,
            Err(error) => {
                logging::warn(
                    "engine.scene3d",
                    format!("runtime-store: failed to load {src}: {error}"),
                );
                continue;
            }
        };
        store.insert(src.clone(), Scene3DRuntimeEntry { def });
    }

    if store.is_empty() {
        None
    } else {
        Some(store)
    }
}

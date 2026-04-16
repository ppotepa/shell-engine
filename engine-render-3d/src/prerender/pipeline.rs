use rayon::prelude::*;

use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::logging;
use engine_core::scene::Scene;

use super::{
    build_work_items, collect_scene3d_sources, load_and_resolve_scene3d, Scene3DAtlas, Scene3DWorkItem,
};

pub fn prerender_scene3d_atlas_with<F>(
    scene: &Scene,
    asset_root: &AssetRoot,
    render_frame: F,
) -> Option<Scene3DAtlas>
where
    F: Fn(&Scene3DWorkItem, &AssetRoot) -> Option<Buffer> + Sync + Send,
{
    let sources = collect_scene3d_sources(&scene.layers);
    if sources.is_empty() {
        return None;
    }

    let scene_id = scene.id.clone();

    logging::info(
        "engine.scene3d",
        format!(
            "scene={scene_id}: prerendering {} scene3d source(s) (parallel)",
            sources.len()
        ),
    );

    let work_items: Vec<Scene3DWorkItem> = sources
        .iter()
        .flat_map(|src| {
            let def = match load_and_resolve_scene3d(asset_root, src) {
                Ok(d) => d,
                Err(e) => {
                    logging::warn(
                        "engine.scene3d",
                        format!("scene={scene_id}: failed to load {src}: {e}"),
                    );
                    return Vec::new();
                }
            };
            build_work_items(src, &def)
        })
        .collect();

    let total = work_items.len();
    logging::info(
        "engine.scene3d",
        format!("scene={scene_id}: rendering {total} scene3d frame(s)"),
    );

    let rendered: Vec<(String, String, Buffer)> = work_items
        .into_par_iter()
        .filter_map(|item| {
            let buf = render_frame(&item, asset_root)?;
            Some((item.src, item.frame_id, buf))
        })
        .collect();

    let count = rendered.len();
    let mut atlas = Scene3DAtlas::new();
    for (src, frame_id, buf) in rendered {
        atlas.insert(&src, &frame_id, buf);
    }

    logging::info(
        "engine.scene3d",
        format!("scene={scene_id}: scene3d prerender complete ({count}/{total} frames cached)"),
    );

    Some(atlas)
}

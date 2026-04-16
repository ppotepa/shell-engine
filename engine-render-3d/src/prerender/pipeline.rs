use rayon::prelude::*;

use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::logging;
use engine_core::scene::Scene;
use engine_core::scene_runtime_types::SceneCamera3D;

use super::{
    build_scene3d_frame_item_at, build_work_items, collect_scene3d_sources, load_and_resolve_scene3d,
    Scene3DAtlas, Scene3DRuntimeEntry, Scene3DWorkItem,
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

pub fn render_scene3d_frame_at_with<F>(
    entry: &Scene3DRuntimeEntry,
    frame_name: &str,
    elapsed_ms: u64,
    asset_root: &AssetRoot,
    camera_override: Option<&SceneCamera3D>,
    render_frame: F,
) -> Option<Buffer>
where
    F: Fn(&Scene3DWorkItem, &AssetRoot) -> Option<Buffer>,
{
    let item = build_scene3d_frame_item_at(entry, frame_name, elapsed_ms, camera_override)?;
    render_frame(&item, asset_root)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use engine_3d::scene3d_format::Scene3DDefinition;
    use engine_core::{assets::AssetRoot, buffer::Buffer};

    use super::render_scene3d_frame_at_with;
    use crate::prerender::Scene3DRuntimeEntry;

    fn parse_scene3d(yaml: &str) -> Scene3DDefinition {
        serde_yaml::from_str(yaml).expect("scene3d yaml should parse")
    }

    #[test]
    fn single_frame_orchestration_invokes_render_callback() {
        let def = parse_scene3d(
            r#"
id: unit
viewport: { width: 32, height: 24 }
materials:
  solid: {}
objects:
  - id: cube
    mesh: /assets/3d/cube.obj
    material: solid
frames:
  orbit:
    show: [cube]
"#,
        );
        let entry = Scene3DRuntimeEntry { def };
        let calls = AtomicUsize::new(0);
        let asset_root = AssetRoot::new(PathBuf::from("."));

        let out = render_scene3d_frame_at_with(
            &entry,
            "orbit",
            0,
            &asset_root,
            None,
            |item, _| {
                calls.fetch_add(1, Ordering::Relaxed);
                Some(Buffer::new(item.viewport_w, item.viewport_h))
            },
        );

        assert!(out.is_some());
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        let out = out.unwrap();
        assert_eq!(out.width, 32);
        assert_eq!(out.height, 24);
    }

    #[test]
    fn single_frame_orchestration_returns_none_for_missing_frame() {
        let def = parse_scene3d(
            r#"
id: unit
viewport: { width: 16, height: 16 }
frames: {}
"#,
        );
        let entry = Scene3DRuntimeEntry { def };
        let asset_root = AssetRoot::new(PathBuf::from("."));

        let out = render_scene3d_frame_at_with(
            &entry,
            "missing",
            0,
            &asset_root,
            None,
            |_item, _| Some(Buffer::new(1, 1)),
        );

        assert!(out.is_none());
    }
}

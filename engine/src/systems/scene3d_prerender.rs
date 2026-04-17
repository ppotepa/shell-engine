//! Scene-pipeline wrapper for compositor Scene3D prerendering.

use crate::scene::Scene;
use crate::scene_pipeline::ScenePreparationStep;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_render_3d::prerender::{build_scene3d_runtime_store, prerender_scene3d_atlas_with};

// ── Scene preparation step ─────────────────────────────────────────────────────

/// Renders every named frame of every `.scene3d.yml` referenced by the scene
/// into the [`Scene3DAtlas`] world resource before the scene is activated.
/// Also builds the [`Scene3DRuntimeStore`] for real-time clip rendering.
pub struct Scene3DPrerenderStep;

impl ScenePreparationStep for Scene3DPrerenderStep {
    fn name(&self) -> &'static str {
        "scene3d-prerender"
    }

    fn run(&self, scene: &Scene, world: &mut World) {
        let Some(asset_root) = world.asset_root().cloned() else {
            return;
        };

        // Build the prerendered atlas (static frames + any explicitly prerendered clips).
        if let Some(atlas) = prerender_scene3d_atlas_with(
            scene,
            &asset_root,
            engine_render_3d::prerender::render_scene3d_work_item,
        ) {
            world.register_scoped(atlas);
        }

        // Build the runtime store (parsed definitions for real-time clip rendering).
        if let Some(store) = build_scene3d_runtime_store(scene, &asset_root) {
            world.register_scoped(store);
        }
    }
}

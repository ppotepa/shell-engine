//! Scene-pipeline wrapper for compositor Scene3D prerendering.

use crate::scene::Scene;
use crate::scene_pipeline::ScenePreparationStep;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_compositor::prerender_scene3d_atlas;

// ── Scene preparation step ─────────────────────────────────────────────────────

/// Renders every named frame of every `.scene3d.yml` referenced by the scene
/// into the [`Scene3DAtlas`] world resource before the scene is activated.
pub struct Scene3DPrerenderStep;

impl ScenePreparationStep for Scene3DPrerenderStep {
    fn name(&self) -> &'static str {
        "scene3d-prerender"
    }

    fn run(&self, scene: &Scene, world: &mut World) {
        let Some(asset_root) = world.asset_root().cloned() else {
            return;
        };
        let Some(atlas) = prerender_scene3d_atlas(scene, &asset_root) else {
            return;
        };
        world.register_scoped(atlas);
    }
}

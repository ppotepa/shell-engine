//! Scene-pipeline wrapper for compositor mesh warmup.

use crate::scene::Scene;
use crate::scene_pipeline::ScenePreparationStep;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_compositor::warmup_scene_meshes;

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
        warmup_scene_meshes(scene, &asset_root);
    }
}

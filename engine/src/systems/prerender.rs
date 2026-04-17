//! Scene-pipeline wrapper for compositor OBJ prerendering.

use crate::obj_prerender::ObjPrerenderStatus;
use crate::scene::Scene;
use crate::scene_pipeline::ScenePreparationStep;
use crate::services::EngineWorldAccess;
use crate::world::World;
use engine_compositor::prerender_scene_sprites;

// ── Scene preparation step ────────────────────────────────────────────────────

/// Scene preparation step: prerenders eligible `type: obj` sprites to in-memory
/// canvases before the scene is activated.
///
/// Runs only when `scene.prerender == true`.  Individual sprites opt out with
/// `prerender: false` on the sprite itself.  Registers scoped
/// [`ObjPrerenderedFrames`] and [`ObjPrerenderStatus::Ready`] into the world.
pub struct ObjPrerenderStep;

impl ScenePreparationStep for ObjPrerenderStep {
    fn name(&self) -> &'static str {
        "obj-prerender"
    }

    fn run(&self, scene: &Scene, world: &mut World) {
        if !scene.prerender {
            return;
        }
        let Some(asset_root) = world.asset_root().cloned() else {
            return;
        };
        let Some(frames) = prerender_scene_sprites(&scene.layers, &scene.id, &asset_root) else {
            return;
        };
        world.register_scoped(frames);
        world.register_scoped(ObjPrerenderStatus::Ready);
    }
}

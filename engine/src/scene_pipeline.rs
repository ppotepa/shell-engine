//! Scene preparation pipeline — the ordered set of steps run once per scene load,
//! before `SceneRuntime` is registered in the world.
//!
//! Every step may register **scoped** world resources that live exactly as long as
//! the active scene.  When the scene transitions, `world.clear_scoped()` is called
//! first, automatically discarding every scoped resource from the previous scene
//! (prerendered frames, caches, etc.) before the next scene's steps run.
//!
//! # Modularity
//!
//! Any engine system or mod can add a preparation step by implementing
//! [`ScenePreparationStep`].  Steps are not limited to 3-D concerns — asset
//! warm-up, audio pre-loading, sidecar handshakes, or arbitrary world mutations
//! are all valid uses.
//!
//! # Default pipeline
//!
//! [`ScenePipeline::default()`] contains the built-in steps in execution order:
//!
//! | Order | Step | Condition |
//! |-------|------|-----------|
//! | 1 | `ObjPrerenderStep` | `scene.prerender == true` |
//!
//! # Usage
//!
//! Register once at startup as an `Arc<ScenePipeline>` world resource:
//!
//! ```ignore
//! world.register(Arc::new(ScenePipeline::default()));
//! ```
//!
//! Before activating a scene:
//!
//! ```ignore
//! world.clear_scoped();
//! if let Some(p) = world.get::<Arc<ScenePipeline>>().cloned() {
//!     p.prepare(&scene, world);
//! }
//! world.register_scoped(SceneRuntime::new(scene));
//! ```

use crate::scene::Scene;
use crate::world::World;

/// A single modular step executed before a scene is activated.
///
/// Implementations may read `scene` fields to decide whether to run, and may
/// register scoped world resources via `world.register_scoped(...)`.
pub trait ScenePreparationStep: Send + Sync {
    /// Short identifier used in log output.
    fn name(&self) -> &'static str;

    /// Execute the step.
    fn run(&self, scene: &Scene, world: &mut World);
}

/// Ordered list of preparation steps run before every scene activation.
///
/// Stored as `Arc<ScenePipeline>` in the world so it can be cloned cheaply
/// inside `apply_transitions` without holding a borrow on `world`.
pub struct ScenePipeline {
    steps: Vec<Box<dyn ScenePreparationStep>>,
}

impl ScenePipeline {
    /// Build a pipeline from an explicit step list.
    pub fn new(steps: Vec<Box<dyn ScenePreparationStep>>) -> Self {
        Self { steps }
    }

    /// Run all preparation steps for `scene` in order.
    pub fn prepare(&self, scene: &Scene, world: &mut World) {
        for step in &self.steps {
            engine_core::logging::info(
                "engine.pipeline",
                format!("scene={}: step '{}'", scene.id, step.name()),
            );
            step.run(scene, world);
        }
    }
}

impl Default for ScenePipeline {
    /// Default pipeline: OBJ prerender (when `scene.prerender == true`).
    fn default() -> Self {
        Self::new(vec![
            Box::new(crate::systems::prerender::ObjPrerenderStep),
        ])
    }
}

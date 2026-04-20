//! Scene domain API: delegated to engine-api.
//!
//! Primary live-handle usage should go through `scene.object(...)` or
//! `runtime.scene.objects.*`. `scene.inspect(...)` remains snapshot-only.
//! Scene scripting registration for the runtime-first surface.

use rhai::Engine as RhaiEngine;

pub(crate) use engine_api::ScriptSceneApi;

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine_api::register_scene_api(engine);
}

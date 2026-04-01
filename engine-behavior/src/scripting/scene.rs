//! Scene domain API: delegated to engine-api.

use rhai::Engine as RhaiEngine;

pub(crate) use engine_api::{ScriptSceneApi, ScriptObjectApi};

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine_api::register_scene_api(engine);
}

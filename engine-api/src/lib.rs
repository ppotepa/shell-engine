//! Script-facing engine API facade.
//!
//! This crate is the landing zone for organizing the engine's exposed scripting
//! surface by domain. It intentionally starts as a minimal skeleton so existing
//! behavior can migrate in small, behavior-preserving steps.

pub mod audio;
pub mod collision;
pub mod commands;
pub mod effects;
pub mod gameplay;
pub mod input;
pub mod namespaces;
pub mod rhai;
pub mod scene;
pub mod testing;

// Re-export key types and functions for easy access
pub use commands::{BehaviorCommand, DebugLogSeverity};
pub use audio::{ScriptAudioApi, register_audio_api};
pub use collision::{ScriptCollisionApi, register_collision_api, filter_hits_by_kind, filter_hits_of_kind};
pub use effects::{ScriptEffectsApi, register_effects_api};
pub use scene::{ScriptSceneApi, ScriptObjectApi, register_scene_api};
pub use namespaces::{WorldNamespace, WorldApi, register_namespaces};
pub use gameplay::api::{CommandQueue, ScriptEntityContext, ScriptWorldContext};
pub use gameplay::emitters::EmitResolved;
pub use gameplay::geometry::{
    jitter_points_i32, points_to_rhai_array, regular_polygon_i32, rhai_array_to_points,
    rotate_points_i32, sin32_i32, to_i32,
};
pub use gameplay::lifecycle::{
    follow_anchor_from_args, is_ephemeral_lifecycle, parse_lifecycle_policy,
};
pub use gameplay::world::EphemeralPrefabResolved;
pub use input::normalization::normalize_input_code;
pub use rhai::conversion::{
    behavior_params_to_rhai_map, json_to_rhai_dynamic, map_bool, map_dynamic,
    map_get_path_dynamic, map_int, map_number, map_set_path_dynamic, map_string, merge_rhai_maps,
    normalize_set_path, region_to_rhai_map, rhai_dynamic_to_json,
};

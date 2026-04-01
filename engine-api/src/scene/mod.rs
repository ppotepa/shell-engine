//! Scene-facing scripting APIs.

pub mod mutation;
pub mod queries;
pub mod api;

pub use api::{ScriptSceneApi, ScriptObjectApi, register_scene_api};


//! Scene-facing scripting APIs.

pub mod api;
pub mod mutation;
pub mod queries;

pub use api::{register_scene_api, ScriptObjectApi, ScriptSceneApi};
pub use mutation::{
    Camera3dMutationRequest, Render3dMutationRequest, Render3dProfileSlot, SceneMutationRequest,
};

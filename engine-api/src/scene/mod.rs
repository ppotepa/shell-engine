//! Scene-facing scripting APIs.
//!
//! The primary mutable path is the live handle surface exposed through
//! `scene.object(...)` and `scene.objects` / `runtime.scene.objects`.
//! `scene.inspect(...)` remains a detached read-only snapshot view.
//!
//! The pure snapshot/query helpers live in the internal `queries` module so the
//! Rhai registration surface in `api.rs` can shrink incrementally without
//! destabilizing the crate.

pub mod api;
pub mod camera;
pub mod mutation;
pub(crate) mod queries;
pub mod render;

pub use api::{register_scene_api, ScriptObjectApi, ScriptSceneApi, ScriptSceneObjectsApi};
pub use camera::{Camera3dNormalizedMutation, Camera3dObjectViewState};
pub use mutation::{
    Camera3dMutationRequest, Render3dMutationRequest, Render3dProfileSlot, SceneMutationError,
    SceneMutationRequest, SceneMutationRequestError, SceneMutationResult, SceneMutationStatus,
};
pub use render::Render3dMutationDomain;

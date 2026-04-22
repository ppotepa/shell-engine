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
pub mod rhai;
pub mod runtime;
pub mod scene;
pub mod testing;
pub mod vehicle;

// Re-export key types and functions for easy access
pub use audio::{register_audio_api, ScriptAudioApi};
pub use collision::{
    filter_hits_by_kind, filter_hits_of_kind, register_collision_api, ScriptCollisionApi,
};
pub use commands::{BehaviorCommand, DebugLogSeverity};
pub use effects::{register_effects_api, ScriptEffectsApi};
pub use gameplay::api::{CommandQueue, ScriptEntityContext, ScriptWorldContext};
pub use gameplay::body::{
    register_world_body_api, GameplayWorldBodyLookupCoreApi, GameplayWorldBodySnapshotCoreApi,
};
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
    behavior_params_to_rhai_map, json_to_rhai_dynamic, map_bool, map_dynamic, map_get_path_dynamic,
    map_int, map_number, map_set_path_dynamic, map_string, merge_rhai_maps, normalize_set_path,
    region_to_rhai_map, rhai_dynamic_to_json,
};
pub use runtime::{
    register_runtime_core_api, ObjectRegistryCoreApi, RuntimeCoreApi, RuntimeSceneCoreApi,
    RuntimeServicesCoreApi, RuntimeStoresCoreApi, RuntimeWorldCoreApi, ScriptRuntimeApi,
};
pub use scene::{
    register_scene_api, Camera3dMutationRequest, Camera3dNormalizedMutation,
    Camera3dObjectViewState, Render3dMutationDomain, Render3dMutationRequest, Render3dProfileSlot,
    SceneMutationError, SceneMutationRequest, SceneMutationRequestError, SceneMutationResult,
    SceneMutationStatus, ScriptObjectApi, ScriptSceneApi,
};
pub use vehicle::{
    normalize_vehicle_profile_id, register_vehicle_api, register_vehicle_core_api,
    AngularBodyConfig, ArcadeConfig, BrakePhase, LinearBrakeConfig, MotionFrame, MotionFrameInput,
    ScriptVehicleApi, ShipModel, ThrusterRampConfig, VehicleAssembly, VehicleAssemblyContext,
    VehicleAssemblyDescriptor, VehicleAssemblyError, VehicleAssemblyModel, VehicleAssemblyPlan,
    VehicleAssemblySink, VehicleAssistState, VehicleBasis3, VehicleBodySnapshot,
    VehicleCapabilities, VehicleControlState, VehicleControllerModel, VehicleCoreApi,
    VehicleDescriptor, VehicleEnvironmentBinding, VehicleEnvironmentSnapshot, VehicleFacing,
    VehicleInputIntent, VehicleKind, VehicleLaunchPacket, VehicleModel, VehicleModelRef,
    VehicleMotionIntent, VehiclePacketEnvelope, VehiclePacketTelemetry, VehiclePacketVehicle,
    VehicleProfile, VehicleProfileInput, VehicleReferenceFrame, VehicleReferenceFrameModel,
    VehicleReturnPacket, VehicleRotationIntent, VehicleSelectionApi, VehicleTelemetry,
    VehicleTelemetryInput, VehicleTelemetryModel, VehicleTelemetrySnapshot,
    VehicleTranslationIntent, DEFAULT_VEHICLE_PROFILE_ID, LEGACY_VEHICLE_HANDOFF_PACKET_KIND,
    VEHICLE_HANDOFF_VERSION, VEHICLE_LAUNCH_PACKET_KIND, VEHICLE_RETURN_PACKET_KIND,
};

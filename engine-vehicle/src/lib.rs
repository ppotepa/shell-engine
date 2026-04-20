//! Vehicle-domain types, telemetry helpers, and neutral assembly plans.
//!
//! This crate owns the neutral vehicle vocabulary used across the engine:
//! profiles, facing/motion decomposition, telemetry, and small input structs
//! that let lower-level gameplay stores feed vehicle snapshots without
//! coupling the vehicle domain back to concrete runtime component types.

pub mod assembly;
pub mod capabilities;
pub mod descriptor;
pub mod handoff;
pub mod input;
pub mod kind;
pub mod models;
pub mod runtime;
mod types;

pub use assembly::{
    AngularBodyConfig, ArcadeConfig, LinearBrakeConfig, ThrusterRampConfig, VehicleAssembly,
    VehicleAssemblyContext, VehicleAssemblyDescriptor, VehicleAssemblyError, VehicleAssemblyPlan,
    VehicleAssemblySink,
};
pub use capabilities::VehicleCapabilities;
pub use descriptor::{VehicleDescriptor, VehicleModel};
pub use handoff::{
    VehicleBasis3, VehicleBodySnapshot, VehicleEnvironmentSnapshot, VehicleLaunchPacket,
    VehiclePacketEnvelope, VehiclePacketTelemetry, VehiclePacketVehicle, VehicleReturnPacket,
    LEGACY_VEHICLE_HANDOFF_PACKET_KIND, VEHICLE_HANDOFF_VERSION, VEHICLE_LAUNCH_PACKET_KIND,
    VEHICLE_RETURN_PACKET_KIND,
};
pub use input::{
    builtin_ship_profile_tuning, next_builtin_ship_profile_id, normalize_vehicle_profile_id,
    VehicleAssistState, VehicleButtonInput, VehicleControlState, VehicleEnvironmentBinding,
    VehicleInputIntent, VehicleMotionIntent, VehicleReferenceFrame, VehicleRotationIntent,
    VehicleShipProfile, VehicleShipProfileTuning, VehicleTelemetrySnapshot,
    VehicleTranslationIntent, DEFAULT_VEHICLE_PROFILE_ID,
};
pub use kind::VehicleKind;
pub use models::{ShipModel, VehicleModelRef};
pub use runtime::{
    ShipReferenceFrameKind, ShipReferenceFrameState, ShipRuntimeInput, ShipRuntimeModel,
    ShipRuntimeOutput, ShipRuntimeState, ShipSurfaceMode, VehicleAssemblyModel,
    VehicleControllerModel, VehicleReferenceFrameModel, VehicleTelemetryModel,
};
pub use types::{
    BrakePhase, MotionFrame, MotionFrameInput, VehicleFacing, VehicleProfile, VehicleProfileInput,
    VehicleTelemetry, VehicleTelemetryInput,
};

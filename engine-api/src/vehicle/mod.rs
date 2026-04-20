//! Vehicle-domain scripting APIs.

pub mod api;

pub use engine_vehicle::{
    normalize_vehicle_profile_id, AngularBodyConfig, ArcadeConfig, BrakePhase, LinearBrakeConfig,
    MotionFrame, MotionFrameInput, ShipModel, ThrusterRampConfig, VehicleAssembly,
    VehicleAssemblyContext, VehicleAssemblyDescriptor, VehicleAssemblyError, VehicleAssemblyModel,
    VehicleAssemblyPlan, VehicleAssemblySink, VehicleAssistState, VehicleBasis3,
    VehicleBodySnapshot, VehicleCapabilities, VehicleControlState, VehicleControllerModel,
    VehicleDescriptor, VehicleEnvironmentBinding, VehicleEnvironmentSnapshot, VehicleFacing,
    VehicleInputIntent, VehicleKind, VehicleLaunchPacket, VehicleModel, VehicleModelRef,
    VehicleMotionIntent, VehiclePacketEnvelope, VehiclePacketTelemetry, VehiclePacketVehicle,
    VehicleProfile, VehicleProfileInput, VehicleReferenceFrame, VehicleReferenceFrameModel,
    VehicleReturnPacket, VehicleRotationIntent, VehicleTelemetry, VehicleTelemetryInput,
    VehicleTelemetryModel, VehicleTelemetrySnapshot, VehicleTranslationIntent,
    DEFAULT_VEHICLE_PROFILE_ID, LEGACY_VEHICLE_HANDOFF_PACKET_KIND, VEHICLE_HANDOFF_VERSION,
    VEHICLE_LAUNCH_PACKET_KIND, VEHICLE_RETURN_PACKET_KIND,
};

pub use api::{
    register_vehicle_api, register_vehicle_core_api, ScriptVehicleApi, VehicleCoreApi,
    VehicleSelectionApi,
};

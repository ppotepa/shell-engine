use serde::{Deserialize, Serialize};

use crate::{
    models::ShipModel, MotionFrame, MotionFrameInput, VehicleAssembly, VehicleCapabilities,
    VehicleControlState, VehicleFacing, VehicleInputIntent, VehicleKind, VehicleTelemetry,
    VehicleTelemetryInput,
};
use crate::{
    VehicleAssemblyModel, VehicleControllerModel, VehicleReferenceFrameModel, VehicleTelemetryModel,
};

/// Enum-based vehicle model dispatch. Avoids a large `dyn Vehicle` surface.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "model", rename_all = "snake_case")]
pub enum VehicleModel {
    Ship(ShipModel),
}

impl VehicleModel {
    pub fn kind(&self) -> VehicleKind {
        match self {
            Self::Ship(_) => VehicleKind::Ship,
        }
    }

    pub fn capabilities(&self) -> VehicleCapabilities {
        match self {
            Self::Ship(model) => model.capabilities(),
        }
    }

    pub fn assembly(&self) -> VehicleAssembly {
        match self {
            Self::Ship(model) => model.vehicle_assembly(),
        }
    }

    pub fn default_control_state(&self) -> VehicleControlState {
        match self {
            Self::Ship(model) => model.default_control_state(),
        }
    }

    pub fn control_state_from_intent(&self, intent: VehicleInputIntent) -> VehicleControlState {
        match self {
            Self::Ship(model) => model.control_state_from_intent(intent),
        }
    }

    pub fn facing_from_heading(&self, heading: f32) -> VehicleFacing {
        match self {
            Self::Ship(model) => model.facing_from_heading(heading),
        }
    }

    pub fn motion_frame_from_input(&self, input: MotionFrameInput, heading: f32) -> MotionFrame {
        match self {
            Self::Ship(model) => model.motion_frame_from_input(input, heading),
        }
    }

    pub fn telemetry_from_input(&self, input: VehicleTelemetryInput) -> VehicleTelemetry {
        match self {
            Self::Ship(model) => model.telemetry_from_input(input),
        }
    }

    pub fn profile_id(&self) -> &str {
        match self {
            Self::Ship(model) => &model.profile_id,
        }
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Ship(model) => model.label.as_deref(),
        }
    }
}

/// Fully-typed descriptor for a concrete vehicle domain model.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleDescriptor {
    pub kind: VehicleKind,
    pub capabilities: VehicleCapabilities,
    pub profile_id: String,
    pub label: Option<String>,
    pub model: VehicleModel,
}

impl Default for VehicleDescriptor {
    fn default() -> Self {
        Self::ship(ShipModel::default())
    }
}

impl VehicleDescriptor {
    pub fn ship(model: ShipModel) -> Self {
        Self {
            kind: VehicleKind::Ship,
            capabilities: model.capabilities(),
            profile_id: model.profile_id.clone(),
            label: model.label.clone(),
            model: VehicleModel::Ship(model),
        }
    }

    pub fn refresh_metadata(&mut self) {
        self.kind = self.model.kind();
        self.capabilities = self.model.capabilities();
        self.profile_id = self.model.profile_id().to_string();
        self.label = self.model.label().map(ToString::to_string);
    }

    pub fn assembly(&self) -> VehicleAssembly {
        self.model.assembly()
    }

    pub fn default_control_state(&self) -> VehicleControlState {
        self.model.default_control_state()
    }

    pub fn control_state_from_intent(&self, intent: VehicleInputIntent) -> VehicleControlState {
        self.model.control_state_from_intent(intent)
    }

    pub fn facing_from_heading(&self, heading: f32) -> VehicleFacing {
        self.model.facing_from_heading(heading)
    }

    pub fn motion_frame_from_input(&self, input: MotionFrameInput, heading: f32) -> MotionFrame {
        self.model.motion_frame_from_input(input, heading)
    }

    pub fn telemetry_from_input(&self, input: VehicleTelemetryInput) -> VehicleTelemetry {
        self.model.telemetry_from_input(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{BrakePhase, MotionFrameInput, VehicleAssistState, VehicleTelemetryInput};

    use super::{ShipModel, VehicleDescriptor, VehicleKind, VehicleModel};

    #[test]
    fn ship_descriptor_uses_typed_dispatch() {
        let descriptor = VehicleDescriptor::ship(ShipModel::new("sim_lite"));
        assert_eq!(descriptor.kind, VehicleKind::Ship);
        assert_eq!(descriptor.profile_id, "sim-lite");
        assert!(descriptor.capabilities.supports_vehicle_stack());

        let frame = descriptor.motion_frame_from_input(
            MotionFrameInput {
                velocity_x: 0.0,
                velocity_y: -2.0,
                accel_x: 0.0,
                accel_y: -1.0,
            },
            0.0,
        );
        assert!((frame.forward_speed - 2.0).abs() < 0.001);

        let telemetry = descriptor.telemetry_from_input(VehicleTelemetryInput {
            heading: 0.0,
            brake_phase: Some(BrakePhase::Idle),
            ..VehicleTelemetryInput::default()
        });
        assert_eq!(telemetry.brake_phase, BrakePhase::Idle);
    }

    #[test]
    fn refresh_metadata_realigns_descriptor_with_model() {
        let mut descriptor = VehicleDescriptor::default();
        descriptor.profile_id = "stale-profile".to_string();
        descriptor.label = None;

        descriptor.model = VehicleModel::Ship(ShipModel {
            profile_id: "sim-lite".to_string(),
            label: Some("Scout".to_string()),
            assists: VehicleAssistState::from_flags(true, false),
            ..ShipModel::default()
        });

        descriptor.refresh_metadata();

        assert_eq!(descriptor.kind, VehicleKind::Ship);
        assert_eq!(descriptor.profile_id, "sim-lite");
        assert_eq!(descriptor.label.as_deref(), Some("Scout"));
        assert_eq!(descriptor.default_control_state().profile_id, "sim-lite");
        assert!(descriptor.default_control_state().assists.alt_hold);
        assert!(descriptor.capabilities.handoff_packets);
    }
}

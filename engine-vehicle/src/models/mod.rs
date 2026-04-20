mod ship;

pub use ship::ShipModel;

use crate::{
    MotionFrame, MotionFrameInput, VehicleAssembly, VehicleCapabilities, VehicleControlState,
    VehicleDescriptor, VehicleFacing, VehicleInputIntent, VehicleKind, VehicleTelemetry,
    VehicleTelemetryInput,
};
use crate::{
    VehicleAssemblyModel, VehicleControllerModel, VehicleReferenceFrameModel, VehicleTelemetryModel,
};

/// Typed reference to a concrete vehicle model without trait objects.
#[derive(Clone, Debug, PartialEq)]
pub enum VehicleModelRef {
    Ship(ShipModel),
}

impl VehicleModelRef {
    pub fn kind(&self) -> VehicleKind {
        match self {
            Self::Ship(model) => model.kind(),
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

    pub fn descriptor(&self) -> VehicleDescriptor {
        match self {
            Self::Ship(model) => model.descriptor(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ShipModel, VehicleModelRef};
    use crate::{VehicleAssistState, VehicleInputIntent, VehicleKind};

    #[test]
    fn vehicle_model_ref_dispatches_through_ship_model() {
        let model = VehicleModelRef::Ship(ShipModel {
            profile_id: "sim-lite".to_string(),
            label: Some("Scout".to_string()),
            assists: VehicleAssistState::from_flags(true, true),
            ..ShipModel::default()
        });

        let descriptor = model.descriptor();
        let control = model.control_state_from_intent(VehicleInputIntent {
            throttle: 0.75,
            boost: true,
            ..VehicleInputIntent::default()
        });

        assert_eq!(model.kind(), VehicleKind::Ship);
        assert!(model.capabilities().handoff_packets);
        assert_eq!(descriptor.profile_id, "sim-lite");
        assert_eq!(descriptor.label.as_deref(), Some("Scout"));
        assert_eq!(control.profile_id, "sim-lite");
        assert!(control.assists.alt_hold);
        assert!(control.assists.heading_hold);
        assert_eq!(control.boost_scale, 2.0);
    }
}

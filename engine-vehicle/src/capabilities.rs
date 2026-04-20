use serde::{Deserialize, Serialize};

/// Typed capability surface for vehicle domain dispatch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VehicleCapabilities {
    pub discrete_heading: bool,
    pub inertial_translation: bool,
    pub angular_inertia: bool,
    pub linear_braking: bool,
    pub thruster_ramp: bool,
    pub handoff_packets: bool,
}

impl VehicleCapabilities {
    pub const fn new() -> Self {
        Self {
            discrete_heading: false,
            inertial_translation: false,
            angular_inertia: false,
            linear_braking: false,
            thruster_ramp: false,
            handoff_packets: false,
        }
    }

    pub const fn ship() -> Self {
        Self {
            discrete_heading: true,
            inertial_translation: true,
            angular_inertia: true,
            linear_braking: true,
            thruster_ramp: true,
            handoff_packets: true,
        }
    }

    pub fn supports_vehicle_stack(self) -> bool {
        self.discrete_heading || self.angular_inertia || self.linear_braking || self.thruster_ramp
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            discrete_heading: self.discrete_heading || other.discrete_heading,
            inertial_translation: self.inertial_translation || other.inertial_translation,
            angular_inertia: self.angular_inertia || other.angular_inertia,
            linear_braking: self.linear_braking || other.linear_braking,
            thruster_ramp: self.thruster_ramp || other.thruster_ramp,
            handoff_packets: self.handoff_packets || other.handoff_packets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VehicleCapabilities;

    #[test]
    fn ship_capabilities_enable_current_stack() {
        let caps = VehicleCapabilities::ship();
        assert!(caps.supports_vehicle_stack());
        assert!(caps.handoff_packets);
    }
}

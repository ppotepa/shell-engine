use serde::{Deserialize, Serialize};

/// Stable top-level vehicle taxonomy for typed dispatch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleKind {
    #[default]
    Ship,
}

impl VehicleKind {
    pub fn from_hint(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "" => None,
            "ship" | "spaceship" | "spacecraft" | "space-ship" | "space_ship" => Some(Self::Ship),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ship => "ship",
        }
    }

    pub fn is_spacecraft(self) -> bool {
        matches!(self, Self::Ship)
    }
}

impl std::fmt::Display for VehicleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::VehicleKind;

    #[test]
    fn vehicle_kind_reports_stable_ids() {
        assert_eq!(VehicleKind::Ship.as_str(), "ship");
        assert!(VehicleKind::Ship.is_spacecraft());
    }

    #[test]
    fn vehicle_kind_accepts_known_hints() {
        assert_eq!(VehicleKind::from_hint(" ship "), Some(VehicleKind::Ship));
        assert_eq!(
            VehicleKind::from_hint("space_ship"),
            Some(VehicleKind::Ship)
        );
        assert_eq!(
            VehicleKind::from_hint("spacecraft"),
            Some(VehicleKind::Ship)
        );
        assert_eq!(VehicleKind::from_hint(""), None);
        assert_eq!(VehicleKind::from_hint("rover"), None);
        assert_eq!(VehicleKind::Ship.to_string(), "ship");
    }
}

use serde::{Deserialize, Serialize};

const CROSS_MOD_PREFIX: &str = "mod+scene:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneTransitionTarget {
    CurrentMod { scene_ref: String },
    OtherMod { mod_ref: String, scene_ref: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CrossModTransitionWire {
    #[serde(rename = "mod")]
    mod_ref: String,
    scene_ref: String,
}

impl SceneTransitionTarget {
    pub fn current_mod(scene_ref: impl Into<String>) -> Option<Self> {
        let scene_ref = scene_ref.into().trim().to_string();
        if scene_ref.is_empty() {
            return None;
        }
        Some(Self::CurrentMod { scene_ref })
    }

    pub fn other_mod(mod_ref: impl Into<String>, scene_ref: impl Into<String>) -> Option<Self> {
        let mod_ref = mod_ref.into().trim().to_string();
        let scene_ref = scene_ref.into().trim().to_string();
        if mod_ref.is_empty() || scene_ref.is_empty() {
            return None;
        }
        Some(Self::OtherMod { mod_ref, scene_ref })
    }

    pub fn parse_wire(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        let Some(payload) = trimmed.strip_prefix(CROSS_MOD_PREFIX) else {
            return Self::current_mod(trimmed);
        };
        let wire: CrossModTransitionWire = serde_json::from_str(payload).ok()?;
        Self::other_mod(wire.mod_ref, wire.scene_ref)
    }

    pub fn to_wire_string(&self) -> String {
        match self {
            Self::CurrentMod { scene_ref } => scene_ref.clone(),
            Self::OtherMod { mod_ref, scene_ref } => {
                let wire = CrossModTransitionWire {
                    mod_ref: mod_ref.clone(),
                    scene_ref: scene_ref.clone(),
                };
                format!(
                    "{CROSS_MOD_PREFIX}{}",
                    serde_json::to_string(&wire).expect("cross-mod transition wire")
                )
            }
        }
    }

    pub fn scene_ref(&self) -> &str {
        match self {
            Self::CurrentMod { scene_ref } | Self::OtherMod { scene_ref, .. } => scene_ref,
        }
    }

    pub fn mod_ref(&self) -> Option<&str> {
        match self {
            Self::CurrentMod { .. } => None,
            Self::OtherMod { mod_ref, .. } => Some(mod_ref),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SceneTransitionTarget;

    #[test]
    fn local_transition_round_trips_as_plain_scene_ref() {
        let target = SceneTransitionTarget::current_mod("flight").expect("target");
        let wire = target.to_wire_string();

        assert_eq!(wire, "flight");
        assert_eq!(SceneTransitionTarget::parse_wire(&wire), Some(target));
    }

    #[test]
    fn cross_mod_transition_round_trips_via_wire_format() {
        let target = SceneTransitionTarget::other_mod("planet-generator", "/scenes/main/scene.yml")
            .expect("target");
        let wire = target.to_wire_string();

        assert!(wire.starts_with("mod+scene:"));
        assert_eq!(SceneTransitionTarget::parse_wire(&wire), Some(target));
    }

    #[test]
    fn malformed_cross_mod_wire_is_rejected() {
        assert_eq!(
            SceneTransitionTarget::parse_wire("mod+scene:{\"mod\":\"playground\"}"),
            None
        );
    }
}

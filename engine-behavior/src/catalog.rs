//! Mod-scoped gameplay catalogs for data-driven helpers.
//! Catalogs allow mods to define prefabs, weapons, emitters, input profiles, etc.
//! via YAML instead of hardcoding in Rust.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Complete set of catalogs for a mod.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModCatalogs {
    pub input_profiles: HashMap<String, InputProfile>,
    pub prefabs: HashMap<String, PrefabTemplate>,
    pub weapons: HashMap<String, WeaponConfig>,
    pub emitters: HashMap<String, EmitterConfig>,
    pub groups: HashMap<String, GroupTemplate>,
    pub waves: HashMap<String, WaveTemplate>,
}

/// Input action bindings: action_name -> list of key codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputProfile {
    pub bindings: HashMap<String, Vec<String>>,
}

/// Prefab template for entity spawning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabTemplate {
    pub kind: String,
    #[serde(default)]
    pub sprite_template: Option<String>,
    #[serde(default)]
    pub init_fields: HashMap<String, JsonValue>,
}

/// Weapon configuration for firing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponConfig {
    pub max_bullets: i64,
    #[serde(default)]
    pub bullet_kind: Option<String>,
    #[serde(default)]
    pub bullet_ttl_ms: Option<i64>,
    #[serde(default)]
    pub cooldown_ms: Option<i64>,
    #[serde(default)]
    pub cooldown_name: Option<String>,
    #[serde(default)]
    pub spawn_offset: Option<f64>,
    #[serde(default)]
    pub speed_scale: Option<f64>,
}

/// Emitter configuration for particle effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterConfig {
    #[serde(default)]
    pub max_count: Option<i64>,
    #[serde(default)]
    pub cooldown_name: Option<String>,
    #[serde(default)]
    pub cooldown_ms: Option<i64>,
    #[serde(default)]
    pub spawn_offset: Option<f64>,
    #[serde(default)]
    pub backward_speed: Option<f64>,
    #[serde(default)]
    pub ttl_ms: Option<i64>,
    #[serde(default)]
    pub radius: Option<i64>,
    #[serde(default)]
    pub velocity_scale: Option<f64>,
}

/// Group template: predefined batch spawn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTemplate {
    pub prefab: String,
    pub spawns: Vec<SpawnSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnSpec {
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub vx: f64,
    #[serde(default)]
    pub vy: f64,
    #[serde(default)]
    pub shape: i64,
    #[serde(default)]
    pub size: i64,
}

/// Wave template: dynamic spawn generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveTemplate {
    pub prefab: String,
    #[serde(default)]
    pub size_distribution: Vec<SizeDistribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeDistribution {
    #[serde(default)]
    pub min_idx: i64,
    #[serde(default)]
    pub max_idx: Option<i64>,
    pub size: i64,
}

impl ModCatalogs {
    /// Create an empty catalog set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load catalogs from a directory (mod_source/catalogs/).
    pub fn load_from_directory(catalogs_dir: &std::path::Path) -> Result<Self, String> {
        let mut catalogs = Self::new();

        // Load input profiles
        let input_path = catalogs_dir.join("input-profiles.yaml");
        if input_path.exists() {
            let content = std::fs::read_to_string(&input_path)
                .map_err(|e| format!("Failed to read input-profiles.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse input-profiles.yaml: {}", e))?;

            if let Some(profiles) = parsed.get("profiles").and_then(|v| v.as_mapping()) {
                for (key, value) in profiles.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(profile) = serde_yaml::from_value::<InputProfile>(value.clone()) {
                            catalogs.input_profiles.insert(key_str.to_string(), profile);
                        }
                    }
                }
            }
        }

        // Load prefabs
        let prefabs_path = catalogs_dir.join("prefabs.yaml");
        if prefabs_path.exists() {
            let content = std::fs::read_to_string(&prefabs_path)
                .map_err(|e| format!("Failed to read prefabs.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse prefabs.yaml: {}", e))?;

            if let Some(prefabs) = parsed.get("prefabs").and_then(|v| v.as_mapping()) {
                for (key, value) in prefabs.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(prefab) = serde_yaml::from_value::<PrefabTemplate>(value.clone())
                        {
                            catalogs.prefabs.insert(key_str.to_string(), prefab);
                        }
                    }
                }
            }
        }

        // Load weapons
        let weapons_path = catalogs_dir.join("weapons.yaml");
        if weapons_path.exists() {
            let content = std::fs::read_to_string(&weapons_path)
                .map_err(|e| format!("Failed to read weapons.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse weapons.yaml: {}", e))?;

            if let Some(weapons) = parsed.get("weapons").and_then(|v| v.as_mapping()) {
                for (key, value) in weapons.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(weapon) = serde_yaml::from_value::<WeaponConfig>(value.clone()) {
                            catalogs.weapons.insert(key_str.to_string(), weapon);
                        }
                    }
                }
            }
        }

        // Load emitters
        let emitters_path = catalogs_dir.join("emitters.yaml");
        if emitters_path.exists() {
            let content = std::fs::read_to_string(&emitters_path)
                .map_err(|e| format!("Failed to read emitters.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse emitters.yaml: {}", e))?;

            if let Some(emitters) = parsed.get("emitters").and_then(|v| v.as_mapping()) {
                for (key, value) in emitters.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(emitter) = serde_yaml::from_value::<EmitterConfig>(value.clone())
                        {
                            catalogs.emitters.insert(key_str.to_string(), emitter);
                        }
                    }
                }
            }
        }

        // Load spawners (groups and waves)
        let spawners_path = catalogs_dir.join("spawners.yaml");
        if spawners_path.exists() {
            let content = std::fs::read_to_string(&spawners_path)
                .map_err(|e| format!("Failed to read spawners.yaml: {}", e))?;
            let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse spawners.yaml: {}", e))?;

            if let Some(groups) = parsed.get("groups").and_then(|v| v.as_mapping()) {
                for (key, value) in groups.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(group) = serde_yaml::from_value::<GroupTemplate>(value.clone()) {
                            catalogs.groups.insert(key_str.to_string(), group);
                        }
                    }
                }
            }

            if let Some(waves) = parsed.get("waves").and_then(|v| v.as_mapping()) {
                for (key, value) in waves.iter() {
                    if let Some(key_str) = key.as_str() {
                        if let Ok(wave) = serde_yaml::from_value::<WaveTemplate>(value.clone()) {
                            catalogs.waves.insert(key_str.to_string(), wave);
                        }
                    }
                }
            }
        }

        Ok(catalogs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_catalogs() {
        let catalogs = ModCatalogs::new();
        assert!(catalogs.input_profiles.is_empty());
        assert!(catalogs.prefabs.is_empty());
        assert!(catalogs.weapons.is_empty());
        assert!(catalogs.emitters.is_empty());
        assert!(catalogs.groups.is_empty());
        assert!(catalogs.waves.is_empty());
    }

    #[test]
    fn test_input_profile_parsing() {
        let yaml = r#"
profiles:
  default:
    bindings:
      turn_left: ["Left", "a", "A"]
      turn_right: ["Right", "d", "D"]
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let profile_val = parsed.get("profiles").unwrap().get("default").unwrap();
        let profile: InputProfile = serde_yaml::from_value(profile_val.clone()).unwrap();

        assert_eq!(profile.bindings.get("turn_left").unwrap().len(), 3);
        assert_eq!(profile.bindings.get("turn_right").unwrap().len(), 3);
    }

    #[test]
    fn test_prefab_parsing() {
        let yaml = r#"
prefabs:
  ship:
    kind: "ship"
    sprite_template: "ship"
    init_fields:
      x: 0
      y: 0
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let prefab_val = parsed.get("prefabs").unwrap().get("ship").unwrap();
        let prefab: PrefabTemplate = serde_yaml::from_value(prefab_val.clone()).unwrap();

        assert_eq!(prefab.kind, "ship");
        assert_eq!(prefab.sprite_template, Some("ship".to_string()));
        assert!(!prefab.init_fields.is_empty());
    }

    #[test]
    fn test_group_parsing() {
        let yaml = r#"
groups:
  asteroids.initial:
    prefab: "asteroid"
    spawns:
      - {x: -300, y: -210, vx: 2.0, vy: 0.0, shape: 0, size: 2}
      - {x: 300, y: -210, vx: 0.0, vy: 2.0, shape: 1, size: 3}
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let group_val = parsed
            .get("groups")
            .unwrap()
            .get("asteroids.initial")
            .unwrap();
        let group: GroupTemplate = serde_yaml::from_value(group_val.clone()).unwrap();

        assert_eq!(group.prefab, "asteroid");
        assert_eq!(group.spawns.len(), 2);
        assert_eq!(group.spawns[0].x, -300.0);
    }

    #[test]
    fn test_wave_parsing() {
        let yaml = r#"
waves:
  asteroids.dynamic:
    prefab: "asteroid"
    size_distribution:
      - {min_idx: 0, max_idx: 2, size: 3}
      - {min_idx: 2, max_idx: 5, size: 2}
      - {min_idx: 5, size: 1}
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let wave_val = parsed
            .get("waves")
            .unwrap()
            .get("asteroids.dynamic")
            .unwrap();
        let wave: WaveTemplate = serde_yaml::from_value(wave_val.clone()).unwrap();

        assert_eq!(wave.prefab, "asteroid");
        assert_eq!(wave.size_distribution.len(), 3);
    }
}

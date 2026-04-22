//! Scene-level planet generation/render contract.
//!
//! This is a typed authored payload that can be declared on a scene and then
//! consumed by higher-level runtime systems (e.g. planet generator mods).

use serde::Deserialize;

/// Scene-level authored planet specification.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct PlanetSpec {
    #[serde(default)]
    pub generator: Option<PlanetSpecGenerator>,
    #[serde(default)]
    pub render: Option<PlanetSpecRender>,
    #[serde(default)]
    pub atmosphere: Option<PlanetSpecAtmosphere>,
    #[serde(default)]
    pub body: Option<PlanetSpecBody>,
}

/// Procedural generation inputs.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PlanetSpecGenerator {
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub detail: Option<u8>,
}

/// Renderer-facing overrides.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PlanetSpecRender {
    #[serde(default)]
    pub mesh_source: Option<String>,
    #[serde(default)]
    pub surface_mode: Option<String>,
    #[serde(default)]
    pub stretch_to_area: Option<bool>,
}

/// Atmosphere authoring parameters.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PlanetSpecAtmosphere {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub density: Option<f32>,
    #[serde(default)]
    pub thickness_km: Option<f32>,
}

/// Body/celestial parameters.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PlanetSpecBody {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub radius_km: Option<f32>,
    #[serde(default)]
    pub gravity_m_s2: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::PlanetSpec;

    #[test]
    fn parses_planet_spec_kebab_case_fields() {
        let spec = serde_yaml::from_str::<PlanetSpec>(
            r#"
generator:
  preset: terrestrial
  seed: 42
  detail: 7
render:
  mesh-source: /assets/3d/sphere.obj
  surface-mode: material
  stretch-to-area: true
atmosphere:
  enabled: true
  density: 0.35
  thickness-km: 120.0
body:
  id: earth-like
  radius-km: 6371.0
  gravity-m-s2: 9.81
"#,
        )
        .expect("planet spec should parse");

        assert_eq!(
            spec.generator.as_ref().and_then(|generator| generator.seed),
            Some(42)
        );
        assert_eq!(
            spec.render
                .as_ref()
                .and_then(|render| render.mesh_source.as_deref()),
            Some("/assets/3d/sphere.obj")
        );
        assert_eq!(
            spec.atmosphere
                .as_ref()
                .and_then(|atmosphere| atmosphere.thickness_km),
            Some(120.0)
        );
        assert_eq!(
            spec.body
                .as_ref()
                .and_then(|body| body.gravity_m_s2)
                .unwrap_or_default()
                .round() as i32,
            10
        );
    }
}

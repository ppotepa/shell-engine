use serde::{Deserialize, Serialize};

/// Selected level-of-detail level. `0` is highest detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct LodLevel(pub u8);

/// Screen-space inputs available to LOD policy selection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct ScreenSpaceMetrics {
    /// Approximate projected object radius in pixels.
    pub projected_radius_px: f32,
    /// Viewport pixel area available to the renderable.
    pub viewport_area_px: u32,
}

/// Engine-level LOD policy contract.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum LodPolicy {
    Disabled,
    Fixed {
        level: LodLevel,
    },
    ScreenSpace {
        min_level: LodLevel,
        max_level: LodLevel,
        /// Hysteresis in pixels used by runtime/pipeline integrations.
        hysteresis_px: f32,
    },
}

impl Default for LodPolicy {
    fn default() -> Self {
        Self::Disabled
    }
}

/// Optional per-node LOD hint carried through scene/runtime/render seams.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct LodHint {
    #[serde(default)]
    pub policy: LodPolicy,
    /// Optional authored bias applied by concrete selectors.
    #[serde(default)]
    pub bias: i8,
}

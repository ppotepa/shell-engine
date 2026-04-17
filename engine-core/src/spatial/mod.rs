use serde::{Deserialize, Serialize};

/// Coordinate-system handedness for world-space math.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Handedness {
    Left,
    #[default]
    Right,
}

/// World up-axis convention used by scene/gameplay/render integrations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum UpAxis {
    #[default]
    Y,
    Z,
}

/// Axis convention for world-space integration points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxisConvention {
    #[serde(default)]
    pub handedness: Handedness,
    #[serde(default)]
    pub up_axis: UpAxis,
}

impl Default for AxisConvention {
    fn default() -> Self {
        Self {
            handedness: Handedness::Right,
            up_axis: UpAxis::Y,
        }
    }
}

/// Unified unit scale used by runtime systems.
///
/// Canonical contract:
/// - gameplay/runtime transforms use world units (`wu`)
/// - physical values use meters/kilometers
/// - renderers consume world-space values and projection policy
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialScale {
    /// Physical meters represented by one world unit.
    #[serde(default = "SpatialScale::default_meters_per_world_unit")]
    pub meters_per_world_unit: f64,
    /// Optional 2D projection helper: virtual pixels represented by one world unit.
    ///
    /// This is a presentation policy value and may be `None` for 3D-only contexts.
    #[serde(default)]
    pub virtual_pixels_per_world_unit: Option<f64>,
}

impl SpatialScale {
    const fn default_meters_per_world_unit() -> f64 {
        1.0
    }

    pub fn world_units_to_meters(self, world_units: f64) -> f64 {
        world_units * self.meters_per_world_unit
    }

    pub fn meters_to_world_units(self, meters: f64) -> f64 {
        let denom = self.meters_per_world_unit.max(f64::MIN_POSITIVE);
        meters / denom
    }

    pub fn world_units_to_kilometers(self, world_units: f64) -> f64 {
        self.world_units_to_meters(world_units) / 1000.0
    }

    pub fn kilometers_to_world_units(self, kilometers: f64) -> f64 {
        self.meters_to_world_units(kilometers * 1000.0)
    }

    pub fn world_units_to_virtual_pixels(self, world_units: f64) -> Option<f64> {
        self.virtual_pixels_per_world_unit
            .map(|ratio| world_units * ratio.max(0.0))
    }

    pub fn virtual_pixels_to_world_units(self, virtual_pixels: f64) -> Option<f64> {
        self.virtual_pixels_per_world_unit.map(|ratio| {
            let denom = ratio.max(f64::MIN_POSITIVE);
            virtual_pixels / denom
        })
    }
}

impl Default for SpatialScale {
    fn default() -> Self {
        Self {
            meters_per_world_unit: Self::default_meters_per_world_unit(),
            virtual_pixels_per_world_unit: None,
        }
    }
}

/// Scene-level spatial context for runtime and renderer seams.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct SpatialContext {
    #[serde(default)]
    pub axes: AxisConvention,
    #[serde(default)]
    pub scale: SpatialScale,
}

#[cfg(test)]
mod tests {
    use super::{SpatialContext, SpatialScale};

    #[test]
    fn default_context_uses_identity_metric_scale() {
        let ctx = SpatialContext::default();
        assert!((ctx.scale.world_units_to_meters(5.0) - 5.0).abs() < f64::EPSILON);
        assert!((ctx.scale.meters_to_world_units(5.0) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn kilometers_round_trip_through_world_units() {
        let scale = SpatialScale {
            meters_per_world_unit: 2.0,
            virtual_pixels_per_world_unit: None,
        };
        let wu = scale.kilometers_to_world_units(1.5);
        let km = scale.world_units_to_kilometers(wu);
        assert!((km - 1.5).abs() < 1e-9);
    }

    #[test]
    fn virtual_pixels_conversion_is_optional() {
        let no_ratio = SpatialScale::default();
        assert_eq!(no_ratio.world_units_to_virtual_pixels(10.0), None);

        let with_ratio = SpatialScale {
            meters_per_world_unit: 1.0,
            virtual_pixels_per_world_unit: Some(8.0),
        };
        assert_eq!(with_ratio.world_units_to_virtual_pixels(2.0), Some(16.0));
        assert_eq!(with_ratio.virtual_pixels_to_world_units(16.0), Some(2.0));
    }
}

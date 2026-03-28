pub mod animation;
pub mod builtin;
pub mod params;

pub use animation::{SpriteAnimation, Transform};
pub use params::AnimationParams;

use crate::authoring::metadata::{FieldMetadata, Requirement, TargetKind, ValueKind, ValueSource};
use crate::scene::Animation;
use std::collections::HashMap;

pub struct AnimationDispatcher {
    registry: HashMap<&'static str, Box<dyn SpriteAnimation>>,
}

impl AnimationDispatcher {
    pub fn new() -> Self {
        let mut d = Self {
            registry: HashMap::new(),
        };
        d.registry
            .insert("float", Box::new(builtin::FloatAnimation));
        d
    }

    /// Compute combined transform from all active animations for a sprite.
    /// Called frequently during sprite rendering — inline to optimize hot path.
    #[inline]
    pub fn compute_transform(&self, animations: &[Animation], elapsed_ms: u64) -> Transform {
        // Fast-path: no animations = no transform
        if animations.is_empty() {
            return Transform::default();
        }

        let mut total = Transform::default();
        for anim in animations {
            if let Some(impl_) = self.registry.get(anim.name.as_str()) {
                let t = impl_.compute(elapsed_ms, &anim.params);
                total.dx += t.dx;
                total.dy += t.dy;
            }
        }
        total
    }

    /// Returns names of all built-in animations.
    pub fn builtin_names() -> Vec<&'static str> {
        vec!["float"]
    }

    /// Returns field metadata for the given animation name.
    pub fn metadata(name: &str) -> Vec<FieldMetadata> {
        match name {
            "float" => vec![
                FieldMetadata {
                    target: TargetKind::Sprite,
                    name: "axis",
                    value_kind: ValueKind::Select,
                    requirement: Requirement::Optional,
                    description: "Axis along which animation moves",
                    default_text: Some("y"),
                    default_number: None,
                    enum_options: Some(&["x", "y"]),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Sprite,
                    name: "amplitude",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Peak displacement in terminal cells",
                    default_text: None,
                    default_number: Some(1.0),
                    enum_options: None,
                    min: Some(0.0),
                    max: Some(100.0),
                    step: Some(1.0),
                    unit: Some("cells"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Sprite,
                    name: "period_ms",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Full cycle duration",
                    default_text: None,
                    default_number: Some(2000.0),
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(10.0),
                    unit: Some("ms"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Sprite,
                    name: "easing",
                    value_kind: ValueKind::Select,
                    requirement: Requirement::Optional,
                    description: "Easing function applied to animation curve",
                    default_text: Some("linear"),
                    default_number: None,
                    enum_options: Some(&["linear", "ease-in", "ease-out", "ease-in-out"]),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authoring::catalog::animation_catalog;

    #[test]
    fn test_all_animations_in_catalog() {
        // Verify that every animation in AnimationDispatcher is present in catalog
        let runtime_animations = AnimationDispatcher::builtin_names();
        let catalog = animation_catalog();
        let catalog_names: Vec<&str> = catalog.iter().map(|(name, _)| *name).collect();

        for animation in &runtime_animations {
            assert!(
                catalog_names.contains(animation),
                "Animation '{}' is registered in runtime but missing from catalog",
                animation
            );
        }

        for catalog_name in &catalog_names {
            assert!(
                runtime_animations.contains(catalog_name),
                "Animation '{}' is in catalog but not registered in AnimationDispatcher",
                catalog_name
            );
        }

        assert_eq!(
            runtime_animations.len(),
            catalog_names.len(),
            "Mismatch between runtime animations ({}) and catalog ({})",
            runtime_animations.len(),
            catalog_names.len()
        );
    }
}

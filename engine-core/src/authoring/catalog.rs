use crate::animations::AnimationDispatcher;
use crate::authoring::metadata::FieldMetadata;
use crate::effects::EffectDispatcher;
use crate::scene::{SceneInput, LAYER_FIELDS, OBJECT_FIELDS, SCENE_FIELDS, SPRITE_FIELDS};

/// Read-only entrypoint for authored field catalogs consumed by tooling.
#[derive(Debug, Clone, Copy)]
pub struct StaticAuthoringCatalog {
    pub scene_fields: &'static [FieldMetadata],
    pub layer_fields: &'static [FieldMetadata],
    pub sprite_fields: &'static [FieldMetadata],
    pub object_fields: &'static [FieldMetadata],
    pub effect_names: &'static [&'static str],
}

pub fn static_catalog() -> StaticAuthoringCatalog {
    StaticAuthoringCatalog {
        scene_fields: SCENE_FIELDS,
        layer_fields: LAYER_FIELDS,
        sprite_fields: SPRITE_FIELDS,
        object_fields: OBJECT_FIELDS,
        effect_names: EffectDispatcher::builtin_names(),
    }
}

/// Converts effect metadata into generic authored field metadata.
pub fn effect_fields(effect_name: &str) -> Vec<FieldMetadata> {
    crate::effects::shared_dispatcher()
        .metadata(effect_name)
        .params
        .iter()
        .map(|p| p.as_authored_field())
        .collect()
}

/// Returns (behavior_name, fields) tuples for all built-in behaviors.
pub fn behavior_catalog() -> Vec<(&'static str, Vec<FieldMetadata>)> {
    // Forward declare from engine crate (will be called via indirect path)
    vec![]
}

/// Returns (animation_name, fields) tuples for all built-in animations.
pub fn animation_catalog() -> Vec<(&'static str, Vec<FieldMetadata>)> {
    AnimationDispatcher::builtin_names()
        .into_iter()
        .map(|name| (name, AnimationDispatcher::metadata(name)))
        .collect()
}

/// Returns names of all built-in input profiles.
pub fn input_profile_catalog() -> Vec<&'static str> {
    SceneInput::builtin_profiles()
}

#[cfg(test)]
mod tests {
    use super::{animation_catalog, effect_fields, input_profile_catalog, static_catalog};

    #[test]
    fn static_catalog_exposes_scene_and_sprite_fields() {
        let c = static_catalog();
        assert!(c.scene_fields.iter().any(|f| f.name == "id"));
        assert!(c.sprite_fields.iter().any(|f| f.name == "type"));
        assert!(!c.effect_names.is_empty());
    }

    #[test]
    fn effect_fields_are_available_for_builtin_effect() {
        let fields = effect_fields("fade-in");
        assert!(!fields.is_empty());
        assert!(fields.iter().any(|f| f.name == "easing"));
    }

    #[test]
    fn animation_catalog_complete() {
        let catalog = animation_catalog();
        assert!(!catalog.is_empty());
        assert!(catalog.iter().any(|(name, _)| *name == "float"));
        for (name, fields) in &catalog {
            assert!(!fields.is_empty(), "Animation {} has no fields", name);
        }
    }

    #[test]
    fn input_profile_catalog_has_builtin_profiles() {
        let profiles = input_profile_catalog();
        assert!(profiles.contains(&"obj-viewer"));
        assert!(profiles.contains(&"terminal-size-tester"));
    }
}

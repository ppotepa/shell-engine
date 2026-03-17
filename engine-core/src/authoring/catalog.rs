use crate::authoring::metadata::FieldMetadata;
use crate::effects::EffectDispatcher;
use crate::scene::{LAYER_FIELDS, OBJECT_FIELDS, SCENE_FIELDS, SPRITE_FIELDS};

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

#[cfg(test)]
mod tests {
    use super::{effect_fields, static_catalog};

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
}

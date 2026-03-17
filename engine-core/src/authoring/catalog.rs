use crate::animations::AnimationDispatcher;
use crate::authoring::metadata::FieldMetadata;
use crate::effects::EffectDispatcher;
use crate::scene::{SceneInput, LAYER_FIELDS, OBJECT_FIELDS, SCENE_FIELDS, SPRITE_FIELDS};

/// Authoring sugar: aliases, shorthands, and normalizers.
#[derive(Debug, Clone)]
pub struct SugarCatalog {
    /// Field aliases (shorthand_name, canonical_name).
    pub aliases: Vec<(&'static str, &'static str)>,
    /// Shorthand transformations (name, description, from_syntax, to_structure).
    pub shorthands: Vec<ShorthandSpec>,
    /// Normalizer function names applied during document processing.
    pub normalizers: Vec<&'static str>,
}

/// Describes one authoring shorthand transformation.
#[derive(Debug, Clone)]
pub struct ShorthandSpec {
    /// Shorthand name (e.g., "pause").
    pub name: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Example authored syntax (e.g., "pause: 1ms").
    pub from_syntax: &'static str,
    /// Resulting canonical structure (e.g., "{duration: N, effects: []}").
    pub to_structure: &'static str,
}

/// Returns catalog of all authoring sugar transformations.
pub fn sugar_catalog() -> SugarCatalog {
    SugarCatalog {
        aliases: vec![
            ("bg", "bg_colour"),
            ("fg", "fg_colour"),
        ],
        shorthands: vec![
            ShorthandSpec {
                name: "pause",
                description: "Duration-only step without effects",
                from_syntax: "pause: 1ms",
                to_structure: "{duration: 1, effects: []}",
            },
            ShorthandSpec {
                name: "at",
                description: "Alignment anchor shorthand",
                from_syntax: "at: cc",
                to_structure: "{align_x: center, align_y: center}",
            },
            ShorthandSpec {
                name: "to",
                description: "Menu option scene target",
                from_syntax: "to: main-menu",
                to_structure: "{scene: main-menu, next: main-menu}",
            },
        ],
        normalizers: vec![
            "normalize_stage",      // engine-authoring/src/document/scene.rs:60
            "normalize_layers",     // engine-authoring/src/document/scene.rs:93
            "normalize_sprites",    // engine-authoring/src/document/scene.rs:108
            "normalize_menu_options", // engine-authoring/src/document/scene.rs:134
            "apply_alias",          // engine-authoring/src/document/scene.rs:159
            "apply_at_anchor",      // engine-authoring/src/document/scene.rs:170
        ],
    }
}

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
    use super::{animation_catalog, effect_fields, input_profile_catalog, static_catalog, sugar_catalog};

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

    #[test]
    fn sugar_catalog_has_all_aliases_and_shorthands() {
        let catalog = sugar_catalog();
        
        // Check aliases
        assert!(catalog.aliases.iter().any(|(from, to)| *from == "bg" && *to == "bg_colour"));
        assert!(catalog.aliases.iter().any(|(from, to)| *from == "fg" && *to == "fg_colour"));
        
        // Check shorthands
        assert!(catalog.shorthands.iter().any(|s| s.name == "pause"));
        assert!(catalog.shorthands.iter().any(|s| s.name == "at"));
        assert!(catalog.shorthands.iter().any(|s| s.name == "to"));
        
        // Check normalizers
        assert!(catalog.normalizers.contains(&"normalize_stage"));
        assert!(catalog.normalizers.contains(&"apply_alias"));
        assert!(catalog.normalizers.contains(&"apply_at_anchor"));
    }
}

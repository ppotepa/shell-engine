//! Builtin effects catalogue sourced directly from engine-core metadata.

use engine_core::scene::EffectTargetKind;
use engine_effects::{shared_dispatcher, EffectDispatcher};

/// Renderable documentation snapshot sourced from engine-core effect metadata.
#[derive(Debug, Clone, Copy)]
pub struct EffectDoc {
    pub summary: &'static str,
    pub sample: &'static str,
    pub category: &'static str,
    pub target_kind: EffectTargetKind,
}

/// Returns a list of all built-in effect names from the engine dispatcher.
pub fn builtin_effect_names() -> Vec<String> {
    EffectDispatcher::builtin_names()
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

/// Returns documentation metadata for the named effect from the engine dispatcher.
pub fn effect_doc(name: &str) -> EffectDoc {
    let meta = shared_dispatcher().metadata(name);
    EffectDoc {
        summary: meta.summary,
        sample: meta.sample,
        category: meta.category,
        target_kind: preferred_target_kind(meta.compatible_targets),
    }
}

fn preferred_target_kind(mask: engine_effects::EffectTargetMask) -> EffectTargetKind {
    if mask.supports(EffectTargetKind::Scene) {
        EffectTargetKind::Scene
    } else if mask.supports(EffectTargetKind::Layer) {
        EffectTargetKind::Layer
    } else if mask.supports(EffectTargetKind::SpriteText) {
        EffectTargetKind::SpriteText
    } else if mask.supports(EffectTargetKind::SpriteBitmap) {
        EffectTargetKind::SpriteBitmap
    } else if mask.supports(EffectTargetKind::Sprite) {
        EffectTargetKind::Sprite
    } else {
        EffectTargetKind::Any
    }
}

#[cfg(test)]
mod tests {
    use super::effect_doc;
    use engine_effects::shared_dispatcher;

    #[test]
    fn effect_doc_is_sourced_from_engine_metadata() {
        let meta = shared_dispatcher().metadata("shine");
        let doc = effect_doc("shine");

        assert_eq!(doc.summary, meta.summary);
        assert_eq!(doc.sample, meta.sample);
        assert_eq!(doc.category, meta.category);
    }
}

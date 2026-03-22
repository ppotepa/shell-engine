//! Built-in visual effect implementations.

pub mod artifact;
pub mod blur;
pub mod brighten;
pub mod clear_to_colour;
pub mod crt_burn_in;
pub mod crt_distort;
pub mod crt_on;
pub mod crt_reflection;
pub mod crt_ruby;
pub mod crt_scan_glitch;
pub mod crt_underlay;
pub mod cutout;
pub mod devour;
pub mod fade;
pub mod fade_to_black;
pub mod glitch;
pub mod lightning;
pub mod neon_edge_glow;
pub mod posterize;
pub mod power_off;
pub mod scanlines;
pub mod shake;
pub mod shatter;
pub mod shine;
pub mod terminal_crt;
pub mod whiteout;

pub use artifact::ArtifactOutEffect;
pub use blur::BlurEffect;
pub use brighten::BrightenEffect;
pub use clear_to_colour::ClearToColourEffect;
pub use crt_burn_in::CrtBurnInEffect;
pub use crt_distort::CtrDistortEffect;
pub use crt_on::CrtOnEffect;
pub use crt_reflection::CrtReflectionEffect;
pub use crt_ruby::CtrRubyEffect;
pub use crt_scan_glitch::CtrScanGlitchEffect;
pub use crt_underlay::CtrUnderlayEffect;
pub use cutout::CutoutEffect;
pub use devour::DevourOutEffect;
pub use fade::{FadeInEffect, FadeOutEffect};
pub use fade_to_black::FadeToBlackEffect;
pub use glitch::GlitchOutEffect;
pub use lightning::{
    LightningAmbientEffect, LightningBranchEffect, LightningFbmEffect, LightningFlashEffect,
    LightningGrowthEffect, LightningNaturalEffect, LightningOptical80sEffect, TeslaOrbEffect,
};
pub use neon_edge_glow::NeonEdgeGlowEffect;
pub use posterize::PosterizeEffect;
pub use power_off::PowerOffEffect;
pub use scanlines::ScanlinesEffect;
pub use shake::ScreenShakeEffect;
pub use shatter::ShatterGlitchEffect;
pub use shine::ShineEffect;
pub use terminal_crt::TerminalCrtEffect;
pub use whiteout::WhiteoutEffect;

use std::sync::OnceLock;

pub type EffectConstructor = fn() -> Box<dyn super::Effect>;

pub struct BuiltinEffectDefinition {
    pub name: &'static str,
    pub constructor: EffectConstructor,
}

pub static BUILTIN_EFFECTS: &[BuiltinEffectDefinition] = &[
    BuiltinEffectDefinition {
        name: "artifact-out",
        constructor: || Box::new(ArtifactOutEffect),
    },
    BuiltinEffectDefinition {
        name: "blur",
        constructor: || Box::new(BlurEffect),
    },
    BuiltinEffectDefinition {
        name: "brighten",
        constructor: || Box::new(BrightenEffect),
    },
    BuiltinEffectDefinition {
        name: "clear-to-colour",
        constructor: || Box::new(ClearToColourEffect),
    },
    BuiltinEffectDefinition {
        name: "crt-on",
        constructor: || Box::new(CrtOnEffect),
    },
    BuiltinEffectDefinition {
        name: "crt-burn-in",
        constructor: || Box::new(CrtBurnInEffect),
    },
    BuiltinEffectDefinition {
        name: "crt-underlay",
        constructor: || Box::new(CtrUnderlayEffect),
    },
    BuiltinEffectDefinition {
        name: "crt-distort",
        constructor: || Box::new(CtrDistortEffect),
    },
    BuiltinEffectDefinition {
        name: "crt-scan-glitch",
        constructor: || Box::new(CtrScanGlitchEffect),
    },
    BuiltinEffectDefinition {
        name: "crt-ruby",
        constructor: || Box::new(CtrRubyEffect),
    },
    BuiltinEffectDefinition {
        name: "cutout",
        constructor: || Box::new(CutoutEffect),
    },
    BuiltinEffectDefinition {
        name: "crt-reflection",
        constructor: || Box::new(CrtReflectionEffect),
    },
    BuiltinEffectDefinition {
        name: "devour-out",
        constructor: || Box::new(DevourOutEffect),
    },
    BuiltinEffectDefinition {
        name: "fade-in",
        constructor: || Box::new(FadeInEffect),
    },
    BuiltinEffectDefinition {
        name: "fade-out",
        constructor: || Box::new(FadeOutEffect),
    },
    BuiltinEffectDefinition {
        name: "fade-to-black",
        constructor: || Box::new(FadeToBlackEffect),
    },
    BuiltinEffectDefinition {
        name: "glitch-out",
        constructor: || Box::new(GlitchOutEffect),
    },
    BuiltinEffectDefinition {
        name: "lightning-ambient",
        constructor: || Box::new(LightningAmbientEffect),
    },
    BuiltinEffectDefinition {
        name: "lightning-branch",
        constructor: || Box::new(LightningBranchEffect),
    },
    BuiltinEffectDefinition {
        name: "lightning-fbm",
        constructor: || Box::new(LightningFbmEffect),
    },
    BuiltinEffectDefinition {
        name: "lightning-flash",
        constructor: || Box::new(LightningFlashEffect),
    },
    BuiltinEffectDefinition {
        name: "lightning-growth",
        constructor: || Box::new(LightningGrowthEffect),
    },
    BuiltinEffectDefinition {
        name: "lightning-natural",
        constructor: || Box::new(LightningNaturalEffect),
    },
    BuiltinEffectDefinition {
        name: "lightning-optical-80s",
        constructor: || Box::new(LightningOptical80sEffect),
    },
    BuiltinEffectDefinition {
        name: "neon-edge-glow",
        constructor: || Box::new(NeonEdgeGlowEffect),
    },
    BuiltinEffectDefinition {
        name: "posterize",
        constructor: || Box::new(PosterizeEffect),
    },
    BuiltinEffectDefinition {
        name: "power-off",
        constructor: || Box::new(PowerOffEffect),
    },
    BuiltinEffectDefinition {
        name: "scanlines",
        constructor: || Box::new(ScanlinesEffect),
    },
    BuiltinEffectDefinition {
        name: "screen-shake",
        constructor: || Box::new(ScreenShakeEffect),
    },
    BuiltinEffectDefinition {
        name: "shatter-glitch",
        constructor: || Box::new(ShatterGlitchEffect),
    },
    BuiltinEffectDefinition {
        name: "shine",
        constructor: || Box::new(ShineEffect),
    },
    BuiltinEffectDefinition {
        name: "tesla-orb",
        constructor: || Box::new(TeslaOrbEffect),
    },
    BuiltinEffectDefinition {
        name: "terminal-crt",
        constructor: || Box::new(TerminalCrtEffect),
    },
    BuiltinEffectDefinition {
        name: "whiteout",
        constructor: || Box::new(WhiteoutEffect),
    },
];

static BUILTIN_EFFECT_NAMES: OnceLock<Box<[&'static str]>> = OnceLock::new();

pub fn builtin_names() -> &'static [&'static str] {
    BUILTIN_EFFECT_NAMES
        .get_or_init(|| {
            BUILTIN_EFFECTS
                .iter()
                .map(|def| def.name)
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
        .as_ref()
}

#[cfg(test)]
mod tests {
    use super::{builtin_names, BUILTIN_EFFECTS};
    use std::collections::BTreeSet;

    #[test]
    fn builtin_effect_names_are_unique() {
        let unique: BTreeSet<_> = builtin_names().iter().copied().collect();
        assert_eq!(unique.len(), builtin_names().len());
    }

    #[test]
    fn builtin_effect_keys_match_metadata_names() {
        for def in BUILTIN_EFFECTS {
            let effect = (def.constructor)();
            assert_eq!(effect.metadata().name, def.name);
        }
    }
}

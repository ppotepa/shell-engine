//! Built-in visual effect implementations.

pub mod artifact;
pub mod brighten;
pub mod clear_to_colour;
pub mod crt_on;
pub mod crt_reflection;
pub mod devour;
pub mod fade;
pub mod fade_to_black;
pub mod glitch;
pub mod lightning;
pub mod power_off;
pub mod scanlines;
pub mod shake;
pub mod shatter;
pub mod shine;
pub mod whiteout;

pub use artifact::ArtifactOutEffect;
pub use brighten::BrightenEffect;
pub use clear_to_colour::ClearToColourEffect;
pub use crt_on::CrtOnEffect;
pub use crt_reflection::CrtReflectionEffect;
pub use devour::DevourOutEffect;
pub use fade::{FadeInEffect, FadeOutEffect};
pub use fade_to_black::FadeToBlackEffect;
pub use glitch::GlitchOutEffect;
pub use lightning::{
    LightningAmbientEffect, LightningBranchEffect, LightningFbmEffect, LightningFlashEffect,
    LightningGrowthEffect, LightningNaturalEffect, LightningOptical80sEffect, TeslaOrbEffect,
};
pub use power_off::PowerOffEffect;
pub use scanlines::ScanlinesEffect;
pub use shake::ScreenShakeEffect;
pub use shatter::ShatterGlitchEffect;
pub use shine::ShineEffect;
pub use whiteout::WhiteoutEffect;

use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::effect::{Effect, Region};
use crate::effects::metadata::{EffectMetadata, ParamControl, ParamMetadata, P_EASING};
use crate::scene::EffectParams;
use crossterm::style::Color;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "clear-to-colour",
    display_name: "Clear to Colour",
    summary: "Clears region to a target terminal colour.",
    category: "colour",
    compatible_targets: crate::effects::effect::EffectTargetMask::ANY,
    params: &[
        ParamMetadata {
            name: "colour",
            label: "Colour",
            description: "Target colour (name or #rrggbb).",
            control: ParamControl::Colour { default: "black" },
        },
        P_EASING,
    ],
    sample: "- name: clear-to-colour\n  duration: 500\n  params:\n    colour: black",
};

pub struct ClearToColourEffect;

impl Effect for ClearToColourEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if progress < 1.0 {
            return;
        }
        let target = params
            .colour
            .as_ref()
            .map(Color::from)
            .unwrap_or(TRUE_BLACK);
        for dy in 0..region.height {
            for dx in 0..region.width {
                buffer.set(region.x + dx, region.y + dy, ' ', TRUE_BLACK, target);
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}

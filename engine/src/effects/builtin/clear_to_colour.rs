use crossterm::style::Color;
use crate::buffer::{Buffer, TRUE_BLACK};
use crate::scene::EffectParams;
use crate::effects::effect::{Effect, Region};

pub struct ClearToColourEffect;

impl Effect for ClearToColourEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if progress < 1.0 { return; }
        let target = params.colour.as_ref().map(Color::from).unwrap_or(TRUE_BLACK);
        for dy in 0..region.height {
            for dx in 0..region.width {
                buffer.set(region.x + dx, region.y + dy, ' ', TRUE_BLACK, target);
            }
        }
    }
}

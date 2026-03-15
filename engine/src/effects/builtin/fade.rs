use crate::buffer::{Buffer, TRUE_BLACK};
use crate::scene::EffectParams;
use crate::effects::effect::{Effect, Region};
use crate::effects::utils::color::lerp_colour;

pub struct FadeInEffect;

impl Effect for FadeInEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        let p = progress.clamp(0.0, 1.0);
        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                if let Some(cell) = buffer.get(x, y) {
                    let symbol = cell.symbol;
                    if symbol == ' ' { continue; }
                    let target_fg = cell.fg;
                    let original_bg = cell.bg;
                    let fg = lerp_colour(TRUE_BLACK, target_fg, p);
                    buffer.set(x, y, symbol, fg, original_bg);
                }
            }
        }
    }
}

pub struct FadeOutEffect;

impl Effect for FadeOutEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        let p = progress.clamp(0.0, 1.0);
        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                if let Some(cell) = buffer.get(x, y) {
                    let symbol = cell.symbol;
                    if symbol == ' ' { continue; }
                    if p >= 0.999 {
                        buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                        continue;
                    }
                    let fg = lerp_colour(cell.fg, TRUE_BLACK, p);
                    buffer.set(x, y, symbol, fg, TRUE_BLACK);
                }
            }
        }
    }
}

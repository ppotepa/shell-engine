use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::effect::{Effect, Region};
use crate::scene::EffectParams;

pub struct ScanlinesEffect;

impl Effect for ScanlinesEffect {
    fn apply(&self, _progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        for dy in (0..region.height).step_by(2) {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                if let Some(cell) = buffer.get(x, y) {
                    let symbol = cell.symbol;
                    let fg = cell.fg;
                    buffer.set(x, y, symbol, fg, TRUE_BLACK);
                }
            }
        }
    }
}

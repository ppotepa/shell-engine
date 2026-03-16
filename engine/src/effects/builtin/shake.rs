use crate::buffer::{Buffer, Cell, TRUE_BLACK};
use crate::effects::effect::{Effect, Region};
use crate::scene::EffectParams;

/// Global-style camera shake implemented as region shift.
/// Best used as a scene effect on full-screen region.
pub struct ScreenShakeEffect;

impl Effect for ScreenShakeEffect {
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let ax = params.amplitude_x.unwrap_or(2.0).max(0.0);
        let ay = params.amplitude_y.unwrap_or(1.0).max(0.0);
        let freq = params.frequency.unwrap_or(22.0).max(0.1);
        let phase = progress * std::f32::consts::TAU * freq;
        let dx = (phase.sin() * ax).round() as i32;
        let dy = ((phase * 1.37).cos() * ay).round() as i32;

        let mut snapshot: Vec<Cell> = Vec::with_capacity((region.width * region.height) as usize);
        for ry in 0..region.height {
            for rx in 0..region.width {
                let x = region.x + rx;
                let y = region.y + ry;
                snapshot.push(buffer.get(x, y).cloned().unwrap_or_default());
            }
        }

        let w = region.width as i32;
        let h = region.height as i32;
        for ry in 0..region.height {
            for rx in 0..region.width {
                let x = region.x + rx;
                let y = region.y + ry;
                let sx = rx as i32 - dx;
                let sy = ry as i32 - dy;
                if sx >= 0 && sx < w && sy >= 0 && sy < h {
                    let idx = sy as usize * region.width as usize + sx as usize;
                    let c = &snapshot[idx];
                    buffer.set(x, y, c.symbol, c.fg, c.bg);
                } else {
                    buffer.set(x, y, ' ', TRUE_BLACK, TRUE_BLACK);
                }
            }
        }
    }
}

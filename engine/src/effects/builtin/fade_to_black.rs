use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::effect::{Effect, Region};
use crate::effects::utils::math::smoothstep;
use crate::scene::EffectParams;
use crossterm::style::Color;

pub struct FadeToBlackEffect;

impl Effect for FadeToBlackEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        let t = smoothstep(progress);
        let cx = region.x as f32 + region.width as f32 * 0.5;
        let cy = region.y as f32 + region.height as f32 * 0.5;
        let max_dx = region.width as f32 * 0.5;
        let max_dy = region.height as f32 * 0.5;
        let max_dist = (max_dx * max_dx + max_dy * max_dy).sqrt().max(1.0);
        let feather = 0.18_f32;

        for dy in 0..region.height {
            for dx in 0..region.width {
                let x = region.x + dx;
                let y = region.y + dy;
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let dist = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
                let dist_n = (dist / max_dist).clamp(0.0, 1.0);

                let radius = (1.0 - t).clamp(0.0, 1.0);
                let edge = ((dist_n - radius) / feather).clamp(0.0, 1.0);
                let edge_mix = smoothstep(edge);
                let brightness = ((1.0 - t) * (1.0 - edge_mix)).clamp(0.0, 1.0);

                let (ch, fg, bg) = if brightness > 0.75 {
                    ('█', Color::White, Color::White)
                } else if brightness > 0.50 {
                    ('█', Color::Grey, Color::Grey)
                } else if brightness > 0.25 {
                    ('▒', Color::DarkGrey, TRUE_BLACK)
                } else {
                    (' ', TRUE_BLACK, TRUE_BLACK)
                };
                buffer.set(x, y, ch, fg, bg);
            }
        }
    }
}

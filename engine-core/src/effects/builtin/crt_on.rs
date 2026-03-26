//! Effect simulating a CRT monitor powering on with phosphor-like reveal phases.

use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::effect::{Effect, EffectTargetMask, Region};
use crate::effects::metadata::{EffectMetadata, P_EASING};
use crate::effects::utils::math::{
    phase_progress, smoothstep, PHASE_BOOT, PHASE_POWER_ON, PHASE_SCAN_END, PHASE_SCAN_START,
    PHASE_WHITE_FLASH,
};
use crate::effects::utils::noise::crt_hash;
use crate::scene::EffectParams;
use crate::color::Color;

/// Static effect metadata exposed to the editor and effect registry.
pub static METADATA: EffectMetadata = EffectMetadata {
    name: "crt-on",
    display_name: "CRT On",
    summary: "CRT startup sweep with phosphor-like reveal.",
    category: "crt",
    compatible_targets: EffectTargetMask::SCENE,
    params: &[P_EASING],
    sample: "- name: crt-on\n  duration: 900\n  params:\n    easing: ease-out",
};

/// Effect that plays a multi-phase CRT startup animation: boot line → scanline expand → white flash → reveal.
pub struct CrtOnEffect;

impl Effect for CrtOnEffect {
    fn apply(&self, progress: f32, _params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.height == 0 || region.width == 0 {
            return;
        }

        let centre_y = region.y + region.height / 2;
        let max_w = region.width as f32;
        let max_x = region.x + region.width.saturating_sub(1);
        let max_y = region.y + region.height.saturating_sub(1);

        if progress >= PHASE_POWER_ON {
            return;
        }

        if progress >= PHASE_WHITE_FLASH {
            let t = smoothstep(phase_progress(progress, PHASE_WHITE_FLASH, PHASE_POWER_ON));
            let (ch, fg, bg) = if t < 0.33 {
                ('█', Color::White, Color::White)
            } else if t < 0.66 {
                ('█', Color::Grey, Color::Grey)
            } else {
                (' ', TRUE_BLACK, TRUE_BLACK)
            };
            for dy in 0..region.height {
                for dx in 0..region.width {
                    buffer.set(region.x + dx, region.y + dy, ch, fg, bg);
                }
            }
            return;
        }

        if progress >= PHASE_SCAN_END {
            for dy in 0..region.height {
                for dx in 0..region.width {
                    buffer.set(
                        region.x + dx,
                        region.y + dy,
                        '█',
                        Color::White,
                        Color::White,
                    );
                }
            }
            return;
        }

        if progress >= PHASE_SCAN_START {
            let v = smoothstep(phase_progress(progress, PHASE_SCAN_START, PHASE_SCAN_END));
            let half = ((region.height as f32 * 0.5) * v).max(1.0) as u16;
            let start_y = centre_y.saturating_sub(half).max(region.y);
            let end_y = centre_y.saturating_add(half).min(max_y);
            for y in start_y..=end_y {
                for x in region.x..=max_x {
                    buffer.set(x, y, '█', Color::White, Color::White);
                }
            }
            return;
        }

        if progress >= PHASE_BOOT {
            let t = smoothstep(phase_progress(progress, PHASE_BOOT, PHASE_SCAN_START));
            let line_len = (max_w * (0.35 + 0.65 * t)).max(1.0) as u16;
            let line_h = (1.0 + t * 2.0) as u16;
            let fg = if t < 0.45 { Color::Grey } else { Color::White };
            let bg = if t < 0.45 { TRUE_BLACK } else { Color::White };
            let end_x = region.x + line_len.saturating_sub(1);
            let line_start_y = centre_y.saturating_sub(line_h / 2).max(region.y);
            let line_end_y = centre_y
                .saturating_add(line_h.saturating_sub(1) / 2)
                .min(max_y);
            for y in line_start_y..=line_end_y {
                for x in region.x..=end_x.min(max_x) {
                    buffer.set(x, y, '█', fg, bg);
                }
            }
            return;
        }

        // Initial centre line: linear, grey, materializes from left to right.
        let t = phase_progress(progress, 0.0, PHASE_BOOT);
        let line_len = (max_w * t).max(1.0) as u16;
        let end_x = region.x + line_len.saturating_sub(1);
        for x in region.x..=end_x.min(max_x) {
            buffer.set(x, centre_y, '─', Color::DarkGrey, TRUE_BLACK);
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}

/// Fills every cell in `y` with blank/black (used to erase collapsed rows).
pub(super) fn crt_blank_row(region: Region, y: u16, buffer: &mut Buffer) {
    for dx in 0..region.width {
        buffer.set(region.x + dx, y, ' ', TRUE_BLACK, TRUE_BLACK);
    }
}

/// Renders a noisy static row using a per-cell hash seeded by `seed`.
pub(super) fn crt_static_row(region: Region, y: u16, fg: Color, seed: u32, buffer: &mut Buffer) {
    const CHARS: &[char] = &['░', '▒', '▓', '╌', '┄', '▪', '▫', '·', '╍', '▒', '░'];
    for dx in 0..region.width {
        let s = crt_hash(region.x + dx, y, seed);
        let ch = CHARS[s as usize % CHARS.len()];
        buffer.set(region.x + dx, y, ch, fg, TRUE_BLACK);
    }
}

/// Renders a dim, sparse static row used for the trailing edge of the collapse.
pub(super) fn crt_dim_row(region: Region, y: u16, seed: u32, buffer: &mut Buffer) {
    const CHARS: &[char] = &['░', ' ', ' ', ' ', '·', ' ', '·', ' ', '░'];
    for dx in 0..region.width {
        let s = crt_hash(region.x + dx, y, seed);
        let ch = CHARS[s as usize % CHARS.len()];
        buffer.set(region.x + dx, y, ch, Color::DarkGrey, TRUE_BLACK);
    }
}

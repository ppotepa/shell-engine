//! Box blur effect — softens terminal cell colours by averaging a neighbourhood.
//!
//! Works on any target. Progress is intentionally ignored so the blur is always
//! applied at full `radius` strength; use `duration: 0` or loop the step for a
//! persistent static blur.

use crate::buffer::{Buffer, TRUE_BLACK};
use crate::effects::effect::{Effect, EffectTargetMask, Region};
use crate::effects::metadata::{slider, EffectMetadata, P_EASING};
use crate::effects::utils::color::colour_to_rgb;
use crate::scene::EffectParams;
use crate::color::Color;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "blur",
    display_name: "Blur",
    summary: "Box blur: softens terminal cell colours by averaging a neighbourhood of pixels.",
    category: "colour",
    compatible_targets: EffectTargetMask::ANY,
    params: &[
        slider(
            "radius",
            "Radius",
            "Blur kernel half-size in cells. 1 = 3×3 neighbourhood, 2 = 5×5, etc.",
            1.0,
            6.0,
            0.5,
            "",
        ),
        P_EASING,
    ],
    sample: "- name: blur\n  duration: 0\n  params:\n    radius: 1",
};

pub struct BlurEffect;

impl Effect for BlurEffect {
    /// `progress` is ignored — blur is always applied at full `radius` strength.
    fn apply(&self, _progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer) {
        if region.width == 0 || region.height == 0 {
            return;
        }

        let radius = params.radius.unwrap_or(1.0).clamp(1.0, 8.0).round() as i32;
        let w = region.width as i32;
        let h = region.height as i32;

        // Snapshot the region before any writes.
        let snapshot: Vec<_> = (0..h)
            .flat_map(|dy| (0..w).map(move |dx| (dy, dx)))
            .map(|(dy, dx)| {
                buffer
                    .get(region.x + dx as u16, region.y + dy as u16)
                    .cloned()
                    .unwrap_or_default()
            })
            .collect();

        for dy in 0..h {
            for dx in 0..w {
                let center = &snapshot[dy as usize * w as usize + dx as usize];
                // Don't write anything into fully transparent cells.
                if center.symbol == ' ' && matches!(center.bg, Color::Reset) {
                    continue;
                }

                let mut fg_r = 0u32;
                let mut fg_g = 0u32;
                let mut fg_b = 0u32;
                let mut bg_r = 0u32;
                let mut bg_g = 0u32;
                let mut bg_b = 0u32;
                let mut count = 0u32;

                for ky in (dy - radius).max(0)..(dy + radius + 1).min(h) {
                    for kx in (dx - radius).max(0)..(dx + radius + 1).min(w) {
                        let cell = &snapshot[ky as usize * w as usize + kx as usize];
                        let (fr, fg, fb) = colour_to_rgb(cell.fg);
                        fg_r += fr as u32;
                        fg_g += fg as u32;
                        fg_b += fb as u32;
                        let bg = if matches!(cell.bg, Color::Reset) {
                            TRUE_BLACK
                        } else {
                            cell.bg
                        };
                        let (br, bgr_ch, bb) = colour_to_rgb(bg);
                        bg_r += br as u32;
                        bg_g += bgr_ch as u32;
                        bg_b += bb as u32;
                        count += 1;
                    }
                }

                if count > 0 {
                    let avg_fg = Color::Rgb {
                        r: (fg_r / count) as u8,
                        g: (fg_g / count) as u8,
                        b: (fg_b / count) as u8,
                    };
                    let avg_bg = Color::Rgb {
                        r: (bg_r / count) as u8,
                        g: (bg_g / count) as u8,
                        b: (bg_b / count) as u8,
                    };
                    buffer.set(
                        region.x + dx as u16,
                        region.y + dy as u16,
                        center.symbol,
                        avg_fg,
                        avg_bg,
                    );
                }
            }
        }
    }

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}

#[cfg(test)]
mod tests {
    use super::{BlurEffect, METADATA};
    use crate::buffer::{Buffer, TRUE_BLACK};
    use crate::effects::effect::{Effect, EffectTargetMask, Region};
    use crate::scene::EffectParams;
    use crate::color::Color;

    #[test]
    fn metadata_is_any_target() {
        assert_eq!(METADATA.compatible_targets, EffectTargetMask::ANY);
    }

    #[test]
    fn blur_averages_neighbouring_cell_colours() {
        let mut buf = Buffer::new(5, 1);
        buf.fill(TRUE_BLACK);
        // Left cell: bright red; right cells: black.
        buf.set(0, 0, '█', Color::Rgb { r: 200, g: 0, b: 0 }, TRUE_BLACK);
        buf.set(1, 0, '█', Color::Rgb { r: 0, g: 0, b: 0 }, TRUE_BLACK);
        buf.set(2, 0, '█', Color::Rgb { r: 0, g: 0, b: 0 }, TRUE_BLACK);

        BlurEffect.apply(
            1.0,
            &EffectParams {
                radius: Some(1.0),
                ..EffectParams::default()
            },
            Region {
                x: 0,
                y: 0,
                width: 3,
                height: 1,
            },
            &mut buf,
        );

        // Cell 0 was averaged with cell 1 only (radius=1, no left neighbour).
        // Expected fg.r = (200 + 0) / 2 = 100.
        let cell0 = buf.get(0, 0).expect("cell 0");
        assert!(
            matches!(cell0.fg, Color::Rgb { r, .. } if r > 0 && r < 200),
            "cell 0 fg should be blurred toward 0: {:?}",
            cell0.fg
        );
        // Cell 2 (pure black) should be softened by cell 1 and cell 3 (both black) → still black.
        let cell2 = buf.get(2, 0).expect("cell 2");
        assert_eq!(cell2.fg, Color::Rgb { r: 0, g: 0, b: 0 });
    }

    #[test]
    fn transparent_cells_are_not_overwritten() {
        let mut buf = Buffer::new(3, 1);
        buf.fill(Color::Reset);
        // Transparent center: symbol ' ', bg Reset.
        buf.set(0, 0, '█', Color::Rgb { r: 100, g: 0, b: 0 }, TRUE_BLACK);

        let before_center = buf.get(1, 0).cloned().expect("center before");
        BlurEffect.apply(
            1.0,
            &EffectParams {
                radius: Some(1.0),
                ..EffectParams::default()
            },
            Region {
                x: 0,
                y: 0,
                width: 3,
                height: 1,
            },
            &mut buf,
        );
        assert_eq!(buf.get(1, 0).cloned().expect("center after"), before_center);
    }
}

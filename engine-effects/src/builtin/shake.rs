use crate::metadata::{slider, EffectMetadata, P_EASING};
use engine_core::buffer::{Buffer, Cell, TRUE_BLACK};
use engine_core::effects::{Effect, EffectTargetMask, Region};
use engine_core::scene::EffectParams;

pub static METADATA: EffectMetadata = EffectMetadata {
    name: "screen-shake",
    display_name: "Screen Shake",
    summary: "Camera-like shake offsetting rendered output.",
    category: "motion",
    compatible_targets: EffectTargetMask::SCENE.union(EffectTargetMask::LAYER),
    params: &[
        slider("amplitude_x", "X Amplitude", "Horizontal shake in cells.", 0.0, 3.0, 0.1, ""),
        slider("amplitude_y", "Y Amplitude", "Vertical shake in cells.", 0.0, 3.0, 0.1, ""),
        slider("frequency", "Frequency", "Oscillation cycles during effect.", 0.0, 20.0, 0.5, ""),
        P_EASING,
    ],
    sample: "- name: screen-shake\n  duration: 260\n  params:\n    amplitude_x: 1.2\n    amplitude_y: 0.4\n    frequency: 8.0",
};

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

        let snapshot_capacity = usize::from(region.width) * usize::from(region.height);
        let mut snapshot: Vec<Cell> = Vec::with_capacity(snapshot_capacity);
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

    fn metadata(&self) -> &'static EffectMetadata {
        &METADATA
    }
}

#[cfg(test)]
mod tests {
    use super::ScreenShakeEffect;
    use engine_core::buffer::Buffer;
    use engine_core::effects::{Effect, Region};
    use engine_core::scene::EffectParams;

    #[test]
    fn does_not_overflow_snapshot_capacity_on_large_regions() {
        let mut buffer = Buffer::new(369, 186);
        let effect = ScreenShakeEffect;
        let region = Region::full(&buffer);
        effect.apply(0.5, &EffectParams::default(), region, &mut buffer);
    }
}

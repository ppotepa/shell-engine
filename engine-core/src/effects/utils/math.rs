//! Math helpers and CRT-phase progress constants for visual effect calculations.

/// Renderer tick interval in milliseconds (~60 fps).
pub const TICK_MS: u64 = 16;
/// Normalised progress at which the CRT power-on sequence completes.
pub const PHASE_POWER_ON: f32 = 0.92;
/// Normalised progress at which the white-flash sub-phase begins.
pub const PHASE_WHITE_FLASH: f32 = 0.78;
/// Normalised progress at which the scanline-expand phase ends.
pub const PHASE_SCAN_END: f32 = 0.60;
/// Normalised progress at which the scanline-expand phase begins.
pub const PHASE_SCAN_START: f32 = 0.35;
/// Normalised progress at which the initial boot-line phase ends.
pub const PHASE_BOOT: f32 = 0.20;

/// Applies the smoothstep curve to `t`, clamped to [0, 1].
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Maps `progress` from the sub-interval \[`start`, `end`\] to \[0, 1\], clamped.
pub fn phase_progress(progress: f32, start: f32, end: f32) -> f32 {
    if end <= start {
        return 1.0;
    }
    ((progress - start) / (end - start)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoothstep_clamps_bounds() {
        assert_eq!(smoothstep(-0.5), 0.0);
        assert_eq!(smoothstep(1.5), 1.0);
    }

    #[test]
    fn phase_progress_maps_interval() {
        assert_eq!(phase_progress(0.5, 0.0, 1.0), 0.5);
        assert_eq!(phase_progress(0.2, 0.3, 0.8), 0.0);
        assert_eq!(phase_progress(0.9, 0.3, 0.8), 1.0);
    }
}

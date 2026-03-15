pub const TICK_MS: u64 = 16;
pub const PHASE_POWER_ON: f32 = 0.92;
pub const PHASE_WHITE_FLASH: f32 = 0.78;
pub const PHASE_SCAN_END: f32 = 0.60;
pub const PHASE_SCAN_START: f32 = 0.35;
pub const PHASE_BOOT: f32 = 0.20;

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn phase_progress(progress: f32, start: f32, end: f32) -> f32 {
    if end <= start {
        return 1.0;
    }
    ((progress - start) / (end - start)).clamp(0.0, 1.0)
}

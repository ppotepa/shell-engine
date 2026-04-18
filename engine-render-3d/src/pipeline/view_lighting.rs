use engine_core::scene::{ResolvedViewProfile, TonemapOperator};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewLightingParams {
    pub ambient_floor: f32,
    pub exposure: f32,
    pub gamma: f32,
    pub tonemap: TonemapOperator,
    pub shadow_contrast: f32,
    pub night_glow_scale: f32,
    pub haze_night_leak: f32,
}

pub fn resolve_view_lighting(view: &ResolvedViewProfile) -> ViewLightingParams {
    let lighting = &view.lighting;
    ViewLightingParams {
        ambient_floor: lighting.black_level.unwrap_or(0.06),
        exposure: lighting.exposure.unwrap_or(1.0).max(0.0),
        gamma: lighting.gamma.unwrap_or(2.2).clamp(0.1, 4.0),
        tonemap: lighting.tonemap.unwrap_or(TonemapOperator::Linear),
        shadow_contrast: lighting.shadow_contrast.unwrap_or(1.0).clamp(0.25, 4.0),
        night_glow_scale: lighting.night_glow_scale.unwrap_or(1.0).clamp(0.0, 2.0),
        haze_night_leak: lighting.haze_night_leak.unwrap_or(0.0).clamp(0.0, 1.0),
    }
}

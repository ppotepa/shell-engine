use engine_core::render_types::{Camera3DState, ViewportRect};

use crate::frame_input::Render3dFrameInput;
use crate::frame_profiles::{
    FrameAtmosphereProfile, FrameEnvironmentProfile, FrameGeometry3D, FrameLightingProfile,
    FramePostProcessProfile, FrameSurfaceProfile,
};

#[derive(Debug, Clone)]
pub struct RenderPassContext {
    pub viewport: ViewportRect,
    pub camera: Camera3DState,
    pub geometry: FrameGeometry3D,
    pub surface: FrameSurfaceProfile,
    pub atmosphere: FrameAtmosphereProfile,
    pub lighting: FrameLightingProfile,
    pub environment: FrameEnvironmentProfile,
    pub postprocess: FramePostProcessProfile,
    pub frame_time_ms: u64,
}

impl From<&Render3dFrameInput> for RenderPassContext {
    fn from(input: &Render3dFrameInput) -> Self {
        Self {
            viewport: input.viewport,
            camera: input.camera,
            geometry: input.geometry.clone(),
            surface: input.surface.clone(),
            atmosphere: input.atmosphere.clone(),
            lighting: input.lighting.clone(),
            environment: input.environment.clone(),
            postprocess: input.postprocess.clone(),
            frame_time_ms: input.frame_time_ms,
        }
    }
}

use engine_core::render_types::{Camera3DState, ViewportRect};

use crate::frame_profiles::{
    FrameAtmosphereProfile, FrameEnvironmentProfile, FrameGeometry3D, FrameLightingProfile,
    FramePostProcessProfile, FrameSurfaceProfile,
};
use crate::pipeline::generated_world_renderer::GeneratedWorldRenderProfile;
use crate::ObjRenderParams;

#[derive(Debug, Clone)]
pub struct Render3dFrameInput {
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

impl Render3dFrameInput {
    pub fn from_obj_params(
        viewport: ViewportRect,
        camera: Camera3DState,
        geometry: FrameGeometry3D,
        params: &ObjRenderParams,
        frame_time_ms: u64,
    ) -> Self {
        Self {
            viewport,
            camera,
            geometry,
            surface: FrameSurfaceProfile::from_obj_params(params),
            atmosphere: FrameAtmosphereProfile::from_obj_params(params),
            lighting: FrameLightingProfile::from_obj_params(params),
            environment: FrameEnvironmentProfile::default(),
            postprocess: FramePostProcessProfile::from_obj_params(params),
            frame_time_ms,
        }
    }

    pub fn from_generated_world_profile(
        viewport: ViewportRect,
        camera: Camera3DState,
        geometry: FrameGeometry3D,
        profile: &GeneratedWorldRenderProfile,
        frame_time_ms: u64,
    ) -> Self {
        Self {
            viewport,
            camera,
            geometry,
            surface: FrameSurfaceProfile::from_generated_world_profile(profile),
            atmosphere: FrameAtmosphereProfile::from_generated_world_profile(profile),
            lighting: FrameLightingProfile::from_generated_world_profile(profile),
            environment: FrameEnvironmentProfile::from_generated_world_profile(profile),
            postprocess: FramePostProcessProfile::from_generated_world_profile(profile),
            frame_time_ms,
        }
    }
}

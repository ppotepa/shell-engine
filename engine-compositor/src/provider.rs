//! CompositorProvider trait — decouples compositor system from engine's World type.

use std::any::Any;
use std::collections::HashMap;

use crate::ObjPrerenderedFrames;
use engine_core::effects::Region;
use engine_core::scene::ResolvedViewProfile;
use engine_core::scene_runtime_types::{
    ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver,
};
use engine_core::spatial::SpatialContext;
use engine_render_2d::{Render2dInput, Render2dPipeline};

#[cfg(feature = "render-3d")]
use crate::generated_world_render_adapter::render_generated_world_sprite as render_generated_world_sprite_adapter;
#[cfg(feature = "render-3d")]
use crate::obj_render_adapter::render_obj_sprite as render_obj_sprite_adapter;
#[cfg(feature = "render-3d")]
use crate::scene_clip_render_adapter::render_scene_clip_sprite as render_scene_clip_sprite_adapter;
use crate::sprite_renderer_2d::render_sprites;
#[cfg(feature = "render-3d")]
use crate::sprite_renderer_2d::{Render3dDelegate, Render3dNodeSpec};

/// Provides access to compositor-needed resources from World.
pub trait CompositorProvider {
    fn buffer_mut(&mut self) -> Option<&mut dyn Any>;
    fn scene_runtime(&self) -> Option<&dyn Any>;
    fn animator(&self) -> Option<&dyn Any>;
    fn asset_root(&self) -> Option<&dyn Any>;
    fn runtime_settings(&self) -> Option<&dyn Any>;
    fn debug_features(&self) -> Option<&dyn Any>;
    fn render_2d_pipeline(&self) -> Option<&dyn Render2dPipeline> {
        None
    }
}

pub(crate) enum ResolvedRender2dPipeline<'a> {
    Provided(&'a dyn Render2dPipeline),
    Default(DefaultCompositorRenderPipelines<'a>),
}

impl<'a> ResolvedRender2dPipeline<'a> {
    pub(crate) fn pipeline(&self) -> &dyn Render2dPipeline {
        match self {
            Self::Provided(pipeline) => *pipeline,
            Self::Default(pipelines) => &pipelines.render_2d,
        }
    }
}

pub(crate) fn resolve_render_2d_pipeline<'a>(
    pipeline: Option<&'a dyn Render2dPipeline>,
    resolved_view_profile: &'a ResolvedViewProfile,
    obj_camera_states: &'a HashMap<String, ObjCameraState>,
    scene_camera_3d: &'a SceneCamera3D,
    spatial_context: SpatialContext,
    celestial_catalogs: Option<&'a engine_celestial::CelestialCatalogs>,
    prerender_frames: Option<&'a ObjPrerenderedFrames>,
) -> ResolvedRender2dPipeline<'a> {
    match pipeline {
        Some(pipeline) => ResolvedRender2dPipeline::Provided(pipeline),
        None => ResolvedRender2dPipeline::Default(DefaultCompositorRenderPipelines::new(
            resolved_view_profile,
            obj_camera_states,
            scene_camera_3d,
            spatial_context,
            celestial_catalogs,
            prerender_frames,
        )),
    }
}

pub struct DefaultCompositorRenderPipelines<'a> {
    pub render_2d: DefaultCompositorRender2dPipeline<'a>,
}

impl<'a> DefaultCompositorRenderPipelines<'a> {
    pub fn new(
        resolved_view_profile: &'a ResolvedViewProfile,
        obj_camera_states: &'a HashMap<String, ObjCameraState>,
        scene_camera_3d: &'a SceneCamera3D,
        spatial_context: SpatialContext,
        celestial_catalogs: Option<&'a engine_celestial::CelestialCatalogs>,
        prerender_frames: Option<&'a ObjPrerenderedFrames>,
    ) -> Self {
        #[cfg(feature = "render-3d")]
        let render_3d = DefaultCompositorRender3dDelegate;
        let render_2d = DefaultCompositorRender2dPipeline {
            resolved_view_profile,
            obj_camera_states,
            scene_camera_3d,
            spatial_context,
            celestial_catalogs,
            prerender_frames,
            #[cfg(feature = "render-3d")]
            render_3d,
        };
        Self { render_2d }
    }
}

pub struct DefaultCompositorRender2dPipeline<'a> {
    resolved_view_profile: &'a ResolvedViewProfile,
    obj_camera_states: &'a HashMap<String, ObjCameraState>,
    scene_camera_3d: &'a SceneCamera3D,
    spatial_context: SpatialContext,
    celestial_catalogs: Option<&'a engine_celestial::CelestialCatalogs>,
    prerender_frames: Option<&'a ObjPrerenderedFrames>,
    #[cfg(feature = "render-3d")]
    render_3d: DefaultCompositorRender3dDelegate,
}

impl Render2dPipeline for DefaultCompositorRender2dPipeline<'_> {
    fn render(&self, input: Render2dInput<'_>, target: &mut engine_core::buffer::Buffer) {
        render_sprites(
            input.layer_idx,
            input.layer,
            input.scene_w,
            input.scene_h,
            input.asset_root,
            input.target_resolver,
            input.object_regions,
            input.root_origin_x,
            input.root_origin_y,
            input.object_states,
            input.scene_elapsed_ms,
            input.current_stage,
            input.step_idx,
            input.elapsed_ms,
            self.obj_camera_states,
            self.scene_camera_3d,
            self.spatial_context,
            self.celestial_catalogs,
            input.is_pixel_backend,
            input.default_font,
            input.ui_font_scale,
            input.ui_layout_scale_x,
            input.ui_layout_scale_y,
            self.resolved_view_profile,
            #[cfg(feature = "render-3d")]
            &self.render_3d,
            self.prerender_frames,
            target,
        );
    }
}

#[cfg(feature = "render-3d")]
pub struct DefaultCompositorRender3dDelegate;

#[cfg(feature = "render-3d")]
impl Render3dDelegate for DefaultCompositorRender3dDelegate {
    fn render_3d_node(
        &self,
        spec: Render3dNodeSpec<'_>,
        area: engine_render_2d::RenderArea,
        target_resolver: Option<&TargetResolver>,
        object_regions: &mut HashMap<String, Region>,
        object_id: Option<&str>,
        object_state: &ObjectRuntimeState,
        appear_at: u64,
        sprite_elapsed: u64,
        ctx: &mut crate::render::RenderCtx<'_>,
    ) {
        match spec {
            Render3dNodeSpec::Obj(obj_spec) => {
                render_obj_sprite_adapter(
                    obj_spec,
                    area,
                    target_resolver,
                    object_regions,
                    object_id,
                    object_state,
                    appear_at,
                    sprite_elapsed,
                    ctx,
                );
            }
            Render3dNodeSpec::GeneratedWorld(world_spec) => {
                render_generated_world_sprite_adapter(
                    world_spec,
                    area,
                    target_resolver,
                    object_regions,
                    object_id,
                    object_state,
                    sprite_elapsed,
                    ctx,
                );
            }
        }
    }

    fn render_scene_clip_sprite(
        &self,
        spec: engine_render_3d::pipeline::SceneClipSpriteSpec<'_>,
        area: engine_render_2d::RenderArea,
        object_id: Option<&str>,
        object_state: &ObjectRuntimeState,
        object_regions: &mut HashMap<String, Region>,
        ctx: &mut crate::render::RenderCtx<'_>,
    ) {
        render_scene_clip_sprite_adapter(spec, area, object_id, object_state, object_regions, ctx);
    }
}

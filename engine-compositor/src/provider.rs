//! CompositorProvider trait — decouples compositor system from engine's World type.

use std::any::Any;
use std::collections::HashMap;

use engine_core::effects::Region;
use engine_core::scene_runtime_types::{ObjCameraState, ObjectRuntimeState, SceneCamera3D, TargetResolver};
use engine_render_2d::{Render2dInput, Render2dPipeline};

use crate::generated_world_render_adapter::render_generated_world_sprite as render_generated_world_sprite_adapter;
use crate::obj_render_adapter::render_obj_sprite as render_obj_sprite_adapter;
use crate::scene_clip_render_adapter::render_scene_clip_sprite as render_scene_clip_sprite_adapter;
use crate::sprite_renderer_2d::{render_sprites, Render3dDelegate};

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

pub struct DefaultCompositorRenderPipelines<'a> {
    pub render_2d: DefaultCompositorRender2dPipeline<'a>,
}

impl<'a> DefaultCompositorRenderPipelines<'a> {
    pub fn new(
        obj_camera_states: &'a HashMap<String, ObjCameraState>,
        scene_camera_3d: &'a SceneCamera3D,
        celestial_catalogs: Option<&'a engine_celestial::CelestialCatalogs>,
    ) -> Self {
        let render_3d = DefaultCompositorRender3dDelegate;
        let render_2d = DefaultCompositorRender2dPipeline {
            obj_camera_states,
            scene_camera_3d,
            celestial_catalogs,
            render_3d,
        };
        Self { render_2d }
    }
}

pub struct DefaultCompositorRender2dPipeline<'a> {
    obj_camera_states: &'a HashMap<String, ObjCameraState>,
    scene_camera_3d: &'a SceneCamera3D,
    celestial_catalogs: Option<&'a engine_celestial::CelestialCatalogs>,
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
            self.celestial_catalogs,
            input.is_pixel_backend,
            input.default_font,
            &self.render_3d,
            target,
        );
    }
}

pub struct DefaultCompositorRender3dDelegate;

impl Render3dDelegate for DefaultCompositorRender3dDelegate {
    fn render_obj_sprite(
        &self,
        spec: engine_render_3d::pipeline::ObjSpriteSpec<'_>,
        area: engine_render_2d::RenderArea,
        target_resolver: Option<&TargetResolver>,
        object_regions: &mut HashMap<String, Region>,
        object_id: Option<&str>,
        object_state: &ObjectRuntimeState,
        appear_at: u64,
        sprite_elapsed: u64,
        ctx: &mut crate::render::RenderCtx<'_>,
    ) {
        render_obj_sprite_adapter(
            spec,
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

    fn render_generated_world_sprite(
        &self,
        spec: engine_render_3d::pipeline::GeneratedWorldSpriteSpec<'_>,
        area: engine_render_2d::RenderArea,
        target_resolver: Option<&TargetResolver>,
        object_regions: &mut HashMap<String, Region>,
        object_id: Option<&str>,
        object_state: &ObjectRuntimeState,
        sprite_elapsed: u64,
        ctx: &mut crate::render::RenderCtx<'_>,
    ) {
        render_generated_world_sprite_adapter(
            spec,
            area,
            target_resolver,
            object_regions,
            object_id,
            object_state,
            sprite_elapsed,
            ctx,
        );
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

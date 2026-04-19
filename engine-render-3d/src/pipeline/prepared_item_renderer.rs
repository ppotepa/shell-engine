use engine_celestial::CelestialCatalogs;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::effects::Region;
use engine_core::scene_runtime_types::{ObjCameraState, SceneCamera3D};
use engine_core::spatial::SpatialContext;

use super::generated_world_sprite_renderer::{
    render_generated_world_sprite_to_buffer, GeneratedWorldSpriteRenderRuntime,
};
use super::obj_sprite_renderer::{
    render_obj_sprite_to_buffer, ObjSpriteRenderRuntime, SpriteRenderArea,
};
use super::producers::{PreparedRender3dItem, PreparedRender3dSource};
use super::scene_clip_renderer::{render_scene_clip_sprite_to_buffer, SceneClipRenderRuntime};
use super::ViewLightingParams;
use crate::prerender::ObjPrerenderedFrames;

#[derive(Clone)]
pub struct PreparedRender3dRuntime<'a> {
    pub scene_elapsed_ms: u64,
    pub sprite_elapsed_ms: u64,
    pub object_offset_x: i32,
    pub object_offset_y: i32,
    pub obj_camera_state: Option<ObjCameraState>,
    pub scene_camera_3d: &'a SceneCamera3D,
    pub view_lighting: ViewLightingParams,
    pub spatial_context: SpatialContext,
    pub celestial_catalogs: Option<&'a CelestialCatalogs>,
    pub asset_root: Option<&'a AssetRoot>,
    pub prerender_frames: Option<&'a ObjPrerenderedFrames>,
}

pub fn render_prepared_render3d_item_to_buffer(
    item: PreparedRender3dItem<'_>,
    area: SpriteRenderArea,
    runtime: PreparedRender3dRuntime<'_>,
    target: &mut Buffer,
) -> Option<Region> {
    match item.source {
        PreparedRender3dSource::Mesh(spec) => render_obj_sprite_to_buffer(
            spec,
            area,
            ObjSpriteRenderRuntime {
                sprite_elapsed_ms: runtime.sprite_elapsed_ms,
                object_offset_x: runtime.object_offset_x,
                object_offset_y: runtime.object_offset_y,
                camera_state: runtime.obj_camera_state.unwrap_or_default(),
                scene_camera_3d: runtime.scene_camera_3d,
                view_lighting: runtime.view_lighting,
                asset_root: runtime.asset_root,
                prerender_frames: runtime.prerender_frames,
            },
            target,
        ),
        PreparedRender3dSource::GeneratedWorld(spec) => render_generated_world_sprite_to_buffer(
            spec,
            area,
            GeneratedWorldSpriteRenderRuntime {
                sprite_elapsed_ms: runtime.sprite_elapsed_ms,
                object_offset_x: runtime.object_offset_x,
                object_offset_y: runtime.object_offset_y,
                scene_camera_3d: runtime.scene_camera_3d,
                view_lighting: runtime.view_lighting,
                spatial_context: runtime.spatial_context,
                celestial_catalogs: runtime.celestial_catalogs,
                asset_root: runtime.asset_root,
            },
            target,
        ),
        PreparedRender3dSource::SceneClip(spec) => render_scene_clip_sprite_to_buffer(
            spec,
            area,
            runtime.object_offset_x,
            runtime.object_offset_y,
            SceneClipRenderRuntime {
                scene_elapsed_ms: runtime.scene_elapsed_ms,
                scene_camera_3d: runtime.scene_camera_3d,
                asset_root: runtime.asset_root,
            },
            target,
        ),
    }
}

use engine_core::assets::AssetRoot;
use engine_core::scene::Layer;
use engine_render_3d::prerender::{prerender_obj_sprites_with, ObjPrerenderedFrames};

use engine_render_3d::raster::{obj_sprite_dimensions, render_obj_to_canvas};

pub fn prerender_scene_sprites(
    layers: &[Layer],
    scene_id: &str,
    asset_root: &AssetRoot,
) -> Option<ObjPrerenderedFrames> {
    prerender_obj_sprites_with(
        layers,
        scene_id,
        asset_root,
        render_obj_to_canvas,
        obj_sprite_dimensions,
    )
}

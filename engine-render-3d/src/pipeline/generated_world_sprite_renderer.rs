use super::{
    apply_world_lod_to_source, build_generated_world_render_profile,
    render_generated_world_sprite_with, GeneratedWorldRenderCallbacks, GeneratedWorldSpriteSpec,
    SpriteRenderArea, ViewLightingParams,
};
use crate::raster::{
    blit_rgba_canvas, composite_rgba_over, obj_sprite_dimensions, render_obj_to_rgba_canvas,
};
use crate::scene::{select_lod_level_stable, Renderable3D};
use engine_asset::MeshBuildKey;
use engine_celestial::CelestialCatalogs;
use engine_core::assets::AssetRoot;
use engine_core::buffer::Buffer;
use engine_core::effects::Region;
use engine_core::render_types::ScreenSpaceMetrics;
use engine_core::scene::{HorizontalAlign, VerticalAlign};
use engine_core::scene_runtime_types::SceneCamera3D;
use engine_core::spatial::SpatialContext;

#[derive(Debug, Clone, Copy)]
pub struct GeneratedWorldSpriteRenderRuntime<'a> {
    pub sprite_elapsed_ms: u64,
    pub object_offset_x: i32,
    pub object_offset_y: i32,
    pub scene_camera_3d: &'a SceneCamera3D,
    pub view_lighting: ViewLightingParams,
    pub spatial_context: SpatialContext,
    pub celestial_catalogs: Option<&'a CelestialCatalogs>,
    pub asset_root: Option<&'a AssetRoot>,
}

pub fn render_generated_world_sprite_to_buffer(
    mut spec: GeneratedWorldSpriteSpec<'_>,
    area: SpriteRenderArea,
    runtime: GeneratedWorldSpriteRenderRuntime<'_>,
    target: &mut Buffer,
) -> Option<Region> {
    let GeneratedWorldSpriteSpec {
        node,
        size,
        width,
        height,
        stretch_to_area,
        observer_altitude_km,
        align_x,
        align_y,
        ..
    } = spec.clone();
    let Renderable3D::GeneratedWorld(generated_world) = node.renderable else {
        return None;
    };

    let catalogs = runtime.celestial_catalogs?;
    let body = catalogs.bodies.get(generated_world.body_id.as_str())?;
    let preset_id = generated_world
        .preset_id
        .as_deref()
        .or(body.planet_type.as_deref());
    let planet = preset_id.and_then(|id| catalogs.planet_types.get(id))?;

    let profile = build_generated_world_render_profile(
        body,
        planet,
        node.transform.scale[0],
        observer_altitude_km.unwrap_or(0.0),
        runtime.spatial_context,
        runtime.view_lighting,
    );

    let (sprite_width, sprite_height) = if stretch_to_area {
        (area.width.max(1), area.height.max(1))
    } else if width.is_some() || height.is_some() || size.is_some() {
        obj_sprite_dimensions(width, height, size)
    } else {
        (area.width.max(1), area.height.max(1))
    };
    let selected_lod = select_lod_level_stable(
        node.id.as_str(),
        node.lod_hint.as_ref(),
        ScreenSpaceMetrics {
            projected_radius_px: (sprite_width.min(sprite_height) as f32) * 0.5,
            viewport_area_px: sprite_width as u32 * sprite_height as u32,
        },
    );
    if let Renderable3D::GeneratedWorld(world) = &mut spec.node.renderable {
        let effective_source = apply_world_lod_to_source(world.mesh_key.as_str(), selected_lod);
        world.mesh_key = MeshBuildKey::from_source(effective_source);
    }
    let base_x = area.origin_x
        + resolve_x(
            node.transform.translation[0].round() as i32,
            &align_x,
            area.width,
            sprite_width,
        );
    let base_y = area.origin_y
        + resolve_y(
            node.transform.translation[1].round() as i32,
            &align_y,
            area.height,
            sprite_height,
        );
    let draw_x = base_x.saturating_add(runtime.object_offset_x);
    let draw_y = base_y.saturating_add(runtime.object_offset_y);

    let rendered = render_generated_world_sprite_with(
        spec,
        &profile,
        sprite_width,
        sprite_height,
        draw_x,
        draw_y,
        runtime.sprite_elapsed_ms,
        runtime.scene_camera_3d,
        runtime.asset_root,
        target,
        GeneratedWorldRenderCallbacks {
            render_obj_to_rgba_canvas,
            composite_rgba_over,
            blit_rgba_canvas,
        },
    );
    if !rendered {
        return None;
    }

    Some(visible_region(
        draw_x,
        draw_y,
        sprite_width,
        sprite_height,
        target,
    ))
}

fn visible_region(draw_x: i32, draw_y: i32, width: u16, height: u16, buf: &Buffer) -> Region {
    let x0 = draw_x.max(0);
    let y0 = draw_y.max(0);
    let x1 = (draw_x + width as i32).min(buf.width as i32).max(x0);
    let y1 = (draw_y + height as i32).min(buf.height as i32).max(y0);
    Region {
        x: x0 as u16,
        y: y0 as u16,
        width: (x1 - x0) as u16,
        height: (y1 - y0) as u16,
    }
}

#[inline]
fn resolve_x(offset_x: i32, align_x: &Option<HorizontalAlign>, area_w: u16, sprite_w: u16) -> i32 {
    let origin = match align_x {
        Some(HorizontalAlign::Left) | None => 0i32,
        Some(HorizontalAlign::Center) => (area_w.saturating_sub(sprite_w) / 2) as i32,
        Some(HorizontalAlign::Right) => area_w.saturating_sub(sprite_w) as i32,
    };
    origin.saturating_add(offset_x)
}

#[inline]
fn resolve_y(offset_y: i32, align_y: &Option<VerticalAlign>, area_h: u16, sprite_h: u16) -> i32 {
    let origin = match align_y {
        Some(VerticalAlign::Top) | None => 0i32,
        Some(VerticalAlign::Center) => (area_h.saturating_sub(sprite_h) / 2) as i32,
        Some(VerticalAlign::Bottom) => area_h.saturating_sub(sprite_h) as i32,
    };
    origin.saturating_add(offset_y)
}

//! Recursive helpers for rendering container sprite children.

use crate::effects::Region;
use crate::scene::{SceneRenderedMode, Sprite};
use crate::scene_runtime::{ObjectRuntimeState, TargetResolver};
use std::collections::HashMap;

use super::super::layout::{GridCellRect, RenderArea};
use super::super::text_render::ClipRect;
use super::common::RenderCtx;

/// Renders all container children using precomputed cell rectangles.
pub(crate) fn render_children_in_cells<F>(
    layer_idx: usize,
    sprite_path: &mut Vec<usize>,
    children: &[Sprite],
    child_cells: &[(usize, GridCellRect)],
    draw_x: i32,
    draw_y: i32,
    resolved_mode: SceneRenderedMode,
    parent_clip: Option<ClipRect>,
    target_resolver: Option<&TargetResolver>,
    object_regions: &mut HashMap<String, Region>,
    object_states: &HashMap<String, ObjectRuntimeState>,
    ctx: &mut RenderCtx<'_>,
    mut render_child: F,
) where
    F: FnMut(
        usize,
        &mut Vec<usize>,
        &Sprite,
        RenderArea,
        SceneRenderedMode,
        Option<ClipRect>,
        Option<&TargetResolver>,
        &mut HashMap<String, Region>,
        &HashMap<String, ObjectRuntimeState>,
        &mut RenderCtx<'_>,
    ),
{
    let base_path_len = sprite_path.len();
    sprite_path.push(0);
    for (idx, rect) in child_cells {
        let Some(child) = children.get(*idx) else {
            continue;
        };
        *sprite_path.last_mut().expect("path element pushed above") = *idx;
        let child_area = RenderArea {
            origin_x: draw_x + rect.x as i32,
            origin_y: draw_y + rect.y as i32,
            width: rect.width.max(1),
            height: rect.height.max(1),
        };
        render_child(
            layer_idx,
            sprite_path,
            child,
            child_area,
            resolved_mode,
            parent_clip,
            target_resolver,
            object_regions,
            object_states,
            ctx,
        );
    }
    sprite_path.truncate(base_path_len);
}

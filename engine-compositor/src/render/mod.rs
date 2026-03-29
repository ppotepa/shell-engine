//! Shared helpers for sprite render dispatch and container traversal.

pub(crate) mod common;
pub(crate) mod containers;

pub(crate) use common::{
    check_visibility, compute_draw_pos, finalize_sprite, is_sprite_offscreen, sprite_transform_offset, RenderCtx,
};
pub(crate) use containers::render_children_in_cells;

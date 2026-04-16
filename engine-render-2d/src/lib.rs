//! 2D sprite rendering primitives shared by the engine compositor.
//!
//! This crate owns text and image rasterization logic. Higher-level scene
//! assembly stays in `engine-compositor`.

pub mod containers;
pub mod image;
pub mod layout;
pub mod text;

pub use containers::render_children_in_cells;
pub use image::{image_sprite_dimensions, render_image_content};
pub use layout::{
    compute_flex_cells, compute_grid_cells, measure_sprite_for_layout, parse_track_spec,
    resolve_x, resolve_y, with_render_context, GridCellRect, RenderArea, TrackSpec,
};
pub use text::{dim_colour, render_text_content, text_sprite_dimensions, ClipRect};

//! Layout helpers for CSS-like grid and flex container resolution.

pub mod area;
pub mod flex;
pub mod grid;
pub mod tracks;

pub use area::{resolve_x, resolve_y, GridCellRect, RenderArea};
pub use flex::compute_flex_cells;
pub use grid::compute_grid_cells;
pub use tracks::{parse_track_spec, TrackSpec};

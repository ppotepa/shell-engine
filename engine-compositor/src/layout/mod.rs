//! Layout helpers for compositor containers and sprite measurement adapters.

pub mod area;
pub mod flex;
pub mod grid;
pub(crate) mod measure;
pub mod tracks;

pub use area::{resolve_x, resolve_y, GridCellRect, RenderArea};
pub use flex::compute_flex_cells;
pub use grid::compute_grid_cells;
pub(crate) use measure::measure_sprite_for_layout;
pub(crate) use measure::with_pixel_backend;
pub use tracks::{parse_track_spec, TrackSpec};

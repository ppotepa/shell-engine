use engine_core::buffer::Buffer;
use crossterm::style::Color;

/// Controls how the virtual (2× height) buffer is packed into the terminal halfblock buffer.
pub trait HalfblockPacker: Send + Sync {
    /// Returns the (x_start, x_end, y_start, y_end) iteration bounds for the packing loop.
    /// `source` is the virtual buffer; `target_height` is the halfblock buffer height.
    fn iteration_bounds(
        &self,
        source: &Buffer,
        target_height: u16,
    ) -> Option<(u16, u16, u16, u16)>;
}

/// Iterates the full buffer every frame. Safe default.
pub struct FullScanPacker;

impl HalfblockPacker for FullScanPacker {
    #[inline]
    fn iteration_bounds(
        &self,
        source: &Buffer,
        target_height: u16,
    ) -> Option<(u16, u16, u16, u16)> {
        Some((
            0,
            source.width.saturating_sub(1),
            0,
            target_height.saturating_sub(1),
        ))
    }
}

/// Narrows packing to the dirty bounding box tracked by `fill()` / `set()`.
/// Only safe when `fill()` ran this frame and `reset_dirty()` was NOT called after it.
/// Gate behind `--opt-comp`.
pub struct DirtyRegionPacker;

impl HalfblockPacker for DirtyRegionPacker {
    #[inline]
    fn iteration_bounds(
        &self,
        source: &Buffer,
        _target_height: u16,
    ) -> Option<(u16, u16, u16, u16)> {
        source.dirty_bounds().map(|(xmin, xmax, ymin, ymax)| {
            let ty_start = ymin / 2;
            let ty_end = ymax / 2;
            (xmin, xmax, ty_start, ty_end)
        })
    }
}

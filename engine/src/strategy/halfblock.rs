use engine_core::buffer::Buffer;
use crossterm::style::Color;

/// Controls how the virtual (2× height) buffer is packed into the terminal halfblock buffer.
pub trait HalfblockPacker: Send + Sync {
    fn iteration_bounds(
        &self,
        source: &Buffer,
        target_height: u16,
    ) -> Option<(u16, u16, u16, u16)>;
    /// Returns `true` when this is the experimental dirty-region variant.
    fn is_dirty_region(&self) -> bool { false }
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
    fn is_dirty_region(&self) -> bool { true }
}

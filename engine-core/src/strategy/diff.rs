use crossterm::style::Color;

use crate::buffer::Buffer;

/// Controls how the double-buffer diff is scanned each frame.
///
/// `FullScanDiff` is the safe default: always scans the entire W×H grid.
/// `DirtyRegionDiff` is an experimental optimisation: restricts the scan to
/// the tracked dirty bounding box. Only safe when `fill()` is guaranteed to
/// have run this frame with no `reset_dirty()` call after it.
/// `RowSkipDiff` skips entire rows marked not dirty. Complements DirtyRegionDiff.
pub trait DiffStrategy: Send + Sync {
    fn diff_into(&self, buf: &Buffer, out: &mut Vec<(u16, u16, char, Color, Color)>);
}

/// Always scans the full buffer. Stable and correct in all circumstances.
pub struct FullScanDiff;

impl DiffStrategy for FullScanDiff {
    #[inline]
    fn diff_into(&self, buf: &Buffer, out: &mut Vec<(u16, u16, char, Color, Color)>) {
        buf.diff_into(out);
    }
}

/// Scans only the dirty bounding box tracked during `fill()` / `set()` calls.
/// Up to ~90 % faster on sparse updates, but unsafe if the dirty invariants are broken.
/// Gate behind `--opt-diff`.
pub struct DirtyRegionDiff;

impl DiffStrategy for DirtyRegionDiff {
    #[inline]
    fn diff_into(&self, buf: &Buffer, out: &mut Vec<(u16, u16, char, Color, Color)>) {
        buf.diff_into_dirty(out);
    }
}

/// Row-level dirty skip: skips entire rows marked not dirty.
/// Complements full scan with per-row early exit. Up to ~10-20% faster on
/// frames with static regions (e.g., UI background not changing each frame).
/// Safe: dirty_rows only set to true during frame, reset after swap().
/// Gate behind `--opt-rowdiff`.
pub struct RowSkipDiff;

impl DiffStrategy for RowSkipDiff {
    #[inline]
    fn diff_into(&self, buf: &Buffer, out: &mut Vec<(u16, u16, char, Color, Color)>) {
        buf.diff_into_row_skip(out);
    }
}

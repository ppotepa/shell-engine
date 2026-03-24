/// Render pipeline diff strategies.
pub mod diff;

pub use diff::{DiffStrategy, DirtyRegionDiff, FullScanDiff};

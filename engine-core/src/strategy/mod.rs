/// Render pipeline diff strategies.
pub mod diff;
/// Mod-defined effect factory.
pub mod effect_factory;

pub use diff::{DiffStrategy, DirtyRegionDiff, FullScanDiff, RowSkipDiff};
pub use effect_factory::ModEffectFactory;

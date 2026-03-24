/// Render pipeline diff strategies.
pub mod diff;
/// Mod-defined effect factory.
pub mod effect_factory;

pub use diff::{DiffStrategy, DirtyRegionDiff, FullScanDiff};
pub use effect_factory::ModEffectFactory;

/// Layer compositor strategy (scratch vs direct).
pub mod layer;
/// Virtual-to-output present strategy (always vs hash-skip).
pub mod present;
/// Frame-skip oracle strategy (unified coordination).
pub mod skip;

pub use layer::{DirectLayerCompositor, LayerCompositor, ScratchLayerCompositor};
pub use present::{AlwaysPresenter, HashSkipPresenter, VirtualPresenter};
pub use skip::{AlwaysRender, CoordinatedSkip, FrameSkipOracle};

use engine_core::strategy::{DiffStrategy, DirtyRegionDiff, FullScanDiff, RowSkipDiff};

/// Aggregated render pipeline strategies, registered as a World resource at startup.
///
/// Systems call the trait methods on these fields instead of branching on boolean flags.
/// Swap any field at runtime for instant behaviour change without restarting.
pub struct PipelineStrategies {
    pub diff: Box<dyn DiffStrategy>,
    pub layer: Box<dyn LayerCompositor>,
    pub present: Box<dyn VirtualPresenter>,
}

impl PipelineStrategies {
    /// Construct safe defaults — all strategies use the full-scan / always-present paths.
    pub fn new() -> Self {
        Self {
            diff: Box::new(FullScanDiff),
            layer: Box::new(ScratchLayerCompositor),
            present: Box::new(AlwaysPresenter),
        }
    }
}

impl Default for PipelineStrategies {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineStrategies {
    /// Construct from CLI optimisation flags.
    ///
    /// | flag           | effect                                              |
    /// |----------------|-----------------------------------------------------|
    /// | `--opt-diff`   | `DirtyRegionDiff` instead of `FullScanDiff`         |
    /// | `--opt-rowdiff`| `RowSkipDiff` (row-level skip in full-scan)          |
    /// | `--opt-comp`   | `DirectLayerCompositor`                             |
    /// | `--opt-present`| `HashSkipPresenter` instead of `AlwaysPresenter`    |
    pub fn from_flags(
        opt_diff: bool,
        opt_comp: bool,
        opt_present: bool,
        opt_rowdiff: bool,
        _opt_async_display: bool,
    ) -> Self {
        Self {
            diff: if opt_diff {
                Box::new(DirtyRegionDiff)
            } else if opt_rowdiff {
                Box::new(RowSkipDiff)
            } else {
                Box::new(FullScanDiff)
            },
            layer: if opt_comp {
                Box::new(DirectLayerCompositor)
            } else {
                Box::new(ScratchLayerCompositor)
            },
            present: if opt_present {
                Box::new(HashSkipPresenter)
            } else {
                Box::new(AlwaysPresenter)
            },
        }
    }
}

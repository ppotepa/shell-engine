/// Layer compositor strategy (scratch vs direct).
pub mod layer;
/// Halfblock packer iteration strategy (full vs dirty-region).
pub mod halfblock;
/// Virtual-to-output present strategy (always vs hash-skip).
pub mod present;
/// Terminal flusher strategy (batched ANSI vs naive).
pub mod flush;
/// Scene rendered-mode compositor strategy (Cell vs HalfBlock path).
pub mod scene_compositor;
/// Behavior factory strategy (built-in vs custom behavior resolution).
pub mod behavior_factory;

pub use layer::{DirectLayerCompositor, LayerCompositor, ScratchLayerCompositor};
pub use halfblock::{DirtyRegionPacker, FullScanPacker, HalfblockPacker};
pub use present::{AlwaysPresenter, HashSkipPresenter, VirtualPresenter};
pub use flush::{AnsiBatchFlusher, NaiveFlusher, TerminalFlusher};
pub use scene_compositor::{
    CellSceneCompositor, CompositeParams, HalfblockSceneCompositor, SceneCompositor,
    compositor_for,
};
pub use behavior_factory::{BehaviorFactory, BuiltInBehaviorFactory};

use engine_core::strategy::{DiffStrategy, FullScanDiff};
use engine_core::strategy::DirtyRegionDiff;

/// Aggregated render pipeline strategies, registered as a World resource at startup.
///
/// Systems call the trait methods on these fields instead of branching on boolean flags.
/// Swap any field at runtime for instant behaviour change without restarting.
pub struct PipelineStrategies {
    pub diff:      Box<dyn DiffStrategy>,
    pub layer:     Box<dyn LayerCompositor>,
    pub halfblock: Box<dyn HalfblockPacker>,
    pub present:   Box<dyn VirtualPresenter>,
    pub flush:     Box<dyn TerminalFlusher>,
}

impl PipelineStrategies {
    /// Construct safe defaults — all strategies use the full-scan / always-present paths.
    pub fn default_safe() -> Self {
        Self {
            diff:      Box::new(FullScanDiff),
            layer:     Box::new(ScratchLayerCompositor),
            halfblock: Box::new(FullScanPacker),
            present:   Box::new(AlwaysPresenter),
            flush:     Box::new(AnsiBatchFlusher),
        }
    }

    /// Construct from CLI optimisation flags.
    ///
    /// | flag           | effect                                              |
    /// |----------------|-----------------------------------------------------|
    /// | `--opt-diff`   | `DirtyRegionDiff` instead of `FullScanDiff`         |
    /// | `--opt-comp`   | `DirectLayerCompositor` + `DirtyRegionPacker`       |
    /// | `--opt-present`| `HashSkipPresenter` instead of `AlwaysPresenter`    |
    pub fn from_flags(opt_diff: bool, opt_comp: bool, opt_present: bool) -> Self {
        Self {
            diff: if opt_diff {
                Box::new(DirtyRegionDiff)
            } else {
                Box::new(FullScanDiff)
            },
            layer: if opt_comp {
                Box::new(DirectLayerCompositor)
            } else {
                Box::new(ScratchLayerCompositor)
            },
            halfblock: if opt_comp {
                Box::new(DirtyRegionPacker)
            } else {
                Box::new(FullScanPacker)
            },
            present: if opt_present {
                Box::new(HashSkipPresenter)
            } else {
                Box::new(AlwaysPresenter)
            },
            flush: Box::new(AnsiBatchFlusher),
        }
    }
}

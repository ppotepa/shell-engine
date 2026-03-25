//! Strategy trait implementations and concrete strategy constructors.
//!
//! Re-exports trait defs from engine-pipeline (low-dep abstractions) and provides
//! concrete implementations that can reference engine internals (renderer, etc.).
//!
//! The strategy pattern is used throughout the render pipeline:
//! each system consults a strategy object instead of branching on boolean flags.

// Re-export trait defs and simple impls from engine-pipeline
pub use engine_pipeline::{
    LayerCompositor, HalfblockPacker, VirtualPresenter, TerminalFlusher,
    DisplaySink, DisplayFrame, FrameSkipOracle,
    DirectLayerCompositor, ScratchLayerCompositor,
    DirtyRegionPacker, FullScanPacker,
    AlwaysPresenter, HashSkipPresenter,
    AlwaysRender, CoordinatedSkip,
    PipelineStrategies,
};

// Concrete implementations that need engine-specific code
pub mod flush;
pub mod display;
pub mod scene_compositor;
pub mod behavior_factory;

pub use flush::{AnsiBatchFlusher, NaiveFlusher};
pub use display::{AsyncDisplaySink, SyncDisplaySink};
pub use scene_compositor::{
    CellSceneCompositor, CompositeParams, HalfblockSceneCompositor, SceneCompositor,
    compositor_for,
};
pub use behavior_factory::{BehaviorFactory, BuiltInBehaviorFactory};

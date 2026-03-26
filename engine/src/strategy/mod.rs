//! Strategy trait implementations and concrete strategy constructors.
//!
//! Re-exports trait defs from engine-pipeline (low-dep abstractions) and provides
//! concrete implementations that can reference engine internals (renderer, etc.).
//!
//! The strategy pattern is used throughout the render pipeline:
//! each system consults a strategy object instead of branching on boolean flags.

// Re-export trait defs and simple impls from engine-pipeline
pub use engine_pipeline::{
    AlwaysPresenter, AlwaysRender, CoordinatedSkip, DirectLayerCompositor, DirtyRegionPacker,
    DisplayFrame, DisplaySink, FrameSkipOracle, FullScanPacker, HalfblockPacker, HashSkipPresenter,
    LayerCompositor, PipelineStrategies, ScratchLayerCompositor, TerminalFlusher, VirtualPresenter,
};

// Re-export terminal-specific strategies from engine-render-terminal
pub use engine_render_terminal::strategy::display::{AsyncDisplaySink, SyncDisplaySink};
pub use engine_render_terminal::strategy::flush::{AnsiBatchFlusher, NaiveFlusher};

pub mod behavior_factory;
pub mod scene_compositor;

pub use behavior_factory::{BehaviorFactory, BuiltInBehaviorFactory};
pub use scene_compositor::{
    CellSceneCompositor, CompositeParams, HalfblockSceneCompositor, SceneCompositor,
};

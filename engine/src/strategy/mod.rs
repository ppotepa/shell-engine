//! Strategy trait implementations and concrete strategy constructors.
//!
//! Re-exports trait defs from engine-pipeline (low-dep abstractions) and provides
//! concrete implementations that can reference engine internals (renderer, etc.).
//!
//! The strategy pattern is used throughout the render pipeline:
//! each system consults a strategy object instead of branching on boolean flags.

// Re-export trait defs and simple impls from engine-pipeline
pub use engine_pipeline::{
    AlwaysPresenter, AlwaysRender, CoordinatedSkip, DirectLayerCompositor,
    FrameSkipOracle, HashSkipPresenter, LayerCompositor, PipelineStrategies,
    ScratchLayerCompositor, VirtualPresenter,
};
pub use engine_compositor::CompositeParams;

pub mod behavior_factory;

pub use behavior_factory::{BehaviorFactory, BuiltInBehaviorFactory};

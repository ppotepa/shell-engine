//! RendererProvider trait — decouples renderer from engine's World type.
use engine_animation::Animator;
use engine_core::buffer::Buffer;
use engine_debug::DebugLogBuffer;
use engine_debug::{DebugFeatures, FpsCounter, ProcessStats, SystemTimings};
use engine_pipeline::{FrameSkipOracle, PipelineStrategies};
use engine_render::{OutputBackend, VectorOverlay};
use engine_runtime::RuntimeSettings;
use std::sync::Mutex;

pub trait RendererProvider {
    fn buffer(&self) -> Option<&Buffer>;
    fn buffer_mut(&mut self) -> Option<&mut Buffer>;
    fn runtime_settings(&self) -> Option<&RuntimeSettings>;
    fn debug_features(&self) -> Option<&DebugFeatures>;
    fn debug_log(&self) -> Option<&DebugLogBuffer>;
    fn animator(&self) -> Option<&Animator>;
    fn fps_counter(&self) -> Option<&FpsCounter>;
    fn process_stats(&self) -> Option<&ProcessStats>;
    /// Optional hook to expose the current object count for debug overlays.
    fn object_count(&self) -> Option<usize> {
        None
    }
    /// Optional hook to expose gameplay entity diagnostics for debug overlays.
    fn gameplay_diagnostics(&self) -> Option<&engine_debug::GameplayDiagnostics> {
        None
    }
    fn system_timings(&self) -> Option<&SystemTimings>;
    fn current_scene_id(&self) -> String;
    /// Returns a raw pointer to PipelineStrategies (safe: singleton, never dropped during frame).
    fn pipeline_strategies_ptr(&self) -> *const PipelineStrategies;
    fn frame_skip_oracle(&self) -> Option<&Mutex<Box<dyn FrameSkipOracle>>>;
    fn renderer_mut(&mut self) -> Option<&mut (dyn OutputBackend + '_)>;
    fn swap_buffers(&mut self);
    fn restore_front_to_back(&mut self);
    /// Returns vector overlay data collected during compositing (for SDL2 native rendering).
    fn vector_overlay(&self) -> Option<&VectorOverlay> {
        None
    }
}

//! RendererProvider trait — decouples renderer from engine's World type.

use crate::renderer::TerminalRenderer;
use engine_animation::Animator;
use engine_core::buffer::{Buffer, VirtualBuffer};
use engine_debug::DebugLogBuffer;
use engine_debug::{DebugFeatures, FpsCounter, ProcessStats, SystemTimings};
use engine_pipeline::{FrameSkipOracle, PipelineStrategies};
use engine_runtime::RuntimeSettings;
use std::sync::Mutex;

pub trait RendererProvider {
    fn buffer(&self) -> Option<&Buffer>;
    fn buffer_mut(&mut self) -> Option<&mut Buffer>;
    fn virtual_buffer(&self) -> Option<&VirtualBuffer>;
    fn runtime_settings(&self) -> Option<&RuntimeSettings>;
    fn debug_features(&self) -> Option<&DebugFeatures>;
    fn debug_log(&self) -> Option<&DebugLogBuffer>;
    fn animator(&self) -> Option<&Animator>;
    fn fps_counter(&self) -> Option<&FpsCounter>;
    fn process_stats(&self) -> Option<&ProcessStats>;
    fn system_timings(&self) -> Option<&SystemTimings>;
    fn current_scene_id(&self) -> String;
    /// Returns a raw pointer to PipelineStrategies (safe: singleton, never dropped during frame).
    fn pipeline_strategies_ptr(&self) -> *const PipelineStrategies;
    fn frame_skip_oracle(&self) -> Option<&Mutex<Box<dyn FrameSkipOracle>>>;
    fn renderer_mut(&mut self) -> Option<&mut TerminalRenderer>;
    fn swap_buffers(&mut self);
    fn restore_front_to_back(&mut self);
    fn with_virtual_and_output<F: FnOnce(&VirtualBuffer, &mut Buffer)>(&mut self, f: F);
}

//! RendererProvider trait — decouples renderer from engine's World type.
//!
//! This trait allows the renderer system in engine-render-terminal to access
//! World resources without depending on the engine crate's World type.

/// Provides access to renderer-needed resources from World.
///
/// Allows renderer system to be generic and independent of engine's type-erased World.
pub trait RendererProvider {
    fn buffer(&self) -> Option<&dyn std::any::Any>;
    fn buffer_mut(&mut self) -> Option<&mut dyn std::any::Any>;
    fn output_buffer(&self) -> Option<&dyn std::any::Any>;
    fn virtual_buffer(&self) -> Option<&dyn std::any::Any>;
    fn virtual_buffer_mut(&mut self) -> Option<&mut dyn std::any::Any>;
    fn runtime_settings(&self) -> Option<&dyn std::any::Any>;
    fn debug_features(&self) -> Option<&dyn std::any::Any>;
    fn debug_log_mut(&mut self) -> Option<&mut dyn std::any::Any>;
}

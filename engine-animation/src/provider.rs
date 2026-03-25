//! LifecycleProvider trait — decouples scene lifecycle system from engine's World type.
//!
//! Enables the lifecycle system (scene transitions, menu navigation) to be generic
//! over any container that provides the necessary resources.

/// Provides access to lifecycle-needed resources from World.
///
/// Allows lifecycle system to be independent of engine's type-erased World.
pub trait LifecycleProvider {
    fn animator(&self) -> Option<&dyn std::any::Any>;
    fn animator_mut(&mut self) -> Option<&mut dyn std::any::Any>;
    fn scene_runtime(&self) -> Option<&dyn std::any::Any>;
    fn scene_runtime_mut(&mut self) -> Option<&mut dyn std::any::Any>;
    fn buffer_mut(&mut self) -> Option<&mut dyn std::any::Any>;
    fn virtual_buffer_mut(&mut self) -> Option<&mut dyn std::any::Any>;
    fn runtime_settings(&self) -> Option<&dyn std::any::Any>;
    fn debug_features(&self) -> Option<&dyn std::any::Any>;
    fn debug_log_mut(&mut self) -> Option<&mut dyn std::any::Any>;
    fn events_mut(&mut self) -> Option<&mut dyn std::any::Any>;
}

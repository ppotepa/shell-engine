//! CompositorProvider trait — decouples compositor system from engine's World type.

use std::any::Any;

/// Provides access to compositor-needed resources from World.
pub trait CompositorProvider {
    fn buffer_mut(&mut self) -> Option<&mut dyn Any>;
    fn scene_runtime(&self) -> Option<&dyn Any>;
    fn animator(&self) -> Option<&dyn Any>;
    fn asset_root(&self) -> Option<&dyn Any>;
    fn runtime_settings(&self) -> Option<&dyn Any>;
    fn debug_features(&self) -> Option<&dyn Any>;
}

//! BehaviorProvider trait — decouples behavior system from engine's World type.

use std::any::Any;

/// Provides access to behavior system resources from World.
pub trait BehaviorProvider {
    fn scene(&self) -> Option<&dyn Any>;
    fn animator(&self) -> Option<&dyn Any>;
    fn scene_runtime_mut(&mut self) -> Option<&mut dyn Any>;
    fn game_state(&self) -> Option<&dyn Any>;
    fn mod_behaviors(&self) -> Option<&dyn Any>;
    fn debug_log_mut(&mut self) -> Option<&mut dyn Any>;
    fn dispatch_audio_command(&mut self, cmd: Box<dyn Any>);
    fn dispatch_behavior_command(&mut self, cmd: Box<dyn Any>);
    fn dispatch_animation_command(&mut self, cmd: Box<dyn Any>);
    fn dispatch_lifecycle_command(&mut self, cmd: Box<dyn Any>);
    fn events_mut(&mut self) -> Option<&mut dyn Any>;
}

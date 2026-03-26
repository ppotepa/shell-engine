//! Typed access to debug resources via [`World`].

use crate::{DebugFeatures, FpsCounter, ProcessStats, SystemTimings};
use crate::log::DebugLogBuffer;
use engine_core::world::World;

/// Typed access to debug/diagnostic resources in World.
pub trait DebugAccess {
    fn debug_features(&self) -> Option<&DebugFeatures>;
    fn debug_features_mut(&mut self) -> Option<&mut DebugFeatures>;
    fn debug_log(&self) -> Option<&DebugLogBuffer>;
    fn debug_log_mut(&mut self) -> Option<&mut DebugLogBuffer>;
    fn fps_counter(&self) -> Option<&FpsCounter>;
    fn process_stats(&self) -> Option<&ProcessStats>;
    fn system_timings(&self) -> Option<&SystemTimings>;
}

impl DebugAccess for World {
    fn debug_features(&self) -> Option<&DebugFeatures> {
        self.get::<DebugFeatures>()
    }
    fn debug_features_mut(&mut self) -> Option<&mut DebugFeatures> {
        self.get_mut::<DebugFeatures>()
    }
    fn debug_log(&self) -> Option<&DebugLogBuffer> {
        self.get::<DebugLogBuffer>()
    }
    fn debug_log_mut(&mut self) -> Option<&mut DebugLogBuffer> {
        self.get_mut::<DebugLogBuffer>()
    }
    fn fps_counter(&self) -> Option<&FpsCounter> {
        self.get::<FpsCounter>()
    }
    fn process_stats(&self) -> Option<&ProcessStats> {
        self.get::<ProcessStats>()
    }
    fn system_timings(&self) -> Option<&SystemTimings> {
        self.get::<SystemTimings>()
    }
}

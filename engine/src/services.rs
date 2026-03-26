//! Internal [`EngineWorldAccess`] trait that gives systems typed access to [`World`] resources.

use crate::assets::AssetRoot;
use crate::audio::AudioRuntime;
use crate::buffer::{Buffer, VirtualBuffer};
use crate::debug_features::DebugFeatures;
use crate::debug_log::DebugLogBuffer;
use crate::events::EventQueue;
use crate::runtime_settings::RuntimeSettings;
use crate::scene_loader::SceneLoader;
use crate::scene_runtime::SceneRuntime;
use engine_animation::Animator;
use crate::systems::renderer::TerminalRenderer;
use crate::world::World;
use engine_audio::AudioProvider;
use engine_animation::{AnimatorProvider, LifecycleProvider};
use engine_render_terminal::RendererProvider;
use engine_behavior_registry::BehaviorProvider;
use engine_compositor::CompositorProvider;
use engine_core::scene::Scene;
use engine_debug::{FpsCounter, ProcessStats, SystemTimings};
use engine_pipeline::{PipelineStrategies, FrameSkipOracle};
use std::sync::Mutex;

/// Typed accessor trait for all engine-managed resources stored in [`World`].
pub(crate) trait EngineWorldAccess {
    fn events_mut(&mut self) -> Option<&mut EventQueue>;
    fn scene_runtime(&self) -> Option<&SceneRuntime>;
    fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime>;
    fn animator(&self) -> Option<&Animator>;
    fn animator_mut(&mut self) -> Option<&mut Animator>;
    fn buffer(&self) -> Option<&Buffer>;
    fn buffer_mut(&mut self) -> Option<&mut Buffer>;
    fn output_buffer(&self) -> Option<&Buffer>;
    fn virtual_buffer(&self) -> Option<&VirtualBuffer>;
    fn virtual_buffer_mut(&mut self) -> Option<&mut VirtualBuffer>;
    fn runtime_settings(&self) -> Option<&RuntimeSettings>;
    fn audio_runtime_mut(&mut self) -> Option<&mut AudioRuntime>;
    fn asset_root(&self) -> Option<&AssetRoot>;
    fn renderer_mut(&mut self) -> Option<&mut TerminalRenderer>;
    fn scene_loader(&self) -> Option<&SceneLoader>;
}

impl EngineWorldAccess for World {
    fn events_mut(&mut self) -> Option<&mut EventQueue> {
        self.get_mut::<EventQueue>()
    }

    fn scene_runtime(&self) -> Option<&SceneRuntime> {
        self.get::<SceneRuntime>()
    }

    fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime> {
        self.get_mut::<SceneRuntime>()
    }

    fn animator(&self) -> Option<&Animator> {
        self.get::<Animator>()
    }

    fn animator_mut(&mut self) -> Option<&mut Animator> {
        self.get_mut::<Animator>()
    }

    fn buffer(&self) -> Option<&Buffer> {
        self.get::<Buffer>()
    }

    fn buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.get_mut::<Buffer>()
    }

    /// The terminal-sized output buffer (alias for `buffer`).
    fn output_buffer(&self) -> Option<&Buffer> {
        self.get::<Buffer>()
    }

    fn virtual_buffer(&self) -> Option<&VirtualBuffer> {
        self.get::<VirtualBuffer>()
    }

    fn virtual_buffer_mut(&mut self) -> Option<&mut VirtualBuffer> {
        self.get_mut::<VirtualBuffer>()
    }

    fn runtime_settings(&self) -> Option<&RuntimeSettings> {
        self.get::<RuntimeSettings>()
    }

    fn audio_runtime_mut(&mut self) -> Option<&mut AudioRuntime> {
        self.get_mut::<AudioRuntime>()
    }

    fn asset_root(&self) -> Option<&AssetRoot> {
        self.get::<AssetRoot>()
    }

    fn renderer_mut(&mut self) -> Option<&mut TerminalRenderer> {
        self.get_mut::<TerminalRenderer>()
    }

    fn scene_loader(&self) -> Option<&SceneLoader> {
        self.get::<SceneLoader>()
    }
}

// Implement AudioProvider for World to work with engine-audio
impl AudioProvider for World {
    fn audio_runtime_mut(&mut self) -> Option<&mut AudioRuntime> {
        self.get_mut::<AudioRuntime>()
    }
}

// Implement AnimatorProvider for World to work with engine-animation
impl AnimatorProvider for World {
    fn scene(&self) -> Option<Scene> {
        EngineWorldAccess::scene_runtime(self).map(|rt| rt.scene().clone())
    }

    fn animator(&self) -> Option<&Animator> {
        self.get::<Animator>()
    }

    fn animator_mut(&mut self) -> Option<&mut Animator> {
        self.get_mut::<Animator>()
    }
}

// Public trait for 3D rendering provider (enables engine-3d extraction)
/// Provides access to 3D assets needed by rendering systems
pub trait Asset3DProvider {
    /// Get mutable access to the asset root (for OBJ loading, etc.)
    fn asset_root_mut(&mut self) -> Option<&mut AssetRoot>;
}

impl Asset3DProvider for World {
    fn asset_root_mut(&mut self) -> Option<&mut AssetRoot> {
        self.get_mut::<AssetRoot>()
    }
}

// Implement Scene3DAssetResolver for AssetRoot (enables backward compat + extraction)
impl crate::scene3d_resolve::Scene3DAssetResolver for AssetRoot {
    fn resolve_and_load_asset(&self, asset_path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let full = self.resolve(asset_path);
        let text = std::fs::read_to_string(full)?;
        Ok(text)
    }
}

// Implement RendererProvider for World to work with engine-render-terminal
impl RendererProvider for World {
    fn buffer(&self) -> Option<&Buffer> {
        self.get::<Buffer>()
    }

    fn buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.get_mut::<Buffer>()
    }

    fn virtual_buffer(&self) -> Option<&VirtualBuffer> {
        self.get::<VirtualBuffer>()
    }

    fn runtime_settings(&self) -> Option<&RuntimeSettings> {
        self.get::<RuntimeSettings>()
    }

    fn debug_features(&self) -> Option<&DebugFeatures> {
        self.get::<DebugFeatures>()
    }

    fn debug_log(&self) -> Option<&DebugLogBuffer> {
        self.get::<DebugLogBuffer>()
    }

    fn animator(&self) -> Option<&Animator> {
        self.get::<Animator>()
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

    fn current_scene_id(&self) -> String {
        EngineWorldAccess::scene_runtime(self)
            .map(|sr| sr.scene().id.clone())
            .unwrap_or_else(|| "unknown".to_string())
    }


    fn frame_skip_oracle(&self) -> Option<&Mutex<Box<dyn FrameSkipOracle>>> {
        self.get::<Mutex<Box<dyn FrameSkipOracle>>>()
    }

    fn renderer_mut(&mut self) -> Option<&mut TerminalRenderer> {
        self.get_mut::<TerminalRenderer>()
    }

    fn swap_buffers(&mut self) {
        if let Some(buf) = self.get_mut::<Buffer>() {
            buf.swap();
        }
    }

    fn restore_front_to_back(&mut self) {
        if let Some(buf) = self.get_mut::<Buffer>() {
            buf.restore_front_to_back();
        }
    }

    fn with_virtual_and_output<F: FnOnce(&VirtualBuffer, &mut Buffer)>(&mut self, f: F) {
        self.with_ref_and_mut::<VirtualBuffer, Buffer, _, _>(f);
    }

    fn pipeline_strategies_ptr(&self) -> *const PipelineStrategies {
        self.get::<PipelineStrategies>()
            .map(|ps| ps as *const PipelineStrategies)
            .unwrap_or(std::ptr::null())
    }
}

// Implement LifecycleProvider for World to work with engine-animation
impl LifecycleProvider for World {
    fn animator(&self) -> Option<&dyn std::any::Any> {
        self.get::<Animator>().map(|a| a as &dyn std::any::Any)
    }

    fn animator_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<Animator>().map(|a| a as &mut dyn std::any::Any)
    }

    fn scene_runtime(&self) -> Option<&dyn std::any::Any> {
        self.get::<SceneRuntime>().map(|sr| sr as &dyn std::any::Any)
    }

    fn scene_runtime_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<SceneRuntime>().map(|sr| sr as &mut dyn std::any::Any)
    }

    fn buffer_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<Buffer>().map(|b| b as &mut dyn std::any::Any)
    }

    fn virtual_buffer_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<VirtualBuffer>().map(|b| b as &mut dyn std::any::Any)
    }

    fn runtime_settings(&self) -> Option<&dyn std::any::Any> {
        self.get::<RuntimeSettings>().map(|r| r as &dyn std::any::Any)
    }

    fn debug_features(&self) -> Option<&dyn std::any::Any> {
        self.get::<DebugFeatures>().map(|d| d as &dyn std::any::Any)
    }

    fn debug_log_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<DebugLogBuffer>().map(|d| d as &mut dyn std::any::Any)
    }

    fn events_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<EventQueue>().map(|e| e as &mut dyn std::any::Any)
    }
}

// Implement BehaviorProvider for World
impl BehaviorProvider for World {
    fn scene(&self) -> Option<&dyn std::any::Any> {
        self.get::<Scene>().map(|s| s as &dyn std::any::Any)
    }

    fn animator(&self) -> Option<&dyn std::any::Any> {
        self.get::<Animator>().map(|a| a as &dyn std::any::Any)
    }

    fn scene_runtime_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<SceneRuntime>().map(|sr| sr as &mut dyn std::any::Any)
    }

    fn game_state(&self) -> Option<&dyn std::any::Any> {
        self.get::<crate::game_state::GameState>().map(|gs| gs as &dyn std::any::Any)
    }

    fn mod_behaviors(&self) -> Option<&dyn std::any::Any> {
        self.get::<engine_behavior_registry::ModBehaviorRegistry>().map(|mb| mb as &dyn std::any::Any)
    }

    fn debug_log_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<DebugLogBuffer>().map(|d| d as &mut dyn std::any::Any)
    }

    fn dispatch_audio_command(&mut self, _cmd: Box<dyn std::any::Any>) {
        // TODO: dispatch to audio system
    }

    fn dispatch_behavior_command(&mut self, _cmd: Box<dyn std::any::Any>) {
        // TODO: dispatch to behavior command queue
    }

    fn dispatch_animation_command(&mut self, _cmd: Box<dyn std::any::Any>) {
        // TODO: dispatch to animation command queue
    }

    fn dispatch_lifecycle_command(&mut self, _cmd: Box<dyn std::any::Any>) {
        // TODO: dispatch to lifecycle command queue
    }

    fn events_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<EventQueue>().map(|e| e as &mut dyn std::any::Any)
    }
}

// Implement CompositorProvider for World
impl CompositorProvider for World {
    fn buffer_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<Buffer>().map(|b| b as &mut dyn std::any::Any)
    }

    fn virtual_buffer_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.get_mut::<VirtualBuffer>().map(|b| b as &mut dyn std::any::Any)
    }

    fn scene_runtime(&self) -> Option<&dyn std::any::Any> {
        self.get::<SceneRuntime>().map(|sr| sr as &dyn std::any::Any)
    }

    fn animator(&self) -> Option<&dyn std::any::Any> {
        self.get::<Animator>().map(|a| a as &dyn std::any::Any)
    }

    fn asset_root(&self) -> Option<&dyn std::any::Any> {
        self.get::<AssetRoot>().map(|ar| ar as &dyn std::any::Any)
    }

    fn runtime_settings(&self) -> Option<&dyn std::any::Any> {
        self.get::<RuntimeSettings>().map(|r| r as &dyn std::any::Any)
    }

    fn debug_features(&self) -> Option<&dyn std::any::Any> {
        self.get::<DebugFeatures>().map(|d| d as &dyn std::any::Any)
    }

}

// Implement CompositorAccess for World
impl engine_compositor::CompositorAccess for World {
    fn scene_runtime(&self) -> Option<&dyn std::any::Any> {
        self.get::<SceneRuntime>().map(|sr| sr as &dyn std::any::Any)
    }

    fn animator(&self) -> Option<&Animator> {
        self.get::<Animator>()
    }

    fn buffer_mut(&self) -> Option<&mut Buffer> {
        // Note: mutating through const ref won't compile, so this would be an issue.
        // In practice, the compositor system will take &mut World, not &World.
        None
    }

    fn virtual_buffer_mut(&self) -> Option<&mut VirtualBuffer> {
        None
    }

    fn runtime_settings(&self) -> Option<&RuntimeSettings> {
        self.get::<RuntimeSettings>()
    }

    fn asset_root(&self) -> Option<&AssetRoot> {
        self.get::<AssetRoot>()
    }

    fn scene3d_atlas(&self) -> Option<&dyn std::any::Any> {
        self.get::<engine_compositor::scene3d_atlas::Scene3DAtlas>()
            .map(|atlas| atlas as &dyn std::any::Any)
    }

    fn obj_prerender_frames(&self) -> Option<&dyn std::any::Any> {
        self.get::<engine_compositor::obj_prerender::ObjPrerenderedFrames>()
            .map(|frames| frames as &dyn std::any::Any)
    }

    fn layer_compositor(&self) -> Option<&dyn std::any::Any> {
        // Layer compositor is a strategy, not stored in World
        None
    }

    fn halfblock_packer(&self) -> Option<&dyn std::any::Any> {
        // Halfblock packer is a strategy, not stored in World
        None
    }
}

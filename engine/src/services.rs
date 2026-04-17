//! Internal [`EngineWorldAccess`] trait that gives systems typed access to [`World`] resources.

use crate::assets::AssetRoot;
use crate::audio::AudioRuntime;
use crate::buffer::Buffer;
use crate::debug_features::DebugFeatures;
use crate::debug_log::DebugLogBuffer;
use crate::events::EventQueue;
use crate::runtime_settings::RuntimeSettings;
use crate::scene_loader::SceneLoader;
use crate::scene_runtime::SceneRuntime;
use crate::world::World;
use engine_animation::Animator;
use engine_animation::{AnimatorProvider, LifecycleProvider};
use engine_audio::AudioProvider;
use engine_behavior_registry::BehaviorProvider;
use engine_compositor::CompositorProvider;
use engine_core::scene::Scene;
use engine_render::RendererBackend;
use std::any::Any;

trait WorldResourceAccess {
    fn resource<T: Any + 'static>(&self) -> Option<&T>;
    fn resource_mut<T: Any + 'static>(&mut self) -> Option<&mut T>;
    fn resource_any<T: Any + 'static>(&self) -> Option<&dyn Any>;
    fn resource_any_mut<T: Any + 'static>(&mut self) -> Option<&mut dyn Any>;
}

impl WorldResourceAccess for World {
    fn resource<T: Any + 'static>(&self) -> Option<&T> {
        self.get::<T>()
    }

    fn resource_mut<T: Any + 'static>(&mut self) -> Option<&mut T> {
        self.get_mut::<T>()
    }

    fn resource_any<T: Any + 'static>(&self) -> Option<&dyn Any> {
        self.resource::<T>().map(|value| value as &dyn Any)
    }

    fn resource_any_mut<T: Any + 'static>(&mut self) -> Option<&mut dyn Any> {
        self.resource_mut::<T>().map(|value| value as &mut dyn Any)
    }
}

/// Typed accessor trait for all engine-managed resources stored in [`World`].
pub(crate) trait EngineWorldAccess {
    fn events_mut(&mut self) -> Option<&mut EventQueue>;
    fn scene_runtime(&self) -> Option<&SceneRuntime>;
    fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime>;
    fn animator(&self) -> Option<&Animator>;
    fn animator_mut(&mut self) -> Option<&mut Animator>;
    #[allow(dead_code)]
    fn buffer(&self) -> Option<&Buffer>;
    fn buffer_mut(&mut self) -> Option<&mut Buffer>;
    fn output_buffer(&self) -> Option<&Buffer>;
    fn output_dimensions(&self) -> Option<(u16, u16)>;
    fn runtime_settings(&self) -> Option<&RuntimeSettings>;
    fn audio_runtime_mut(&mut self) -> Option<&mut AudioRuntime>;
    fn asset_root(&self) -> Option<&AssetRoot>;
    fn renderer_mut(&mut self) -> Option<&mut (dyn RendererBackend + '_)>;
    fn scene_loader(&self) -> Option<&SceneLoader>;
}

impl EngineWorldAccess for World {
    fn events_mut(&mut self) -> Option<&mut EventQueue> {
        self.resource_mut::<EventQueue>()
    }

    fn scene_runtime(&self) -> Option<&SceneRuntime> {
        self.resource::<SceneRuntime>()
    }

    fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime> {
        self.resource_mut::<SceneRuntime>()
    }

    fn animator(&self) -> Option<&Animator> {
        self.resource::<Animator>()
    }

    fn animator_mut(&mut self) -> Option<&mut Animator> {
        self.resource_mut::<Animator>()
    }

    fn buffer(&self) -> Option<&Buffer> {
        self.resource::<Buffer>()
    }

    fn buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.resource_mut::<Buffer>()
    }

    /// The active render buffer.
    fn output_buffer(&self) -> Option<&Buffer> {
        self.resource::<Buffer>()
    }

    fn output_dimensions(&self) -> Option<(u16, u16)> {
        self.resource::<Box<dyn RendererBackend>>()
            .map(|renderer| renderer.output_size())
            .or_else(|| {
                self.resource::<Buffer>()
                    .map(|buffer| (buffer.width.max(1), buffer.height.max(1)))
            })
    }

    fn runtime_settings(&self) -> Option<&RuntimeSettings> {
        self.resource::<RuntimeSettings>()
    }

    fn audio_runtime_mut(&mut self) -> Option<&mut AudioRuntime> {
        self.resource_mut::<AudioRuntime>()
    }

    fn asset_root(&self) -> Option<&AssetRoot> {
        self.resource::<AssetRoot>()
    }

    fn renderer_mut(&mut self) -> Option<&mut (dyn RendererBackend + '_)> {
        let renderer = self.get_mut::<Box<dyn RendererBackend>>()?;
        Some(renderer.as_mut())
    }

    fn scene_loader(&self) -> Option<&SceneLoader> {
        self.resource::<SceneLoader>()
    }
}

// Implement AudioProvider for World to work with engine-audio
impl AudioProvider for World {
    fn audio_runtime_mut(&mut self) -> Option<&mut AudioRuntime> {
        self.resource_mut::<AudioRuntime>()
    }
}

// Implement AnimatorProvider for World to work with engine-animation
impl AnimatorProvider for World {
    fn scene(&self) -> Option<Scene> {
        EngineWorldAccess::scene_runtime(self).map(|rt| rt.scene().clone())
    }

    fn animator(&self) -> Option<&Animator> {
        self.resource::<Animator>()
    }

    fn animator_mut(&mut self) -> Option<&mut Animator> {
        self.resource_mut::<Animator>()
    }
}

// Public trait for 3D rendering provider (enables engine-3d extraction)
/// Provides access to 3D assets needed by rendering systems
#[allow(dead_code)]
pub trait Asset3DProvider {
    /// Get mutable access to the asset root (for OBJ loading, etc.)
    fn asset_root_mut(&mut self) -> Option<&mut AssetRoot>;
}

impl Asset3DProvider for World {
    fn asset_root_mut(&mut self) -> Option<&mut AssetRoot> {
        self.resource_mut::<AssetRoot>()
    }
}

// Implement Scene3DAssetResolver for AssetRoot (enables backward compat + extraction)
impl crate::scene3d_resolve::Scene3DAssetResolver for AssetRoot {
    fn resolve_and_load_asset(
        &self,
        asset_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let full = self.resolve(asset_path);
        let text = std::fs::read_to_string(full)?;
        Ok(text)
    }
}

// Implement LifecycleProvider for World to work with engine-animation
impl LifecycleProvider for World {
    fn animator(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<Animator>()
    }

    fn animator_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.resource_any_mut::<Animator>()
    }

    fn scene_runtime(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<SceneRuntime>()
    }

    fn scene_runtime_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.resource_any_mut::<SceneRuntime>()
    }

    fn buffer_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.resource_any_mut::<Buffer>()
    }

    fn runtime_settings(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<RuntimeSettings>()
    }

    fn debug_features(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<DebugFeatures>()
    }

    fn debug_log_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.resource_any_mut::<DebugLogBuffer>()
    }

    fn events_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.resource_any_mut::<EventQueue>()
    }
}

// Implement BehaviorProvider for World
impl BehaviorProvider for World {
    fn scene(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<Scene>()
    }

    fn animator(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<Animator>()
    }

    fn scene_runtime_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.resource_any_mut::<SceneRuntime>()
    }

    fn game_state(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<crate::game_state::GameState>()
    }

    fn mod_behaviors(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<engine_behavior_registry::ModBehaviorRegistry>()
    }

    fn debug_log_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.resource_any_mut::<DebugLogBuffer>()
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
        self.resource_any_mut::<EventQueue>()
    }
}

// Implement CompositorProvider for World
impl CompositorProvider for World {
    fn buffer_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        self.resource_any_mut::<Buffer>()
    }

    fn scene_runtime(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<SceneRuntime>()
    }

    fn animator(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<Animator>()
    }

    fn asset_root(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<AssetRoot>()
    }

    fn runtime_settings(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<RuntimeSettings>()
    }

    fn debug_features(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<DebugFeatures>()
    }
}

// Implement CompositorAccess for World
impl engine_compositor::CompositorAccess for World {
    fn scene_runtime(&self) -> Option<&dyn std::any::Any> {
        self.resource_any::<SceneRuntime>()
    }

    fn animator(&self) -> Option<&Animator> {
        self.resource::<Animator>()
    }

    fn buffer_mut(&self) -> Option<&mut Buffer> {
        // Note: mutating through const ref won't compile, so this would be an issue.
        // In practice, the compositor system will take &mut World, not &World.
        None
    }

    fn runtime_settings(&self) -> Option<&RuntimeSettings> {
        self.resource::<RuntimeSettings>()
    }

    fn asset_root(&self) -> Option<&AssetRoot> {
        self.resource::<AssetRoot>()
    }

    fn scene3d_atlas(&self) -> Option<&dyn std::any::Any> {
        #[cfg(feature = "render-3d")]
        {
            self.resource_any::<engine_render_3d::prerender::Scene3DAtlas>()
        }
        #[cfg(not(feature = "render-3d"))]
        {
            None
        }
    }

    fn obj_prerender_frames(&self) -> Option<&dyn std::any::Any> {
        #[cfg(feature = "render-3d")]
        {
            self.resource_any::<engine_render_3d::prerender::ObjPrerenderedFrames>()
        }
        #[cfg(not(feature = "render-3d"))]
        {
            None
        }
    }

    fn layer_compositor(&self) -> Option<&dyn std::any::Any> {
        // Layer compositor is a strategy, not stored in World
        None
    }
}

// Implement SceneRuntimeAccess for World
impl engine_scene_runtime::SceneRuntimeAccess for World {
    fn scene_runtime(&self) -> Option<&engine_scene_runtime::SceneRuntime> {
        self.resource::<engine_scene_runtime::SceneRuntime>()
    }

    fn scene_runtime_mut(&mut self) -> Option<&mut engine_scene_runtime::SceneRuntime> {
        self.resource_mut::<engine_scene_runtime::SceneRuntime>()
    }
}

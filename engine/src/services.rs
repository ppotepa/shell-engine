//! Internal [`EngineWorldAccess`] trait that gives systems typed access to [`World`] resources.

use crate::assets::AssetRoot;
use crate::audio::AudioRuntime;
use crate::buffer::{Buffer, VirtualBuffer};
use crate::events::EventQueue;
use crate::runtime_settings::RuntimeSettings;
use crate::scene_loader::SceneLoader;
use crate::scene_runtime::SceneRuntime;
use crate::systems::animator::Animator;
use crate::systems::renderer::TerminalRenderer;
use crate::world::World;

/// Typed accessor trait for all engine-managed resources stored in [`World`].
pub(crate) trait EngineWorldAccess {
    fn events_mut(&mut self) -> Option<&mut EventQueue>;
    fn scene_runtime(&self) -> Option<&SceneRuntime>;
    fn scene_runtime_mut(&mut self) -> Option<&mut SceneRuntime>;
    fn animator(&self) -> Option<&Animator>;
    fn animator_mut(&mut self) -> Option<&mut Animator>;
    fn buffer(&self) -> Option<&Buffer>;
    fn buffer_mut(&mut self) -> Option<&mut Buffer>;
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

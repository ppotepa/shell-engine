//! Domain access traits for typed resource retrieval from [`World`].
//!
//! Each trait provides typed accessors for resources that live in this crate.
//! Sub-crates define their own `XxxAccess` traits for their domain types.

use crate::buffer::{Buffer, VirtualBuffer};
use crate::game_state::GameState;
use crate::assets::AssetRoot;
use crate::world::World;

/// Typed access to the terminal and virtual frame buffers.
pub trait BufferAccess {
    fn buffer(&self) -> Option<&Buffer>;
    fn buffer_mut(&mut self) -> Option<&mut Buffer>;
    fn virtual_buffer(&self) -> Option<&VirtualBuffer>;
    fn virtual_buffer_mut(&mut self) -> Option<&mut VirtualBuffer>;
}

impl BufferAccess for World {
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
}

/// Typed access to persistent cross-scene game state.
pub trait GameStateAccess {
    fn game_state(&self) -> Option<&GameState>;
    fn game_state_mut(&mut self) -> Option<&mut GameState>;
}

impl GameStateAccess for World {
    fn game_state(&self) -> Option<&GameState> {
        self.get::<GameState>()
    }
    fn game_state_mut(&mut self) -> Option<&mut GameState> {
        self.get_mut::<GameState>()
    }
}

/// Typed access to the mod asset root directory.
pub trait AssetAccess {
    fn asset_root(&self) -> Option<&AssetRoot>;
}

impl AssetAccess for World {
    fn asset_root(&self) -> Option<&AssetRoot> {
        self.get::<AssetRoot>()
    }
}

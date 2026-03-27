#[cfg(test)]
mod access_trait_integration_tests {
    use crate::access::{AssetAccess, BufferAccess, GameStateAccess};
    use crate::assets::AssetRoot;
    use crate::buffer::Buffer;
    use crate::game_state::GameState;
    use crate::world::World;
    use std::path::PathBuf;

    #[test]
    fn engine_core_access_traits_work() {
        let mut world = World::new();

        // Register resources from engine-core
        world.register(Buffer::new(80, 24));
        world.register(GameState::new());
        world.register(AssetRoot::new(PathBuf::from("/tmp")));

        // Test BufferAccess
        assert!(world.buffer().is_some());
        assert!(world.buffer_mut().is_some());

        // Test GameStateAccess
        assert!(world.game_state().is_some());
        assert!(world.game_state_mut().is_some());

        // Test AssetAccess
        assert!(world.asset_root().is_some());
    }

    #[test]
    fn access_traits_enable_generic_system_signatures() {
        // This demonstrates that systems can be generic over access traits
        // without depending on engine

        let mut world = World::new();
        world.register(Buffer::new(80, 24));
        world.register(GameState::new());

        // System that only uses access traits (no engine deps)
        fn system_uses_traits<T>(world: &mut T)
        where
            T: BufferAccess + GameStateAccess,
        {
            assert!(world.buffer().is_some());
            assert!(world.game_state().is_some());
        }

        system_uses_traits(&mut world);
    }

    #[test]
    fn multiple_core_resources_accessible() {
        let mut world = World::new();

        // Register in any order
        world.register(GameState::new());
        world.register(Buffer::new(80, 24));
        world.register(AssetRoot::new(PathBuf::from("/")));

        // All retrievable
        assert!(world.buffer().is_some());
        assert!(world.game_state().is_some());
        assert!(world.asset_root().is_some());
    }
}

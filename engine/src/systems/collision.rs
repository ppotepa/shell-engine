use engine_game::{CollisionStrategies, GameplayWorld};

/// Runs collision detection for gameplay entities and returns the hits.
pub fn collision_system(world: &mut engine_core::world::World) -> Vec<engine_game::CollisionHit> {
    let (Some(strategies), Some(gameplay_world)) = (
        world.get::<CollisionStrategies>(),
        world.get::<GameplayWorld>(),
    ) else {
        return Vec::new();
    };
    engine_game::collision::collision_system(gameplay_world, strategies)
}

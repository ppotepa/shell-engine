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

/// Runs collision detection for particles (those with ParticlePhysics.collision=true)
/// against collidable entities whose tags match the particle's collision_mask.
pub fn particle_collision_system(world: &mut engine_core::world::World) -> Vec<engine_game::CollisionHit> {
    let (Some(strategies), Some(gameplay_world)) = (
        world.get::<CollisionStrategies>(),
        world.get::<GameplayWorld>(),
    ) else {
        return Vec::new();
    };
    engine_game::collision::particle_collision_system(gameplay_world, strategies)
}

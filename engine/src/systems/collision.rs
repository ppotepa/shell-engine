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

/// Applies impulse-based velocity response for all rigid body collision pairs.
pub fn apply_collision_response(
    world: &mut engine_core::world::World,
    hits: &[engine_game::CollisionHit],
) {
    let Some(gameplay_world) = world.get::<GameplayWorld>() else {
        return;
    };
    engine_game::collision::apply_collision_response(gameplay_world, hits);
}

/// Runs collision detection for particles (those with ParticlePhysics.collision=true)
/// against collidable entities whose tags match the particle's collision_mask.
pub fn particle_collision_system(
    world: &mut engine_core::world::World,
) -> Vec<engine_game::CollisionHit> {
    let (Some(strategies), Some(gameplay_world)) = (
        world.get::<CollisionStrategies>(),
        world.get::<GameplayWorld>(),
    ) else {
        return Vec::new();
    };
    engine_game::collision::particle_collision_system(gameplay_world, strategies)
}

/// Applies bounce response for particles that have ParticlePhysics.bounce > 0.0.
pub fn apply_particle_bounce(
    world: &mut engine_core::world::World,
    hits: &[engine_game::CollisionHit],
) {
    let Some(gameplay_world) = world.get::<GameplayWorld>() else {
        return;
    };
    engine_game::collision::apply_particle_bounce(gameplay_world, hits);
}

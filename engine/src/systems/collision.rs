use engine_game::{CollisionHit, CollisionStrategies, GameplayWorld, WrapStrategy};

/// Runs collision detection for gameplay entities and returns the hits.
pub fn collision_system(
    world: &mut engine_core::world::World,
) -> Vec<CollisionHit> {
    // Keep collision wrap in sync with the active render buffer so games that
    // use centered coordinates (e.g. -W/2..W/2) get toroidal edge collisions.
    if let Some((w, h)) = world
        .get::<engine_core::buffer::Buffer>()
        .map(|buf| (buf.width.max(1), buf.height.max(1)))
    {
        if let Some(strategies) = world.get_mut::<CollisionStrategies>() {
            let half_w = (w as f32) * 0.5;
            let half_h = (h as f32) * 0.5;
            strategies.wrap_strategy = WrapStrategy::Toroid {
                min_x: -half_w,
                max_x: half_w,
                min_y: -half_h,
                max_y: half_h,
            };
        }
    }

    let (Some(strategies), Some(gameplay_world)) = (
        world.get::<CollisionStrategies>(),
        world.get::<GameplayWorld>(),
    ) else {
        return Vec::new();
    };
    engine_game::collision::collision_system(gameplay_world, strategies)
}

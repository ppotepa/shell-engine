use engine_game::{GameplayStrategies, GameplayWorld};

/// Gameplay driver: runs physics, wrap, timer, and lifetime systems over gameplay components.
pub fn gameplay_system(world: &mut engine_core::world::World, dt_ms: u64) {
    // Run physics integration
    if let (Some(strategies), Some(gameplay_world)) = (
        world.get::<GameplayStrategies>(),
        world.get::<GameplayWorld>(),
    ) {
        strategies.physics.step(gameplay_world, dt_ms);
    }

    // Apply toroidal wrap after physics (entities with WrapBounds)
    if let Some(gameplay_world) = world.get::<GameplayWorld>() {
        gameplay_world.apply_wrap();
    }

    // Tick entity timers (cooldowns + statuses)
    if let Some(gameplay_world) = world.get::<GameplayWorld>() {
        gameplay_world.tick_timers(dt_ms);
    }

    // Lifetime decrement and cleanup
    if let Some(gameplay_world) = world.get::<GameplayWorld>().cloned() {
        let ids = gameplay_world.ids_with_lifetime();
        for id in ids {
            if let Some(mut lt) = gameplay_world.lifetime(id) {
                lt.ttl_ms -= dt_ms as i32;
                if lt.ttl_ms <= 0 {
                    if let Some(binding) = gameplay_world.visual(id) {
                        for target in binding.all_visual_ids() {
                            super::visual_binding::queue_visual_despawn(
                                world,
                                target.to_string(),
                            );
                        }
                    }
                    let _ = gameplay_world.despawn(id);
                    continue;
                }
                let _ = gameplay_world.set_lifetime(id, lt);
            }
        }
    }
}

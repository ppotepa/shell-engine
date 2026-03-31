use engine_behavior::EmitterState;
use engine_game::components::DespawnReason;
use engine_game::{GameplayStrategies, GameplayWorld};

fn despawn_lifecycle_entity(
    world: &mut engine_core::world::World,
    gameplay_world: &GameplayWorld,
    emitter_state: Option<&EmitterState>,
    id: u64,
    reason: DespawnReason,
) {
    let should_cleanup_visuals = matches!(
        reason,
        DespawnReason::Manual
            | DespawnReason::Expired
            | DespawnReason::OwnerDestroyed
            | DespawnReason::Collision
            | DespawnReason::SceneReset
            | DespawnReason::InvalidLifecycle
    );

    if should_cleanup_visuals {
        for tree_id in gameplay_world.despawn_tree_ids(id) {
            if let Some(binding) = gameplay_world.visual(tree_id) {
                for target in binding.all_visual_ids() {
                    super::visual_binding::queue_visual_despawn(world, target.to_string());
                }
            }
        }
    }

    let _ = gameplay_world.despawn(id);
    if let Some(state) = emitter_state {
        state.remove_entity(id);
    }
}

/// Gameplay driver: runs ship controllers, physics, wrap, timer, and lifetime systems over gameplay components.
pub fn gameplay_system(world: &mut engine_core::world::World, dt_ms: u64) {
    // Run ship controller logic BEFORE physics (controller sets acceleration)
    if let Some(gameplay_world) = world.get::<GameplayWorld>() {
        super::ship_controller::ship_controller_system(&gameplay_world, dt_ms);
    }

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

    // Tick entity timers (cooldowns + statuses) and world-level one-shot timers.
    if let Some(gameplay_world) = world.get::<GameplayWorld>() {
        gameplay_world.tick_timers(dt_ms);
        gameplay_world.tick_world_timers(dt_ms);
    }

    // Lifecycle decrement and cleanup
    if let Some(gameplay_world) = world.get::<GameplayWorld>().cloned() {
        let emitter_state = world.get::<EmitterState>().cloned();
        let ids = gameplay_world.ids_with_lifecycle();
        for id in ids {
            if !gameplay_world.exists(id) {
                continue;
            }
            let Some(policy) = gameplay_world.lifecycle(id) else {
                continue;
            };

            if policy.is_owner_bound() {
                let Some(ownership) = gameplay_world.ownership(id) else {
                    despawn_lifecycle_entity(
                        world,
                        &gameplay_world,
                        emitter_state.as_ref(),
                        id,
                        DespawnReason::InvalidLifecycle,
                    );
                    continue;
                };
                if !gameplay_world.exists(ownership.owner_id) {
                    despawn_lifecycle_entity(
                        world,
                        &gameplay_world,
                        emitter_state.as_ref(),
                        id,
                        DespawnReason::OwnerDestroyed,
                    );
                    continue;
                }
            }

            if policy.uses_ttl() {
                let Some(mut lt) = gameplay_world.lifetime(id) else {
                    despawn_lifecycle_entity(
                        world,
                        &gameplay_world,
                        emitter_state.as_ref(),
                        id,
                        DespawnReason::InvalidLifecycle,
                    );
                    continue;
                };
                lt.ttl_ms -= dt_ms as i32;
                if lt.ttl_ms <= 0 {
                    despawn_lifecycle_entity(
                        world,
                        &gameplay_world,
                        emitter_state.as_ref(),
                        id,
                        DespawnReason::Expired,
                    );
                    continue;
                }
                let _ = gameplay_world.set_lifetime(id, lt);
            }
        }
    }
}

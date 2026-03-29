//! Destructible system: manages entity destruction and split spawning.
//!
//! When an entity with SplitOnDestroy receives a health_zero event, this system:
//! 1. Marks the split as triggered
//! 2. Counts down the animation delay
//! 3. Emits a split_ready event when children are ready to be spawned
//!
//! Scripts can then poll split_ready events and spawn child entities with configured
//! parameters (count, size_delta, velocity_factor).

use engine_game::GameplayWorld;

/// Run destructible logic for all entities with SplitOnDestroy.
///
/// Processes health_zero events and triggers split animations/spawning.
///
/// For each entity that just died and has SplitOnDestroy:
/// - Trigger the split animation
/// - Count down delay_ms
/// - When delay is complete, is_ready() returns true for scripts to spawn children
pub fn destructible_system(world: &GameplayWorld, dt_ms: u64) {
    // Collect health_zero events this frame: returns (entity, killer) tuples
    let health_zero_events = world.poll_events("health_zero");
    let mut entities_to_split = Vec::new();

    for (entity, _killer) in health_zero_events {
        if world.split_on_destroy(entity).is_some() {
            entities_to_split.push(entity);
        }
    }

    // Trigger split animation for each dead entity with SplitOnDestroy
    for id in entities_to_split {
        let _ = world.with_split_on_destroy(id, |split| {
            if !split.triggered {
                split.triggered = true;
                split.elapsed_ms = 0;
            }
        });
    }

    // Advance split timers for all entities marked for splitting
    let split_ids = world.ids_with_split_on_destroy();
    for id in split_ids {
        let _ = world.with_split_on_destroy(id, |split| {
            if split.triggered {
                split.elapsed_ms += dt_ms as u32;
            }
        });
    }
}

use engine_behavior::EmitterState;
use engine_game::components::{DespawnReason, LifecyclePolicy};
use engine_game::{GameplayStrategies, GameplayWorld};
use rayon::prelude::*;

/// Result of lifecycle check for a single entity.
enum LifecycleAction {
    /// Entity is still alive, update its TTL.
    UpdateTtl(u64, i32),
    /// Entity should be despawned.
    Despawn(u64, DespawnReason),
    /// No action needed.
    None,
}

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
        super::arcade_controller::arcade_controller_system(&gameplay_world, dt_ms);
    }

    // Run physics integration
    if let (Some(strategies), Some(gameplay_world)) = (
        world.get::<GameplayStrategies>(),
        world.get::<GameplayWorld>(),
    ) {
        strategies.physics.step(gameplay_world, dt_ms);
        gameplay_world.apply_angular_velocity(dt_ms);
    }

    // Apply toroidal wrap after physics (entities with WrapBounds)
    if let Some(gameplay_world) = world.get::<GameplayWorld>() {
        gameplay_world.apply_wrap();
        gameplay_world.apply_follow_anchors();
    }

    // Tick entity timers (cooldowns + statuses) and world-level one-shot timers.
    if let Some(gameplay_world) = world.get::<GameplayWorld>() {
        gameplay_world.tick_timers(dt_ms);
        gameplay_world.tick_world_timers(dt_ms);
    }

    // Lifecycle decrement and cleanup (optimized with batch operations)
    if let Some(gameplay_world) = world.get::<GameplayWorld>().cloned() {
        let emitter_state = world.get::<EmitterState>().cloned();
        let ids = gameplay_world.ids_with_lifecycle();
        
        // PHASE 1: Single-lock batch read of all lifecycle data
        let lifecycle_data = gameplay_world.batch_read_lifecycle(&ids);
        
        // PHASE 2: Compute lifecycle actions (parallel only if >32 entities)
        const PARALLEL_THRESHOLD: usize = 32;
        
        let compute_action = |item: &(u64, i32, LifecyclePolicy, Option<u64>)| {
            let (id, ttl_ms, policy, owner_id) = *item;
            // Check owner-bound lifecycle
            if policy.is_owner_bound() {
                match owner_id {
                    None => return LifecycleAction::Despawn(id, DespawnReason::InvalidLifecycle),
                    Some(oid) if !gameplay_world.exists(oid) => {
                        return LifecycleAction::Despawn(id, DespawnReason::OwnerDestroyed);
                    }
                    _ => {}
                }
            }

            // Check TTL-based lifecycle
            if policy.uses_ttl() {
                let new_ttl = ttl_ms - dt_ms as i32;
                if new_ttl <= 0 {
                    return LifecycleAction::Despawn(id, DespawnReason::Expired);
                }
                return LifecycleAction::UpdateTtl(id, new_ttl);
            }

            LifecycleAction::None
        };
        
        let actions: Vec<LifecycleAction> = if lifecycle_data.len() > PARALLEL_THRESHOLD {
            lifecycle_data.par_iter().map(compute_action).collect()
        } else {
            lifecycle_data.iter().map(compute_action).collect()
        };

        // PHASE 3: Collect TTL updates for batch write
        let ttl_updates: Vec<(u64, i32)> = actions
            .iter()
            .filter_map(|a| match a {
                LifecycleAction::UpdateTtl(id, ttl) => Some((*id, *ttl)),
                _ => None,
            })
            .collect();

        // Single-lock batch write all TTL updates
        if !ttl_updates.is_empty() {
            gameplay_world.batch_write_ttl(&ttl_updates);
        }

        // PHASE 4: Apply despawns sequentially (requires world mutation)
        for action in actions {
            if let LifecycleAction::Despawn(id, reason) = action {
                despawn_lifecycle_entity(
                    world,
                    &gameplay_world,
                    emitter_state.as_ref(),
                    id,
                    reason,
                );
            }
        }
    }

    // Update gameplay diagnostics for debug overlay
    if let Some(gameplay_world) = world.get::<GameplayWorld>() {
        let snap = gameplay_world.diagnostic_snapshot();
        let mut parts: Vec<String> = snap
            .by_kind
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect();
        if parts.is_empty() {
            parts.push("(empty)".into());
        }
        let visual_count = gameplay_world.total_visual_count();
        let diag = crate::debug_features::GameplayDiagnostics {
            entity_count: snap.total,
            visual_count,
            summary: parts.join(" "),
        };
        world.register(diag);
    }
}

use engine_behavior::EmitterState;
use engine_game::components::DespawnReason;
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

    // Lifecycle decrement and cleanup (parallel TTL computation)
    if let Some(gameplay_world) = world.get::<GameplayWorld>().cloned() {
        let emitter_state = world.get::<EmitterState>().cloned();
        let ids = gameplay_world.ids_with_lifecycle();
        
        // PHASE 1: Parallel computation of lifecycle actions
        // Each entity is checked independently, no locks during compute
        let actions: Vec<LifecycleAction> = ids
            .par_iter()
            .map(|&id| {
                if !gameplay_world.exists(id) {
                    return LifecycleAction::None;
                }
                let Some(policy) = gameplay_world.lifecycle(id) else {
                    return LifecycleAction::None;
                };

                // Check owner-bound lifecycle
                if policy.is_owner_bound() {
                    let Some(ownership) = gameplay_world.ownership(id) else {
                        return LifecycleAction::Despawn(id, DespawnReason::InvalidLifecycle);
                    };
                    if !gameplay_world.exists(ownership.owner_id) {
                        return LifecycleAction::Despawn(id, DespawnReason::OwnerDestroyed);
                    }
                }

                // Check TTL-based lifecycle
                if policy.uses_ttl() {
                    let Some(lt) = gameplay_world.lifetime(id) else {
                        return LifecycleAction::Despawn(id, DespawnReason::InvalidLifecycle);
                    };
                    let new_ttl = lt.ttl_ms - dt_ms as i32;
                    if new_ttl <= 0 {
                        return LifecycleAction::Despawn(id, DespawnReason::Expired);
                    }
                    return LifecycleAction::UpdateTtl(id, new_ttl);
                }

                LifecycleAction::None
            })
            .collect();

        // PHASE 2: Apply actions sequentially (requires mutable world access)
        for action in actions {
            match action {
                LifecycleAction::Despawn(id, reason) => {
                    despawn_lifecycle_entity(
                        world,
                        &gameplay_world,
                        emitter_state.as_ref(),
                        id,
                        reason,
                    );
                }
                LifecycleAction::UpdateTtl(id, new_ttl) => {
                    if let Some(mut lt) = gameplay_world.lifetime(id) {
                        lt.ttl_ms = new_ttl;
                        let _ = gameplay_world.set_lifetime(id, lt);
                    }
                }
                LifecycleAction::None => {}
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

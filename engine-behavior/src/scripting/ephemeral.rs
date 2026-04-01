use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::BehaviorCommand;
use engine_game::components::DespawnVisual;
use engine_game::{
    FollowAnchor2D, GameplayWorld, LifecyclePolicy, Lifetime, PhysicsBody2D, Transform2D,
    VisualBinding,
};

pub(crate) struct EphemeralSpawn {
    pub(crate) kind: &'static str,
    pub(crate) template: &'static str,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) heading: f32,
    pub(crate) vx: f32,
    pub(crate) vy: f32,
    pub(crate) drag: f32,
    pub(crate) max_speed: f32,
    pub(crate) ttl_ms: Option<i32>,
    pub(crate) owner_id: Option<u64>,
    pub(crate) lifecycle: LifecyclePolicy,
    pub(crate) follow_anchor: Option<FollowAnchor2D>,
    pub(crate) extra_data: BTreeMap<String, JsonValue>,
}

fn queue_scene_cleanup(queue: &Arc<Mutex<Vec<BehaviorCommand>>>, visual_id: &str) {
    if let Ok(mut commands) = queue.lock() {
        commands.push(BehaviorCommand::SceneDespawn {
            target: visual_id.to_string(),
        });
    }
}

pub(crate) fn spawn_ephemeral_visual(
    world: &GameplayWorld,
    queue: &Arc<Mutex<Vec<BehaviorCommand>>>,
    spec: EphemeralSpawn,
) -> Option<u64> {
    if spec.kind.trim().is_empty() || spec.template.trim().is_empty() {
        return None;
    }
    if spec.lifecycle.uses_ttl() && spec.ttl_ms.unwrap_or(0) <= 0 {
        return None;
    }
    if spec.lifecycle.is_owner_bound() && spec.owner_id.is_none() {
        return None;
    }

    let entity_id = world.spawn(spec.kind, JsonValue::Object(JsonMap::new()))?;
    let visual_id = format!("{}-{}", spec.kind, entity_id);

    {
        let Ok(mut commands) = queue.lock() else {
            let _ = world.despawn(entity_id);
            return None;
        };
        commands.push(BehaviorCommand::SceneSpawn {
            template: spec.template.to_string(),
            target: visual_id.clone(),
        });
    }

    if !world.set_visual(
        entity_id,
        VisualBinding {
            visual_id: Some(visual_id.clone()),
            additional_visuals: Vec::new(),
        },
    ) {
        let _ = world.despawn(entity_id);
        queue_scene_cleanup(queue, &visual_id);
        return None;
    }

    if !world.set_lifecycle(entity_id, spec.lifecycle) {
        let _ = world.despawn(entity_id);
        queue_scene_cleanup(queue, &visual_id);
        return None;
    }

    if !world.set_transform(
        entity_id,
        Transform2D {
            x: spec.x,
            y: spec.y,
            heading: spec.heading,
        },
    ) {
        let _ = world.despawn(entity_id);
        queue_scene_cleanup(queue, &visual_id);
        return None;
    }

    if spec.lifecycle.uses_ttl() {
        let ttl = spec.ttl_ms.unwrap_or_default();
        if !world.set_lifetime(
            entity_id,
            Lifetime {
                ttl_ms: ttl,
                original_ttl_ms: ttl,
                on_expire: DespawnVisual::None,
            },
        ) {
            let _ = world.despawn(entity_id);
            queue_scene_cleanup(queue, &visual_id);
            return None;
        }
    }

    if !world.set_physics(
        entity_id,
        PhysicsBody2D {
            vx: spec.vx,
            vy: spec.vy,
            ax: 0.0,
            ay: 0.0,
            drag: spec.drag,
            max_speed: spec.max_speed,
        },
    ) {
        let _ = world.despawn(entity_id);
        queue_scene_cleanup(queue, &visual_id);
        return None;
    }

    if spec.lifecycle.is_transient() && !world.tag_add(entity_id, "ephemeral") {
        let _ = world.despawn(entity_id);
        queue_scene_cleanup(queue, &visual_id);
        return None;
    }

    if let Some(owner_id) = spec.owner_id {
        if !world.exists(owner_id) || !world.register_child(owner_id, entity_id) {
            let _ = world.despawn(entity_id);
            queue_scene_cleanup(queue, &visual_id);
            return None;
        }
    }

    if spec.lifecycle.follows_owner() {
        if !world.set_follow_anchor(entity_id, spec.follow_anchor.unwrap_or_default()) {
            let _ = world.despawn(entity_id);
            queue_scene_cleanup(queue, &visual_id);
            return None;
        }
    }

    if !spec.extra_data.is_empty() && !world.set_many(entity_id, &spec.extra_data) {
        let _ = world.despawn(entity_id);
        queue_scene_cleanup(queue, &visual_id);
        return None;
    }

    Some(entity_id)
}

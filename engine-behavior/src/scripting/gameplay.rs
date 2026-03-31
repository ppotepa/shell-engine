//! Gameplay domain APIs: ScriptGameplayApi for world management, ScriptGameplayEntityApi for entity interaction.

use rhai::{Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};

use crate::geometry::{points_to_rhai_array, rhai_array_to_points, rotate_points_i32, sin32_i32, to_i32};

pub(crate) use super::gameplay_impl::{ScriptGameplayApi, ScriptGameplayEntityApi};

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptGameplayApi>("GameplayApi");
    engine.register_type_with_name::<ScriptGameplayEntityApi>("GameplayEntityApi");

    // Gameplay API - world/collection operations
    engine.register_fn("clear", |world: &mut ScriptGameplayApi| {
        world.clear();
    });
    engine.register_fn("reset_dynamic_entities", |world: &mut ScriptGameplayApi| {
        world.reset_dynamic_entities()
    });
    engine.register_fn("count", |world: &mut ScriptGameplayApi| world.count());
    engine.register_fn("count_kind", |world: &mut ScriptGameplayApi, kind: &str| {
        world.count_kind(kind)
    });
    engine.register_fn("count_tag", |world: &mut ScriptGameplayApi, tag: &str| {
        world.count_tag(tag)
    });
    engine.register_fn("first_kind", |world: &mut ScriptGameplayApi, kind: &str| {
        world.first_kind(kind)
    });
    engine.register_fn("first_tag", |world: &mut ScriptGameplayApi, tag: &str| {
        world.first_tag(tag)
    });
    engine.register_fn("diagnostic_info", |world: &mut ScriptGameplayApi| {
        world.diagnostic_info()
    });
    engine.register_fn(
        "spawn_object",
        |world: &mut ScriptGameplayApi, kind: &str, payload: RhaiDynamic| {
            world.spawn(kind, payload)
        },
    );
    engine.register_fn(
        "despawn_object",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.despawn(id),
    );
    engine.register_fn("despawn", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.despawn(id)
    });
    engine.register_fn("exists", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.exists(id)
    });
    engine.register_fn("kind", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.kind(id)
    });
    engine.register_fn("tags", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.tags(id)
    });
    engine.register_fn("ids", |world: &mut ScriptGameplayApi| world.ids());
    engine.register_fn("entity", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.entity(id)
    });
    engine.register_fn("query_kind", |world: &mut ScriptGameplayApi, kind: &str| {
        world.query_kind(kind)
    });
    engine.register_fn("query_tag", |world: &mut ScriptGameplayApi, tag: &str| {
        world.query_tag(tag)
    });
    engine.register_fn(
        "get",
        |world: &mut ScriptGameplayApi, id: rhai::INT, path: &str| world.get(id, path),
    );
    engine.register_fn(
        "set",
        |world: &mut ScriptGameplayApi, id: rhai::INT, path: &str, value: RhaiDynamic| {
            world.set(id, path, value)
        },
    );
    engine.register_fn(
        "has",
        |world: &mut ScriptGameplayApi, id: rhai::INT, path: &str| world.has(id, path),
    );
    engine.register_fn(
        "remove",
        |world: &mut ScriptGameplayApi, id: rhai::INT, path: &str| world.remove(id, path),
    );
    engine.register_fn(
        "push",
        |world: &mut ScriptGameplayApi, id: rhai::INT, path: &str, value: RhaiDynamic| {
            world.push(id, path, value)
        },
    );
    engine.register_fn(
        "set_transform",
        |world: &mut ScriptGameplayApi,
         id: rhai::INT,
         x: rhai::FLOAT,
         y: rhai::FLOAT,
         heading: rhai::FLOAT| { world.set_transform(id, x, y, heading) },
    );
    engine.register_fn(
        "transform",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.transform(id),
    );
    // Physics operations now exclusively through ScriptPhysicsApi domain
    engine.register_fn(
        "set_lifetime",
        |world: &mut ScriptGameplayApi, id: rhai::INT, ttl_ms: rhai::INT| {
            world.set_lifetime(id, ttl_ms)
        },
    );
    engine.register_fn(
        "set_visual",
        |world: &mut ScriptGameplayApi, id: rhai::INT, visual_id: &str| {
            world.set_visual(id, visual_id)
        },
    );
    engine.register_fn(
        "bind_visual",
        |world: &mut ScriptGameplayApi, id: rhai::INT, visual_id: &str| {
            world.bind_visual(id, visual_id)
        },
    );
    engine.register_fn(
        "spawn_visual",
        |world: &mut ScriptGameplayApi, kind: &str, template: &str, data: RhaiMap| {
            world.spawn_visual(kind, template, data)
        },
    );
    engine.register_fn(
        "spawn_prefab",
        |world: &mut ScriptGameplayApi, name: &str, args: RhaiMap| world.spawn_prefab(name, args),
    );
    engine.register_fn(
        "spawn_group",
        |world: &mut ScriptGameplayApi, group_name: &str, prefab_name: &str| {
            world.spawn_group(group_name, prefab_name)
        },
    );
    engine.register_fn(
        "collisions",
        |world: &mut ScriptGameplayApi| {
            world.collisions()
        },
    );
    engine.register_fn(
        "collisions_between",
        |world: &mut ScriptGameplayApi, kind_a: &str, kind_b: &str| {
            world.collisions_between(kind_a, kind_b)
        },
    );
    engine.register_fn(
        "collisions_of",
        |world: &mut ScriptGameplayApi, kind: &str| world.collisions_of(kind),
    );
    // ── Collision enter/stay/exit events ──────────────────────────────────
    engine.register_fn(
        "collision_enters",
        |world: &mut ScriptGameplayApi, kind_a: &str, kind_b: &str| {
            world.collision_enters_between(kind_a, kind_b)
        },
    );
    engine.register_fn(
        "collision_stays",
        |world: &mut ScriptGameplayApi, kind_a: &str, kind_b: &str| {
            world.collision_stays_between(kind_a, kind_b)
        },
    );
    engine.register_fn(
        "collision_exits",
        |world: &mut ScriptGameplayApi, kind_a: &str, kind_b: &str| {
            world.collision_exits_between(kind_a, kind_b)
        },
    );
    // ── Child entity API ──────────────────────────────────────────────────
    engine.register_fn(
        "spawn_child",
        |world: &mut ScriptGameplayApi,
         parent_id: rhai::INT,
         kind: &str,
         template: &str,
         data: RhaiMap| { world.spawn_child_entity(parent_id, kind, template, data) },
    );
    engine.register_fn(
        "despawn_children",
        |world: &mut ScriptGameplayApi, parent_id: rhai::INT| world.despawn_children_of(parent_id),
    );
    engine.register_fn(
        "distance",
        |world: &mut ScriptGameplayApi, a: rhai::INT, b: rhai::INT| -> rhai::FLOAT {
            world.distance(a, b)
        },
    );
    engine.register_fn(
        "any_alive",
        |world: &mut ScriptGameplayApi, kind: &str| -> bool { world.any_alive(kind) },
    );
    engine.register_fn(
        "set_world_bounds",
        |world: &mut ScriptGameplayApi,
         min_x: rhai::FLOAT,
         min_y: rhai::FLOAT,
         max_x: rhai::FLOAT,
         max_y: rhai::FLOAT| { world.set_world_bounds(min_x, min_y, max_x, max_y) },
    );
    engine.register_fn("world_bounds", |world: &mut ScriptGameplayApi| -> RhaiMap {
        world.world_bounds()
    });
    engine.register_fn(
        "rand_seed",
        |world: &mut ScriptGameplayApi, seed: rhai::INT| world.rand_seed(seed),
    );
    engine.register_fn(
        "tag_add",
        |world: &mut ScriptGameplayApi, id: rhai::INT, tag: &str| world.tag_add(id, tag),
    );
    engine.register_fn(
        "tag_remove",
        |world: &mut ScriptGameplayApi, id: rhai::INT, tag: &str| world.tag_remove(id, tag),
    );
    engine.register_fn(
        "tag_has",
        |world: &mut ScriptGameplayApi, id: rhai::INT, tag: &str| world.tag_has(id, tag),
    );
    engine.register_fn(
        "after_ms",
        |world: &mut ScriptGameplayApi, label: &str, delay_ms: rhai::INT| {
            world.after_ms(label, delay_ms)
        },
    );
    engine.register_fn(
        "timer_fired",
        |world: &mut ScriptGameplayApi, label: &str| world.timer_fired(label),
    );
    engine.register_fn(
        "cancel_timer",
        |world: &mut ScriptGameplayApi, label: &str| world.cancel_timer(label),
    );
    engine.register_fn(
        "ship_set_turn",
        |world: &mut ScriptGameplayApi, id: rhai::INT, dir: rhai::INT| world.ship_set_turn(id, dir),
    );
    engine.register_fn(
        "ship_set_thrust",
        |world: &mut ScriptGameplayApi, id: rhai::INT, on: bool| world.ship_set_thrust(id, on),
    );
    engine.register_fn(
        "ship_heading",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.ship_heading(id),
    );
    engine.register_fn(
        "ship_heading_vector",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.ship_heading_vector(id),
    );
    engine.register_fn(
        "ship_velocity",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.ship_velocity(id),
    );
    engine.register_fn("poll_collision_events", |world: &mut ScriptGameplayApi| {
        world.poll_collision_events()
    });
    engine.register_fn("clear_events", |world: &mut ScriptGameplayApi| {
        world.clear_events()
    });
    engine.register_fn(
        "spawn_batch",
        |world: &mut ScriptGameplayApi, specs: rhai::Array| world.spawn_batch(specs),
    );
    engine.register_fn(
        "enable_wrap",
        |world: &mut ScriptGameplayApi,
         id: rhai::INT,
         min_x: rhai::FLOAT,
         max_x: rhai::FLOAT,
         min_y: rhai::FLOAT,
         max_y: rhai::FLOAT| { world.enable_wrap(id, min_x, max_x, min_y, max_y) },
    );
    engine.register_fn(
        "disable_wrap",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.disable_wrap(id),
    );
    engine.register_fn("physics", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.physics(id)
    });
    engine.register_fn(
        "set_physics",
        |world: &mut ScriptGameplayApi,
         id: rhai::INT,
         vx: rhai::FLOAT,
         vy: rhai::FLOAT,
         ax: rhai::FLOAT,
         ay: rhai::FLOAT,
         drag: rhai::FLOAT,
         max_speed: rhai::FLOAT| { world.set_physics(id, vx, vy, ax, ay, drag, max_speed) },
    );

    // Gameplay Entity API
    engine.register_fn("exists", |entity: &mut ScriptGameplayEntityApi| {
        entity.exists()
    });
    engine.register_fn("get", |entity: &mut ScriptGameplayEntityApi, path: &str| {
        entity.get(path)
    });
    engine.register_fn(
        "get_i",
        |entity: &mut ScriptGameplayEntityApi, path: &str, fallback: rhai::INT| {
            entity.get_i(path, fallback)
        },
    );
    engine.register_fn(
        "get_bool",
        |entity: &mut ScriptGameplayEntityApi, path: &str, fallback: bool| {
            entity.get_bool(path, fallback)
        },
    );
    engine.register_fn(
        "get_b",
        |entity: &mut ScriptGameplayEntityApi, path: &str, fallback: bool| {
            entity.get_bool(path, fallback)
        },
    );
    engine.register_fn(
        "set",
        |entity: &mut ScriptGameplayEntityApi, path: &str, value: RhaiDynamic| {
            entity.set(path, value)
        },
    );
    engine.register_fn("kind", |entity: &mut ScriptGameplayEntityApi| entity.kind());
    engine.register_fn("tags", |entity: &mut ScriptGameplayEntityApi| entity.tags());
    engine.register_fn("get_metadata", |entity: &mut ScriptGameplayEntityApi| {
        entity.get_metadata()
    });
    engine.register_fn("get_components", |entity: &mut ScriptGameplayEntityApi| {
        entity.get_components()
    });
    engine.register_fn("transform", |entity: &mut ScriptGameplayEntityApi| {
        entity.transform()
    });
    engine.register_fn(
        "set_position",
        |entity: &mut ScriptGameplayEntityApi, x: rhai::FLOAT, y: rhai::FLOAT| {
            entity.set_position(x, y)
        },
    );
    engine.register_fn(
        "set_heading",
        |entity: &mut ScriptGameplayEntityApi, heading: rhai::FLOAT| entity.set_heading(heading),
    );

    // Physics as a property: ship.physics.velocity(), ship.physics.set_velocity(), etc.
    engine.register_get("physics", |entity: &mut ScriptGameplayEntityApi| {
        entity.physics.clone()
    });

    engine.register_fn(
        "lifetime_remaining",
        |entity: &mut ScriptGameplayEntityApi| entity.lifetime_remaining(),
    );
    engine.register_fn(
        "set_many",
        |entity: &mut ScriptGameplayEntityApi, map: RhaiMap| entity.set_many(map),
    );
    engine.register_fn("data", |entity: &mut ScriptGameplayEntityApi| entity.data());
    engine.register_fn(
        "get_f",
        |entity: &mut ScriptGameplayEntityApi, path: &str, fallback: rhai::FLOAT| {
            entity.get_f(path, fallback)
        },
    );
    engine.register_fn(
        "get_s",
        |entity: &mut ScriptGameplayEntityApi, path: &str, fallback: &str| {
            entity.get_s(path, fallback)
        },
    );
    engine.register_fn("despawn", |entity: &mut ScriptGameplayEntityApi| {
        entity.despawn()
    });
    engine.register_fn("id", |entity: &mut ScriptGameplayEntityApi| -> rhai::INT {
        entity.id()
    });
    engine.register_fn(
        "flag",
        |entity: &mut ScriptGameplayEntityApi, name: &str| -> bool { entity.flag(name) },
    );
    engine.register_fn(
        "set_flag",
        |entity: &mut ScriptGameplayEntityApi, name: &str, value: bool| -> bool {
            entity.set_flag(name, value)
        },
    );

    // ── Cooldown API ──────────────────────────────────────────────────────
    engine.register_fn(
        "cooldown_start",
        |entity: &mut ScriptGameplayEntityApi, name: &str, ms: rhai::INT| {
            entity.cooldown_start(name, ms)
        },
    );
    engine.register_fn(
        "cooldown_ready",
        |entity: &mut ScriptGameplayEntityApi, name: &str| entity.cooldown_ready(name),
    );
    engine.register_fn(
        "cooldown_remaining",
        |entity: &mut ScriptGameplayEntityApi, name: &str| -> rhai::INT {
            entity.cooldown_remaining(name)
        },
    );

    // ── Status API ────────────────────────────────────────────────────────
    engine.register_fn(
        "status_add",
        |entity: &mut ScriptGameplayEntityApi, name: &str, ms: rhai::INT| {
            entity.status_add(name, ms)
        },
    );
    engine.register_fn(
        "status_has",
        |entity: &mut ScriptGameplayEntityApi, name: &str| entity.status_has(name),
    );
    engine.register_fn(
        "status_remaining",
        |entity: &mut ScriptGameplayEntityApi, name: &str| -> rhai::INT {
            entity.status_remaining(name)
        },
    );

    // ── Ship Controller API (on entity ref) ───────────────────────────────
    engine.register_fn(
        "attach_ship_controller",
        |entity: &mut ScriptGameplayEntityApi, config: RhaiMap| {
            entity.attach_ship_controller(config)
        },
    );
    engine.register_fn(
        "set_turn",
        |entity: &mut ScriptGameplayEntityApi, dir: rhai::INT| entity.set_turn(dir),
    );
    engine.register_fn(
        "set_thrust",
        |entity: &mut ScriptGameplayEntityApi, on: bool| entity.set_thrust(on),
    );
    engine.register_fn("physics", |entity: &mut ScriptGameplayEntityApi| {
        entity.physics.clone()
    });
    engine.register_fn(
        "set_acceleration",
        |entity: &mut ScriptGameplayEntityApi, ax: rhai::FLOAT, ay: rhai::FLOAT| {
            entity.set_acceleration(ax, ay)
        },
    );
    engine.register_fn("collider", |entity: &mut ScriptGameplayEntityApi| {
        entity.collider()
    });
    engine.register_fn("heading", |entity: &mut ScriptGameplayEntityApi| {
        entity.heading()
    });
    engine.register_fn("heading_vector", |entity: &mut ScriptGameplayEntityApi| {
        entity.heading_vector()
    });

    // ── Geometry utilities ───────────────────────────────────────────────────────
    // TODO: Move to mod-level shared script once Rhai module system is added (A4)
    engine.register_fn(
        "rotate_points",
        |points: rhai::Array, heading: rhai::INT| -> rhai::Array {
            let points = rhai_array_to_points(&points);
            points_to_rhai_array(rotate_points_i32(&points, to_i32(heading)))
        },
    );
    engine.register_fn("sin32", |idx: rhai::INT| -> rhai::INT {
        sin32_i32(to_i32(idx)) as rhai::INT
    });

    // ── Numeric utility functions ────────────────────────────────────────────
    engine.register_fn("to_i", |v: rhai::INT| -> rhai::INT { v });
    engine.register_fn("to_i", |v: rhai::FLOAT| -> rhai::INT { v as rhai::INT });
    engine.register_fn(
        "clamp_i",
        |v: rhai::INT, min_v: rhai::INT, max_v: rhai::INT| -> rhai::INT {
            if v < min_v {
                min_v
            } else if v > max_v {
                max_v
            } else {
                v
            }
        },
    );
}

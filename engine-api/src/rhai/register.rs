//! Domain-oriented Rhai registration entry points.

use rhai::Engine as RhaiEngine;

use crate::gameplay::api::{GameplayEntityCoreApi, GameplayWorldCoreApi};
use crate::gameplay::geometry::{
    crack_polygon_i32, dent_polygon_i32, jitter_points_i32, points_to_rhai_array,
    regular_polygon_i32, rhai_array_to_points, rotate_points_i32, sin32_i32, to_i32,
};

pub fn register_geometry_api(engine: &mut RhaiEngine) {
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
    engine.register_fn(
        "regular_polygon",
        |sides: rhai::INT, radius: rhai::INT| -> rhai::Array {
            points_to_rhai_array(regular_polygon_i32(to_i32(sides), to_i32(radius)))
        },
    );
    engine.register_fn(
        "jitter_points",
        |points: rhai::Array, amount: rhai::INT, seed: rhai::INT| -> rhai::Array {
            let points = rhai_array_to_points(&points);
            points_to_rhai_array(jitter_points_i32(&points, to_i32(amount), to_i32(seed)))
        },
    );
    engine.register_fn(
        "dent_polygon",
        |points: rhai::Array,
         impact_x: rhai::INT,
         impact_y: rhai::INT,
         strength: rhai::INT|
         -> rhai::Array {
            let pts = rhai_array_to_points(&points);
            points_to_rhai_array(dent_polygon_i32(
                &pts,
                to_i32(impact_x),
                to_i32(impact_y),
                to_i32(strength),
            ))
        },
    );
    engine.register_fn(
        "crack_polygon",
        |points: rhai::Array,
         impact_x: rhai::INT,
         impact_y: rhai::INT,
         depth: rhai::INT|
         -> rhai::Array {
            let pts = rhai_array_to_points(&points);
            points_to_rhai_array(crack_polygon_i32(
                &pts,
                to_i32(impact_x),
                to_i32(impact_y),
                to_i32(depth),
            ))
        },
    );
    engine.register_fn(
        "subtract_polygon",
        |poly_a: rhai::Array, poly_b: rhai::Array| -> rhai::Array {
            let a = rhai_array_to_points(&poly_a);
            let b = rhai_array_to_points(&poly_b);
            let results = engine_physics::subtract_polygons(&a, &b);
            results
                .into_iter()
                .map(|poly| -> rhai::Dynamic { points_to_rhai_array(poly).into() })
                .collect()
        },
    );
    engine.register_fn(
        "polygon_area",
        |points: rhai::Array| -> rhai::INT {
            let pts = rhai_array_to_points(&points);
            engine_physics::polygon_area(&pts) as rhai::INT
        },
    );
}

pub fn register_numeric_api(engine: &mut RhaiEngine) {
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

pub fn register_gameplay_core_api<TWorld, TEntity>(engine: &mut RhaiEngine)
where
    TWorld: GameplayWorldCoreApi<TEntity>,
    TEntity: GameplayEntityCoreApi,
{
    engine.register_type_with_name::<TWorld>("GameplayApi");
    engine.register_type_with_name::<TEntity>("GameplayEntityApi");

    engine.register_fn("clear", |world: &mut TWorld| {
        world.clear();
    });
    engine.register_fn("reset_dynamic_entities", |world: &mut TWorld| {
        world.reset_dynamic_entities()
    });
    engine.register_fn("count", |world: &mut TWorld| world.count());
    engine.register_fn("count_kind", |world: &mut TWorld, kind: &str| world.count_kind(kind));
    engine.register_fn("count_tag", |world: &mut TWorld, tag: &str| world.count_tag(tag));
    engine.register_fn("first_kind", |world: &mut TWorld, kind: &str| world.first_kind(kind));
    engine.register_fn("first_tag", |world: &mut TWorld, tag: &str| world.first_tag(tag));
    engine.register_fn("diagnostic_info", |world: &mut TWorld| world.diagnostic_info());
    engine.register_fn(
        "spawn_object",
        |world: &mut TWorld, kind: &str, payload: rhai::Dynamic| world.spawn(kind, payload),
    );
    engine.register_fn("despawn_object", |world: &mut TWorld, id: rhai::INT| world.despawn(id));
    engine.register_fn("despawn", |world: &mut TWorld, id: rhai::INT| world.despawn(id));
    engine.register_fn("exists", |world: &mut TWorld, id: rhai::INT| world.exists(id));
    engine.register_fn("kind", |world: &mut TWorld, id: rhai::INT| world.kind(id));
    engine.register_fn("tags", |world: &mut TWorld, id: rhai::INT| world.tags(id));
    engine.register_fn("ids", |world: &mut TWorld| world.ids());
    engine.register_fn("entity", |world: &mut TWorld, id: rhai::INT| world.entity(id));
    engine.register_fn("query_kind", |world: &mut TWorld, kind: &str| world.query_kind(kind));
    engine.register_fn("query_tag", |world: &mut TWorld, tag: &str| world.query_tag(tag));
    engine.register_fn("get", |world: &mut TWorld, id: rhai::INT, path: &str| {
        world.get(id, path)
    });
    engine.register_fn(
        "set",
        |world: &mut TWorld, id: rhai::INT, path: &str, value: rhai::Dynamic| {
            world.set(id, path, value)
        },
    );
    engine.register_fn("has", |world: &mut TWorld, id: rhai::INT, path: &str| {
        world.has(id, path)
    });
    engine.register_fn("remove", |world: &mut TWorld, id: rhai::INT, path: &str| {
        world.remove(id, path)
    });
    engine.register_fn(
        "push",
        |world: &mut TWorld, id: rhai::INT, path: &str, value: rhai::Dynamic| {
            world.push(id, path, value)
        },
    );
    engine.register_fn(
        "set_transform",
        |world: &mut TWorld,
         id: rhai::INT,
         x: rhai::FLOAT,
         y: rhai::FLOAT,
         heading: rhai::FLOAT| { world.set_transform(id, x, y, heading) },
    );
    engine.register_fn("transform", |world: &mut TWorld, id: rhai::INT| world.transform(id));
    engine.register_fn(
        "set_physics",
        |world: &mut TWorld,
         id: rhai::INT,
         vx: rhai::FLOAT,
         vy: rhai::FLOAT,
         ax: rhai::FLOAT,
         ay: rhai::FLOAT,
         drag: rhai::FLOAT,
         max_speed: rhai::FLOAT| { world.set_physics(id, vx, vy, ax, ay, drag, max_speed) },
    );
    engine.register_fn("physics", |world: &mut TWorld, id: rhai::INT| world.physics(id));
    engine.register_fn("set_lifetime", |world: &mut TWorld, id: rhai::INT, ttl_ms: rhai::INT| {
        world.set_lifetime(id, ttl_ms)
    });
    engine.register_fn("collisions", |world: &mut TWorld| world.collisions());
    engine.register_fn(
        "collisions_between",
        |world: &mut TWorld, kind_a: &str, kind_b: &str| world.collisions_between(kind_a, kind_b),
    );
    engine.register_fn("collisions_of", |world: &mut TWorld, kind: &str| {
        world.collisions_of(kind)
    });
    engine.register_fn(
        "collision_enters",
        |world: &mut TWorld, kind_a: &str, kind_b: &str| {
            world.collision_enters_between(kind_a, kind_b)
        },
    );
    engine.register_fn(
        "collision_stays",
        |world: &mut TWorld, kind_a: &str, kind_b: &str| {
            world.collision_stays_between(kind_a, kind_b)
        },
    );
    engine.register_fn(
        "collision_exits",
        |world: &mut TWorld, kind_a: &str, kind_b: &str| {
            world.collision_exits_between(kind_a, kind_b)
        },
    );
    engine.register_fn(
        "spawn_child",
        |world: &mut TWorld,
         parent_id: rhai::INT,
         kind: &str,
         template: &str,
         data: rhai::Map| { world.spawn_child_entity(parent_id, kind, template, data) },
    );
    engine.register_fn("despawn_children", |world: &mut TWorld, parent_id: rhai::INT| {
        world.despawn_children_of(parent_id)
    });
    engine.register_fn("distance", |world: &mut TWorld, a: rhai::INT, b: rhai::INT| {
        world.distance(a, b)
    });
    engine.register_fn("any_alive", |world: &mut TWorld, kind: &str| world.any_alive(kind));
    engine.register_fn(
        "set_world_bounds",
        |world: &mut TWorld,
         min_x: rhai::FLOAT,
         min_y: rhai::FLOAT,
         max_x: rhai::FLOAT,
         max_y: rhai::FLOAT| { world.set_world_bounds(min_x, min_y, max_x, max_y) },
    );
    engine.register_fn("world_bounds", |world: &mut TWorld| world.world_bounds());
    engine.register_fn("rand_i", |world: &mut TWorld, min: rhai::INT, max: rhai::INT| {
        world.rand_i(min, max)
    });
    engine.register_fn("rand_seed", |world: &mut TWorld, seed: rhai::INT| {
        world.rand_seed(seed)
    });
    engine.register_fn("tag_add", |world: &mut TWorld, id: rhai::INT, tag: &str| {
        world.tag_add(id, tag)
    });
    engine.register_fn("tag_remove", |world: &mut TWorld, id: rhai::INT, tag: &str| {
        world.tag_remove(id, tag)
    });
    engine.register_fn("tag_has", |world: &mut TWorld, id: rhai::INT, tag: &str| {
        world.tag_has(id, tag)
    });
    engine.register_fn("after_ms", |world: &mut TWorld, label: &str, delay_ms: rhai::INT| {
        world.after_ms(label, delay_ms)
    });
    engine.register_fn("timer_fired", |world: &mut TWorld, label: &str| {
        world.timer_fired(label)
    });
    engine.register_fn("cancel_timer", |world: &mut TWorld, label: &str| {
        world.cancel_timer(label)
    });
    engine.register_fn(
        "enable_wrap",
        |world: &mut TWorld,
         id: rhai::INT,
         min_x: rhai::FLOAT,
         max_x: rhai::FLOAT,
         min_y: rhai::FLOAT,
         max_y: rhai::FLOAT| { world.enable_wrap(id, min_x, max_x, min_y, max_y) },
    );
    engine.register_fn("disable_wrap", |world: &mut TWorld, id: rhai::INT| {
        world.disable_wrap(id)
    });
    engine.register_fn("poll_collision_events", |world: &mut TWorld| {
        world.poll_collision_events()
    });
    engine.register_fn("clear_events", |world: &mut TWorld| {
        world.clear_events();
    });
    engine.register_fn("spawn_batch", |world: &mut TWorld, specs: rhai::Array| {
        world.spawn_batch(specs)
    });

    engine.register_fn("exists", |entity: &mut TEntity| entity.exists());
    engine.register_fn("get", |entity: &mut TEntity, path: &str| entity.get(path));
    engine.register_fn(
        "get_i",
        |entity: &mut TEntity, path: &str, fallback: rhai::INT| entity.get_i(path, fallback),
    );
    engine.register_fn(
        "get_bool",
        |entity: &mut TEntity, path: &str, fallback: bool| entity.get_bool(path, fallback),
    );
    engine.register_fn(
        "get_b",
        |entity: &mut TEntity, path: &str, fallback: bool| entity.get_bool(path, fallback),
    );
    engine.register_fn(
        "set",
        |entity: &mut TEntity, path: &str, value: rhai::Dynamic| entity.set(path, value),
    );
    engine.register_fn("kind", |entity: &mut TEntity| entity.kind());
    engine.register_fn("tags", |entity: &mut TEntity| entity.tags());
    engine.register_fn("get_metadata", |entity: &mut TEntity| entity.get_metadata());
    engine.register_fn("get_components", |entity: &mut TEntity| entity.get_components());
    engine.register_fn("transform", |entity: &mut TEntity| entity.transform());
    engine.register_fn("set_position", |entity: &mut TEntity, x: rhai::FLOAT, y: rhai::FLOAT| {
        entity.set_position(x, y)
    });
    engine.register_fn("set_heading", |entity: &mut TEntity, heading: rhai::FLOAT| {
        entity.set_heading(heading)
    });
    engine.register_fn("lifetime_remaining", |entity: &mut TEntity| {
        entity.lifetime_remaining()
    });
    engine.register_fn("set_many", |entity: &mut TEntity, map: rhai::Map| entity.set_many(map));
    engine.register_fn("data", |entity: &mut TEntity| entity.data());
    engine.register_fn(
        "get_f",
        |entity: &mut TEntity, path: &str, fallback: rhai::FLOAT| entity.get_f(path, fallback),
    );
    engine.register_fn("get_s", |entity: &mut TEntity, path: &str, fallback: &str| {
        entity.get_s(path, fallback)
    });
    engine.register_fn("despawn", |entity: &mut TEntity| entity.despawn());
    engine.register_fn("id", |entity: &mut TEntity| entity.id());
    engine.register_fn("flag", |entity: &mut TEntity, name: &str| entity.flag(name));
    engine.register_fn("set_flag", |entity: &mut TEntity, name: &str, value: bool| {
        entity.set_flag(name, value)
    });
    engine.register_fn(
        "cooldown_start",
        |entity: &mut TEntity, name: &str, ms: rhai::INT| entity.cooldown_start(name, ms),
    );
    engine.register_fn("cooldown_ready", |entity: &mut TEntity, name: &str| {
        entity.cooldown_ready(name)
    });
    engine.register_fn("cooldown_remaining", |entity: &mut TEntity, name: &str| {
        entity.cooldown_remaining(name)
    });
    engine.register_fn(
        "status_add",
        |entity: &mut TEntity, name: &str, ms: rhai::INT| entity.status_add(name, ms),
    );
    engine.register_fn("status_has", |entity: &mut TEntity, name: &str| {
        entity.status_has(name)
    });
    engine.register_fn("status_remaining", |entity: &mut TEntity, name: &str| {
        entity.status_remaining(name)
    });
    engine.register_fn(
        "set_acceleration",
        |entity: &mut TEntity, ax: rhai::FLOAT, ay: rhai::FLOAT| entity.set_acceleration(ax, ay),
    );
    engine.register_fn("collider", |entity: &mut TEntity| entity.collider());
    engine.register_fn("heading", |entity: &mut TEntity| entity.heading());
    engine.register_fn("heading_vector", |entity: &mut TEntity| entity.heading_vector());
    engine.register_fn("attach_controller", |entity: &mut TEntity, config: rhai::Map| {
        entity.attach_controller(config)
    });
    engine.register_fn("set_turn", |entity: &mut TEntity, dir: rhai::INT| {
        entity.set_turn(dir)
    });
    engine.register_fn("set_thrust", |entity: &mut TEntity, on: bool| {
        entity.set_thrust(on)
    });
}

pub fn register_all(_engine: &mut RhaiEngine) {}

//! Gameplay domain APIs: ScriptGameplayApi for world management, ScriptGameplayEntityApi for entity interaction.

use engine_api::gameplay::api::{GameplayEntityCoreApi, GameplayWorldCoreApi};
use engine_api::rhai::register::{
    register_gameplay_core_api, register_geometry_api, register_numeric_api,
};
use rhai::{Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};

pub(crate) use super::gameplay_impl::{ScriptGameplayApi, ScriptGameplayEntityApi};

impl GameplayWorldCoreApi<ScriptGameplayEntityApi> for ScriptGameplayApi {
    fn clear(&mut self) { self.clear() }
    fn reset_dynamic_entities(&mut self) -> bool { self.reset_dynamic_entities() }
    fn count(&mut self) -> rhai::INT { self.count() }
    fn count_kind(&mut self, kind: &str) -> rhai::INT { self.count_kind(kind) }
    fn count_tag(&mut self, tag: &str) -> rhai::INT { self.count_tag(tag) }
    fn first_kind(&mut self, kind: &str) -> rhai::INT { self.first_kind(kind) }
    fn first_tag(&mut self, tag: &str) -> rhai::INT { self.first_tag(tag) }
    fn diagnostic_info(&mut self) -> RhaiMap { self.diagnostic_info() }
    fn spawn(&mut self, kind: &str, payload: RhaiDynamic) -> rhai::INT { self.spawn(kind, payload) }
    fn despawn(&mut self, id: rhai::INT) -> bool { self.despawn(id) }
    fn exists(&mut self, id: rhai::INT) -> bool { self.exists(id) }
    fn kind(&mut self, id: rhai::INT) -> String { self.kind(id) }
    fn tags(&mut self, id: rhai::INT) -> rhai::Array { self.tags(id) }
    fn ids(&mut self) -> rhai::Array { self.ids() }
    fn entity(&mut self, id: rhai::INT) -> ScriptGameplayEntityApi { self.entity(id) }
    fn query_kind(&mut self, kind: &str) -> rhai::Array { self.query_kind(kind) }
    fn query_tag(&mut self, tag: &str) -> rhai::Array { self.query_tag(tag) }
    fn get(&mut self, id: rhai::INT, path: &str) -> RhaiDynamic { self.get(id, path) }
    fn set(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool { self.set(id, path, value) }
    fn has(&mut self, id: rhai::INT, path: &str) -> bool { self.has(id, path) }
    fn remove(&mut self, id: rhai::INT, path: &str) -> bool { self.remove(id, path) }
    fn push(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool { self.push(id, path, value) }
    fn set_transform(&mut self, id: rhai::INT, x: rhai::FLOAT, y: rhai::FLOAT, heading: rhai::FLOAT) -> bool {
        self.set_transform(id, x, y, heading)
    }
    fn transform(&mut self, id: rhai::INT) -> RhaiDynamic { self.transform(id) }
    fn set_physics(
        &mut self,
        id: rhai::INT,
        vx: rhai::FLOAT,
        vy: rhai::FLOAT,
        ax: rhai::FLOAT,
        ay: rhai::FLOAT,
        drag: rhai::FLOAT,
        max_speed: rhai::FLOAT,
    ) -> bool {
        self.set_physics(id, vx, vy, ax, ay, drag, max_speed)
    }
    fn physics(&mut self, id: rhai::INT) -> RhaiDynamic { self.physics(id) }
    fn set_lifetime(&mut self, id: rhai::INT, ttl_ms: rhai::INT) -> bool { self.set_lifetime(id, ttl_ms) }
    fn collisions(&mut self) -> rhai::Array { self.collisions() }
    fn collisions_between(&mut self, kind_a: &str, kind_b: &str) -> rhai::Array { self.collisions_between(kind_a, kind_b) }
    fn collisions_of(&mut self, kind: &str) -> rhai::Array { self.collisions_of(kind) }
    fn collision_enters_between(&mut self, kind_a: &str, kind_b: &str) -> rhai::Array {
        self.collision_enters_between(kind_a, kind_b)
    }
    fn collision_stays_between(&mut self, kind_a: &str, kind_b: &str) -> rhai::Array {
        self.collision_stays_between(kind_a, kind_b)
    }
    fn collision_exits_between(&mut self, kind_a: &str, kind_b: &str) -> rhai::Array {
        self.collision_exits_between(kind_a, kind_b)
    }
    fn spawn_child_entity(
        &mut self,
        parent_id: rhai::INT,
        kind: &str,
        template: &str,
        data: RhaiMap,
    ) -> rhai::INT {
        self.spawn_child_entity(parent_id, kind, template, data)
    }
    fn despawn_children_of(&mut self, parent_id: rhai::INT) { self.despawn_children_of(parent_id) }
    fn distance(&mut self, a: rhai::INT, b: rhai::INT) -> rhai::FLOAT { self.distance(a, b) }
    fn any_alive(&mut self, kind: &str) -> bool { self.any_alive(kind) }
    fn set_world_bounds(
        &mut self,
        min_x: rhai::FLOAT,
        min_y: rhai::FLOAT,
        max_x: rhai::FLOAT,
        max_y: rhai::FLOAT,
    ) {
        self.set_world_bounds(min_x, min_y, max_x, max_y)
    }
    fn world_bounds(&mut self) -> RhaiMap { self.world_bounds() }
    fn rand_i(&mut self, min: rhai::INT, max: rhai::INT) -> rhai::INT { self.rand_i(min, max) }
    fn rand_seed(&mut self, seed: rhai::INT) { self.rand_seed(seed) }
    fn tag_add(&mut self, id: rhai::INT, tag: &str) -> bool { self.tag_add(id, tag) }
    fn tag_remove(&mut self, id: rhai::INT, tag: &str) -> bool { self.tag_remove(id, tag) }
    fn tag_has(&mut self, id: rhai::INT, tag: &str) -> bool { self.tag_has(id, tag) }
    fn after_ms(&mut self, label: &str, delay_ms: rhai::INT) { self.after_ms(label, delay_ms) }
    fn timer_fired(&mut self, label: &str) -> bool { self.timer_fired(label) }
    fn cancel_timer(&mut self, label: &str) -> bool { self.cancel_timer(label) }
    fn enable_wrap(
        &mut self,
        id: rhai::INT,
        min_x: rhai::FLOAT,
        max_x: rhai::FLOAT,
        min_y: rhai::FLOAT,
        max_y: rhai::FLOAT,
    ) -> bool {
        self.enable_wrap(id, min_x, max_x, min_y, max_y)
    }
    fn disable_wrap(&mut self, id: rhai::INT) -> bool { self.disable_wrap(id) }
    fn poll_collision_events(&mut self) -> rhai::Array { self.poll_collision_events() }
    fn clear_events(&mut self) { self.clear_events() }
    fn ship_set_turn(&mut self, id: rhai::INT, dir: rhai::INT) -> bool { self.ship_set_turn(id, dir) }
    fn ship_set_thrust(&mut self, id: rhai::INT, on: bool) -> bool { self.ship_set_thrust(id, on) }
    fn ship_heading(&mut self, id: rhai::INT) -> i32 { self.ship_heading(id) }
    fn ship_heading_vector(&mut self, id: rhai::INT) -> RhaiMap { self.ship_heading_vector(id) }
    fn ship_velocity(&mut self, id: rhai::INT) -> RhaiMap { self.ship_velocity(id) }
    fn spawn_batch(&mut self, specs: rhai::Array) -> rhai::Array { self.spawn_batch(specs) }
}

impl GameplayEntityCoreApi for ScriptGameplayEntityApi {
    fn exists(&mut self) -> bool { self.exists() }
    fn get(&mut self, path: &str) -> RhaiDynamic { self.get(path) }
    fn get_i(&mut self, path: &str, fallback: rhai::INT) -> rhai::INT { self.get_i(path, fallback) }
    fn get_bool(&mut self, path: &str, fallback: bool) -> bool { self.get_bool(path, fallback) }
    fn set(&mut self, path: &str, value: RhaiDynamic) -> bool { self.set(path, value) }
    fn kind(&mut self) -> String { self.kind() }
    fn tags(&mut self) -> rhai::Array { self.tags() }
    fn get_metadata(&mut self) -> RhaiMap { self.get_metadata() }
    fn get_components(&mut self) -> RhaiMap { self.get_components() }
    fn transform(&mut self) -> RhaiMap { self.transform() }
    fn set_position(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) -> bool { self.set_position(x, y) }
    fn set_heading(&mut self, heading: rhai::FLOAT) -> bool { self.set_heading(heading) }
    fn lifetime_remaining(&mut self) -> rhai::INT { self.lifetime_remaining() }
    fn set_many(&mut self, map: RhaiMap) -> bool { self.set_many(map) }
    fn data(&mut self) -> RhaiMap { self.data() }
    fn get_f(&mut self, path: &str, fallback: rhai::FLOAT) -> rhai::FLOAT { self.get_f(path, fallback) }
    fn get_s(&mut self, path: &str, fallback: &str) -> String { self.get_s(path, fallback) }
    fn despawn(&mut self) -> bool { self.despawn() }
    fn id(&mut self) -> rhai::INT { self.id() }
    fn flag(&mut self, name: &str) -> bool { self.flag(name) }
    fn set_flag(&mut self, name: &str, value: bool) -> bool { self.set_flag(name, value) }
    fn cooldown_start(&mut self, name: &str, ms: rhai::INT) -> bool { self.cooldown_start(name, ms) }
    fn cooldown_ready(&mut self, name: &str) -> bool { self.cooldown_ready(name) }
    fn cooldown_remaining(&mut self, name: &str) -> rhai::INT { self.cooldown_remaining(name) }
    fn status_add(&mut self, name: &str, ms: rhai::INT) -> bool { self.status_add(name, ms) }
    fn status_has(&mut self, name: &str) -> bool { self.status_has(name) }
    fn status_remaining(&mut self, name: &str) -> rhai::INT { self.status_remaining(name) }
    fn set_acceleration(&mut self, ax: rhai::FLOAT, ay: rhai::FLOAT) -> bool {
        self.set_acceleration(ax, ay)
    }
    fn collider(&mut self) -> RhaiMap { self.collider() }
    fn heading(&mut self) -> rhai::INT { self.heading() }
    fn heading_vector(&mut self) -> RhaiMap { self.heading_vector() }
    fn attach_ship_controller(&mut self, config: RhaiMap) -> bool { self.attach_ship_controller(config) }
    fn set_turn(&mut self, dir: rhai::INT) -> bool { self.set_turn(dir) }
    fn set_thrust(&mut self, on: bool) -> bool { self.set_thrust(on) }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    register_gameplay_core_api::<ScriptGameplayApi, ScriptGameplayEntityApi>(engine);

    // ═══════════════════════════════════════════════════════════════════════════════
    // PHASE 3: WORLD.* NAMESPACE WITH DUAL-NAME REGISTRATION
    // Both flat names (for backward compatibility) and world.* names work
    // ═══════════════════════════════════════════════════════════════════════════════

    // Gameplay API - remaining world/collection operations not yet moved to engine-api
    
    // --- SPAWN OPERATIONS (world.spawn_*) ---
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
        "world.spawn_visual",
        |world: &mut ScriptGameplayApi, kind: &str, template: &str, data: RhaiMap| {
            world.spawn_visual(kind, template, data)
        },
    );
    
    engine.register_fn(
        "spawn_prefab",
        |world: &mut ScriptGameplayApi, name: &str, args: RhaiMap| world.spawn_prefab(name, args),
    );
    engine.register_fn(
        "world.spawn_prefab",
        |world: &mut ScriptGameplayApi, name: &str, args: RhaiMap| world.spawn_prefab(name, args),
    );
    
    engine.register_fn(
        "spawn_group",
        |world: &mut ScriptGameplayApi, group_name: &str, prefab_name: &str| {
            world.spawn_group(group_name, prefab_name)
        },
    );
    engine.register_fn(
        "world.spawn_group",
        |world: &mut ScriptGameplayApi, group_name: &str, prefab_name: &str| {
            world.spawn_group(group_name, prefab_name)
        },
    );
    
    // --- EFFECT OPERATIONS (world.emit, world.effects.*) ---
    engine.register_fn(
        "emit",
        |world: &mut ScriptGameplayApi, emitter_name: &str, owner_id: rhai::INT, args: RhaiMap| {
            world.emit(emitter_name, owner_id, args)
        },
    );
    engine.register_fn(
        "world.emit",
        |world: &mut ScriptGameplayApi, emitter_name: &str, owner_id: rhai::INT, args: RhaiMap| {
            world.emit(emitter_name, owner_id, args)
        },
    );

    // --- QUERY OPERATIONS (world.query_*, world.count_*, world.entity, world.exists) ---
    // Note: count, count_kind, count_tag, query_kind, query_tag, entity, exists
    //       are already registered in register_gameplay_core_api (flat names only for now)
    //       Future expansion: add world.count(), world.query(), world.entity() versions

    // --- BOUNDS OPERATIONS (world.set_bounds, world.get_bounds) ---
    // These are accessed via set_world_bounds and world_bounds currently
    // Pattern for future expansion:
    engine.register_fn(
        "world.set_bounds",
        |world: &mut ScriptGameplayApi, min_x: rhai::FLOAT, min_y: rhai::FLOAT, max_x: rhai::FLOAT, max_y: rhai::FLOAT| {
            world.set_world_bounds(min_x, min_y, max_x, max_y)
        },
    );
    engine.register_fn(
        "world.get_bounds",
        |world: &mut ScriptGameplayApi| world.world_bounds()
    );

    // --- TIMER OPERATIONS (world.timer_*) ---
    // Pattern for future: world.timer_ms, world.timer_sec, world.timer_fired
    
    // Gameplay Entity API - remaining entity operations not yet moved to engine-api
    // Physics as a property: ship.physics.velocity(), ship.physics.set_velocity(), etc.
    engine.register_get("physics", |entity: &mut ScriptGameplayEntityApi| {
        entity.physics.clone()
    });

    engine.register_fn("physics", |entity: &mut ScriptGameplayEntityApi| {
        entity.physics.clone()
    });

    register_geometry_api(engine);
    register_numeric_api(engine);
}

//! Gameplay domain APIs: ScriptGameplayApi for world management, ScriptGameplayEntityApi for entity interaction.

use engine_api::gameplay::api::{GameplayEntityCoreApi, GameplayWorldCoreApi};
use engine_api::gameplay::body::{
    register_world_body_api, GameplayWorldBodyLookupCoreApi, GameplayWorldBodySnapshotCoreApi,
};
use engine_api::gameplay::objects::{GameplayWorldObjectCoreApi, GameplayWorldObjectsCoreApi};
use engine_api::rhai::register::{
    register_gameplay_core_api, register_geometry_api, register_numeric_api,
};
use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};

pub(crate) use super::gameplay_impl::{
    ScriptGameplayApi, ScriptGameplayBodySnapshotApi, ScriptGameplayEntityApi,
    ScriptGameplayObjectApi, ScriptGameplayObjectsApi,
};

impl GameplayWorldCoreApi<ScriptGameplayEntityApi> for ScriptGameplayApi {
    fn clear(&mut self) {
        self.clear()
    }
    fn reset_dynamic_entities(&mut self) -> bool {
        self.reset_dynamic_entities()
    }
    fn count(&mut self) -> rhai::INT {
        self.count()
    }
    fn count_kind(&mut self, kind: &str) -> rhai::INT {
        self.count_kind(kind)
    }
    fn count_tag(&mut self, tag: &str) -> rhai::INT {
        self.count_tag(tag)
    }
    fn first_kind(&mut self, kind: &str) -> rhai::INT {
        self.first_kind(kind)
    }
    fn first_tag(&mut self, tag: &str) -> rhai::INT {
        self.first_tag(tag)
    }
    fn diagnostic_info(&mut self) -> RhaiMap {
        self.diagnostic_info()
    }
    fn spawn(&mut self, kind: &str, payload: RhaiDynamic) -> rhai::INT {
        self.spawn(kind, payload)
    }
    fn despawn(&mut self, id: rhai::INT) -> bool {
        self.despawn(id)
    }
    fn exists(&mut self, id: rhai::INT) -> bool {
        self.exists(id)
    }
    fn kind(&mut self, id: rhai::INT) -> String {
        self.kind(id)
    }
    fn tags(&mut self, id: rhai::INT) -> rhai::Array {
        self.tags(id)
    }
    fn ids(&mut self) -> rhai::Array {
        self.ids()
    }
    fn entity(&mut self, id: rhai::INT) -> ScriptGameplayEntityApi {
        self.entity(id)
    }
    fn query_kind(&mut self, kind: &str) -> rhai::Array {
        self.query_kind(kind)
    }
    fn query_tag(&mut self, tag: &str) -> rhai::Array {
        self.query_tag(tag)
    }
    fn query_circle(&mut self, x: rhai::FLOAT, y: rhai::FLOAT, radius: rhai::FLOAT) -> rhai::Array {
        self.query_circle(x, y, radius)
    }
    fn query_rect(
        &mut self,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        w: rhai::FLOAT,
        h: rhai::FLOAT,
    ) -> rhai::Array {
        self.query_rect(x, y, w, h)
    }
    fn query_nearest(
        &mut self,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        max_dist: rhai::FLOAT,
    ) -> rhai::INT {
        self.query_nearest(x, y, max_dist)
    }
    fn query_nearest_kind(
        &mut self,
        kind: &str,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        max_dist: rhai::FLOAT,
    ) -> rhai::INT {
        self.query_nearest_kind(kind, x, y, max_dist)
    }
    fn get(&mut self, id: rhai::INT, path: &str) -> RhaiDynamic {
        self.get(id, path)
    }
    fn set(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
        self.set(id, path, value)
    }
    fn has(&mut self, id: rhai::INT, path: &str) -> bool {
        self.has(id, path)
    }
    fn remove(&mut self, id: rhai::INT, path: &str) -> bool {
        self.remove(id, path)
    }
    fn push(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
        self.push(id, path, value)
    }
    fn set_transform(
        &mut self,
        id: rhai::INT,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        heading: rhai::FLOAT,
    ) -> bool {
        self.set_transform(id, x, y, heading)
    }
    fn set_transform_3d(
        &mut self,
        id: rhai::INT,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        z: rhai::FLOAT,
        heading: rhai::FLOAT,
    ) -> bool {
        self.set_transform_3d(id, x, y, z, heading)
    }
    fn transform(&mut self, id: rhai::INT) -> RhaiDynamic {
        self.transform(id)
    }
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
    #[allow(clippy::too_many_arguments)]
    fn set_physics_3d(
        &mut self,
        id: rhai::INT,
        vx: rhai::FLOAT,
        vy: rhai::FLOAT,
        vz: rhai::FLOAT,
        ax: rhai::FLOAT,
        ay: rhai::FLOAT,
        az: rhai::FLOAT,
        drag: rhai::FLOAT,
        max_speed: rhai::FLOAT,
    ) -> bool {
        self.set_physics_3d(id, vx, vy, vz, ax, ay, az, drag, max_speed)
    }
    fn physics(&mut self, id: rhai::INT) -> RhaiDynamic {
        self.physics(id)
    }
    fn set_lifetime(&mut self, id: rhai::INT, ttl_ms: rhai::INT) -> bool {
        self.set_lifetime(id, ttl_ms)
    }
    fn collisions(&mut self) -> rhai::Array {
        self.collisions()
    }
    fn collisions_between(&mut self, kind_a: &str, kind_b: &str) -> rhai::Array {
        self.collisions_between(kind_a, kind_b)
    }
    fn collisions_of(&mut self, kind: &str) -> rhai::Array {
        self.collisions_of(kind)
    }
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
    fn set_controlled_entity(&mut self, id: rhai::INT) -> bool {
        self.set_controlled_entity(id)
    }
    fn controlled_entity(&mut self) -> rhai::INT {
        self.controlled_entity()
    }
    fn clear_controlled_entity(&mut self) -> bool {
        self.clear_controlled_entity()
    }
    fn despawn_children_of(&mut self, parent_id: rhai::INT) {
        self.despawn_children_of(parent_id)
    }
    fn distance(&mut self, a: rhai::INT, b: rhai::INT) -> rhai::FLOAT {
        self.distance(a, b)
    }
    fn any_alive(&mut self, kind: &str) -> bool {
        self.any_alive(kind)
    }
    fn set_world_bounds(
        &mut self,
        min_x: rhai::FLOAT,
        min_y: rhai::FLOAT,
        max_x: rhai::FLOAT,
        max_y: rhai::FLOAT,
    ) {
        self.set_world_bounds(min_x, min_y, max_x, max_y)
    }
    fn world_bounds(&mut self) -> RhaiMap {
        self.world_bounds()
    }
    fn world_width(&mut self) -> rhai::FLOAT {
        self.world_width()
    }
    fn world_height(&mut self) -> rhai::FLOAT {
        self.world_height()
    }
    fn set_camera(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) {
        self.set_camera(x, y)
    }
    fn set_camera_zoom(&mut self, zoom: rhai::FLOAT) {
        self.set_camera_zoom(zoom)
    }
    fn set_camera_3d_look_at(
        &mut self,
        eye_x: rhai::FLOAT,
        eye_y: rhai::FLOAT,
        eye_z: rhai::FLOAT,
        target_x: rhai::FLOAT,
        target_y: rhai::FLOAT,
        target_z: rhai::FLOAT,
    ) {
        self.set_camera_3d_look_at(eye_x, eye_y, eye_z, target_x, target_y, target_z)
    }
    fn set_camera_3d_up(&mut self, up_x: rhai::FLOAT, up_y: rhai::FLOAT, up_z: rhai::FLOAT) {
        self.set_camera_3d_up(up_x, up_y, up_z)
    }
    fn angular_body_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        self.angular_body_attach(id, config)
    }
    fn set_angular_input(&mut self, id: rhai::INT, input: rhai::FLOAT) -> bool {
        self.set_angular_input(id, input)
    }
    fn angular_vel(&mut self, id: rhai::INT) -> rhai::FLOAT {
        self.angular_vel(id)
    }
    fn linear_brake_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        self.linear_brake_attach(id, config)
    }
    fn set_linear_brake_active(&mut self, id: rhai::INT, active: bool) -> bool {
        self.set_linear_brake_active(id, active)
    }
    fn thruster_ramp_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        self.thruster_ramp_attach(id, config)
    }
    fn thruster_ramp(&mut self, id: rhai::INT) -> RhaiMap {
        self.thruster_ramp(id)
    }
    fn thruster_ramp_detach(&mut self, id: rhai::INT) -> bool {
        self.thruster_ramp_detach(id)
    }
    fn rand_i(&mut self, min: rhai::INT, max: rhai::INT) -> rhai::INT {
        self.rand_i(min, max)
    }
    fn rand_seed(&mut self, seed: rhai::INT) {
        self.rand_seed(seed)
    }
    fn tag_add(&mut self, id: rhai::INT, tag: &str) -> bool {
        self.tag_add(id, tag)
    }
    fn tag_remove(&mut self, id: rhai::INT, tag: &str) -> bool {
        self.tag_remove(id, tag)
    }
    fn tag_has(&mut self, id: rhai::INT, tag: &str) -> bool {
        self.tag_has(id, tag)
    }
    fn after_ms(&mut self, label: &str, delay_ms: rhai::INT) {
        self.after_ms(label, delay_ms)
    }
    fn timer_fired(&mut self, label: &str) -> bool {
        self.timer_fired(label)
    }
    fn cancel_timer(&mut self, label: &str) -> bool {
        self.cancel_timer(label)
    }
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
    fn disable_wrap(&mut self, id: rhai::INT) -> bool {
        self.disable_wrap(id)
    }
    fn poll_collision_events(&mut self) -> rhai::Array {
        self.poll_collision_events()
    }
    fn clear_events(&mut self) {
        self.clear_events()
    }
    fn spawn_batch(&mut self, specs: rhai::Array) -> rhai::Array {
        self.spawn_batch(specs)
    }
}

impl GameplayEntityCoreApi for ScriptGameplayEntityApi {
    fn exists(&mut self) -> bool {
        self.exists()
    }
    fn get(&mut self, path: &str) -> RhaiDynamic {
        self.get(path)
    }
    fn get_i(&mut self, path: &str, fallback: rhai::INT) -> rhai::INT {
        self.get_i(path, fallback)
    }
    fn get_bool(&mut self, path: &str, fallback: bool) -> bool {
        self.get_bool(path, fallback)
    }
    fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        self.set(path, value)
    }
    fn kind(&mut self) -> String {
        self.kind()
    }
    fn tags(&mut self) -> rhai::Array {
        self.tags()
    }
    fn get_metadata(&mut self) -> RhaiMap {
        self.get_metadata()
    }
    fn get_components(&mut self) -> RhaiMap {
        self.get_components()
    }
    fn transform(&mut self) -> RhaiMap {
        self.transform()
    }
    fn set_position(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) -> bool {
        self.set_position(x, y)
    }
    fn set_heading(&mut self, heading: rhai::FLOAT) -> bool {
        self.set_heading(heading)
    }
    fn lifetime_remaining(&mut self) -> rhai::INT {
        self.lifetime_remaining()
    }
    fn set_many(&mut self, map: RhaiMap) -> bool {
        self.set_many(map)
    }
    fn data(&mut self) -> RhaiMap {
        self.data()
    }
    fn get_f(&mut self, path: &str, fallback: rhai::FLOAT) -> rhai::FLOAT {
        self.get_f(path, fallback)
    }
    fn get_s(&mut self, path: &str, fallback: &str) -> String {
        self.get_s(path, fallback)
    }
    fn despawn(&mut self) -> bool {
        self.despawn()
    }
    fn id(&mut self) -> rhai::INT {
        self.id()
    }
    fn flag(&mut self, name: &str) -> bool {
        self.flag(name)
    }
    fn set_flag(&mut self, name: &str, value: bool) -> bool {
        self.set_flag(name, value)
    }
    fn cooldown_start(&mut self, name: &str, ms: rhai::INT) -> bool {
        self.cooldown_start(name, ms)
    }
    fn cooldown_ready(&mut self, name: &str) -> bool {
        self.cooldown_ready(name)
    }
    fn cooldown_remaining(&mut self, name: &str) -> rhai::INT {
        self.cooldown_remaining(name)
    }
    fn status_add(&mut self, name: &str, ms: rhai::INT) -> bool {
        self.status_add(name, ms)
    }
    fn status_has(&mut self, name: &str) -> bool {
        self.status_has(name)
    }
    fn status_remaining(&mut self, name: &str) -> rhai::INT {
        self.status_remaining(name)
    }
    fn set_acceleration(&mut self, ax: rhai::FLOAT, ay: rhai::FLOAT) -> bool {
        self.set_acceleration(ax, ay)
    }
    fn apply_impulse(&mut self, vx: rhai::FLOAT, vy: rhai::FLOAT) -> bool {
        self.apply_impulse(vx, vy)
    }
    fn velocity_magnitude(&mut self) -> rhai::FLOAT {
        self.velocity_magnitude()
    }
    fn velocity_angle(&mut self) -> rhai::FLOAT {
        self.velocity_angle()
    }
    fn set_velocity_polar(&mut self, speed: rhai::FLOAT, angle: rhai::FLOAT) -> bool {
        self.set_velocity_polar(speed, angle)
    }
    fn collider(&mut self) -> RhaiMap {
        self.collider()
    }
    fn heading(&mut self) -> rhai::INT {
        self.heading()
    }
    fn heading_vector(&mut self) -> RhaiMap {
        self.heading_vector()
    }
    fn attach_controller(&mut self, config: RhaiMap) -> bool {
        self.attach_controller(config)
    }
    fn set_turn(&mut self, dir: rhai::INT) -> bool {
        self.set_turn(dir)
    }
    fn set_thrust(&mut self, on: bool) -> bool {
        self.set_thrust(on)
    }
    fn lifetime_fraction(&mut self) -> rhai::FLOAT {
        self.lifetime_fraction()
    }
    fn set_fg(&mut self, color: &str) -> bool {
        self.set_fg(color)
    }
    fn set_radius(&mut self, r: rhai::INT) -> bool {
        self.set_radius(r)
    }
}

impl GameplayWorldObjectsCoreApi<ScriptGameplayObjectApi> for ScriptGameplayObjectsApi {
    fn find(&mut self, target: &str) -> ScriptGameplayObjectApi {
        self.find(target)
    }

    fn find_id(&mut self, id: rhai::INT) -> ScriptGameplayObjectApi {
        self.find_id(id)
    }

    fn all(&mut self) -> rhai::Array {
        self.all()
    }

    fn by_tag(&mut self, tag: &str) -> rhai::Array {
        self.by_tag(tag)
    }

    fn by_name(&mut self, name: &str) -> rhai::Array {
        self.by_name(name)
    }
}

impl GameplayWorldBodyLookupCoreApi<ScriptGameplayBodySnapshotApi> for ScriptGameplayApi {
    fn body(&mut self, id: &str) -> ScriptGameplayBodySnapshotApi {
        self.body(id)
    }
}

impl GameplayWorldBodySnapshotCoreApi for ScriptGameplayBodySnapshotApi {
    fn exists(&mut self) -> bool {
        self.exists()
    }

    fn id(&mut self) -> String {
        self.id()
    }

    fn center_x(&mut self) -> rhai::FLOAT {
        self.center_x()
    }

    fn center_y(&mut self) -> rhai::FLOAT {
        self.center_y()
    }

    fn orbit_radius(&mut self) -> rhai::FLOAT {
        self.orbit_radius()
    }

    fn orbit_period_sec(&mut self) -> rhai::FLOAT {
        self.orbit_period_sec()
    }

    fn orbit_phase_deg(&mut self) -> rhai::FLOAT {
        self.orbit_phase_deg()
    }

    fn radius_px(&mut self) -> rhai::FLOAT {
        self.radius_px()
    }

    fn surface_radius(&mut self) -> rhai::FLOAT {
        self.surface_radius()
    }

    fn gravity_mu(&mut self) -> rhai::FLOAT {
        self.gravity_mu()
    }

    fn gravity_mu_km3_s2(&mut self) -> rhai::FLOAT {
        self.gravity_mu_km3_s2()
    }

    fn km_per_px(&mut self) -> rhai::FLOAT {
        self.km_per_px()
    }

    fn km_per_world_unit(&mut self) -> rhai::FLOAT {
        self.km_per_world_unit()
    }

    fn radius_km(&mut self) -> rhai::FLOAT {
        self.radius_km()
    }

    fn resolved_radius_km(&mut self) -> rhai::FLOAT {
        self.resolved_radius_km()
    }

    fn resolved_gravity_mu(&mut self) -> rhai::FLOAT {
        self.resolved_gravity_mu()
    }

    fn atmosphere_top_km(&mut self) -> rhai::FLOAT {
        self.atmosphere_top_km()
    }

    fn atmosphere_dense_start_km(&mut self) -> rhai::FLOAT {
        self.atmosphere_dense_start_km()
    }

    fn resolved_atmosphere_top_km(&mut self) -> rhai::FLOAT {
        self.resolved_atmosphere_top_km()
    }

    fn resolved_atmosphere_dense_start_km(&mut self) -> rhai::FLOAT {
        self.resolved_atmosphere_dense_start_km()
    }

    fn atmosphere_drag_max(&mut self) -> rhai::FLOAT {
        self.atmosphere_drag_max()
    }

    fn cloud_bottom_km(&mut self) -> rhai::FLOAT {
        self.cloud_bottom_km()
    }

    fn cloud_top_km(&mut self) -> rhai::FLOAT {
        self.cloud_top_km()
    }

    fn planet_type(&mut self) -> String {
        self.planet_type()
    }

    fn parent(&mut self) -> String {
        self.parent()
    }

    fn inspect(&mut self) -> RhaiMap {
        self.inspect()
    }
}

impl GameplayWorldObjectCoreApi for ScriptGameplayObjectApi {
    fn exists(&mut self) -> bool {
        self.exists()
    }

    fn id(&mut self) -> rhai::INT {
        self.id()
    }

    fn kind(&mut self) -> String {
        self.kind()
    }

    fn tags(&mut self) -> rhai::Array {
        self.tags()
    }

    fn inspect(&mut self) -> RhaiMap {
        self.inspect()
    }

    fn get(&mut self, path: &str) -> RhaiDynamic {
        self.get(path)
    }

    fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        self.set(path, value)
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    register_gameplay_core_api::<ScriptGameplayApi, ScriptGameplayEntityApi>(engine);
    register_world_body_api::<ScriptGameplayApi, ScriptGameplayBodySnapshotApi>(engine);
    engine_api::gameplay::objects::register_world_objects_api::<
        ScriptGameplayObjectsApi,
        ScriptGameplayObjectApi,
    >(engine);
    engine.register_get("objects", |world: &mut ScriptGameplayApi| world.objects());
    engine.register_fn("objects", |world: &mut ScriptGameplayApi| world.objects());

    // Additional grouped `world.*` registrations that still live in behavior.
    // Core typed gameplay contracts are registered above via `engine-api`.

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
        "spawn_from_heading",
        |world: &mut ScriptGameplayApi, owner_id: rhai::INT, prefab: &str, args: RhaiMap| {
            world.spawn_from_heading(owner_id, prefab, args)
        },
    );
    engine.register_fn(
        "heading_drift",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.heading_drift(id),
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
    // `register_gameplay_core_api` already exposes the typed count/query/entity surface.
    // Add grouped `world.*` aliases here when those calls are promoted together.

    // --- BOUNDS OPERATIONS (world.set_bounds, world.get_bounds) ---
    // These are accessed via set_world_bounds and world_bounds currently
    // Pattern for future expansion:
    engine.register_fn(
        "world.set_bounds",
        |world: &mut ScriptGameplayApi,
         min_x: rhai::FLOAT,
         min_y: rhai::FLOAT,
         max_x: rhai::FLOAT,
         max_y: rhai::FLOAT| { world.set_world_bounds(min_x, min_y, max_x, max_y) },
    );
    engine.register_fn("world.get_bounds", |world: &mut ScriptGameplayApi| {
        world.world_bounds()
    });

    // --- CATALOG QUERY OPERATIONS ---
    // `world.body(...)` is the public Rhai surface for celestial body snapshots.
    engine.register_fn(
        "body_upsert",
        |world: &mut ScriptGameplayApi, id: &str, patch: RhaiMap| world.body_upsert(id, patch),
    );
    engine.register_fn(
        "world.body_upsert",
        |world: &mut ScriptGameplayApi, id: &str, patch: RhaiMap| world.body_upsert(id, patch),
    );
    engine.register_fn(
        "body_patch",
        |world: &mut ScriptGameplayApi, id: &str, patch: RhaiMap| world.body_patch(id, patch),
    );
    engine.register_fn(
        "world.body_patch",
        |world: &mut ScriptGameplayApi, id: &str, patch: RhaiMap| world.body_patch(id, patch),
    );
    engine.register_fn(
        "apply_planet_spec",
        |world: &mut ScriptGameplayApi, target: &str, body_id: &str, spec_map: RhaiMap| {
            world.apply_planet_spec(target, body_id, spec_map)
        },
    );
    engine.register_fn(
        "world.apply_planet_spec",
        |world: &mut ScriptGameplayApi, target: &str, body_id: &str, spec_map: RhaiMap| {
            world.apply_planet_spec(target, body_id, spec_map)
        },
    );
    engine.register_fn(
        "body_position",
        |world: &mut ScriptGameplayApi, id: &str, elapsed_sec: rhai::FLOAT| {
            world.body_position(id, elapsed_sec)
        },
    );
    engine.register_fn(
        "world.body_position",
        |world: &mut ScriptGameplayApi, id: &str, elapsed_sec: rhai::FLOAT| {
            world.body_position(id, elapsed_sec)
        },
    );
    engine.register_fn(
        "body_pose",
        |world: &mut ScriptGameplayApi, id: &str, elapsed_sec: rhai::FLOAT| {
            world.body_pose(id, elapsed_sec)
        },
    );
    engine.register_fn(
        "world.body_pose",
        |world: &mut ScriptGameplayApi, id: &str, elapsed_sec: rhai::FLOAT| {
            world.body_pose(id, elapsed_sec)
        },
    );
    engine.register_fn(
        "body_surface",
        |world: &mut ScriptGameplayApi,
         body_id: &str,
         latitude_deg: rhai::FLOAT,
         longitude_deg: rhai::FLOAT,
         altitude_world: rhai::FLOAT,
         elapsed_sec: rhai::FLOAT| {
            world.body_surface(
                body_id,
                latitude_deg,
                longitude_deg,
                altitude_world,
                elapsed_sec,
            )
        },
    );
    engine.register_fn(
        "world.body_surface",
        |world: &mut ScriptGameplayApi,
         body_id: &str,
         latitude_deg: rhai::FLOAT,
         longitude_deg: rhai::FLOAT,
         altitude_world: rhai::FLOAT,
         elapsed_sec: rhai::FLOAT| {
            world.body_surface(
                body_id,
                latitude_deg,
                longitude_deg,
                altitude_world,
                elapsed_sec,
            )
        },
    );
    engine.register_fn(
        "body_frame",
        |world: &mut ScriptGameplayApi,
         body_id: &str,
         x: rhai::FLOAT,
         y: rhai::FLOAT,
         z: rhai::FLOAT,
         elapsed_sec: rhai::FLOAT| { world.body_frame(body_id, x, y, z, elapsed_sec) },
    );
    engine.register_fn(
        "world.body_frame",
        |world: &mut ScriptGameplayApi,
         body_id: &str,
         x: rhai::FLOAT,
         y: rhai::FLOAT,
         z: rhai::FLOAT,
         elapsed_sec: rhai::FLOAT| { world.body_frame(body_id, x, y, z, elapsed_sec) },
    );
    engine.register_fn(
        "site_pose",
        |world: &mut ScriptGameplayApi, site_id: &str, elapsed_sec: rhai::FLOAT| {
            world.site_pose(site_id, elapsed_sec)
        },
    );
    engine.register_fn(
        "world.site_pose",
        |world: &mut ScriptGameplayApi, site_id: &str, elapsed_sec: rhai::FLOAT| {
            world.site_pose(site_id, elapsed_sec)
        },
    );
    engine.register_fn(
        "system_query",
        |world: &mut ScriptGameplayApi, system_id: &str, elapsed_sec: rhai::FLOAT| {
            world.system_query(system_id, elapsed_sec)
        },
    );
    engine.register_fn(
        "world.system_query",
        |world: &mut ScriptGameplayApi, system_id: &str, elapsed_sec: rhai::FLOAT| {
            world.system_query(system_id, elapsed_sec)
        },
    );
    engine.register_fn(
        "planet_type_info",
        |world: &mut ScriptGameplayApi, id: &str| world.planet_type_info(id),
    );
    engine.register_fn(
        "world.planet_type_info",
        |world: &mut ScriptGameplayApi, id: &str| world.planet_type_info(id),
    );
    engine.register_fn(
        "gravity_attach",
        |world: &mut ScriptGameplayApi, id: rhai::INT, config: RhaiMap| {
            world.gravity_attach(id, config)
        },
    );
    engine.register_fn(
        "world.gravity_attach",
        |world: &mut ScriptGameplayApi, id: rhai::INT, config: RhaiMap| {
            world.gravity_attach(id, config)
        },
    );
    engine.register_fn("gravity", |world: &mut ScriptGameplayApi, id: rhai::INT| {
        world.gravity(id)
    });
    engine.register_fn(
        "world.gravity",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.gravity(id),
    );
    engine.register_fn(
        "body_gravity",
        |world: &mut ScriptGameplayApi,
         body_id: &str,
         x: rhai::FLOAT,
         y: rhai::FLOAT,
         z: rhai::FLOAT| { world.body_gravity(body_id, x, y, z) },
    );
    engine.register_fn(
        "world.body_gravity",
        |world: &mut ScriptGameplayApi,
         body_id: &str,
         x: rhai::FLOAT,
         y: rhai::FLOAT,
         z: rhai::FLOAT| { world.body_gravity(body_id, x, y, z) },
    );
    engine.register_fn(
        "body_atmosphere",
        |world: &mut ScriptGameplayApi,
         body_id: &str,
         x: rhai::FLOAT,
         y: rhai::FLOAT,
         z: rhai::FLOAT,
         elapsed_sec: rhai::FLOAT| { world.body_atmosphere(body_id, x, y, z, elapsed_sec) },
    );
    engine.register_fn(
        "world.body_atmosphere",
        |world: &mut ScriptGameplayApi,
         body_id: &str,
         x: rhai::FLOAT,
         y: rhai::FLOAT,
         z: rhai::FLOAT,
         elapsed_sec: rhai::FLOAT| { world.body_atmosphere(body_id, x, y, z, elapsed_sec) },
    );
    engine.register_fn(
        "atmosphere_attach",
        |world: &mut ScriptGameplayApi, id: rhai::INT, config: RhaiMap| {
            world.atmosphere_attach(id, config)
        },
    );
    engine.register_fn(
        "world.atmosphere_attach",
        |world: &mut ScriptGameplayApi, id: rhai::INT, config: RhaiMap| {
            world.atmosphere_attach(id, config)
        },
    );
    engine.register_fn(
        "find_planet_spawn_angle",
        |world: &mut ScriptGameplayApi, config: RhaiMap, preferred_biomes: RhaiArray| {
            world.find_planet_spawn_angle(config, preferred_biomes)
        },
    );
    engine.register_fn(
        "world.find_planet_spawn_angle",
        |world: &mut ScriptGameplayApi, config: RhaiMap, preferred_biomes: RhaiArray| {
            world.find_planet_spawn_angle(config, preferred_biomes)
        },
    );
    engine.register_fn(
        "find_planet_spawn",
        |world: &mut ScriptGameplayApi, config: RhaiMap, preferred_biomes: RhaiArray| {
            world.find_planet_spawn(config, preferred_biomes)
        },
    );
    engine.register_fn(
        "world.find_planet_spawn",
        |world: &mut ScriptGameplayApi, config: RhaiMap, preferred_biomes: RhaiArray| {
            world.find_planet_spawn(config, preferred_biomes)
        },
    );
    engine.register_fn(
        "atmosphere",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.atmosphere(id),
    );
    engine.register_fn(
        "world.atmosphere",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.atmosphere(id),
    );

    engine.register_fn(
        "angular_body_attach",
        |world: &mut ScriptGameplayApi, id: rhai::INT, config: RhaiMap| {
            world.angular_body_attach(id, config)
        },
    );
    engine.register_fn(
        "set_angular_input",
        |world: &mut ScriptGameplayApi, id: rhai::INT, input: rhai::FLOAT| {
            world.set_angular_input(id, input)
        },
    );
    engine.register_fn(
        "angular_vel",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.angular_vel(id),
    );
    engine.register_fn(
        "linear_brake_attach",
        |world: &mut ScriptGameplayApi, id: rhai::INT, config: RhaiMap| {
            world.linear_brake_attach(id, config)
        },
    );
    engine.register_fn(
        "set_linear_brake_active",
        |world: &mut ScriptGameplayApi, id: rhai::INT, active: bool| {
            world.set_linear_brake_active(id, active)
        },
    );
    engine.register_fn(
        "thruster_ramp_attach",
        |world: &mut ScriptGameplayApi, id: rhai::INT, config: RhaiMap| {
            world.thruster_ramp_attach(id, config)
        },
    );
    engine.register_fn(
        "thruster_ramp",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.thruster_ramp(id),
    );
    engine.register_fn(
        "thruster_ramp_detach",
        |world: &mut ScriptGameplayApi, id: rhai::INT| world.thruster_ramp_detach(id),
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use engine_game::GameplayWorld;
    use rhai::{Engine as RhaiEngine, Map as RhaiMap, Scope as RhaiScope};

    use super::{register_with_rhai, ScriptGameplayApi};
    use crate::{catalog, palette::PaletteStore};

    fn build_world_api(world: GameplayWorld) -> ScriptGameplayApi {
        ScriptGameplayApi::new(
            Some(world),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            None,
            Arc::new(catalog::ModCatalogs::default()),
            None,
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(PaletteStore::default()),
            None,
            None,
        )
    }

    #[test]
    fn register_with_rhai_exposes_world_objects_lookup_by_visual_and_tag() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let ship_id = world.spawn_object("ship", #{ hp: 12, pilot: "Ada" });
                    let rock_id = world.spawn_object("asteroid", #{ hp: 3 });
                    world.set_visual(ship_id, "ship-main");
                    world.bind_visual(ship_id, "ship-shadow");
                    world.tag_add(ship_id, "player");
                    world.tag_add(rock_id, "hazard");

                    let found = world.objects.find("ship-shadow");
                    let by_id = world.objects.find(ship_id);
                    let all = world.objects.all();
                    let tagged = world.objects.by_tag("player");
                    found.set("hp", 42);

                    #{
                        found_exists: found.exists(),
                        found_id: found.id(),
                        found_kind: found.kind(),
                        found_hp: found.get("hp"),
                        inspect_visual: found.inspect()["visual_id"],
                        inspect_visual_count: found.inspect()["visual_ids"].len(),
                        by_id_kind: by_id.kind(),
                        all_len: all.len(),
                        tagged_len: tagged.len(),
                        tagged_id: tagged[0].id()
                    }
                "#,
            )
            .expect("world.objects API should evaluate in behavior-owned engine");

        assert_eq!(
            result
                .get("found_exists")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(true)
        );
        assert_eq!(
            result
                .get("found_id")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(1)
        );
        assert_eq!(
            result
                .get("found_kind")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("ship".to_string())
        );
        assert_eq!(
            result
                .get("found_hp")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(42)
        );
        assert_eq!(
            result
                .get("inspect_visual")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("ship-main".to_string())
        );
        assert_eq!(
            result
                .get("inspect_visual_count")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(2)
        );
        assert_eq!(
            result
                .get("by_id_kind")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("ship".to_string())
        );
        assert_eq!(
            result
                .get("all_len")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(2)
        );
        assert_eq!(
            result
                .get("tagged_len")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(1)
        );
        assert_eq!(
            result
                .get("tagged_id")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(1)
        );
    }

    #[test]
    fn register_with_rhai_exposes_world_objects_lookup_by_name() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let ship_id = world.spawn_object("ship", #{ name: "Ace" });
                    let wing_id = world.spawn_object("wingman", #{ name: "Ace" });
                    let named = world.objects.by_name("Ace");
                    #{
                        ship_id: ship_id,
                        wing_id: wing_id,
                        named_len: named.len(),
                        first_named_id: named[0].id(),
                        second_named_id: named[1].id()
                    }
                "#,
            )
            .expect("world.objects.by_name should evaluate in behavior-owned engine");

        assert_eq!(
            result
                .get("ship_id")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(1)
        );
        assert_eq!(
            result
                .get("wing_id")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(2)
        );
        assert_eq!(
            result
                .get("named_len")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(2)
        );
        assert_eq!(
            result
                .get("first_named_id")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(1)
        );
        assert_eq!(
            result
                .get("second_named_id")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(2)
        );
    }

    #[test]
    fn register_with_rhai_exposes_full_world_objects_registry_and_iteration() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let ship_id = world.spawn_object("ship", #{ name: "Ace" });
                    let wing_id = world.spawn_object("wingman", #{ name: "Ace" });
                    let rock_id = world.spawn_object("asteroid", #{ name: "Rock" });
                    world.set_visual(ship_id, "ship-main");
                    world.bind_visual(ship_id, "ship-shadow");
                    world.tag_add(ship_id, "player");
                    world.tag_add(wing_id, "player");
                    world.tag_add(rock_id, "hazard");

                    let found_visual = world.objects.find("ship-shadow");
                    let found_name = world.objects.find("Ace");
                    let all_ids = [];
                    for object in world.objects.all() {
                        all_ids.push(object.id());
                    }
                    let player_kinds = [];
                    for object in world.objects.by_tag("player") {
                        player_kinds.push(object.kind());
                    }
                    let ace_ids = world.objects.by_name("Ace").map(|object| object.id());

                    #{
                        found_visual_id: found_visual.id(),
                        found_name_id: found_name.id(),
                        all_ids: all_ids,
                        player_kinds: player_kinds,
                        ace_ids: ace_ids
                    }
                "#,
            )
            .expect("world.objects registry queries and iteration should evaluate");

        let all_ids = result
            .get("all_ids")
            .and_then(|value| value.clone().into_array().ok())
            .expect("all_ids");
        let player_kinds = result
            .get("player_kinds")
            .and_then(|value| value.clone().into_array().ok())
            .expect("player_kinds");
        let ace_ids = result
            .get("ace_ids")
            .and_then(|value| value.clone().into_array().ok())
            .expect("ace_ids");

        let all_ids: Vec<rhai::INT> = all_ids
            .into_iter()
            .filter_map(|value| value.try_cast::<rhai::INT>())
            .collect();
        let player_kinds: Vec<String> = player_kinds
            .into_iter()
            .filter_map(|value| value.try_cast::<String>())
            .collect();
        let ace_ids: Vec<rhai::INT> = ace_ids
            .into_iter()
            .filter_map(|value| value.try_cast::<rhai::INT>())
            .collect();

        assert_eq!(
            result
                .get("found_visual_id")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(1)
        );
        assert_eq!(
            result
                .get("found_name_id")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(1)
        );
        assert_eq!(all_ids, vec![1, 2, 3]);
        assert_eq!(
            player_kinds,
            vec!["ship".to_string(), "wingman".to_string()]
        );
        assert_eq!(ace_ids, vec![1, 2]);
    }

    #[test]
    fn register_with_rhai_returns_missing_world_object_handle_for_unknown_target() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let missing = world.objects.find("missing-visual");
                    #{
                        exists: missing.exists(),
                        id: missing.id(),
                        kind: missing.kind(),
                        tags_len: missing.tags().len(),
                        inspect_len: missing.inspect().len()
                    }
                "#,
            )
            .expect("missing world object handle should still evaluate");

        assert_eq!(
            result
                .get("exists")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(false)
        );
        assert_eq!(
            result
                .get("id")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(0)
        );
        assert_eq!(
            result
                .get("kind")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some(String::new())
        );
        assert_eq!(
            result
                .get("tags_len")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(0)
        );
        assert_eq!(
            result
                .get("inspect_len")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(0)
        );
    }

    #[test]
    fn register_with_rhai_returns_false_for_stale_world_object_handle_set() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let ship_id = world.spawn_object("ship", #{ hp: 12 });
                    world.set_visual(ship_id, "ship-main");
                    let found = world.objects.find("ship-main");
                    let despawned = world.despawn(ship_id);
                    let stale_exists = found.exists();
                    let stale_id = found.id();
                    let stale_set = found.set("hp", 99);
                    let stale_inspect_len = found.inspect().len();

                    #{
                        despawned: despawned,
                        stale_exists: stale_exists,
                        stale_id: stale_id,
                        stale_set: stale_set,
                        stale_inspect_len: stale_inspect_len
                    }
                "#,
            )
            .expect("stale world object handle should still evaluate in behavior-owned engine");

        assert_eq!(
            result
                .get("despawned")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(true)
        );
        assert_eq!(
            result
                .get("stale_exists")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(false)
        );
        assert_eq!(
            result
                .get("stale_id")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(0)
        );
        assert_eq!(
            result
                .get("stale_set")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(false)
        );
        assert_eq!(
            result
                .get("stale_inspect_len")
                .and_then(|value| value.clone().try_cast::<rhai::INT>()),
            Some(0)
        );
    }

    #[test]
    fn register_with_rhai_exposes_typed_world_body_snapshot_surface() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let ok = world.body_patch("generated-planet", #{
                        center_x: 12.0,
                        center_y: -4.0,
                        radius_px: 210.0,
                        surface_radius: 205.0,
                        radius_km: 210.0,
                        gravity_mu_km3_s2: 4321.5,
                        atmosphere_top_km: 88.0,
                        atmosphere_dense_start_km: 18.0,
                        atmosphere_drag_max: 1.5,
                        cloud_bottom_km: 6.0,
                        cloud_top_km: 12.0,
                        planet_type: "earth_like"
                    });
                    let body = world.body("generated-planet");
                    let alias = world.body_snapshot("generated-planet");

                    #{
                        ok: ok,
                        exists: body.exists,
                        id: body.id,
                        center_x: body.center_x,
                        surface_radius: body.surface_radius,
                        gravity_mu: body.gravity_mu,
                        gravity_mu_km3_s2: body.gravity_mu_km3_s2,
                        atmosphere_top_km: body.atmosphere_top_km,
                        atmosphere_dense_start_km: body.atmosphere_dense_start_km,
                        cloud_top_km: body.cloud_top_km,
                        planet_type: body.planet_type,
                        alias_exists: alias.exists,
                        inspect_id: body.inspect()["id"]
                    }
                "#,
            )
            .expect("typed world body snapshot should evaluate in behavior-owned engine");

        assert_eq!(
            result
                .get("ok")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(true)
        );
        assert_eq!(
            result
                .get("exists")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(true)
        );
        assert_eq!(
            result
                .get("id")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("generated-planet".to_string())
        );
        assert_eq!(
            result
                .get("center_x")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(12.0)
        );
        assert_eq!(
            result
                .get("surface_radius")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(205.0)
        );
        assert_eq!(
            result
                .get("gravity_mu")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(4321.5)
        );
        assert_eq!(
            result
                .get("gravity_mu_km3_s2")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(4321.5)
        );
        assert_eq!(
            result
                .get("atmosphere_top_km")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(88.0)
        );
        assert_eq!(
            result
                .get("atmosphere_dense_start_km")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(18.0)
        );
        assert_eq!(
            result
                .get("cloud_top_km")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            Some(12.0)
        );
        assert_eq!(
            result
                .get("planet_type")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("earth_like".to_string())
        );
        assert_eq!(
            result
                .get("alias_exists")
                .and_then(|value| value.clone().try_cast::<bool>()),
            Some(true)
        );
        assert_eq!(
            result
                .get("inspect_id")
                .and_then(|value| value.clone().try_cast::<String>()),
            Some("generated-planet".to_string())
        );
    }

    #[test]
    fn register_with_rhai_resolves_canonical_km_fields_for_body_inspect() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let ok = world.body_patch("derived-planet", #{
                        radius_px: 210.0,
                        surface_radius: 205.0,
                        km_per_px: 2.0,
                        atmosphere_top: 40.0,
                        atmosphere_dense_start: 8.0
                    });
                    let body = world.body("derived-planet");
                    let inspect = body.inspect();

                    #{
                        ok: ok,
                        radius_km: body.radius_km,
                        inspect_radius_km: inspect["radius_km"],
                        atmosphere_top_km: body.atmosphere_top_km,
                        inspect_atmosphere_top_km: inspect["atmosphere_top_km"],
                        atmosphere_dense_start_km: body.atmosphere_dense_start_km,
                        inspect_atmosphere_dense_start_km: inspect["atmosphere_dense_start_km"],
                        km_per_px: body.km_per_px,
                        inspect_km_per_px: inspect["km_per_px"]
                    }
                "#,
            )
            .expect("typed world body inspect should resolve canonical km fields");

        for key in [
            "radius_km",
            "inspect_radius_km",
            "atmosphere_top_km",
            "inspect_atmosphere_top_km",
            "atmosphere_dense_start_km",
            "inspect_atmosphere_dense_start_km",
        ] {
            assert_eq!(
                result
                    .get(key)
                    .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
                Some(match key {
                    "radius_km" | "inspect_radius_km" | "info_radius_km" => 420.0,
                    "atmosphere_top_km"
                    | "inspect_atmosphere_top_km"
                    | "info_atmosphere_top_km" => 80.0,
                    _ => 16.0,
                }),
                "unexpected value for {key}"
            );
        }
        for key in ["km_per_px", "inspect_km_per_px"] {
            assert_eq!(
                result
                    .get(key)
                    .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
                Some(2.0),
                "unexpected km_per_px for {key}"
            );
        }
    }

    #[test]
    fn register_with_rhai_finds_planet_spawn_angle_for_preferred_biomes() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let angle: rhai::FLOAT = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    world.find_planet_spawn_angle(
                        #{
                            seed: 1847,
                            has_ocean: true,
                            ocean: 0.55,
                            cscale: 2.5,
                            cwarp: 0.65,
                            coct: 5,
                            mscale: 6.0,
                            mstr: 0.45,
                            mroct: 5,
                            moisture: 3.0,
                            ice: 1.0,
                            lapse: 0.6,
                            rain: 0.4
                        },
                        ["beach", "desert", "grassland"]
                    )
                "#,
            )
            .expect("spawn angle lookup should evaluate");

        assert!((0.0..=360.0).contains(&angle));
    }

    #[test]
    fn register_with_rhai_finds_planet_spawn_with_surface_data() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    world.find_planet_spawn(
                        #{
                            seed: 1847,
                            has_ocean: true,
                            ocean: 0.55,
                            cscale: 2.5,
                            cwarp: 0.65,
                            coct: 5,
                            mscale: 6.0,
                            mstr: 0.45,
                            mroct: 5,
                            moisture: 3.0,
                            ice: 1.0,
                            lapse: 0.6,
                            rain: 0.4,
                            disp: 0.22
                        },
                        ["beach", "desert", "grassland"]
                    )
                "#,
            )
            .expect("spawn lookup with surface data should evaluate");

        let longitude = result
            .get("longitude_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("longitude_deg");
        let latitude = result
            .get("latitude_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("latitude_deg");
        let nx = result
            .get("normal_x")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("normal_x");
        let ny = result
            .get("normal_y")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("normal_y");
        let nz = result
            .get("normal_z")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("normal_z");
        let surface_radius_scale = result
            .get("surface_radius_scale")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("surface_radius_scale");
        let biome = result
            .get("biome")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("biome");

        assert!((0.0..=360.0).contains(&longitude));
        assert!((-90.0..=90.0).contains(&latitude));
        assert!(surface_radius_scale > 0.0);
        assert!(!biome.is_empty());

        let normal_len = (nx * nx + ny * ny + nz * nz).sqrt();
        assert!(
            (normal_len - 1.0).abs() < 0.001,
            "expected unit surface normal, got {normal_len}"
        );
    }

    #[test]
    fn register_with_rhai_uses_default_biomes_when_spawn_preferences_are_empty() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let config = #{
                        seed: 1847,
                        has_ocean: true,
                        ocean: 0.55,
                        cscale: 2.5,
                        cwarp: 0.65,
                        coct: 5,
                        mscale: 6.0,
                        mstr: 0.45,
                        mroct: 5,
                        moisture: 3.0,
                        ice: 1.0,
                        lapse: 0.6,
                        rain: 0.4
                    };
                    #{
                        empty: world.find_planet_spawn_angle(config, []),
                        explicit: world.find_planet_spawn_angle(config, ["beach", "desert", "grassland"])
                    }
                "#,
            )
            .expect("empty spawn preference list should evaluate");

        assert_eq!(
            result
                .get("empty")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            result
                .get("explicit")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
        );
    }

    #[test]
    fn register_with_rhai_accepts_alias_and_string_spawn_config_fields() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_with_rhai(&mut engine);

        let mut scope = RhaiScope::new();
        scope.push("world", build_world_api(world));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let canonical = world.find_planet_spawn_angle(
                        #{
                            seed: 1847,
                            has_ocean: true,
                            ocean: 0.55,
                            cscale: 2.5,
                            cwarp: 0.65,
                            coct: 5,
                            mscale: 6.0,
                            mstr: 0.45,
                            mroct: 5,
                            moisture: 3.0,
                            ice: 1.0,
                            lapse: 0.6,
                            rain: 0.4
                        },
                        ["beach", "desert", "grassland"]
                    );
                    let alias = world.find_planet_spawn_angle(
                        #{
                            seed: "1847",
                            "has-ocean": "yes",
                            "ocean-fraction": "0.55",
                            "continent-scale": "2.5",
                            "continent-warp": "0.65",
                            "continent-octaves": "5",
                            "mountain-scale": "6.0",
                            "mountain-strength": "0.45",
                            "mountain-ridge-octaves": "5",
                            "moisture-scale": "3.0",
                            "ice-cap-strength": "1.0",
                            "lapse-rate": "0.6",
                            "rain-shadow": "0.4"
                        },
                        ["beach", "desert", "grassland"]
                    );
                    #{
                        canonical: canonical,
                        alias: alias
                    }
                "#,
            )
            .expect("string/alias spawn config should evaluate");

        assert_eq!(
            result
                .get("canonical")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>()),
            result
                .get("alias")
                .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
        );
    }
}

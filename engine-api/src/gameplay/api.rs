//! Generic script runtime context shared by gameplay-facing APIs.

use std::sync::{Arc, Mutex};

use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Map as RhaiMap};

use engine_game::{CollisionHit, GameplayWorld};

use crate::BehaviorCommand;

pub type CommandQueue = Arc<Mutex<Vec<BehaviorCommand>>>;

#[derive(Clone)]
pub struct ScriptWorldContext {
    pub world: Option<GameplayWorld>,
    pub collisions: Arc<Vec<CollisionHit>>,
    pub collision_enters: Arc<Vec<CollisionHit>>,
    pub collision_stays: Arc<Vec<CollisionHit>>,
    pub collision_exits: Arc<Vec<CollisionHit>>,
    pub queue: CommandQueue,
}

impl ScriptWorldContext {
    pub fn new(
        world: Option<GameplayWorld>,
        collisions: Arc<Vec<CollisionHit>>,
        collision_enters: Arc<Vec<CollisionHit>>,
        collision_stays: Arc<Vec<CollisionHit>>,
        collision_exits: Arc<Vec<CollisionHit>>,
        queue: CommandQueue,
    ) -> Self {
        Self {
            world,
            collisions,
            collision_enters,
            collision_stays,
            collision_exits,
            queue,
        }
    }
}

#[derive(Clone)]
pub struct ScriptEntityContext {
    pub world: Option<GameplayWorld>,
    pub id: u64,
    pub queue: CommandQueue,
}

impl ScriptEntityContext {
    pub fn new(world: Option<GameplayWorld>, id: u64, queue: CommandQueue) -> Self {
        Self { world, id, queue }
    }
}

pub trait GameplayEntityCoreApi: Clone + 'static {
    fn exists(&mut self) -> bool;
    fn get(&mut self, path: &str) -> RhaiDynamic;
    fn get_i(&mut self, path: &str, fallback: rhai::INT) -> rhai::INT;
    fn get_bool(&mut self, path: &str, fallback: bool) -> bool;
    fn set(&mut self, path: &str, value: RhaiDynamic) -> bool;
    fn kind(&mut self) -> String;
    fn tags(&mut self) -> RhaiArray;
    fn get_metadata(&mut self) -> RhaiMap;
    fn get_components(&mut self) -> RhaiMap;
    fn transform(&mut self) -> RhaiMap;
    fn set_position(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) -> bool;
    fn set_heading(&mut self, heading: rhai::FLOAT) -> bool;
    fn lifetime_remaining(&mut self) -> rhai::INT;
    fn set_many(&mut self, map: RhaiMap) -> bool;
    fn data(&mut self) -> RhaiMap;
    fn get_f(&mut self, path: &str, fallback: rhai::FLOAT) -> rhai::FLOAT;
    fn get_s(&mut self, path: &str, fallback: &str) -> String;
    fn despawn(&mut self) -> bool;
    fn id(&mut self) -> rhai::INT;
    fn flag(&mut self, name: &str) -> bool;
    fn set_flag(&mut self, name: &str, value: bool) -> bool;
    fn cooldown_start(&mut self, name: &str, ms: rhai::INT) -> bool;
    fn cooldown_ready(&mut self, name: &str) -> bool;
    fn cooldown_remaining(&mut self, name: &str) -> rhai::INT;
    fn status_add(&mut self, name: &str, ms: rhai::INT) -> bool;
    fn status_has(&mut self, name: &str) -> bool;
    fn status_remaining(&mut self, name: &str) -> rhai::INT;
    fn set_acceleration(&mut self, ax: rhai::FLOAT, ay: rhai::FLOAT) -> bool;
    fn collider(&mut self) -> RhaiMap;
    fn heading(&mut self) -> rhai::INT;
    fn heading_vector(&mut self) -> RhaiMap;
    fn attach_controller(&mut self, config: RhaiMap) -> bool;
    fn set_turn(&mut self, dir: rhai::INT) -> bool;
    fn set_thrust(&mut self, on: bool) -> bool;
    fn lifetime_fraction(&mut self) -> rhai::FLOAT;
    fn set_fg(&mut self, color: &str) -> bool;
    fn set_radius(&mut self, r: rhai::INT) -> bool;
}

pub trait GameplayWorldCoreApi<TEntity>: Clone + 'static
where
    TEntity: GameplayEntityCoreApi,
{
    fn clear(&mut self);
    fn reset_dynamic_entities(&mut self) -> bool;
    fn count(&mut self) -> rhai::INT;
    fn count_kind(&mut self, kind: &str) -> rhai::INT;
    fn count_tag(&mut self, tag: &str) -> rhai::INT;
    fn first_kind(&mut self, kind: &str) -> rhai::INT;
    fn first_tag(&mut self, tag: &str) -> rhai::INT;
    fn diagnostic_info(&mut self) -> RhaiMap;
    fn spawn(&mut self, kind: &str, payload: RhaiDynamic) -> rhai::INT;
    fn despawn(&mut self, id: rhai::INT) -> bool;
    fn exists(&mut self, id: rhai::INT) -> bool;
    fn kind(&mut self, id: rhai::INT) -> String;
    fn tags(&mut self, id: rhai::INT) -> RhaiArray;
    fn ids(&mut self) -> RhaiArray;
    fn entity(&mut self, id: rhai::INT) -> TEntity;
    fn query_kind(&mut self, kind: &str) -> RhaiArray;
    fn query_tag(&mut self, tag: &str) -> RhaiArray;
    fn get(&mut self, id: rhai::INT, path: &str) -> RhaiDynamic;
    fn set(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool;
    fn has(&mut self, id: rhai::INT, path: &str) -> bool;
    fn remove(&mut self, id: rhai::INT, path: &str) -> bool;
    fn push(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool;
    fn set_transform(
        &mut self,
        id: rhai::INT,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        heading: rhai::FLOAT,
    ) -> bool;
    fn transform(&mut self, id: rhai::INT) -> RhaiDynamic;
    fn set_physics(
        &mut self,
        id: rhai::INT,
        vx: rhai::FLOAT,
        vy: rhai::FLOAT,
        ax: rhai::FLOAT,
        ay: rhai::FLOAT,
        drag: rhai::FLOAT,
        max_speed: rhai::FLOAT,
    ) -> bool;
    fn physics(&mut self, id: rhai::INT) -> RhaiDynamic;
    fn set_lifetime(&mut self, id: rhai::INT, ttl_ms: rhai::INT) -> bool;
    fn collisions(&mut self) -> RhaiArray;
    fn collisions_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray;
    fn collisions_of(&mut self, kind: &str) -> RhaiArray;
    fn collision_enters_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray;
    fn collision_stays_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray;
    fn collision_exits_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray;
    fn spawn_child_entity(
        &mut self,
        parent_id: rhai::INT,
        kind: &str,
        template: &str,
        data: RhaiMap,
    ) -> rhai::INT;
    fn despawn_children_of(&mut self, parent_id: rhai::INT);
    fn distance(&mut self, a: rhai::INT, b: rhai::INT) -> rhai::FLOAT;
    fn any_alive(&mut self, kind: &str) -> bool;
    fn set_world_bounds(
        &mut self,
        min_x: rhai::FLOAT,
        min_y: rhai::FLOAT,
        max_x: rhai::FLOAT,
        max_y: rhai::FLOAT,
    );
    fn world_bounds(&mut self) -> RhaiMap;
    fn angular_body_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool;
    fn set_angular_input(&mut self, id: rhai::INT, input: rhai::FLOAT) -> bool;
    fn angular_vel(&mut self, id: rhai::INT) -> rhai::FLOAT;
    fn linear_brake_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool;
    fn set_linear_brake_active(&mut self, id: rhai::INT, active: bool) -> bool;
    fn rand_i(&mut self, min: rhai::INT, max: rhai::INT) -> rhai::INT;
    fn rand_seed(&mut self, seed: rhai::INT);
    fn tag_add(&mut self, id: rhai::INT, tag: &str) -> bool;
    fn tag_remove(&mut self, id: rhai::INT, tag: &str) -> bool;
    fn tag_has(&mut self, id: rhai::INT, tag: &str) -> bool;
    fn after_ms(&mut self, label: &str, delay_ms: rhai::INT);
    fn timer_fired(&mut self, label: &str) -> bool;
    fn cancel_timer(&mut self, label: &str) -> bool;
    fn enable_wrap(
        &mut self,
        id: rhai::INT,
        min_x: rhai::FLOAT,
        max_x: rhai::FLOAT,
        min_y: rhai::FLOAT,
        max_y: rhai::FLOAT,
    ) -> bool;
    fn disable_wrap(&mut self, id: rhai::INT) -> bool;
    fn poll_collision_events(&mut self) -> RhaiArray;
    fn clear_events(&mut self);
    fn spawn_batch(&mut self, specs: RhaiArray) -> RhaiArray;
}

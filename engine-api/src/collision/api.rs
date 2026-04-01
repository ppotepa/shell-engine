//! Collision domain API: script-facing namespace for querying collision events.

use std::sync::Arc;

use rhai::{Array as RhaiArray, Engine as RhaiEngine, Map as RhaiMap};

use engine_game::{CollisionHit, GameplayWorld};

use crate::gameplay::api::ScriptWorldContext;

/// Filters a collision hit slice by kind pair, returning `{kind_a: id, kind_b: id}` maps.
///
/// This is a shared helper used by both `ScriptCollisionApi` and `ScriptGameplayApi`.
pub fn filter_hits_by_kind(
    hits: &[CollisionHit],
    world: &GameplayWorld,
    kind_a: &str,
    kind_b: &str,
) -> RhaiArray {
    hits.iter()
        .filter_map(|hit| {
            let ka = world.kind_of(hit.a).unwrap_or_default();
            let kb = world.kind_of(hit.b).unwrap_or_default();
            if ka == kind_a && kb == kind_b {
                let mut map = RhaiMap::new();
                map.insert(kind_a.into(), (hit.a as rhai::INT).into());
                map.insert(kind_b.into(), (hit.b as rhai::INT).into());
                Some(map.into())
            } else if ka == kind_b && kb == kind_a {
                let mut map = RhaiMap::new();
                map.insert(kind_a.into(), (hit.b as rhai::INT).into());
                map.insert(kind_b.into(), (hit.a as rhai::INT).into());
                Some(map.into())
            } else {
                None
            }
        })
        .collect()
}

/// Filters a collision hit slice to those involving the given kind (either side).
///
/// Returns `#{self: id, other: id}` maps where `self` is the entity of the given kind.
pub fn filter_hits_of_kind(
    hits: &[CollisionHit],
    world: &GameplayWorld,
    kind: &str,
) -> RhaiArray {
    hits.iter()
        .filter_map(|hit| {
            let ka = world.kind_of(hit.a).unwrap_or_default();
            let kb = world.kind_of(hit.b).unwrap_or_default();
            if ka == kind {
                let mut map = RhaiMap::new();
                map.insert("self".into(), (hit.a as rhai::INT).into());
                map.insert("other".into(), (hit.b as rhai::INT).into());
                Some(map.into())
            } else if kb == kind {
                let mut map = RhaiMap::new();
                map.insert("self".into(), (hit.b as rhai::INT).into());
                map.insert("other".into(), (hit.a as rhai::INT).into());
                Some(map.into())
            } else {
                None
            }
        })
        .collect()
}

/// Script-facing API for querying collision events this frame.
///
/// Pushed as `collision` in the Rhai scope when gameplay is active.
///
/// ```rhai
/// for hit in collision.enters("bullet", "asteroid") {
///     let bullet_id   = hit["bullet"];
///     let asteroid_id = hit["asteroid"];
/// }
/// for hit in collision.of("ship") {
///     let ship_id  = hit["self"];
///     let other_id = hit["other"];
/// }
/// ```
#[derive(Clone)]
pub struct ScriptCollisionApi {
    ctx: ScriptWorldContext,
}

impl ScriptCollisionApi {
    pub fn new(ctx: ScriptWorldContext) -> Self {
        Self { ctx }
    }

    fn world(&self) -> Option<&GameplayWorld> {
        self.ctx.world.as_ref()
    }

    /// All collision-enter events between `kind_a` and `kind_b` this frame.
    pub fn enters(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world() else { return vec![]; };
        filter_hits_by_kind(&self.ctx.collision_enters, world, kind_a, kind_b)
    }

    /// All collision-stay events between `kind_a` and `kind_b` this frame.
    pub fn stays(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world() else { return vec![]; };
        filter_hits_by_kind(&self.ctx.collision_stays, world, kind_a, kind_b)
    }

    /// All collision-exit events between `kind_a` and `kind_b` this frame.
    pub fn exits(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world() else { return vec![]; };
        filter_hits_by_kind(&self.ctx.collision_exits, world, kind_a, kind_b)
    }

    /// All collision-enter events involving `kind` on either side.
    pub fn enters_of(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.world() else { return vec![]; };
        filter_hits_of_kind(&self.ctx.collision_enters, world, kind)
    }

    /// All collision-stay events involving `kind` on either side.
    pub fn stays_of(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.world() else { return vec![]; };
        filter_hits_of_kind(&self.ctx.collision_stays, world, kind)
    }

    /// All raw collision-enter events for this frame (unfiltered).
    pub fn all_enters(&mut self) -> RhaiArray {
        self.ctx.collision_enters.iter().map(|hit| {
            let mut map = RhaiMap::new();
            map.insert("a".into(), (hit.a as rhai::INT).into());
            map.insert("b".into(), (hit.b as rhai::INT).into());
            map.into()
        }).collect()
    }

    /// Returns the number of enter events between `kind_a` and `kind_b`.
    pub fn count_enters(&mut self, kind_a: &str, kind_b: &str) -> rhai::INT {
        self.enters(kind_a, kind_b).len() as rhai::INT
    }

    /// Returns true if any enter event occurred between `kind_a` and `kind_b`.
    pub fn any_enter(&mut self, kind_a: &str, kind_b: &str) -> bool {
        !self.enters(kind_a, kind_b).is_empty()
    }

    /// Builds a `ScriptCollisionApi` from the raw Arc slices (convenience for construction).
    pub fn from_arcs(
        world: Option<engine_game::GameplayWorld>,
        collisions: Arc<Vec<CollisionHit>>,
        collision_enters: Arc<Vec<CollisionHit>>,
        collision_stays: Arc<Vec<CollisionHit>>,
        collision_exits: Arc<Vec<CollisionHit>>,
        queue: crate::gameplay::api::CommandQueue,
    ) -> Self {
        Self {
            ctx: ScriptWorldContext::new(
                world,
                collisions,
                collision_enters,
                collision_stays,
                collision_exits,
                queue,
            ),
        }
    }
}

pub fn register_collision_api(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptCollisionApi>("CollisionApi");

    engine.register_fn("enters", |api: &mut ScriptCollisionApi, a: &str, b: &str| {
        api.enters(a, b)
    });
    engine.register_fn("stays", |api: &mut ScriptCollisionApi, a: &str, b: &str| {
        api.stays(a, b)
    });
    engine.register_fn("exits", |api: &mut ScriptCollisionApi, a: &str, b: &str| {
        api.exits(a, b)
    });
    engine.register_fn("enters_of", |api: &mut ScriptCollisionApi, kind: &str| {
        api.enters_of(kind)
    });
    engine.register_fn("stays_of", |api: &mut ScriptCollisionApi, kind: &str| {
        api.stays_of(kind)
    });
    engine.register_fn("all_enters", |api: &mut ScriptCollisionApi| {
        api.all_enters()
    });
    engine.register_fn(
        "count_enters",
        |api: &mut ScriptCollisionApi, a: &str, b: &str| api.count_enters(a, b),
    );
    engine.register_fn(
        "any_enter",
        |api: &mut ScriptCollisionApi, a: &str, b: &str| api.any_enter(a, b),
    );
}

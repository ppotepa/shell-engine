//! Physics domain API: ScriptPhysicsApi for velocity, acceleration, colliders, and constraints.
//!
//! Generic physics operations for 2D entities. Auto-detects 2D context from GameplayWorld.
//! Exposes clean velocity/acceleration manipulation without game-specific concerns.

use std::sync::{Arc, Mutex};

use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};

use engine_game::{GameplayWorld, PhysicsBody2D, Collider2D, ColliderShape};

use crate::BehaviorCommand;

// ── ScriptPhysicsApi ──────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptPhysicsApi {
    world: Option<GameplayWorld>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptPhysicsApi {
    pub(crate) fn new(
        world: Option<GameplayWorld>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self { world, queue }
    }

    /// Get velocity as [vx, vy] array.
    pub(crate) fn velocity(&mut self, id: rhai::INT) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        let Some(body) = world.physics(id as u64) else {
            return RhaiArray::new();
        };
        let mut arr = RhaiArray::with_capacity(2);
        arr.push((body.vx as rhai::FLOAT).into());
        arr.push((body.vy as rhai::FLOAT).into());
        arr
    }

    /// Set velocity to (vx, vy).
    pub(crate) fn set_velocity(&mut self, id: rhai::INT, vx: rhai::FLOAT, vy: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(id as u64) else {
            return false;
        };
        body.vx = vx as f32;
        body.vy = vy as f32;
        world.set_physics(id as u64, body)
    }

    /// Add (dvx, dvy) to velocity.
    pub(crate) fn add_velocity(&mut self, id: rhai::INT, dvx: rhai::FLOAT, dvy: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(id as u64) else {
            return false;
        };
        body.vx += dvx as f32;
        body.vy += dvy as f32;
        world.set_physics(id as u64, body)
    }

    /// Get acceleration as [ax, ay] array.
    pub(crate) fn acceleration(&mut self, id: rhai::INT) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        let Some(body) = world.physics(id as u64) else {
            return RhaiArray::new();
        };
        let mut arr = RhaiArray::with_capacity(2);
        arr.push((body.ax as rhai::FLOAT).into());
        arr.push((body.ay as rhai::FLOAT).into());
        arr
    }

    /// Set acceleration to (ax, ay).
    pub(crate) fn set_acceleration(&mut self, id: rhai::INT, ax: rhai::FLOAT, ay: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(id as u64) else {
            return false;
        };
        body.ax = ax as f32;
        body.ay = ay as f32;
        world.set_physics(id as u64, body)
    }

    /// Add (dax, day) to acceleration.
    pub(crate) fn add_acceleration(&mut self, id: rhai::INT, dax: rhai::FLOAT, day: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(id as u64) else {
            return false;
        };
        body.ax += dax as f32;
        body.ay += day as f32;
        world.set_physics(id as u64, body)
    }

    /// Get drag factor (0.0 = no drag, 1.0 = full stop per second).
    pub(crate) fn drag(&mut self, id: rhai::INT) -> rhai::FLOAT {
        let Some(world) = self.world.as_ref() else {
            return 0.0;
        };
        world.physics(id as u64).map(|b| b.drag as rhai::FLOAT).unwrap_or(0.0)
    }

    /// Set drag factor.
    pub(crate) fn set_drag(&mut self, id: rhai::INT, drag: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(id as u64) else {
            return false;
        };
        body.drag = (drag as f32).clamp(0.0, 1.0);
        world.set_physics(id as u64, body)
    }

    /// Get maximum speed limit (0.0 = unlimited).
    pub(crate) fn max_speed(&mut self, id: rhai::INT) -> rhai::FLOAT {
        let Some(world) = self.world.as_ref() else {
            return 0.0;
        };
        world
            .physics(id as u64)
            .map(|b| b.max_speed as rhai::FLOAT)
            .unwrap_or(0.0)
    }

    /// Set maximum speed limit.
    pub(crate) fn set_max_speed(&mut self, id: rhai::INT, max_speed: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(id as u64) else {
            return false;
        };
        body.max_speed = (max_speed as f32).max(0.0);
        world.set_physics(id as u64, body)
    }

    /// Get collider as {shape, layer, mask}.
    pub(crate) fn collider(&mut self, id: rhai::INT) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(coll) = world.collider(id as u64) else {
            return RhaiMap::new();
        };
        let mut map = RhaiMap::new();
        match coll.shape {
            ColliderShape::Circle { radius } => {
                map.insert("type".into(), "circle".into());
                map.insert("radius".into(), (radius as rhai::FLOAT).into());
            }
            ColliderShape::Polygon { points } => {
                map.insert("type".into(), "polygon".into());
                let pts: RhaiArray = points
                    .iter()
                    .map(|[x, y]| {
                        let mut pair = RhaiArray::with_capacity(2);
                        pair.push((*x as rhai::FLOAT).into());
                        pair.push((*y as rhai::FLOAT).into());
                        pair.into()
                    })
                    .collect();
                map.insert("points".into(), pts.into());
            }
        }
        map.insert("layer".into(), (coll.layer as rhai::INT).into());
        map.insert("mask".into(), (coll.mask as rhai::INT).into());
        map
    }

    /// Set circle collider.
    pub(crate) fn set_collider_circle(
        &mut self,
        id: rhai::INT,
        radius: rhai::FLOAT,
        layer: rhai::INT,
        mask: rhai::INT,
    ) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let collider = Collider2D {
            shape: ColliderShape::Circle {
                radius: (radius as f32).max(0.0),
            },
            layer: (layer as u32),
            mask: (mask as u32),
        };
        world.set_collider(id as u64, collider)
    }

    /// Set polygon collider.
    pub(crate) fn set_collider_polygon(
        &mut self,
        id: rhai::INT,
        points: RhaiArray,
        layer: rhai::INT,
        mask: rhai::INT,
    ) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        // Convert Rhai array of [x, y] pairs to Vec<[f32; 2]>
        let mut polygon_points = Vec::new();
        for item in points {
            if let Some(pair) = item.clone().try_cast::<RhaiArray>() {
                if pair.len() >= 2 {
                    let x = pair.get(0).and_then(|v| v.clone().try_cast::<rhai::FLOAT>());
                    let y = pair.get(1).and_then(|v| v.clone().try_cast::<rhai::FLOAT>());
                    if let (Some(x), Some(y)) = (x, y) {
                        polygon_points.push([x as f32, y as f32]);
                    }
                }
            }
        }
        let collider = Collider2D {
            shape: ColliderShape::Polygon {
                points: polygon_points,
            },
            layer: (layer as u32),
            mask: (mask as u32),
        };
        world.set_collider(id as u64, collider)
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptPhysicsApi>("PhysicsApi");

    engine.register_fn("velocity", |physics: &mut ScriptPhysicsApi, id: rhai::INT| {
        physics.velocity(id)
    });
    engine.register_fn(
        "set_velocity",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT, vx: rhai::FLOAT, vy: rhai::FLOAT| {
            physics.set_velocity(id, vx, vy)
        },
    );
    engine.register_fn(
        "add_velocity",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT, dvx: rhai::FLOAT, dvy: rhai::FLOAT| {
            physics.add_velocity(id, dvx, dvy)
        },
    );
    engine.register_fn("acceleration", |physics: &mut ScriptPhysicsApi, id: rhai::INT| {
        physics.acceleration(id)
    });
    engine.register_fn(
        "set_acceleration",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT, ax: rhai::FLOAT, ay: rhai::FLOAT| {
            physics.set_acceleration(id, ax, ay)
        },
    );
    engine.register_fn(
        "add_acceleration",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT, dax: rhai::FLOAT, day: rhai::FLOAT| {
            physics.add_acceleration(id, dax, day)
        },
    );
    engine.register_fn("drag", |physics: &mut ScriptPhysicsApi, id: rhai::INT| physics.drag(id));
    engine.register_fn(
        "set_drag",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT, drag: rhai::FLOAT| physics.set_drag(id, drag),
    );
    engine.register_fn("max_speed", |physics: &mut ScriptPhysicsApi, id: rhai::INT| {
        physics.max_speed(id)
    });
    engine.register_fn(
        "set_max_speed",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT, max_speed: rhai::FLOAT| {
            physics.set_max_speed(id, max_speed)
        },
    );
    engine.register_fn("collider", |physics: &mut ScriptPhysicsApi, id: rhai::INT| {
        physics.collider(id)
    });
    engine.register_fn(
        "set_collider_circle",
        |physics: &mut ScriptPhysicsApi,
         id: rhai::INT,
         radius: rhai::FLOAT,
         layer: rhai::INT,
         mask: rhai::INT| {
            physics.set_collider_circle(id, radius, layer, mask)
        },
    );
    engine.register_fn(
        "set_collider_polygon",
        |physics: &mut ScriptPhysicsApi,
         id: rhai::INT,
         points: RhaiArray,
         layer: rhai::INT,
         mask: rhai::INT| {
            physics.set_collider_polygon(id, points, layer, mask)
        },
    );
}

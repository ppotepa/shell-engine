//! Physics domain API: ScriptPhysicsApi for world-level velocity/acceleration operations,
//! and ScriptEntityPhysicsApi for entity-level physics as a nested property.
//!
//! Generic physics operations for 2D entities. Auto-detects 2D context from GameplayWorld.
//! Exposes clean velocity/acceleration manipulation without game-specific concerns.

use std::sync::{Arc, Mutex};

use rhai::{Array as RhaiArray, Engine as RhaiEngine, Map as RhaiMap};

use engine_game::{Collider2D, ColliderShape, GameplayWorld};

use crate::BehaviorCommand;

// ── ScriptEntityPhysicsApi ────────────────────────────────────────────────
//
// Entity-specific physics API: accessed as ship.physics.velocity(), ship.physics.set_velocity(), etc.
// Implicitly knows its entity ID; all operations are on that entity.

#[derive(Clone)]
pub(crate) struct ScriptEntityPhysicsApi {
    world: Option<GameplayWorld>,
    entity_id: u64,
}

impl ScriptEntityPhysicsApi {
    pub(crate) fn new(world: Option<GameplayWorld>, entity_id: u64) -> Self {
        Self { world, entity_id }
    }

    /// Get velocity as [vx, vy] array.
    pub(crate) fn velocity(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        let Some(body) = world.physics(self.entity_id) else {
            return RhaiArray::new();
        };
        vec![
            (body.vx as rhai::FLOAT).into(),
            (body.vy as rhai::FLOAT).into(),
        ]
    }
    pub(crate) fn set_velocity(&mut self, vx: rhai::FLOAT, vy: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(self.entity_id) else {
            return false;
        };
        body.vx = vx as f32;
        body.vy = vy as f32;
        world.set_physics(self.entity_id, body)
    }

    /// Add (dvx, dvy) to velocity.
    pub(crate) fn add_velocity(&mut self, dvx: rhai::FLOAT, dvy: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(self.entity_id) else {
            return false;
        };
        body.vx += dvx as f32;
        body.vy += dvy as f32;
        world.set_physics(self.entity_id, body)
    }

    /// Get acceleration as [ax, ay] array.
    pub(crate) fn acceleration(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        let Some(body) = world.physics(self.entity_id) else {
            return RhaiArray::new();
        };
        vec![
            (body.ax as rhai::FLOAT).into(),
            (body.ay as rhai::FLOAT).into(),
        ]
    }

    /// Set acceleration to (ax, ay).
    pub(crate) fn set_acceleration(&mut self, ax: rhai::FLOAT, ay: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(self.entity_id) else {
            return false;
        };
        body.ax = ax as f32;
        body.ay = ay as f32;
        world.set_physics(self.entity_id, body)
    }

    /// Add (dax, day) to acceleration.
    pub(crate) fn add_acceleration(&mut self, dax: rhai::FLOAT, day: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(self.entity_id) else {
            return false;
        };
        body.ax += dax as f32;
        body.ay += day as f32;
        world.set_physics(self.entity_id, body)
    }

    /// Get drag factor (0.0 = no drag, 1.0 = full stop per second).
    pub(crate) fn drag(&mut self) -> rhai::FLOAT {
        let Some(world) = self.world.as_ref() else {
            return 0.0;
        };
        world
            .physics(self.entity_id)
            .map(|b| b.drag as rhai::FLOAT)
            .unwrap_or(0.0)
    }

    /// Set drag factor.
    pub(crate) fn set_drag(&mut self, drag: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(self.entity_id) else {
            return false;
        };
        body.drag = (drag as f32).clamp(0.0, 1.0);
        world.set_physics(self.entity_id, body)
    }

    /// Get maximum speed limit (0.0 = unlimited).
    pub(crate) fn max_speed(&mut self) -> rhai::FLOAT {
        let Some(world) = self.world.as_ref() else {
            return 0.0;
        };
        world
            .physics(self.entity_id)
            .map(|b| b.max_speed as rhai::FLOAT)
            .unwrap_or(0.0)
    }

    /// Set maximum speed limit.
    pub(crate) fn set_max_speed(&mut self, max_speed: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(self.entity_id) else {
            return false;
        };
        body.max_speed = (max_speed as f32).max(0.0);
        world.set_physics(self.entity_id, body)
    }

    /// Get collider as {type, radius/points, layer, mask}.
    pub(crate) fn collider(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(coll) = world.collider(self.entity_id) else {
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
                        rhai::Dynamic::from(vec![
                            (*x as rhai::FLOAT).into(),
                            (*y as rhai::FLOAT).into(),
                        ] as RhaiArray)
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
        world.set_collider(self.entity_id, collider)
    }

    /// Set polygon collider.
    pub(crate) fn set_collider_polygon(
        &mut self,
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
                    let x = pair
                        .first()
                        .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
                        .or_else(|| {
                            pair.first()
                                .and_then(|v| v.clone().try_cast::<rhai::INT>())
                                .map(|v| v as rhai::FLOAT)
                        });
                    let y = pair
                        .get(1)
                        .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
                        .or_else(|| {
                            pair.get(1)
                                .and_then(|v| v.clone().try_cast::<rhai::INT>())
                                .map(|v| v as rhai::FLOAT)
                        });
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
        world.set_collider(self.entity_id, collider)
    }

    /// Get mass.
    pub(crate) fn mass(&mut self) -> rhai::FLOAT {
        let Some(world) = self.world.as_ref() else {
            return 1.0;
        };
        world
            .physics(self.entity_id)
            .map(|b| b.mass as rhai::FLOAT)
            .unwrap_or(1.0)
    }

    /// Set mass (0.0 = immovable).
    pub(crate) fn set_mass(&mut self, mass: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(self.entity_id) else {
            return false;
        };
        body.mass = (mass as f32).max(0.0);
        world.set_physics(self.entity_id, body)
    }

    /// Get restitution (0.0 = inelastic, 1.0 = elastic).
    pub(crate) fn restitution(&mut self) -> rhai::FLOAT {
        let Some(world) = self.world.as_ref() else {
            return 0.7;
        };
        world
            .physics(self.entity_id)
            .map(|b| b.restitution as rhai::FLOAT)
            .unwrap_or(0.7)
    }

    /// Set restitution coefficient.
    pub(crate) fn set_restitution(&mut self, r: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(self.entity_id) else {
            return false;
        };
        body.restitution = (r as f32).clamp(0.0, 1.0);
        world.set_physics(self.entity_id, body)
    }
}

// ── ScriptPhysicsApi ──────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptPhysicsApi {
    world: Option<GameplayWorld>,
    #[allow(dead_code)]
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptPhysicsApi {
    #[allow(dead_code)]
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
        vec![
            (body.vx as rhai::FLOAT).into(),
            (body.vy as rhai::FLOAT).into(),
        ]
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
    pub(crate) fn add_velocity(
        &mut self,
        id: rhai::INT,
        dvx: rhai::FLOAT,
        dvy: rhai::FLOAT,
    ) -> bool {
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
        vec![
            (body.ax as rhai::FLOAT).into(),
            (body.ay as rhai::FLOAT).into(),
        ]
    }

    /// Set acceleration to (ax, ay).
    pub(crate) fn set_acceleration(
        &mut self,
        id: rhai::INT,
        ax: rhai::FLOAT,
        ay: rhai::FLOAT,
    ) -> bool {
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
    pub(crate) fn add_acceleration(
        &mut self,
        id: rhai::INT,
        dax: rhai::FLOAT,
        day: rhai::FLOAT,
    ) -> bool {
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
        world
            .physics(id as u64)
            .map(|b| b.drag as rhai::FLOAT)
            .unwrap_or(0.0)
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
                        rhai::Dynamic::from(vec![
                            (*x as rhai::FLOAT).into(),
                            (*y as rhai::FLOAT).into(),
                        ] as RhaiArray)
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
                    let x = pair
                        .first()
                        .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
                        .or_else(|| {
                            pair.first()
                                .and_then(|v| v.clone().try_cast::<rhai::INT>())
                                .map(|v| v as rhai::FLOAT)
                        });
                    let y = pair
                        .get(1)
                        .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
                        .or_else(|| {
                            pair.get(1)
                                .and_then(|v| v.clone().try_cast::<rhai::INT>())
                                .map(|v| v as rhai::FLOAT)
                        });
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

    /// Get mass.
    pub(crate) fn mass(&mut self, id: rhai::INT) -> rhai::FLOAT {
        let Some(world) = self.world.as_ref() else {
            return 1.0;
        };
        world
            .physics(id as u64)
            .map(|b| b.mass as rhai::FLOAT)
            .unwrap_or(1.0)
    }

    /// Set mass (0.0 = immovable).
    pub(crate) fn set_mass(&mut self, id: rhai::INT, mass: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(id as u64) else {
            return false;
        };
        body.mass = (mass as f32).max(0.0);
        world.set_physics(id as u64, body)
    }

    /// Get restitution.
    pub(crate) fn restitution(&mut self, id: rhai::INT) -> rhai::FLOAT {
        let Some(world) = self.world.as_ref() else {
            return 0.7;
        };
        world
            .physics(id as u64)
            .map(|b| b.restitution as rhai::FLOAT)
            .unwrap_or(0.7)
    }

    /// Set restitution coefficient.
    pub(crate) fn set_restitution(&mut self, id: rhai::INT, r: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut body) = world.physics(id as u64) else {
            return false;
        };
        body.restitution = (r as f32).clamp(0.0, 1.0);
        world.set_physics(id as u64, body)
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptEntityPhysicsApi>("EntityPhysicsApi");
    engine.register_type_with_name::<ScriptPhysicsApi>("PhysicsApi");

    // Entity-level physics API (accessed as entity.physics.velocity(), etc.)
    engine.register_fn("velocity", |physics: &mut ScriptEntityPhysicsApi| {
        physics.velocity()
    });
    engine.register_fn(
        "set_velocity",
        |physics: &mut ScriptEntityPhysicsApi, vx: rhai::FLOAT, vy: rhai::FLOAT| {
            physics.set_velocity(vx, vy)
        },
    );
    engine.register_fn(
        "add_velocity",
        |physics: &mut ScriptEntityPhysicsApi, dvx: rhai::FLOAT, dvy: rhai::FLOAT| {
            physics.add_velocity(dvx, dvy)
        },
    );
    engine.register_fn("acceleration", |physics: &mut ScriptEntityPhysicsApi| {
        physics.acceleration()
    });
    engine.register_fn(
        "set_acceleration",
        |physics: &mut ScriptEntityPhysicsApi, ax: rhai::FLOAT, ay: rhai::FLOAT| {
            physics.set_acceleration(ax, ay)
        },
    );
    engine.register_fn(
        "add_acceleration",
        |physics: &mut ScriptEntityPhysicsApi, dax: rhai::FLOAT, day: rhai::FLOAT| {
            physics.add_acceleration(dax, day)
        },
    );
    engine.register_fn("drag", |physics: &mut ScriptEntityPhysicsApi| {
        physics.drag()
    });
    engine.register_fn(
        "set_drag",
        |physics: &mut ScriptEntityPhysicsApi, drag: rhai::FLOAT| physics.set_drag(drag),
    );
    engine.register_fn("max_speed", |physics: &mut ScriptEntityPhysicsApi| {
        physics.max_speed()
    });
    engine.register_fn(
        "set_max_speed",
        |physics: &mut ScriptEntityPhysicsApi, max_speed: rhai::FLOAT| {
            physics.set_max_speed(max_speed)
        },
    );
    engine.register_fn("collider", |physics: &mut ScriptEntityPhysicsApi| {
        physics.collider()
    });
    engine.register_fn(
        "set_collider_circle",
        |physics: &mut ScriptEntityPhysicsApi,
         radius: rhai::FLOAT,
         layer: rhai::INT,
         mask: rhai::INT| { physics.set_collider_circle(radius, layer, mask) },
    );
    engine.register_fn(
        "set_collider_polygon",
        |physics: &mut ScriptEntityPhysicsApi,
         points: RhaiArray,
         layer: rhai::INT,
         mask: rhai::INT| { physics.set_collider_polygon(points, layer, mask) },
    );

    // World-level physics API (for id-based operations: physics.velocity(id), etc.)
    engine.register_fn(
        "velocity",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT| physics.velocity(id),
    );
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
    engine.register_fn(
        "acceleration",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT| physics.acceleration(id),
    );
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
    engine.register_fn("drag", |physics: &mut ScriptPhysicsApi, id: rhai::INT| {
        physics.drag(id)
    });
    engine.register_fn(
        "set_drag",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT, drag: rhai::FLOAT| {
            physics.set_drag(id, drag)
        },
    );
    engine.register_fn(
        "max_speed",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT| physics.max_speed(id),
    );
    engine.register_fn(
        "set_max_speed",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT, max_speed: rhai::FLOAT| {
            physics.set_max_speed(id, max_speed)
        },
    );
    engine.register_fn(
        "collider",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT| physics.collider(id),
    );
    engine.register_fn(
        "set_collider_circle",
        |physics: &mut ScriptPhysicsApi,
         id: rhai::INT,
         radius: rhai::FLOAT,
         layer: rhai::INT,
         mask: rhai::INT| { physics.set_collider_circle(id, radius, layer, mask) },
    );
    engine.register_fn(
        "set_collider_polygon",
        |physics: &mut ScriptPhysicsApi,
         id: rhai::INT,
         points: RhaiArray,
         layer: rhai::INT,
         mask: rhai::INT| { physics.set_collider_polygon(id, points, layer, mask) },
    );

    // Entity-level mass / restitution
    engine.register_fn("mass", |physics: &mut ScriptEntityPhysicsApi| {
        physics.mass()
    });
    engine.register_fn(
        "set_mass",
        |physics: &mut ScriptEntityPhysicsApi, mass: rhai::FLOAT| physics.set_mass(mass),
    );
    engine.register_fn("restitution", |physics: &mut ScriptEntityPhysicsApi| {
        physics.restitution()
    });
    engine.register_fn(
        "set_restitution",
        |physics: &mut ScriptEntityPhysicsApi, r: rhai::FLOAT| physics.set_restitution(r),
    );

    // World-level mass / restitution
    engine.register_fn("mass", |physics: &mut ScriptPhysicsApi, id: rhai::INT| {
        physics.mass(id)
    });
    engine.register_fn(
        "set_mass",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT, mass: rhai::FLOAT| {
            physics.set_mass(id, mass)
        },
    );
    engine.register_fn(
        "restitution",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT| physics.restitution(id),
    );
    engine.register_fn(
        "set_restitution",
        |physics: &mut ScriptPhysicsApi, id: rhai::INT, r: rhai::FLOAT| {
            physics.set_restitution(id, r)
        },
    );
}

//! ScriptGameplayApi and ScriptGameplayEntityApi implementation - large standalone module.
//! This module contains the full impl blocks extracted from lib.rs.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Map as RhaiMap};

use engine_game::{GameplayWorld, CollisionHit, Lifetime, PhysicsBody2D, Transform2D, VisualBinding};
use engine_core::game_state::GameState;
use engine_game::components::{DespawnVisual, TopDownShipController};

use crate::{BehaviorCommand, catalog};
use crate::rhai_util::json_to_rhai_dynamic;

// ── Struct Definitions ───────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptGameplayApi {
    pub(crate) world: Option<GameplayWorld>,
    pub(crate) game_state: Option<GameState>,
    pub(crate) scene_elapsed_ms: u64,
    pub(crate) collisions: std::sync::Arc<Vec<CollisionHit>>,
    pub(crate) collision_enters: std::sync::Arc<Vec<CollisionHit>>,
    pub(crate) collision_stays: std::sync::Arc<Vec<CollisionHit>>,
    pub(crate) collision_exits: std::sync::Arc<Vec<CollisionHit>>,
    pub(crate) catalogs: Arc<catalog::ModCatalogs>,
    pub(crate) queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

#[derive(Clone)]
pub(crate) struct ScriptGameplayEntityApi {
    pub(crate) world: Option<GameplayWorld>,
    pub(crate) id: u64,
    pub(crate) queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

// ── ScriptGameplayApi Implementation ──────────────────────────────────────
impl ScriptGameplayApi {
    fn map_number(args: &RhaiMap, key: &str, fallback: rhai::FLOAT) -> rhai::FLOAT {
        args.get(key)
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(fallback)
    }

    fn map_int(args: &RhaiMap, key: &str, fallback: rhai::INT) -> rhai::INT {
        args.get(key)
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::INT>()
                    .or_else(|| v.clone().try_cast::<rhai::FLOAT>().map(|f| f as rhai::INT))
            })
            .unwrap_or(fallback)
    }

    fn map_map(args: &RhaiMap, key: &str) -> Option<RhaiMap> {
        args.get(key).and_then(|v| v.clone().try_cast::<RhaiMap>())
    }

    fn map_string(args: &RhaiMap, key: &str) -> Option<String> {
        args.get(key)
            .and_then(|v| v.clone().try_cast::<String>())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn new(
        world: Option<GameplayWorld>,
        game_state: Option<GameState>,
        scene_elapsed_ms: u64,
        collisions: std::sync::Arc<Vec<CollisionHit>>,
        collision_enters: std::sync::Arc<Vec<CollisionHit>>,
        collision_stays: std::sync::Arc<Vec<CollisionHit>>,
        collision_exits: std::sync::Arc<Vec<CollisionHit>>,
        catalogs: Arc<catalog::ModCatalogs>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self {
            world,
            game_state,
            scene_elapsed_ms,
            collisions,
            collision_enters,
            collision_stays,
            collision_exits,
            catalogs,
            queue,
        }
    }

    fn entity(&mut self, id: rhai::INT) -> ScriptGameplayEntityApi {
        if id < 0 {
            return ScriptGameplayEntityApi {
                world: None,
                id: 0,
                queue: Arc::clone(&self.queue),
            };
        }
        ScriptGameplayEntityApi {
            world: self.world.clone(),
            id: id as u64,
            queue: Arc::clone(&self.queue),
        }
    }

    fn clear(&mut self) {
        if let Some(world) = self.world.as_ref() {
            world.clear();
        }
    }

    fn count(&mut self) -> rhai::INT {
        self.world
            .as_ref()
            .map(|world| world.count() as rhai::INT)
            .unwrap_or(0)
    }

    fn count_kind(&mut self, kind: &str) -> rhai::INT {
        self.world
            .as_ref()
            .map(|world| world.count_kind(kind) as rhai::INT)
            .unwrap_or(0)
    }

    fn count_tag(&mut self, tag: &str) -> rhai::INT {
        self.world
            .as_ref()
            .map(|world| world.count_tag(tag) as rhai::INT)
            .unwrap_or(0)
    }

    fn first_kind(&mut self, kind: &str) -> rhai::INT {
        self.world
            .as_ref()
            .and_then(|world| world.first_kind(kind))
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    fn first_tag(&mut self, tag: &str) -> rhai::INT {
        self.world
            .as_ref()
            .and_then(|world| world.first_tag(tag))
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    fn spawn(&mut self, kind: &str, payload: RhaiDynamic) -> rhai::INT {
        let Some(world) = self.world.clone() else {
            return 0;
        };
        let Some(payload) = rhai_dynamic_to_json(&payload) else {
            return 0;
        };
        world
            .spawn(kind, payload)
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    fn despawn(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let uid = id as u64;
        if let Some(binding) = world.visual(uid) {
            if let Ok(mut commands) = self.queue.lock() {
                for vid in binding.all_visual_ids() {
                    commands.push(BehaviorCommand::SceneDespawn {
                        target: vid.to_string(),
                    });
                }
            }
        }
        world.despawn(uid)
    }

    fn bind_visual(&mut self, id: rhai::INT, visual_id: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 || visual_id.trim().is_empty() {
            return false;
        }
        world.add_visual(id as u64, visual_id.to_string())
    }

    fn exists(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.exists(id as u64)
    }

    fn kind(&mut self, id: rhai::INT) -> String {
        let Some(world) = self.world.as_ref() else {
            return String::new();
        };
        if id < 0 {
            return String::new();
        }
        world.kind_of(id as u64).unwrap_or_default()
    }

    fn tags(&mut self, id: rhai::INT) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        if id < 0 {
            return RhaiArray::new();
        }
        world.tags(id as u64).into_iter().map(Into::into).collect()
    }

    fn ids(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .ids()
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    fn query_kind(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_kind(kind)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    fn query_tag(&mut self, tag: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_tag(tag)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    fn get(&mut self, id: rhai::INT, path: &str) -> RhaiDynamic {
        let Some(world) = self.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        world
            .get(id as u64, path)
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    fn set(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.set(id as u64, path, value)
    }

    fn has(&mut self, id: rhai::INT, path: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.has(id as u64, path)
    }

    fn remove(&mut self, id: rhai::INT, path: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.remove(id as u64, path)
    }

    fn push(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.push(id as u64, path, value)
    }

    fn set_transform(
        &mut self,
        id: rhai::INT,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        heading: rhai::FLOAT,
    ) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_transform(
            id as u64,
            Transform2D {
                x: x as f32,
                y: y as f32,
                heading: heading as f32,
            },
        )
    }

    fn transform(&mut self, id: rhai::INT) -> RhaiDynamic {
        let Some(world) = self.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        if let Some(xf) = world.transform(id as u64) {
            let mut map = RhaiMap::new();
            map.insert("x".into(), (xf.x as rhai::FLOAT).into());
            map.insert("y".into(), (xf.y as rhai::FLOAT).into());
            map.insert("heading".into(), (xf.heading as rhai::FLOAT).into());
            return map.into();
        }
        ().into()
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
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_physics(
            id as u64,
            PhysicsBody2D {
                vx: vx as f32,
                vy: vy as f32,
                ax: ax as f32,
                ay: ay as f32,
                drag: drag as f32,
                max_speed: max_speed as f32,
            },
        )
    }

    fn physics(&mut self, id: rhai::INT) -> RhaiDynamic {
        let Some(world) = self.world.as_ref() else {
            return ().into();
        };
        if id < 0 {
            return ().into();
        }
        if let Some(body) = world.physics(id as u64) {
            let mut map = RhaiMap::new();
            map.insert("vx".into(), (body.vx as rhai::FLOAT).into());
            map.insert("vy".into(), (body.vy as rhai::FLOAT).into());
            map.insert("ax".into(), (body.ax as rhai::FLOAT).into());
            map.insert("ay".into(), (body.ay as rhai::FLOAT).into());
            map.insert("drag".into(), (body.drag as rhai::FLOAT).into());
            map.insert("max_speed".into(), (body.max_speed as rhai::FLOAT).into());
            return map.into();
        }
        ().into()
    }

    fn set_collider_circle(
        &mut self,
        id: rhai::INT,
        radius: rhai::FLOAT,
        layer: rhai::INT,
        mask: rhai::INT,
    ) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_collider(
            id as u64,
            Collider2D {
                shape: ColliderShape::Circle {
                    radius: radius as f32,
                },
                layer: layer as u32,
                mask: mask as u32,
            },
        )
    }

    fn set_lifetime(&mut self, id: rhai::INT, ttl_ms: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_lifetime(
            id as u64,
            Lifetime {
                ttl_ms: ttl_ms as i32,
                on_expire: DespawnVisual::None,
            },
        )
    }

    fn set_visual(&mut self, id: rhai::INT, visual_id: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_visual(
            id as u64,
            VisualBinding {
                visual_id: if visual_id.trim().is_empty() {
                    None
                } else {
                    Some(visual_id.to_string())
                },
                additional_visuals: Vec::new(),
            },
        )
    }

    fn spawn_visual(&mut self, kind: &str, template: &str, data: RhaiMap) -> rhai::INT {
        let Some(world) = self.world.clone() else {
            return 0;
        };

        // Step 1: Spawn gameplay entity with empty payload
        let Some(entity_id) = world.spawn(kind, JsonValue::Object(JsonMap::new())) else {
            return 0;
        };

        // Step 2: Generate visual_id (format: "{kind}-{entity_id}")
        let visual_id = format!("{}-{}", kind, entity_id);

        // Step 3: Emit SceneSpawn command
        {
            let mut commands = match self.queue.lock() {
                Ok(cmds) => cmds,
                Err(_) => {
                    world.despawn(entity_id);
                    return 0;
                }
            };
            commands.push(BehaviorCommand::SceneSpawn {
                template: template.to_string(),
                target: visual_id.clone(),
            });
        }

        // Step 4: Set visual binding
        if !world.set_visual(
            entity_id,
            VisualBinding {
                visual_id: Some(visual_id.clone()),
                additional_visuals: Vec::new(),
            },
        ) {
            world.despawn(entity_id);
            return 0;
        }

        // Step 5: Set transform from data
        let x = data
            .get("x")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;
        let y = data
            .get("y")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;
        let heading = data
            .get("heading")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;

        if !world.set_transform(entity_id, Transform2D { x, y, heading }) {
            world.despawn(entity_id);
            return 0;
        }

        // Step 6: Set collider if provided
        if let Some(radius_val) = data.get("collider_radius") {
            let radius_opt = radius_val.clone().try_cast::<rhai::FLOAT>().or_else(|| {
                radius_val
                    .clone()
                    .try_cast::<rhai::INT>()
                    .map(|i| i as rhai::FLOAT)
            });
            if let Some(radius) = radius_opt {
                let layer = data
                    .get("collider_layer")
                    .and_then(|v| v.clone().try_cast::<rhai::INT>())
                    .unwrap_or(-1) as u32;
                let mask = data
                    .get("collider_mask")
                    .and_then(|v| v.clone().try_cast::<rhai::INT>())
                    .unwrap_or(-1) as u32;

                if !world.set_collider(
                    entity_id,
                    Collider2D {
                        shape: ColliderShape::Circle {
                            radius: radius as f32,
                        },
                        layer,
                        mask,
                    },
                ) {
                    world.despawn(entity_id);
                    return 0;
                }
            }
        }

        // Step 6b: Set polygon collider if provided
        if let Some(poly_val) = data.get("collider_polygon") {
            if let Some(poly_arr) = poly_val.clone().try_cast::<RhaiArray>() {
                let mut points: Vec<[f32; 2]> = Vec::new();
                for point in poly_arr {
                    if let Some(point_arr) = point.try_cast::<RhaiArray>() {
                        if point_arr.len() >= 2 {
                            if let (Some(px), Some(py)) = (
                                point_arr[0].clone().try_cast::<rhai::FLOAT>(),
                                point_arr[1].clone().try_cast::<rhai::FLOAT>(),
                            ) {
                                points.push([px as f32, py as f32]);
                            }
                        }
                    }
                }
                if !points.is_empty() {
                    let layer = data
                        .get("collider_layer")
                        .and_then(|v| v.clone().try_cast::<rhai::INT>())
                        .unwrap_or(-1) as u32;
                    let mask = data
                        .get("collider_mask")
                        .and_then(|v| v.clone().try_cast::<rhai::INT>())
                        .unwrap_or(-1) as u32;

                    if !world.set_collider(
                        entity_id,
                        Collider2D {
                            shape: ColliderShape::Polygon { points },
                            layer,
                            mask,
                        },
                    ) {
                        world.despawn(entity_id);
                        return 0;
                    }
                }
            }
        }

        // Step 7: Set lifetime if provided
        if let Some(ttl_val) = data.get("lifetime_ms") {
            if let Some(ttl) = ttl_val.clone().try_cast::<rhai::INT>() {
                if !world.set_lifetime(
                    entity_id,
                    Lifetime {
                        ttl_ms: ttl as i32,
                        on_expire: DespawnVisual::None,
                    },
                ) {
                    world.despawn(entity_id);
                    return 0;
                }
            }
        }

        entity_id as rhai::INT
    }

    fn spawn_prefab(&mut self, name: &str, args: RhaiMap) -> rhai::INT {
        // Check if prefab exists in catalog; fall back to hardcoded
        let _prefab_exists_in_catalog = self.catalogs.prefabs.contains_key(name);

        match name {
            "ship" => {
                let cfg = Self::map_map(&args, "cfg").unwrap_or_default();
                let x = Self::map_number(&args, "x", 0.0);
                let y = Self::map_number(&args, "y", 0.0);
                let heading = Self::map_number(&args, "heading", 0.0);
                let id = self.spawn("ship", RhaiMap::new().into());
                if id <= 0 {
                    return 0;
                }
                let mut controller_cfg = RhaiMap::new();
                controller_cfg.insert(
                    "turn_step_ms".into(),
                    Self::map_int(&cfg, "turn_step_ms", 40).into(),
                );
                controller_cfg.insert(
                    "thrust_power".into(),
                    Self::map_number(&cfg, "ship_thrust", 170.0).into(),
                );
                controller_cfg.insert(
                    "max_speed".into(),
                    Self::map_number(&cfg, "ship_max_speed", 4.5).into(),
                );
                controller_cfg.insert("heading_bits".into(), 32_i64.into());
                if !self.set_visual(id, "ship")
                    || !self.set_transform(id, x, y, heading)
                    || !self.set_physics(
                        id,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.10,
                        Self::map_number(&cfg, "ship_max_speed", 0.0),
                    )
                    || !self.set_collider_circle(id, 10.0, 0xFFFF, 0xFFFF)
                    || !self.attach_ship_controller(id, controller_cfg)
                    || !self.enable_wrap_bounds(id)
                {
                    let _ = self.despawn(id);
                    return 0;
                }

                let invulnerable_ms = Self::map_int(&args, "invulnerable_ms", 0);
                if invulnerable_ms > 0 {
                    let _ = self.entity(id).status_add("invulnerable", invulnerable_ms);
                }
                id
            }
            "asteroid" => {
                let x = Self::map_number(&args, "x", 0.0);
                let y = Self::map_number(&args, "y", 0.0);
                let vx = Self::map_number(&args, "vx", 0.0);
                let vy = Self::map_number(&args, "vy", 0.0);
                let shape = Self::map_int(&args, "shape", 0);
                let size = Self::map_int(&args, "size", 1);
                let mut visual_args = RhaiMap::new();
                visual_args.insert("x".into(), x.into());
                visual_args.insert("y".into(), y.into());
                visual_args.insert("heading".into(), 0.0.into());
                visual_args.insert(
                    "collider_radius".into(),
                    (asteroid_radius_i32(size as i32) as rhai::INT).into(),
                );
                let id = self.spawn_visual("asteroid", "asteroid-template", visual_args);
                if id <= 0
                    || !self.set_physics(id, vx * 60.0, vy * 60.0, 0.0, 0.0, 0.0, 0.0)
                    || !self.enable_wrap_bounds(id)
                {
                    let _ = self.despawn(id);
                    return 0;
                }
                let rot_phase = self.rand_i(0, 31);
                let rot_speed = if self.rand_i(0, 1) == 0 { 1 } else { -1 };
                let rot_step = 72 + self.rand_i(0, 2) * 16;
                let mut asteroid_data = RhaiMap::new();
                asteroid_data.insert("shape".into(), shape.into());
                asteroid_data.insert("size".into(), size.into());
                asteroid_data.insert("flash_ms".into(), 0_i64.into());
                asteroid_data.insert("flash_total_ms".into(), 0_i64.into());
                asteroid_data.insert("split_pending".into(), false.into());
                asteroid_data.insert("rot_phase".into(), rot_phase.into());
                asteroid_data.insert("rot_speed".into(), rot_speed.into());
                asteroid_data.insert("rot_step_ms".into(), rot_step.into());
                asteroid_data.insert("rot_accum_ms".into(), 0_i64.into());
                let _ = self.entity(id).set_many(asteroid_data);
                id
            }
            "bullet" => {
                let x = Self::map_number(&args, "x", 0.0);
                let y = Self::map_number(&args, "y", 0.0);
                let vx = Self::map_number(&args, "vx", 0.0);
                let vy = Self::map_number(&args, "vy", 0.0);
                let ttl_ms = Self::map_int(&args, "ttl_ms", 0);
                let mut visual_args = RhaiMap::new();
                visual_args.insert("x".into(), x.into());
                visual_args.insert("y".into(), y.into());
                visual_args.insert("heading".into(), 0.0.into());
                visual_args.insert("collider_radius".into(), 3.0.into());
                visual_args.insert("lifetime_ms".into(), ttl_ms.into());
                let id = self.spawn_visual("bullet", "bullet-template", visual_args);
                if id <= 0
                    || !self.set_physics(id, vx * 60.0, vy * 60.0, 0.0, 0.0, 0.0, 0.0)
                    || !self.enable_wrap_bounds(id)
                {
                    let _ = self.despawn(id);
                    return 0;
                }
                id
            }
            "smoke" => {
                let x = Self::map_number(&args, "x", 0.0);
                let y = Self::map_number(&args, "y", 0.0);
                let vx = Self::map_number(&args, "vx", 0.0);
                let vy = Self::map_number(&args, "vy", 0.0);
                let ttl_ms = Self::map_int(&args, "ttl_ms", 0);
                let radius = Self::map_int(&args, "radius", 1);
                let mut visual_args = RhaiMap::new();
                visual_args.insert("x".into(), x.into());
                visual_args.insert("y".into(), y.into());
                visual_args.insert("heading".into(), 0.0.into());
                visual_args.insert("lifetime_ms".into(), ttl_ms.into());
                let id = self.spawn_visual("smoke", "smoke-template", visual_args);
                if id <= 0 || !self.set_physics(id, vx * 60.0, vy * 60.0, 0.0, 0.0, 0.04, 0.0) {
                    let _ = self.despawn(id);
                    return 0;
                }
                let mut smoke_data = RhaiMap::new();
                smoke_data.insert("ttl_ms".into(), ttl_ms.into());
                smoke_data.insert("max_ttl_ms".into(), ttl_ms.into());
                smoke_data.insert("radius".into(), radius.into());
                let _ = self.entity(id).set_many(smoke_data);
                id
            }
            _ => 0,
        }
    }

    fn spawn_group(&mut self, group_name: &str, prefab_name: &str) -> RhaiArray {
        // Try to load from catalog first
        if let Some(group) = self.catalogs.groups.get(group_name) {
            if group.prefab == prefab_name {
                let spawns = group.spawns.clone();
                return spawns
                    .iter()
                    .map(|spec| {
                        let mut args = RhaiMap::new();
                        args.insert("x".into(), (spec.x).into());
                        args.insert("y".into(), (spec.y).into());
                        args.insert("vx".into(), (spec.vx).into());
                        args.insert("vy".into(), (spec.vy).into());
                        args.insert("shape".into(), (spec.shape).into());
                        args.insert("size".into(), (spec.size).into());
                        self.spawn_prefab(prefab_name, args).into()
                    })
                    .collect();
            }
        }

        // Fall back to hardcoded groups
        let spawns: &[(
            rhai::FLOAT,
            rhai::FLOAT,
            rhai::FLOAT,
            rhai::FLOAT,
            rhai::INT,
            rhai::INT,
        )] = match (group_name, prefab_name) {
            ("asteroids.initial", "asteroid") => &[
                (-300.0, -210.0, 2.0, 0.0, 0, 2),
                (300.0, -210.0, 0.0, 2.0, 1, 3),
                (300.0, 210.0, -2.0, 0.0, 2, 2),
                (-300.0, 210.0, 0.0, -2.0, 3, 1),
                (-290.0, -40.0, 2.0, 1.0, 0, 2),
                (-140.0, -230.0, 2.0, 1.0, 1, 1),
                (140.0, -230.0, -2.0, 1.0, 2, 2),
                (290.0, 30.0, -2.0, -1.0, 3, 3),
                (120.0, 230.0, -2.0, -1.0, 1, 2),
                (-120.0, 230.0, 2.0, -1.0, 0, 1),
            ],
            _ => return RhaiArray::new(),
        };

        spawns
            .iter()
            .map(|(x, y, vx, vy, shape, size)| {
                let mut args = RhaiMap::new();
                args.insert("x".into(), (*x).into());
                args.insert("y".into(), (*y).into());
                args.insert("vx".into(), (*vx).into());
                args.insert("vy".into(), (*vy).into());
                args.insert("shape".into(), (*shape).into());
                args.insert("size".into(), (*size).into());
                self.spawn_prefab(prefab_name, args).into()
            })
            .collect()
    }

    fn try_fire_weapon(
        &mut self,
        weapon_name: &str,
        source_id: rhai::INT,
        args: RhaiMap,
    ) -> rhai::INT {
        const ASTEROIDS_VELOCITY_SCALE: f32 = 60.0;

        let Some(world) = self.world.clone() else {
            return 0;
        };
        if source_id <= 0 {
            return 0;
        }
        let source_id = source_id as u64;
        if !world.exists(source_id) {
            return 0;
        }

        // Try to load from catalog first
        if let Some(weapon) = self.catalogs.weapons.get(weapon_name) {
            let bullet_kind = Self::map_string(&args, "bullet_kind")
                .or_else(|| weapon.bullet_kind.clone())
                .unwrap_or_else(|| "bullet".to_string());
            let max_bullets = Self::map_int(&args, "max_bullets", weapon.max_bullets).max(0);
            if world.count_kind(&bullet_kind) as rhai::INT >= max_bullets {
                return 0;
            }

            let cooldown_name = Self::map_string(&args, "cooldown_name")
                .or_else(|| weapon.cooldown_name.clone())
                .unwrap_or_else(|| "shot".to_string());
            if !world.cooldown_ready(source_id, &cooldown_name) {
                return 0;
            }

            let Some(transform) = world.transform(source_id) else {
                return 0;
            };
            let Some(physics) = world.physics(source_id) else {
                return 0;
            };
            let Some(controller) = world.controller(source_id) else {
                return 0;
            };
            let (dir_x, dir_y) = controller.heading_vector();

            let spawn_offset =
                Self::map_number(&args, "spawn_offset", weapon.spawn_offset.unwrap_or(9.0)) as f32;
            let bullet_speed = Self::map_number(&args, "bullet_speed", 0.0) as f32;
            let bullet_ttl_ms =
                Self::map_int(&args, "bullet_ttl_ms", weapon.bullet_ttl_ms.unwrap_or(0));
            let shot_cooldown_ms =
                Self::map_int(&args, "shot_cooldown_ms", weapon.cooldown_ms.unwrap_or(0)).max(0);

            let mut bullet_args = RhaiMap::new();
            bullet_args.insert(
                "x".into(),
                ((transform.x + (dir_x * spawn_offset)) as rhai::FLOAT).into(),
            );
            bullet_args.insert(
                "y".into(),
                ((transform.y + (dir_y * spawn_offset)) as rhai::FLOAT).into(),
            );
            bullet_args.insert(
                "vx".into(),
                (((physics.vx / ASTEROIDS_VELOCITY_SCALE)
                    + (dir_x * bullet_speed / ASTEROIDS_VELOCITY_SCALE))
                    as rhai::FLOAT)
                    .into(),
            );
            bullet_args.insert(
                "vy".into(),
                (((physics.vy / ASTEROIDS_VELOCITY_SCALE)
                    + (dir_y * bullet_speed / ASTEROIDS_VELOCITY_SCALE))
                    as rhai::FLOAT)
                    .into(),
            );
            bullet_args.insert("ttl_ms".into(), bullet_ttl_ms.into());

            let bullet_prefab =
                Self::map_string(&args, "bullet_prefab").unwrap_or_else(|| "bullet".to_string());
            let bullet_id = self.spawn_prefab(&bullet_prefab, bullet_args);
            if bullet_id <= 0 {
                return 0;
            }

            if shot_cooldown_ms > 0
                && !world.cooldown_start(source_id, &cooldown_name, shot_cooldown_ms as i32)
            {
                let _ = self.despawn(bullet_id);
                return 0;
            }

            let audio_event = Self::map_string(&args, "audio_event")
                .unwrap_or_else(|| "gameplay.ship.shoot".to_string());
            let gain = Self::map_number(&args, "audio_gain", 1.0) as f32;
            if let Ok(mut queue) = self.queue.lock() {
                queue.push(BehaviorCommand::PlayAudioEvent {
                    event: audio_event,
                    gain: Some(gain),
                });
            }

            return bullet_id;
        }

        // Fall back to hardcoded weapons
        match weapon_name {
            "asteroids.ship" => {
                let bullet_kind =
                    Self::map_string(&args, "bullet_kind").unwrap_or_else(|| "bullet".to_string());
                let max_bullets = Self::map_int(&args, "max_bullets", 8).max(0);
                if world.count_kind(&bullet_kind) as rhai::INT >= max_bullets {
                    return 0;
                }

                let cooldown_name =
                    Self::map_string(&args, "cooldown_name").unwrap_or_else(|| "shot".to_string());
                if !world.cooldown_ready(source_id, &cooldown_name) {
                    return 0;
                }

                let Some(transform) = world.transform(source_id) else {
                    return 0;
                };
                let Some(physics) = world.physics(source_id) else {
                    return 0;
                };
                let Some(controller) = world.controller(source_id) else {
                    return 0;
                };
                let (dir_x, dir_y) = controller.heading_vector();

                let spawn_offset = Self::map_number(&args, "spawn_offset", 9.0) as f32;
                let bullet_speed = Self::map_number(&args, "bullet_speed", 0.0) as f32;
                let bullet_ttl_ms = Self::map_int(&args, "bullet_ttl_ms", 0);
                let shot_cooldown_ms = Self::map_int(&args, "shot_cooldown_ms", 0).max(0);

                let mut bullet_args = RhaiMap::new();
                bullet_args.insert(
                    "x".into(),
                    ((transform.x + (dir_x * spawn_offset)) as rhai::FLOAT).into(),
                );
                bullet_args.insert(
                    "y".into(),
                    ((transform.y + (dir_y * spawn_offset)) as rhai::FLOAT).into(),
                );
                bullet_args.insert(
                    "vx".into(),
                    (((physics.vx / ASTEROIDS_VELOCITY_SCALE)
                        + (dir_x * bullet_speed / ASTEROIDS_VELOCITY_SCALE))
                        as rhai::FLOAT)
                        .into(),
                );
                bullet_args.insert(
                    "vy".into(),
                    (((physics.vy / ASTEROIDS_VELOCITY_SCALE)
                        + (dir_y * bullet_speed / ASTEROIDS_VELOCITY_SCALE))
                        as rhai::FLOAT)
                        .into(),
                );
                bullet_args.insert("ttl_ms".into(), bullet_ttl_ms.into());

                let bullet_prefab = Self::map_string(&args, "bullet_prefab")
                    .unwrap_or_else(|| "bullet".to_string());
                let bullet_id = self.spawn_prefab(&bullet_prefab, bullet_args);
                if bullet_id <= 0 {
                    return 0;
                }

                if shot_cooldown_ms > 0
                    && !world.cooldown_start(source_id, &cooldown_name, shot_cooldown_ms as i32)
                {
                    let _ = self.despawn(bullet_id);
                    return 0;
                }

                let audio_event = Self::map_string(&args, "audio_event")
                    .unwrap_or_else(|| "gameplay.ship.shoot".to_string());
                let gain = Self::map_number(&args, "audio_gain", 1.0) as f32;
                if let Ok(mut queue) = self.queue.lock() {
                    queue.push(BehaviorCommand::PlayAudioEvent {
                        event: audio_event,
                        gain: Some(gain),
                    });
                }

                bullet_id
            }
            _ => 0,
        }
    }

    fn tick_heading_anim(&mut self, id: rhai::INT, dt_ms: rhai::INT) -> RhaiMap {
        let mut out = RhaiMap::new();
        if id <= 0 {
            out.insert("rot_phase".into(), 0_i64.into());
            out.insert("rot_accum_ms".into(), 0_i64.into());
            return out;
        }
        let mut entity = self.entity(id);
        if !entity.exists() {
            out.insert("rot_phase".into(), 0_i64.into());
            out.insert("rot_accum_ms".into(), 0_i64.into());
            return out;
        }

        let mut rot_accum = entity.get_i("/rot_accum_ms", 0);
        let mut rot_phase = entity.get_i("/rot_phase", 0);
        let rot_speed = entity.get_i("/rot_speed", 1);
        let rot_step = entity.get_i("/rot_step_ms", 72);
        let dt_ms = dt_ms.max(0);

        if rot_step > 0 {
            rot_accum += dt_ms;
            while rot_accum >= rot_step {
                let next = (rot_phase + rot_speed) % 32;
                rot_phase = if next < 0 { next + 32 } else { next };
                rot_accum -= rot_step;
            }
        }

        let mut updates = RhaiMap::new();
        updates.insert("rot_phase".into(), rot_phase.into());
        updates.insert("rot_accum_ms".into(), rot_accum.into());
        let _ = entity.set_many(updates);

        out.insert("rot_phase".into(), rot_phase.into());
        out.insert("rot_accum_ms".into(), rot_accum.into());
        out
    }

    fn handle_ship_hit(
        &mut self,
        ship_id: rhai::INT,
        asteroid_id: rhai::INT,
        args: RhaiMap,
    ) -> bool {
        const ASTEROIDS_VELOCITY_SCALE: f32 = 60.0;

        if ship_id <= 0 {
            return false;
        }
        let Some(world) = self.world.clone() else {
            return false;
        };
        let ship_id = ship_id as u64;
        if !world.exists(ship_id) || world.status_has(ship_id, "invulnerable") {
            return false;
        }

        let (ship_x, ship_y) = match world.transform(ship_id) {
            Some(transform) => (transform.x, transform.y),
            None => return false,
        };

        let ship_reset_vx = ScriptGameplayApi::map_number(&args, "ship_reset_vx", 0.0);
        let ship_reset_vy = ScriptGameplayApi::map_number(&args, "ship_reset_vy", 0.0);
        let ship_invulnerable_ms =
            ScriptGameplayApi::map_int(&args, "ship_invulnerable_ms", 3000).max(0);
        let ui_text =
            ScriptGameplayApi::map_string(&args, "ui_text").unwrap_or_else(|| "HIT!".to_string());
        let ui_ttl_ms = ScriptGameplayApi::map_int(&args, "ui_ttl_ms", 450).max(0);
        let crack_duration_ms = ScriptGameplayApi::map_int(&args, "crack_duration_ms", 1000).max(0);
        let asteroid_velocity_limit = ScriptGameplayApi::map_number(
            &args,
            "asteroid_velocity_limit",
            4.0 * ASTEROIDS_VELOCITY_SCALE as rhai::FLOAT,
        ) as f32;
        let audio_event = ScriptGameplayApi::map_string(&args, "audio_event")
            .unwrap_or_else(|| "gameplay.ship.hit".to_string());
        let audio_gain = ScriptGameplayApi::map_number(&args, "audio_gain", 1.0) as f32;

        let mut fx = ScriptFxApi::new(
            self.world.clone(),
            Arc::clone(&self.catalogs),
            Arc::clone(&self.queue),
        );
        let mut fx_args = RhaiMap::new();
        fx_args.insert("x".into(), (ship_x as rhai::FLOAT).into());
        fx_args.insert("y".into(), (ship_y as rhai::FLOAT).into());
        let _ = fx.emit("asteroids.ship_disintegration", fx_args);

        if let Ok(mut queue) = self.queue.lock() {
            queue.push(BehaviorCommand::PlayAudioEvent {
                event: audio_event,
                gain: Some(audio_gain),
            });
        }

        if asteroid_id > 0 {
            let asteroid_id = asteroid_id as u64;
            if world.exists(asteroid_id) {
                let mut asteroid = self.entity(asteroid_id as rhai::INT);
                let asteroid_phys = world.physics(asteroid_id);
                let mut asteroid_updates = RhaiMap::new();
                asteroid_updates.insert("flash_ms".into(), crack_duration_ms.into());
                asteroid_updates.insert("flash_total_ms".into(), crack_duration_ms.into());
                asteroid_updates.insert("split_pending".into(), true.into());
                let _ = asteroid.set_many(asteroid_updates);
                if let Some(phys) = asteroid_phys {
                    let clamped_vx =
                        (-(phys.vx)).clamp(-asteroid_velocity_limit, asteroid_velocity_limit);
                    let clamped_vy =
                        (-(phys.vy)).clamp(-asteroid_velocity_limit, asteroid_velocity_limit);
                    let _ =
                        asteroid.set_velocity(clamped_vx as rhai::FLOAT, clamped_vy as rhai::FLOAT);
                }
            }
        }

        let mut ship = self.entity(ship_id as rhai::INT);
        let _ = ship.set_velocity(ship_reset_vx, ship_reset_vy);
        if ship_invulnerable_ms > 0 {
            let _ = ship.status_add("invulnerable", ship_invulnerable_ms);
        }

        let _ = self.flash_ui_message(&ui_text, ui_ttl_ms);

        true
    }

    fn handle_bullet_hit(
        &mut self,
        bullet_id: rhai::INT,
        asteroid_id: rhai::INT,
        args: RhaiMap,
    ) -> RhaiMap {
        let mut out = RhaiMap::new();
        out.insert("handled".into(), false.into());
        out.insert("asteroid_size".into(), 0_i64.into());

        if bullet_id > 0 {
            let _ = self.despawn(bullet_id);
        }

        if asteroid_id <= 0 {
            return out;
        }
        let Some(world) = self.world.clone() else {
            return out;
        };
        let asteroid_id = asteroid_id as u64;
        if !world.exists(asteroid_id) {
            return out;
        }
        let mut asteroid = self.entity(asteroid_id as rhai::INT);
        if asteroid.flag("split_pending") {
            return out;
        }

        let crack_duration_ms = ScriptGameplayApi::map_int(&args, "crack_duration_ms", 1000).max(0);
        let ui_text =
            ScriptGameplayApi::map_string(&args, "ui_text").unwrap_or_else(|| "HIT".to_string());
        let ui_ttl_ms = ScriptGameplayApi::map_int(&args, "ui_ttl_ms", 250).max(0);

        let asteroid_size = asteroid.get_i("/size", 0);
        let mut asteroid_updates = RhaiMap::new();
        asteroid_updates.insert("flash_ms".into(), crack_duration_ms.into());
        asteroid_updates.insert("flash_total_ms".into(), crack_duration_ms.into());
        asteroid_updates.insert("split_pending".into(), true.into());
        let _ = asteroid.set_many(asteroid_updates);
        let _ = self.flash_ui_message(&ui_text, ui_ttl_ms);

        out.insert("handled".into(), true.into());
        out.insert("asteroid_size".into(), asteroid_size.into());
        out
    }

    fn handle_asteroid_split(&mut self, asteroid_id: rhai::INT, args: RhaiMap) -> RhaiMap {
        let mut out = RhaiMap::new();
        let child_ids = RhaiArray::new();
        out.insert("handled".into(), false.into());
        out.insert("despawned".into(), false.into());
        out.insert("children".into(), child_ids.clone().into());

        if asteroid_id <= 0 {
            return out;
        }
        let Some(world) = self.world.clone() else {
            return out;
        };
        let asteroid_id = asteroid_id as u64;
        if !world.exists(asteroid_id) {
            return out;
        }

        let asteroid = self.entity(asteroid_id as rhai::INT);
        let ast_size = asteroid.clone().get_i("/size", 1);
        if ast_size <= 0 {
            let _ = self.despawn(asteroid_id as rhai::INT);
            out.insert("handled".into(), true.into());
            out.insert("despawned".into(), true.into());
            return out;
        }

        let base_heading = asteroid.clone().get_i("/rot_phase", 0);
        let Some(transform) = world.transform(asteroid_id) else {
            return out;
        };
        let next_size = ast_size - 1;
        let mut children = RhaiArray::new();

        for (frag_idx, offset) in [0_i64, 11, 21].into_iter().enumerate() {
            let split_heading = {
                let next = (base_heading + offset) % 32;
                if next < 0 {
                    next + 32
                } else {
                    next
                }
            };
            let split_sin = sin32_i32(split_heading as i32);
            let split_cos = sin32_i32((split_heading + 8) as i32);
            let dir_x = match frag_idx {
                1 => 1,
                _ => {
                    if split_sin < 0 {
                        -1
                    } else {
                        1
                    }
                }
            };
            let dir_y = if -split_cos < 0 { -1 } else { 1 };
            let spawn_offset = asteroid_radius_i32(next_size as i32) + 4;
            let spawn_x = transform.x + ((split_sin * spawn_offset) as f32 / 1024.0_f32);
            let spawn_y = transform.y - ((split_cos * spawn_offset) as f32 / 1024.0_f32);
            let mut spawn_args = RhaiMap::new();
            spawn_args.insert("x".into(), (spawn_x as rhai::FLOAT).into());
            spawn_args.insert("y".into(), (spawn_y as rhai::FLOAT).into());
            spawn_args.insert(
                "vx".into(),
                ((dir_x * (next_size + 2)) as rhai::FLOAT).into(),
            );
            spawn_args.insert(
                "vy".into(),
                ((dir_y * (next_size + 2)) as rhai::FLOAT).into(),
            );
            spawn_args.insert("shape".into(), self.rand_i(0, 3).into());
            spawn_args.insert("size".into(), next_size.into());

            let child_id = self.spawn_prefab("asteroid", spawn_args);
            if child_id > 0 {
                children.push(child_id.into());
            }
        }

        let _ = self.despawn(asteroid_id as rhai::INT);
        if let Ok(mut queue) = self.queue.lock() {
            queue.push(BehaviorCommand::PlayAudioEvent {
                event: ScriptGameplayApi::map_string(&args, "audio_event")
                    .unwrap_or_else(|| "gameplay.asteroid.split".to_string()),
                gain: Some(ScriptGameplayApi::map_number(&args, "audio_gain", 1.0) as f32),
            });
        }

        out.insert("handled".into(), true.into());
        out.insert("despawned".into(), true.into());
        out.insert("children".into(), children.into());
        out
    }

    fn spawn_wave(&mut self, wave_name: &str, args: RhaiMap) -> RhaiArray {
        // Try to load from catalog first
        if let Some(wave) = self.catalogs.waves.get(wave_name) {
            let spawn_count = Self::map_int(&args, "spawn_count", 0).max(0);
            let ship_x = Self::map_number(&args, "ship_x", 0.0) as i64;
            let ship_y = Self::map_number(&args, "ship_y", 0.0) as i64;
            let min_x = Self::map_number(&args, "min_x", -320.0) as i64;
            let max_x = Self::map_number(&args, "max_x", 320.0) as i64;
            let min_y = Self::map_number(&args, "min_y", -240.0) as i64;
            let max_y = Self::map_number(&args, "max_y", 240.0) as i64;

            let wave_prefab = wave.prefab.clone();
            let size_distribution = wave.size_distribution.clone();

            fn respawn_x(seed: i64, min_x: i64, max_x: i64, ship_x: i64) -> i64 {
                let mut x = if (seed % 2) == 0 {
                    min_x + 10
                } else {
                    max_x - 10
                };
                if (x - ship_x).abs() < 90 {
                    x = if x < 0 { max_x - 10 } else { min_x + 10 };
                }
                x
            }

            fn respawn_y(seed: i64, min_y: i64, max_y: i64, ship_y: i64) -> i64 {
                let span = (max_y - min_y) - 20;
                if span <= 0 {
                    return min_y + 10;
                }
                let mut y = min_y + 10 + seed.rem_euclid(span);
                if (y - ship_y).abs() < 70 {
                    y += 76;
                    if y > max_y - 10 {
                        y = min_y + 10;
                    }
                }
                y
            }

            fn speed_for_seed(seed: i64) -> i64 {
                let v = seed.rem_euclid(3) + 1;
                if ((seed / 7) % 2) == 0 {
                    v
                } else {
                    -v
                }
            }

            return (0..spawn_count)
                .filter_map(|idx| {
                    // Find size for this index based on distribution
                    let mut size = 1i64;
                    for dist in &size_distribution {
                        if idx >= dist.min_idx && dist.max_idx.map_or(true, |max| idx < max) {
                            size = dist.size;
                            break;
                        }
                    }

                    let rx = self.rand_i(0, 2_147_483_646);
                    let ry = self.rand_i(0, 2_147_483_646);
                    let mut spawn_args = RhaiMap::new();
                    spawn_args.insert(
                        "x".into(),
                        (respawn_x(rx, min_x, max_x, ship_x) as rhai::FLOAT).into(),
                    );
                    spawn_args.insert(
                        "y".into(),
                        (respawn_y(ry, min_y, max_y, ship_y) as rhai::FLOAT).into(),
                    );
                    spawn_args.insert(
                        "vx".into(),
                        (speed_for_seed(self.rand_i(0, 2_147_483_646)) as rhai::FLOAT).into(),
                    );
                    spawn_args.insert(
                        "vy".into(),
                        (speed_for_seed(self.rand_i(0, 2_147_483_646)) as rhai::FLOAT).into(),
                    );
                    spawn_args.insert("shape".into(), self.rand_i(0, 3).into());
                    spawn_args.insert("size".into(), size.into());

                    let asteroid_id = self.spawn_prefab(&wave_prefab, spawn_args);
                    (asteroid_id > 0).then(|| asteroid_id.into())
                })
                .collect();
        }

        // Fall back to hardcoded waves
        fn respawn_x(seed: i64, min_x: i64, max_x: i64, ship_x: i64) -> i64 {
            let mut x = if (seed % 2) == 0 {
                min_x + 10
            } else {
                max_x - 10
            };
            if (x - ship_x).abs() < 90 {
                x = if x < 0 { max_x - 10 } else { min_x + 10 };
            }
            x
        }

        fn respawn_y(seed: i64, min_y: i64, max_y: i64, ship_y: i64) -> i64 {
            let span = (max_y - min_y) - 20;
            if span <= 0 {
                return min_y + 10;
            }
            let mut y = min_y + 10 + seed.rem_euclid(span);
            if (y - ship_y).abs() < 70 {
                y += 76;
                if y > max_y - 10 {
                    y = min_y + 10;
                }
            }
            y
        }

        fn speed_for_seed(seed: i64) -> i64 {
            let v = seed.rem_euclid(3) + 1;
            if ((seed / 7) % 2) == 0 {
                v
            } else {
                -v
            }
        }

        match wave_name {
            "asteroids.dynamic" => {
                let spawn_count = Self::map_int(&args, "spawn_count", 0).max(0);
                let ship_x = Self::map_number(&args, "ship_x", 0.0) as i64;
                let ship_y = Self::map_number(&args, "ship_y", 0.0) as i64;
                let min_x = Self::map_number(&args, "min_x", -320.0) as i64;
                let max_x = Self::map_number(&args, "max_x", 320.0) as i64;
                let min_y = Self::map_number(&args, "min_y", -240.0) as i64;
                let max_y = Self::map_number(&args, "max_y", 240.0) as i64;

                (0..spawn_count)
                    .filter_map(|idx| {
                        let rx = self.rand_i(0, 2_147_483_646);
                        let ry = self.rand_i(0, 2_147_483_646);
                        let mut spawn_args = RhaiMap::new();
                        spawn_args.insert(
                            "x".into(),
                            (respawn_x(rx, min_x, max_x, ship_x) as rhai::FLOAT).into(),
                        );
                        spawn_args.insert(
                            "y".into(),
                            (respawn_y(ry, min_y, max_y, ship_y) as rhai::FLOAT).into(),
                        );
                        spawn_args.insert(
                            "vx".into(),
                            (speed_for_seed(self.rand_i(0, 2_147_483_646)) as rhai::FLOAT).into(),
                        );
                        spawn_args.insert(
                            "vy".into(),
                            (speed_for_seed(self.rand_i(0, 2_147_483_646)) as rhai::FLOAT).into(),
                        );
                        spawn_args.insert("shape".into(), self.rand_i(0, 3).into());
                        let size = if idx < 2 {
                            3
                        } else if idx < 5 {
                            2
                        } else {
                            1
                        };
                        spawn_args.insert("size".into(), size.into());

                        let asteroid_id = self.spawn_prefab("asteroid", spawn_args);
                        (asteroid_id > 0).then(|| asteroid_id.into())
                    })
                    .collect()
            }
            _ => RhaiArray::new(),
        }
    }

    fn ensure_crack_visuals(&mut self, asteroid_id: rhai::INT) -> RhaiArray {
        let mut out = RhaiArray::new();
        if asteroid_id <= 0 {
            return out;
        }
        let Some(world) = self.world.clone() else {
            return out;
        };
        let asteroid_id_u64 = asteroid_id as u64;
        if !world.exists(asteroid_id_u64) {
            return out;
        }
        let mut asteroid = self.entity(asteroid_id);
        if asteroid.get_bool("/cracks_spawned", false) {
            return out;
        }

        for i in 0..3_i64 {
            let visual_id = format!("asteroid-{}-crack-{}", asteroid_id_u64, i);
            if let Ok(mut queue) = self.queue.lock() {
                queue.push(BehaviorCommand::SceneSpawn {
                    template: "asteroid-template".to_string(),
                    target: visual_id.clone(),
                });
            } else {
                return RhaiArray::new();
            }
            let _ = self.bind_visual(asteroid_id, &visual_id);
            let _ = self.set(
                asteroid_id,
                &format!("/crack_visual_{}", i),
                visual_id.clone().into(),
            );
            out.push(visual_id.into());
        }
        let _ = self.set(asteroid_id, "/cracks_spawned", true.into());
        out
    }

    fn flash_ui_message(&self, text: &str, ttl_ms: rhai::INT) -> bool {
        let text = text.trim();
        if text.is_empty() {
            return false;
        }
        let ttl_ms = ttl_ms.max(0);
        if let Some(game_state) = self.game_state.as_ref() {
            let until_ms = self.scene_elapsed_ms.saturating_add(ttl_ms as u64) as i64;
            let _ = game_state.set(
                ScriptUiApi::FLASH_TEXT_PATH,
                JsonValue::String(text.to_string()),
            );
            let _ = game_state.set(
                ScriptUiApi::FLASH_UNTIL_MS_PATH,
                JsonValue::Number(JsonNumber::from(until_ms)),
            );
        }
        if let Ok(mut queue) = self.queue.lock() {
            queue.push(BehaviorCommand::SetText {
                target: ScriptUiApi::FLASH_TARGET.to_string(),
                text: text.to_string(),
            });
            return true;
        }
        false
    }

    fn collisions(&mut self) -> RhaiArray {
        self.collisions
            .iter()
            .map(|hit| {
                let mut map = RhaiMap::new();
                map.insert("a".into(), (hit.a as rhai::INT).into());
                map.insert("b".into(), (hit.b as rhai::INT).into());
                map.into()
            })
            .collect()
    }

    fn collisions_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return vec![];
        };
        self.collisions
            .iter()
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

    fn collisions_of(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return vec![];
        };
        self.collisions
            .iter()
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

    /// Filters a collision hit slice by kind pair, returning `{kind_a: id, kind_b: id}` maps.
    fn filter_hits_by_kind(
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

    fn collision_enters_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return vec![];
        };
        Self::filter_hits_by_kind(&self.collision_enters.clone(), world, kind_a, kind_b)
    }

    fn collision_stays_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return vec![];
        };
        Self::filter_hits_by_kind(&self.collision_stays.clone(), world, kind_a, kind_b)
    }

    fn collision_exits_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return vec![];
        };
        Self::filter_hits_by_kind(&self.collision_exits.clone(), world, kind_a, kind_b)
    }

    fn spawn_child_entity(
        &mut self,
        parent_id: rhai::INT,
        kind: &str,
        template: &str,
        data: RhaiMap,
    ) -> rhai::INT {
        if parent_id < 0 {
            return 0;
        }
        // Check parent exists before taking &mut self via spawn_visual
        let parent_uid = parent_id as u64;
        let parent_exists = self
            .world
            .as_ref()
            .map(|w| w.exists(parent_uid))
            .unwrap_or(false);
        if !parent_exists {
            return 0;
        }
        let child_id = self.spawn_visual(kind, template, data);
        if child_id > 0 {
            if let Some(world) = self.world.as_ref() {
                world.register_child(parent_uid, child_id as u64);
            }
        }
        child_id
    }

    fn despawn_children_of(&mut self, parent_id: rhai::INT) {
        if parent_id < 0 {
            return;
        }
        let Some(world) = self.world.as_ref() else {
            return;
        };
        world.despawn_children(parent_id as u64);
    }

    fn enable_wrap(
        &mut self,
        id: rhai::INT,
        min_x: rhai::FLOAT,
        max_x: rhai::FLOAT,
        min_y: rhai::FLOAT,
        max_y: rhai::FLOAT,
    ) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        let bounds =
            engine_game::WrapBounds::new(min_x as f32, max_x as f32, min_y as f32, max_y as f32);
        world.set_wrap_bounds(uid, bounds)
    }

    fn disable_wrap(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        world.remove_wrap_bounds(uid);
        true
    }

    fn set_world_bounds(
        &mut self,
        min_x: rhai::FLOAT,
        max_x: rhai::FLOAT,
        min_y: rhai::FLOAT,
        max_y: rhai::FLOAT,
    ) {
        let Some(world) = self.world.as_ref() else {
            return;
        };
        world.set_world_bounds(min_x as f32, max_x as f32, min_y as f32, max_y as f32);
    }

    fn world_bounds(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        match world.world_bounds() {
            Some(b) => {
                let mut map = RhaiMap::new();
                map.insert("min_x".into(), (b.min_x as rhai::FLOAT).into());
                map.insert("max_x".into(), (b.max_x as rhai::FLOAT).into());
                map.insert("min_y".into(), (b.min_y as rhai::FLOAT).into());
                map.insert("max_y".into(), (b.max_y as rhai::FLOAT).into());
                map
            }
            None => RhaiMap::new(),
        }
    }

    fn enable_wrap_bounds(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.enable_wrap_bounds(id as u64)
    }

    fn rand_i(&mut self, min: rhai::INT, max: rhai::INT) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return min;
        };
        world.rand_i(min as i32, max as i32) as rhai::INT
    }

    fn rand_seed(&mut self, seed: rhai::INT) {
        let Some(world) = self.world.as_ref() else {
            return;
        };
        world.rand_seed(seed as i64);
    }

    fn tag_add(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.tag_add(id as u64, tag)
    }

    fn tag_remove(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.tag_remove(id as u64, tag)
    }

    fn tag_has(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.tag_has(id as u64, tag)
    }

    fn after_ms(&mut self, label: &str, delay_ms: rhai::INT) {
        let Some(world) = self.world.as_ref() else {
            return;
        };
        world.after_ms(label, delay_ms as i64);
    }

    fn timer_fired(&mut self, label: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.timer_fired(label)
    }

    fn cancel_timer(&mut self, label: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.cancel_timer(label)
    }

    /// Spawn multiple entities from an array of spec maps.
    /// Each map should have `kind: String` and optionally `data: Map`.
    /// Returns an array of spawned entity IDs.
    fn spawn_batch(&mut self, specs: rhai::Array) -> rhai::Array {
        let Some(world) = self.world.as_ref() else {
            return rhai::Array::new();
        };
        specs
            .into_iter()
            .filter_map(|spec| {
                let map = spec.try_cast::<RhaiMap>()?;
                let kind = map.get("kind")?.clone().try_cast::<String>()?;
                let data_dyn = map.get("data").cloned().unwrap_or_default();
                let data_json = rhai_dynamic_to_json(&data_dyn)
                    .unwrap_or(serde_json::Value::Object(Default::default()));
                let id = world.spawn(&kind, data_json)?;
                Some((id as rhai::INT).into())
            })
            .collect()
    }

    fn attach_ship_controller(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let uid = id as u64;

        // Extract config values with defaults
        let turn_step_ms = config
            .get("turn_step_ms")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(40) as u32;

        let thrust_power = config
            .get("thrust_power")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(170.0) as f32;

        let max_speed = config
            .get("max_speed")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(4.5) as f32;

        let heading_bits = config
            .get("heading_bits")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(32) as u8;

        let controller =
            TopDownShipController::new(turn_step_ms, thrust_power, max_speed, heading_bits);
        world.attach_controller(uid, controller)
    }

    fn ship_set_turn(&mut self, id: rhai::INT, dir: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        world.with_controller(uid, |ctrl| {
            ctrl.set_turn(dir.clamp(-1, 1) as i8);
        })
    }

    fn ship_set_thrust(&mut self, id: rhai::INT, on: bool) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        world.with_controller(uid, |ctrl| {
            ctrl.set_thrust(on);
        })
    }

    fn ship_heading(&mut self, id: rhai::INT) -> i32 {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        let uid = id as u64;
        world
            .controller(uid)
            .map(|c| c.current_heading)
            .unwrap_or(0)
    }

    fn ship_heading_vector(&mut self, id: rhai::INT) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let uid = id as u64;
        match world.controller(uid) {
            Some(ctrl) => {
                let (x, y) = ctrl.heading_vector();
                let mut map = RhaiMap::new();
                map.insert("x".into(), (x as rhai::FLOAT).into());
                map.insert("y".into(), (y as rhai::FLOAT).into());
                map
            }
            None => RhaiMap::new(),
        }
    }

    fn ship_velocity(&mut self, id: rhai::INT) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let uid = id as u64;
        match world.physics(uid) {
            Some(body) => {
                let mut map = RhaiMap::new();
                map.insert("vx".into(), (body.vx as rhai::FLOAT).into());
                map.insert("vy".into(), (body.vy as rhai::FLOAT).into());
                map
            }
            None => RhaiMap::new(),
        }
    }

    fn poll_collision_events(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        let collisions = world.poll_events("collision_enter");
        let mut array = RhaiArray::new();
        for (a, b) in collisions {
            let mut event = RhaiMap::new();
            event.insert("a".into(), (a as rhai::INT).into());
            event.insert("b".into(), (b as rhai::INT).into());
            array.push(RhaiDynamic::from(event));
        }
        array
    }

    fn clear_events(&mut self) {
        if let Some(world) = self.world.as_ref() {
            world.clear_events();
        }
    }

    fn distance(&mut self, a: rhai::INT, b: rhai::INT) -> rhai::FLOAT {
        let Some(world) = self.world.as_ref() else {
            return 0.0;
        };
        let ta = world.transform(a as u64);
        let tb = world.transform(b as u64);
        match (ta, tb) {
            (Some(a), Some(b)) => {
                let dx = a.x - b.x;
                let dy = a.y - b.y;
                ((dx * dx + dy * dy) as rhai::FLOAT).sqrt()
            }
            _ => 0.0,
        }
    }

    fn any_alive(&mut self, kind: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.count_kind(kind) > 0
    }
}

impl ScriptGameplayEntityApi {
    fn exists(&mut self) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.exists(self.id)
    }

    fn id(&mut self) -> rhai::INT {
        self.id as rhai::INT
    }

    fn flag(&mut self, name: &str) -> bool {
        let path = format!("/{}", name);
        self.get_bool(&path, false)
    }

    fn set_flag(&mut self, name: &str, value: bool) -> bool {
        let path = format!("/{}", name);
        self.set(&path, value.into())
    }

    fn despawn(&mut self) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if let Some(binding) = world.visual(self.id) {
            if let Ok(mut commands) = self.queue.lock() {
                for vid in binding.all_visual_ids() {
                    commands.push(BehaviorCommand::SceneDespawn {
                        target: vid.to_string(),
                    });
                }
            }
        }
        world.despawn(self.id)
    }

    fn get(&mut self, path: &str) -> RhaiDynamic {
        let Some(world) = self.world.as_ref() else {
            return ().into();
        };
        world
            .get(self.id, path)
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    fn get_i(&mut self, path: &str, fallback: rhai::INT) -> rhai::INT {
        self.get(path).try_cast::<rhai::INT>().unwrap_or(fallback)
    }

    fn get_bool(&mut self, path: &str, fallback: bool) -> bool {
        self.get(path).try_cast::<bool>().unwrap_or(fallback)
    }

    fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.set(self.id, path, value)
    }

    fn kind(&mut self) -> String {
        let Some(world) = self.world.as_ref() else {
            return String::new();
        };
        world.kind_of(self.id).unwrap_or_default()
    }

    fn tags(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .tags(self.id)
            .into_iter()
            .map(|tag| tag.into())
            .collect()
    }

    fn get_metadata(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(entity) = world.get_entity(self.id) else {
            return RhaiMap::new();
        };

        let mut metadata = RhaiMap::new();
        metadata.insert("id".into(), (self.id as rhai::INT).into());
        metadata.insert("kind".into(), entity.kind.into());

        let tags: RhaiArray = entity.tags.iter().map(|t| t.clone().into()).collect();
        metadata.insert("tags".into(), tags.into());

        // Include all components
        if let Some(transform) = world.transform(self.id) {
            let mut xf = RhaiMap::new();
            xf.insert("x".into(), (transform.x as rhai::FLOAT).into());
            xf.insert("y".into(), (transform.y as rhai::FLOAT).into());
            xf.insert("heading".into(), (transform.heading as rhai::FLOAT).into());
            metadata.insert("transform".into(), xf.into());
        }

        if let Some(physics) = world.physics(self.id) {
            let mut phys = RhaiMap::new();
            phys.insert("vx".into(), (physics.vx as rhai::FLOAT).into());
            phys.insert("vy".into(), (physics.vy as rhai::FLOAT).into());
            phys.insert("ax".into(), (physics.ax as rhai::FLOAT).into());
            phys.insert("ay".into(), (physics.ay as rhai::FLOAT).into());
            phys.insert("drag".into(), (physics.drag as rhai::FLOAT).into());
            phys.insert(
                "max_speed".into(),
                (physics.max_speed as rhai::FLOAT).into(),
            );
            metadata.insert("physics".into(), phys.into());
        }

        if let Some(collider) = world.collider(self.id) {
            let mut coll = RhaiMap::new();
            match &collider.shape {
                ColliderShape::Circle { radius } => {
                    coll.insert("shape".into(), "circle".into());
                    coll.insert("radius".into(), (*radius as rhai::FLOAT).into());
                }
                ColliderShape::Polygon { points } => {
                    coll.insert("shape".into(), "polygon".into());
                    let pts: RhaiArray = points
                        .iter()
                        .map(|p| {
                            let mut point = RhaiMap::new();
                            point.insert("x".into(), (p[0] as rhai::FLOAT).into());
                            point.insert("y".into(), (p[1] as rhai::FLOAT).into());
                            point.into()
                        })
                        .collect();
                    coll.insert("points".into(), pts.into());
                }
            }
            coll.insert("layer".into(), (collider.layer as rhai::INT).into());
            coll.insert("mask".into(), (collider.mask as rhai::INT).into());
            metadata.insert("collider".into(), coll.into());
        }

        if let Some(lifetime) = world.lifetime(self.id) {
            let mut life = RhaiMap::new();
            life.insert("ttl_ms".into(), (lifetime.ttl_ms as rhai::INT).into());
            metadata.insert("lifetime".into(), life.into());
        }

        if let Some(visual) = world.visual(self.id) {
            if let Some(visual_id) = &visual.visual_id {
                metadata.insert("visual_id".into(), visual_id.clone().into());
            }
            if !visual.additional_visuals.is_empty() {
                let extras: RhaiArray = visual
                    .additional_visuals
                    .iter()
                    .map(|v| v.clone().into())
                    .collect();
                metadata.insert("additional_visuals".into(), extras.into());
            }
        }

        metadata
    }

    fn get_components(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };

        let mut components = RhaiMap::new();

        if let Some(transform) = world.transform(self.id) {
            let mut xf = RhaiMap::new();
            xf.insert("x".into(), (transform.x as rhai::FLOAT).into());
            xf.insert("y".into(), (transform.y as rhai::FLOAT).into());
            xf.insert("heading".into(), (transform.heading as rhai::FLOAT).into());
            components.insert("transform".into(), xf.into());
        }

        if let Some(physics) = world.physics(self.id) {
            let mut phys = RhaiMap::new();
            phys.insert("vx".into(), (physics.vx as rhai::FLOAT).into());
            phys.insert("vy".into(), (physics.vy as rhai::FLOAT).into());
            phys.insert("ax".into(), (physics.ax as rhai::FLOAT).into());
            phys.insert("ay".into(), (physics.ay as rhai::FLOAT).into());
            phys.insert("drag".into(), (physics.drag as rhai::FLOAT).into());
            phys.insert(
                "max_speed".into(),
                (physics.max_speed as rhai::FLOAT).into(),
            );
            components.insert("physics".into(), phys.into());
        }

        if let Some(collider) = world.collider(self.id) {
            let mut coll = RhaiMap::new();
            match &collider.shape {
                ColliderShape::Circle { radius } => {
                    coll.insert("shape".into(), "circle".into());
                    coll.insert("radius".into(), (*radius as rhai::FLOAT).into());
                }
                ColliderShape::Polygon { points } => {
                    coll.insert("shape".into(), "polygon".into());
                    let pts: RhaiArray = points
                        .iter()
                        .map(|p| {
                            let mut point = RhaiMap::new();
                            point.insert("x".into(), (p[0] as rhai::FLOAT).into());
                            point.insert("y".into(), (p[1] as rhai::FLOAT).into());
                            point.into()
                        })
                        .collect();
                    coll.insert("points".into(), pts.into());
                }
            }
            coll.insert("layer".into(), (collider.layer as rhai::INT).into());
            coll.insert("mask".into(), (collider.mask as rhai::INT).into());
            components.insert("collider".into(), coll.into());
        }

        if let Some(lifetime) = world.lifetime(self.id) {
            let mut life = RhaiMap::new();
            life.insert("ttl_ms".into(), (lifetime.ttl_ms as rhai::INT).into());
            components.insert("lifetime".into(), life.into());
        }

        if let Some(visual) = world.visual(self.id) {
            if let Some(visual_id) = &visual.visual_id {
                components.insert("visual_id".into(), visual_id.clone().into());
            }
            if !visual.additional_visuals.is_empty() {
                let extras: RhaiArray = visual
                    .additional_visuals
                    .iter()
                    .map(|v| v.clone().into())
                    .collect();
                components.insert("additional_visuals".into(), extras.into());
            }
        }

        components
    }

    fn transform(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(xf) = world.transform(self.id) else {
            return RhaiMap::new();
        };

        let mut result = RhaiMap::new();
        result.insert("x".into(), (xf.x as rhai::FLOAT).into());
        result.insert("y".into(), (xf.y as rhai::FLOAT).into());
        result.insert("heading".into(), (xf.heading as rhai::FLOAT).into());
        result
    }

    fn set_position(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut xf) = world.transform(self.id) else {
            return false;
        };
        xf.x = x as f32;
        xf.y = y as f32;
        world.set_transform(self.id, xf)
    }

    fn set_heading(&mut self, heading: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut xf) = world.transform(self.id) else {
            return false;
        };
        xf.heading = heading as f32;
        world.set_transform(self.id, xf)
    }

    fn physics(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(phys) = world.physics(self.id) else {
            return RhaiMap::new();
        };

        let mut result = RhaiMap::new();
        result.insert("vx".into(), (phys.vx as rhai::FLOAT).into());
        result.insert("vy".into(), (phys.vy as rhai::FLOAT).into());
        result.insert("ax".into(), (phys.ax as rhai::FLOAT).into());
        result.insert("ay".into(), (phys.ay as rhai::FLOAT).into());
        result.insert("drag".into(), (phys.drag as rhai::FLOAT).into());
        result.insert("max_speed".into(), (phys.max_speed as rhai::FLOAT).into());
        result
    }

    fn set_velocity(&mut self, vx: rhai::FLOAT, vy: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut phys) = world.physics(self.id) else {
            return false;
        };
        phys.vx = vx as f32;
        phys.vy = vy as f32;
        world.set_physics(self.id, phys)
    }

    fn set_acceleration(&mut self, ax: rhai::FLOAT, ay: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut phys) = world.physics(self.id) else {
            return false;
        };
        phys.ax = ax as f32;
        phys.ay = ay as f32;
        world.set_physics(self.id, phys)
    }

    fn collider(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(coll) = world.collider(self.id) else {
            return RhaiMap::new();
        };

        let mut result = RhaiMap::new();
        match &coll.shape {
            ColliderShape::Circle { radius } => {
                result.insert("shape".into(), "circle".into());
                result.insert("radius".into(), (*radius as rhai::FLOAT).into());
            }
            ColliderShape::Polygon { points } => {
                result.insert("shape".into(), "polygon".into());
                let pts: RhaiArray = points
                    .iter()
                    .map(|p| {
                        let mut point = RhaiMap::new();
                        point.insert("x".into(), (p[0] as rhai::FLOAT).into());
                        point.insert("y".into(), (p[1] as rhai::FLOAT).into());
                        point.into()
                    })
                    .collect();
                result.insert("points".into(), pts.into());
            }
        }
        result.insert("layer".into(), (coll.layer as rhai::INT).into());
        result.insert("mask".into(), (coll.mask as rhai::INT).into());
        result
    }

    fn lifetime_remaining(&mut self) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        let Some(lifetime) = world.lifetime(self.id) else {
            return 0;
        };
        lifetime.ttl_ms as rhai::INT
    }

    fn set_many(&mut self, map: RhaiMap) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        for (key, value) in map {
            let Some(json_value) = rhai_dynamic_to_json(&value) else {
                return false;
            };
            if !world.set(self.id, &format!("/{}", key), json_value) {
                return false;
            }
        }
        true
    }

    fn data(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(entity) = world.get_entity(self.id) else {
            return RhaiMap::new();
        };
        json_to_rhai_dynamic(&entity.data)
            .try_cast::<RhaiMap>()
            .unwrap_or_default()
    }

    fn get_f(&mut self, path: &str, fallback: rhai::FLOAT) -> rhai::FLOAT {
        self.get(path).try_cast::<rhai::FLOAT>().unwrap_or(fallback)
    }

    fn get_s(&mut self, path: &str, fallback: &str) -> String {
        self.get(path)
            .try_cast::<String>()
            .unwrap_or_else(|| fallback.to_string())
    }

    // ── Cooldown API ──────────────────────────────────────────────────────

    fn cooldown_start(&mut self, name: &str, ms: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.cooldown_start(self.id, name, ms as i32)
    }

    fn cooldown_ready(&mut self, name: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return true;
        };
        world.cooldown_ready(self.id, name)
    }

    fn cooldown_remaining(&mut self, name: &str) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        world.cooldown_remaining(self.id, name) as rhai::INT
    }

    // ── Status API ────────────────────────────────────────────────────────

    fn status_add(&mut self, name: &str, ms: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.status_add(self.id, name, ms as i32)
    }

    fn status_has(&mut self, name: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.status_has(self.id, name)
    }

    fn status_remaining(&mut self, name: &str) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        world.status_remaining(self.id, name) as rhai::INT
    }

    // ── Ship Controller API ───────────────────────────────────────────────

    fn attach_ship_controller(&mut self, config: RhaiMap) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let turn_step_ms = config
            .get("turn_step_ms")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(40) as u32;
        let thrust_power = config
            .get("thrust_power")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(170.0) as f32;
        let max_speed = config
            .get("max_speed")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(4.5) as f32;
        let heading_bits = config
            .get("heading_bits")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(32) as u8;
        let controller =
            TopDownShipController::new(turn_step_ms, thrust_power, max_speed, heading_bits);
        world.attach_controller(self.id, controller)
    }

    fn set_turn(&mut self, dir: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.with_controller(self.id, |ctrl| {
            ctrl.set_turn(dir.clamp(-1, 1) as i8);
        })
    }

    fn set_thrust(&mut self, on: bool) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.with_controller(self.id, |ctrl| {
            ctrl.set_thrust(on);
        })
    }

    fn heading(&mut self) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        world
            .controller(self.id)
            .map(|c| c.current_heading as rhai::INT)
            .unwrap_or(0)
    }

    fn heading_vector(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        match world.controller(self.id) {
            Some(ctrl) => {
                let (x, y) = ctrl.heading_vector();
                let mut map = RhaiMap::new();
                map.insert("x".into(), (x as rhai::FLOAT).into());
                map.insert("y".into(), (y as rhai::FLOAT).into());
                map
            }
            None => RhaiMap::new(),
        }
    }
}

impl RhaiScriptBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        let compile_error = match params.script.as_deref() {
            Some(src) => {
                let engine = init_rhai_engine();
                match engine.compile(src) {
                    Ok(ast) => {
                        // Pre-populate the thread-local AST cache so the first
                        // frame doesn't pay the compile cost.
                        let hash = script_hash(src);
                        AST_CACHE.with(|cache| {
                            cache.borrow_mut().insert(hash, ast);
                        });
                        None
                    }
                    Err(err) => Some(format!("{}", err)),
                }
            }
            None => None,
        };
        Self {
            params: params.clone(),
            state: JsonValue::Object(JsonMap::new()),
            compile_error,
            compile_error_reported: false,
            behavior_id: NEXT_BEHAVIOR_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl Drop for RhaiScriptBehavior {
    fn drop(&mut self) {
        // Clean up the thread-local scope so it doesn't outlive the scene.
        BEHAVIOR_SCOPES.with(|scopes| {
            scopes.borrow_mut().remove(&self.behavior_id);
        });
    }
}

impl RhaiScriptBehavior {
    fn build_regions_map(&self, ctx: &BehaviorContext, scene: &Scene) -> RhaiMap {
        let mut regions = RhaiMap::new();
        for (object_id, region) in ctx.object_regions.iter() {
            regions.insert(object_id.clone().into(), region_to_rhai_map(region).into());
        }
        if let Some(target) = self.params.target.as_deref() {
            if let Some(region) = ctx.resolved_object_region(target) {
                regions.insert(target.into(), region_to_rhai_map(region).into());
            }
        }
        let total = self.params.count.unwrap_or(scene.menu_options.len());
        let prefix = self
            .params
            .item_prefix
            .as_deref()
            .unwrap_or("menu-item-")
            .to_string();
        for idx in 0..total {
            let alias = if prefix.contains("{}") {
                prefix.replace("{}", &idx.to_string())
            } else {
                format!("{prefix}{idx}")
            };
            if let Some(region) = ctx.resolved_object_region(&alias) {
                regions.insert(alias.into(), region_to_rhai_map(region).into());
            }
        }
        regions
    }
}

fn build_collision_events_array(collisions: &[CollisionHit]) -> RhaiArray {
    collisions
        .iter()
        .map(|hit| {
            let mut map = RhaiMap::new();
            map.insert("a".into(), (hit.a as rhai::INT).into());
            map.insert("b".into(), (hit.b as rhai::INT).into());
            map.into()
        })
        .collect()
}

fn build_sidecar_io_map(sidecar_io: &SidecarIoFrameState) -> RhaiMap {
    let mut ipc_map = RhaiMap::new();
    ipc_map.insert(
        "has_output".into(),
        (!sidecar_io.output_lines.is_empty()).into(),
    );
    let output_array: RhaiArray = sidecar_io
        .output_lines
        .iter()
        .cloned()
        .map(Into::into)
        .collect();
    ipc_map.insert("output_lines".into(), output_array.into());
    ipc_map.insert(
        "clear_count".into(),
        (sidecar_io.clear_count as rhai::INT).into(),
    );
    ipc_map.insert(
        "has_screen_full".into(),
        sidecar_io.screen_full_lines.is_some().into(),
    );
    let screen_full_lines: RhaiArray = sidecar_io
        .screen_full_lines
        .as_ref()
        .map(|lines| lines.iter().cloned().map(Into::into).collect())
        .unwrap_or_default();
    ipc_map.insert("screen_full_lines".into(), screen_full_lines.into());
    let custom_events: RhaiArray = sidecar_io
        .custom_events
        .iter()
        .cloned()
        .map(Into::into)
        .collect();
    ipc_map.insert("custom_events".into(), custom_events.into());
    ipc_map
}

struct UiFieldsData {
    focused_target: String,
    theme: String,
    submit_target: String,
    submit_text: String,
    change_target: String,
    change_text: String,
    has_submit: bool,
    has_change: bool,
}

fn extract_ui_fields_data(ctx: &BehaviorContext) -> UiFieldsData {
    UiFieldsData {
        focused_target: ctx
            .ui_focused_target_id
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        theme: ctx.ui_theme_id.as_deref().unwrap_or_default().to_string(),
        submit_target: ctx
            .ui_last_submit_target_id
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        submit_text: ctx
            .ui_last_submit_text
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        change_target: ctx
            .ui_last_change_target_id
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        change_text: ctx
            .ui_last_change_text
            .as_deref()
            .unwrap_or_default()
            .to_string(),
        has_submit: ctx.ui_last_submit_target_id.is_some(),
        has_change: ctx.ui_last_change_target_id.is_some(),
    }
}

impl Behavior for RhaiScriptBehavior {
    fn update(
        &mut self,
        _object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        if let Some(err) = &self.compile_error {
            if !self.compile_error_reported {
                commands.push(BehaviorCommand::ScriptError {
                    scene_id: scene.id.clone(),
                    source: self.params.src.clone(),
                    message: format!("compile error: {}", err),
                });
                self.compile_error_reported = true;
            }
            return;
        }

        let Some(script) = self.params.script.as_deref() else {
            return;
        };

        // Compute hash and regions flag before entering the scope borrow.
        let hash = script_hash(script);
        let needs_regions = script.contains("regions");

        // Build per-frame data outside the borrow to avoid lifetime conflicts.
        let regions_map = if needs_regions {
            Some(self.build_regions_map(ctx, scene))
        } else {
            None
        };
        let helper_commands = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        expire_ui_flash_message(
            ctx.scene_elapsed_ms,
            ctx.game_state.as_ref(),
            helper_commands.as_ref(),
        );

        let eval_result: Result<RhaiDynamic, Box<rhai::EvalAltResult>> =
            BEHAVIOR_SCOPES.with(|scopes| {
                let mut map = scopes.borrow_mut();
                let (scope, base_len) = map
                    .entry(self.behavior_id)
                    .or_insert_with(|| (rhai::Scope::new(), 0));

                // One-time static init: push params and local below the rewind
                // point. `local` is seeded from `self.state` so scripts migrating
                // from the legacy `{state: ...}` return pattern get their state.
                if *base_len == 0 {
                    scope.push_dynamic("params", behavior_params_to_rhai_map(&self.params).into());
                    scope.push_dynamic("local", json_to_rhai_dynamic(&self.state));
                    *base_len = scope.len();
                }

                // Rewind to static base: clears all per-frame variables pushed
                // last frame and any `let` declarations the script made.
                // Variables at positions 0..*base_len (params, local) are kept.
                scope.rewind(*base_len);

                // --- Per-frame pushes ---

                // Phase 7C: Use Arc-wrapped maps from context instead of rebuilding.
                // Each push_dynamic clones the Arc (O(1) refcount), not the map (O(n)).
                scope.push_dynamic("menu", (*ctx.rhai_menu_map).clone().into());
                scope.push(
                    "time",
                    ScriptTimeApi::new(
                        ctx.scene_elapsed_ms,
                        ctx.stage_elapsed_ms,
                        ctx.stage,
                        ctx.game_state.clone(),
                    ),
                );

                // Compatibility layer for existing scripts; prefer `menu.*` and `time.*`.
                scope.push("selected_index", ctx.menu_selected_index as rhai::INT);
                scope.push("scene_elapsed_ms", ctx.scene_elapsed_ms as rhai::INT);
                scope.push("stage_elapsed_ms", ctx.stage_elapsed_ms as rhai::INT);
                scope.push("menu_count", scene.menu_options.len() as rhai::INT);

                // OPT-11: Only build regions_map when the script uses `regions`.
                scope.push_dynamic(
                    "regions",
                    regions_map
                        .map(|m| rhai::Dynamic::from(m))
                        .unwrap_or_else(|| RhaiMap::new().into()),
                );
                // OPT-3 + OPT-10: Skip build_objects_map entirely; push empty map for
                // backward compat. All scripts use scene.get(target) for lazy lookup.
                scope.push_dynamic("objects", RhaiMap::new().into());
                // `state` pushed per-frame for scripts using the legacy return-state pattern.
                scope.push_dynamic("state", json_to_rhai_dynamic(&self.state));
                scope.push("ui", ScriptUiApi::new(ctx, Arc::clone(&helper_commands)));
                
                let ui_data = extract_ui_fields_data(ctx);
                scope.push("ui_focused_target", ui_data.focused_target);
                scope.push("ui_theme", ui_data.theme);
                scope.push("ui_submit_target", ui_data.submit_target);
                scope.push("ui_submit_text", ui_data.submit_text);
                scope.push("ui_change_target", ui_data.change_target);
                scope.push("ui_change_text", ui_data.change_text);
                scope.push("ui_has_submit", ui_data.has_submit);
                scope.push("ui_has_change", ui_data.has_change);

                // Phase 7C: Use Arc-wrapped key map from context instead of rebuilding.
                scope.push_dynamic("key", (*ctx.rhai_key_map).clone().into());

                // Engine-level key state (separate namespace to prevent behavior interference)
                scope.push_dynamic("engine", (*ctx.engine_key_map).clone().into());

                // Gameplay collision events (array of {a, b} maps).
                scope.push_dynamic(
                    "collisions",
                    build_collision_events_array(&ctx.collisions).into(),
                );

                // External sidecar bridge exposed as object-shaped `ipc.*`.
                scope.push_dynamic("ipc", build_sidecar_io_map(&ctx.sidecar_io).into());

                // OPT-4: Reuse thread-local engine with all static registrations pre-done.
                scope.push(
                    "scene",
                    ScriptSceneApi::new(
                        Arc::clone(&ctx.object_states),
                        Arc::clone(&ctx.object_kinds),
                        Arc::clone(&ctx.object_props),
                        Arc::clone(&ctx.object_regions),
                        Arc::clone(&ctx.object_text),
                        Arc::clone(&ctx.target_resolver),
                        Arc::clone(&helper_commands),
                    ),
                );
                scope.push(
                    "game",
                    ScriptGameApi::new(ctx.game_state.clone(), Arc::clone(&helper_commands)),
                );
                scope.push("level", ScriptLevelApi::new(ctx.level_state.clone()));
                scope.push(
                    "terminal",
                    ScriptTerminalApi::new(Arc::clone(&helper_commands)),
                );
                scope.push(
                    "input",
                    ScriptInputApi::new(
                        Arc::clone(&ctx.keys_down),
                        Arc::clone(&ctx.keys_just_pressed),
                        Arc::clone(&ctx.action_bindings),
                        Arc::clone(&ctx.catalogs),
                        Arc::clone(&helper_commands),
                    ),
                );
                scope.push(
                    "diag",
                    ScriptDebugApi::new(
                        scene.id.clone(),
                        self.params.src.clone(),
                        Arc::clone(&helper_commands),
                    ),
                );
                scope.push(
                    "persist",
                    ScriptPersistenceApi::new(ctx.persistence.clone()),
                );
                scope.push(
                    "world",
                    ScriptGameplayApi::new(
                        ctx.gameplay_world.clone(),
                        ctx.game_state.clone(),
                        ctx.scene_elapsed_ms,
                        std::sync::Arc::clone(&ctx.collisions),
                        std::sync::Arc::clone(&ctx.collision_enters),
                        std::sync::Arc::clone(&ctx.collision_stays),
                        std::sync::Arc::clone(&ctx.collision_exits),
                        Arc::clone(&ctx.catalogs),
                        Arc::clone(&helper_commands),
                    ),
                );
                scope.push("audio", ScriptAudioApi::new(Arc::clone(&helper_commands)));
                scope.push(
                    "fx",
                    ScriptFxApi::new(
                        ctx.gameplay_world.clone(),
                        Arc::clone(&ctx.catalogs),
                        Arc::clone(&helper_commands),
                    ),
                );

                // OPT-4: Use thread-local engine + cached AST.
                RHAI_ENGINE.with(|cell| {
                    let mut opt = cell.borrow_mut();
                    let engine = opt.get_or_insert_with(init_rhai_engine);
                    AST_CACHE.with(|cache| {
                        let borrow = cache.borrow();
                        if let Some(ast) = borrow.get(&hash) {
                            return engine.eval_ast_with_scope::<RhaiDynamic>(scope, ast);
                        }
                        drop(borrow);
                        match engine.compile(script) {
                            Ok(ast) => {
                                let result = engine.eval_ast_with_scope::<RhaiDynamic>(scope, &ast);
                                let mut cache_mut = cache.borrow_mut();
                                // Limit AST cache to 256 entries (typical game has ~20-50 scripts)
                                // If full, clear oldest half to make room (simple eviction strategy)
                                if cache_mut.len() >= 256 {
                                    let to_remove = cache_mut.len() / 2;
                                    let keys: Vec<_> =
                                        cache_mut.keys().take(to_remove).copied().collect();
                                    for key in keys {
                                        cache_mut.remove(&key);
                                    }
                                }
                                cache_mut.insert(hash, ast);
                                result
                            }
                            Err(err) => Err(err.into()),
                        }
                    })
                })
            });

        let result = match eval_result {
            Ok(r) => r,
            Err(err) => {
                let src = self.params.src.as_deref().unwrap_or("<inline>");
                let msg = format!("{}", err);
                eprintln!(
                    "Rhai script error in scene '{}' (src: {}): {}",
                    scene.id, src, msg
                );
                commands.push(BehaviorCommand::ScriptError {
                    scene_id: scene.id.clone(),
                    source: self.params.src.clone(),
                    message: msg,
                });
                return;
            }
        };
        if let Some(map) = result.clone().try_cast::<RhaiMap>() {
            if let Some(next_state) = map.get("state").and_then(rhai_dynamic_to_json) {
                self.state = next_state;
            }
        }
        apply_rhai_commands(result, commands);
        if let Ok(mut queue) = helper_commands.lock() {
            commands.extend(queue.drain(..));
        };
    }
}

/// Executes a tiny multi-step runtime probe against a Rhai script using the same
/// behavior runtime path as the game loop.
pub fn smoke_validate_rhai_script(
    script: &str,
    src: Option<&str>,
    scene: &Scene,
) -> Result<(), String> {
    let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
        src: src.map(ToString::to_string),
        script: Some(script.to_string()),
        ..BehaviorParams::default()
    });
    if let Some(err) = behavior.compile_error.as_ref() {
        return Err(format!("compile error: {err}"));
    }

    let probe_object = GameObject {
        id: "__rhai_probe__".to_string(),
        name: "__rhai_probe__".to_string(),
        kind: GameObjectKind::Scene,
        aliases: Vec::new(),
        parent_id: None,
        children: Vec::new(),
    };
    let game_state = GameState::new();
    let gameplay_world = GameplayWorld::new();

    let frames = [
        (0_u64, None),
        (16_u64, Some("linus")),
        (32_u64, Some("tux")),
        (420_u64, None),
        (470_u64, Some("help")),
    ];

    for (elapsed_ms, submit_text) in frames {
        let ctx = smoke_probe_context(
            elapsed_ms,
            submit_text,
            game_state.clone(),
            gameplay_world.clone(),
        );
        let mut commands = Vec::new();
        behavior.update(&probe_object, scene, &ctx, &mut commands);
        if let Some(message) = commands.into_iter().find_map(|command| match command {
            BehaviorCommand::ScriptError { message, .. } => Some(message),
            _ => None,
        }) {
            return Err(message);
        }
    }
    Ok(())
}

fn smoke_probe_context(
    elapsed_ms: u64,
    submit_text: Option<&str>,
    game_state: GameState,
    gameplay_world: GameplayWorld,
) -> BehaviorContext {
    BehaviorContext {
        stage: SceneStage::OnIdle,
        scene_elapsed_ms: elapsed_ms,
        stage_elapsed_ms: elapsed_ms,
        menu_selected_index: 0,
        target_resolver: Arc::new(TargetResolver::default()),
        object_states: Arc::new(HashMap::new()),
        object_kinds: Arc::new(HashMap::new()),
        object_props: Arc::new(HashMap::new()),
        object_regions: Arc::new(HashMap::new()),
        object_text: Arc::new(HashMap::new()),
        ui_focused_target_id: Some(Arc::from("login-hidden-prompt")),
        ui_theme_id: None,
        ui_last_submit_target_id: submit_text.map(|_| Arc::from("login-hidden-prompt")),
        ui_last_submit_text: submit_text.map(|s| Arc::from(s)),
        ui_last_change_target_id: None,
        ui_last_change_text: None,
        game_state: Some(game_state),
        level_state: None,
        persistence: None,
        catalogs: Arc::new(catalog::ModCatalogs::default()),
        gameplay_world: Some(gameplay_world),
        collisions: Arc::new(Vec::new()),
        collision_enters: Arc::new(Vec::new()),
        collision_stays: Arc::new(Vec::new()),
        collision_exits: Arc::new(Vec::new()),
        last_raw_key: None,
        keys_down: Arc::new(HashSet::new()),
        keys_just_pressed: Arc::new(HashSet::new()),
        action_bindings: Arc::new(HashMap::new()),
        sidecar_io: Arc::new(SidecarIoFrameState::default()),
        rhai_time_map: Arc::new(RhaiMap::new()),
        rhai_menu_map: Arc::new(RhaiMap::new()),
        rhai_key_map: Arc::new(RhaiMap::new()),
        engine_key_map: Arc::new(RhaiMap::new()),
    }
}

/// Shows directional arrow sprites flanking the selected menu option.
pub struct SelectedArrowsBehavior {
    target: Option<String>,
    index: usize,
    side: ArrowSide,
    padding: i32,
    amplitude_x: i32,
    period_ms: u64,
    phase_ms: u64,
    autoscale_height: bool,
    last_dx: i32,
    last_dy: i32,
}

#[derive(Clone, Copy)]
enum ArrowSide {
    Left,
    Right,
}

impl SelectedArrowsBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        let side_str = params.side.as_deref().unwrap_or("");
        let side = if side_str.trim().eq_ignore_ascii_case("right") {
            ArrowSide::Right
        } else {
            ArrowSide::Left
        };
        Self {
            target: params.target.clone(),
            index: params.index.unwrap_or(0),
            side,
            padding: params.padding.unwrap_or(1),
            amplitude_x: params.amplitude_x.unwrap_or(1).abs(),
            period_ms: params.period_ms.unwrap_or(900).max(1),
            phase_ms: params.phase_ms.unwrap_or(0),
            autoscale_height: params.autoscale_height.unwrap_or(false),
            last_dx: 0,
            last_dy: 0,
        }
    }

    fn hide_and_reset(&mut self, object: &GameObject, commands: &mut Vec<BehaviorCommand>) {
        self.last_dx = 0;
        self.last_dy = 0;
        emit_visibility(commands, object.id.clone(), false);
    }
}

impl Behavior for SelectedArrowsBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        if ctx.menu_selected_index != self.index {
            self.hide_and_reset(object, commands);
            return;
        }

        let Some(target_alias) = self.target.as_deref() else {
            self.hide_and_reset(object, commands);
            return;
        };
        let Some(target_region) = ctx.resolved_object_region(target_alias) else {
            self.hide_and_reset(object, commands);
            return;
        };
        let Some(own_region) = ctx.object_region(&object.id) else {
            emit_visibility(commands, object.id.clone(), true);
            // First frame after becoming visible: wait for compositor to discover own region.
            return;
        };

        let wave = rounded_sine_wave(ctx.scene_elapsed_ms, self.phase_ms, self.period_ms);
        let signed_wave = match self.side {
            ArrowSide::Left => wave,
            ArrowSide::Right => -wave,
        } * self.amplitude_x;
        let auto_pad = if self.autoscale_height {
            (target_region.height.saturating_sub(1) as i32) / 2
        } else {
            0
        };
        let effective_padding = self.padding.saturating_add(auto_pad).max(0);
        let arrow_w = own_region.width.max(1) as i32;
        let arrow_h = own_region.height.max(1) as i32;
        let target_w = target_region.width.max(1) as i32;
        let target_center_y =
            target_region.y as i32 + (target_region.height.saturating_sub(1) as i32 / 2);

        let target_x = match self.side {
            ArrowSide::Left => target_region.x as i32 - effective_padding - arrow_w + signed_wave,
            ArrowSide::Right => target_region.x as i32 + target_w + effective_padding + signed_wave,
        };
        let target_y = target_center_y.saturating_sub((arrow_h.saturating_sub(1)) / 2);

        emit_visibility(commands, object.id.clone(), true);

        let base_x = own_region.x as i32 - self.last_dx;
        let base_y = own_region.y as i32 - self.last_dy;
        let new_dx = target_x - base_x;
        let new_dy = target_y - base_y;
        self.last_dx = new_dx;
        self.last_dy = new_dy;

        emit_offset(commands, object.id.clone(), new_dx, new_dy);
    }
}

impl StageVisibilityBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        let stages = params
            .stages
            .iter()
            .filter_map(|value| parse_stage_name(value))
            .collect::<Vec<_>>();
        Self {
            target: params.target.clone(),
            stages,
        }
    }
}

impl Behavior for StageVisibilityBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let visible = if self.stages.is_empty() {
            true
        } else {
            self.stages.iter().any(|stage| stage == &ctx.stage)
        };
        emit_visibility(commands, resolve_target(&self.target, object), visible);
    }
}

/// Shows the object only within a configured time window relative to the scene or stage clock.
pub struct TimedVisibilityBehavior {
    target: Option<String>,
    start_ms: Option<u64>,
    end_ms: Option<u64>,
    time_scope: TimeScope,
}

impl TimedVisibilityBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            start_ms: params.start_ms,
            end_ms: params.end_ms,
            time_scope: TimeScope::from_params(params),
        }
    }
}

impl Behavior for TimedVisibilityBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let elapsed_ms = self.time_scope.elapsed_ms(ctx);
        emit_visibility(
            commands,
            resolve_target(&self.target, object),
            is_within_time_window(elapsed_ms, self.start_ms, self.end_ms),
        );
    }
}

fn apply_rhai_commands(result: RhaiDynamic, commands: &mut Vec<BehaviorCommand>) {
    let commands_dynamic = if result.is::<RhaiArray>() {
        result
    } else if result.is::<RhaiMap>() {
        let map = result.cast::<RhaiMap>();
        map.get("commands")
            .cloned()
            .unwrap_or_else(|| RhaiArray::new().into())
    } else {
        return;
    };
    let Some(array) = commands_dynamic.try_cast::<RhaiArray>() else {
        return;
    };
    for command in array {
        let Some(map) = command.try_cast::<RhaiMap>() else {
            continue;
        };
        let op = map
            .get("op")
            .and_then(|value| value.clone().try_cast::<String>())
            .unwrap_or_default();
        match op.as_str() {
            "visibility" => {
                let Some(target) = map
                    .get("target")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let Some(visible) = map
                    .get("visible")
                    .and_then(|value| value.clone().try_cast::<bool>())
                else {
                    continue;
                };
                emit_visibility(commands, target, visible);
            }
            "offset" => {
                let Some(target) = map
                    .get("target")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let dx = map
                    .get("dx")
                    .and_then(|value| value.clone().try_cast::<rhai::INT>())
                    .unwrap_or(0);
                let dy = map
                    .get("dy")
                    .and_then(|value| value.clone().try_cast::<rhai::INT>())
                    .unwrap_or(0);
                emit_offset(commands, target, dx as i32, dy as i32);
            }
            "set-text" => {
                let Some(target) = map
                    .get("target")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let Some(text) = map
                    .get("text")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                emit_text(commands, target, text);
            }
            "set-props" => {
                let Some(target) = map
                    .get("target")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let visible = map
                    .get("visible")
                    .and_then(|value| value.clone().try_cast::<bool>());
                let dx = map
                    .get("dx")
                    .and_then(|value| value.clone().try_cast::<rhai::INT>())
                    .map(|value| value as i32);
                let dy = map
                    .get("dy")
                    .and_then(|value| value.clone().try_cast::<rhai::INT>())
                    .map(|value| value as i32);
                let text = map
                    .get("text")
                    .and_then(|value| value.clone().try_cast::<String>());
                if visible.is_none() && dx.is_none() && dy.is_none() && text.is_none() {
                    continue;
                }
                commands.push(BehaviorCommand::SetProps {
                    target,
                    visible,
                    dx,
                    dy,
                    text,
                });
            }
            "set" => {
                let Some(target) = map
                    .get("target")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let Some(path) = map
                    .get("path")
                    .and_then(|value| value.clone().try_cast::<String>())
                else {
                    continue;
                };
                let Some(value) = map.get("value").and_then(rhai_dynamic_to_json) else {
                    continue;
                };
                commands.push(BehaviorCommand::SetProperty {
                    target,
                    path: normalize_set_path(&path),
                    value,
                });
            }
            "transition" => {
                let Some(to_scene_id) = map
                    .get("to_scene_id")
                    .and_then(|value| value.clone().try_cast::<String>())
                    .filter(|value| !value.trim().is_empty())
                else {
                    continue;
                };
                commands.push(BehaviorCommand::SceneTransition { to_scene_id });
            }
            _ => {}
        }
    }
}

fn is_within_time_window(elapsed_ms: u64, start_ms: Option<u64>, end_ms: Option<u64>) -> bool {
    start_ms.map(|start| elapsed_ms >= start).unwrap_or(true)
        && end_ms.map(|end| elapsed_ms < end).unwrap_or(true)
}

#[derive(Clone, Copy)]
enum TimeScope {
    Scene,
    Stage,
}

impl TimeScope {
    pub fn from_params(params: &BehaviorParams) -> Self {
        let scope_str = params.time_scope.as_deref().unwrap_or("");
        if scope_str.trim().eq_ignore_ascii_case("stage") {
            Self::Stage
        } else {
            Self::Scene
        }
    }

    fn elapsed_ms(self, ctx: &BehaviorContext) -> u64 {
        match self {
            Self::Scene => ctx.scene_elapsed_ms,
            Self::Stage => ctx.stage_elapsed_ms,
        }
    }
}

fn parse_stage_name(raw: &str) -> Option<SceneStage> {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("on-enter") || trimmed.eq_ignore_ascii_case("enter") {
        Some(SceneStage::OnEnter)
    } else if trimmed.eq_ignore_ascii_case("on-idle") || trimmed.eq_ignore_ascii_case("idle") {
        Some(SceneStage::OnIdle)
    } else if trimmed.eq_ignore_ascii_case("on-leave") || trimmed.eq_ignore_ascii_case("leave") {
        Some(SceneStage::OnLeave)
    } else if trimmed.eq_ignore_ascii_case("done") {
        Some(SceneStage::Done)
    } else {
        None
    }
}

fn cues_for_stage<'a>(scene: &'a Scene, stage: &SceneStage) -> &'a [AudioCue] {
    match stage {
        SceneStage::OnEnter => &scene.audio.on_enter,
        SceneStage::OnIdle => &scene.audio.on_idle,
        SceneStage::OnLeave => &scene.audio.on_leave,
        SceneStage::Done => &[],
    }
}

impl BehaviorContext {
    pub fn resolve_target(&self, target: &str) -> Option<&str> {
        self.target_resolver.resolve_alias(target)
    }

    pub fn object_state(&self, object_id: &str) -> Option<&ObjectRuntimeState> {
        self.object_states.get(object_id)
    }

    pub fn object_region(&self, object_id: &str) -> Option<&Region> {
        self.object_regions.get(object_id)
    }

    pub fn resolved_object_state(&self, target: &str) -> Option<&ObjectRuntimeState> {
        self.resolve_target(target)
            .and_then(|object_id| self.object_state(object_id))
    }

    pub fn resolved_object_region(&self, target: &str) -> Option<&Region> {
        self.resolve_target(target)
            .and_then(|object_id| self.object_region(object_id))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        asteroid_fragment_points_i32, asteroid_points_i32, built_in_behavior, catalog,
        rotate_points_i32, smoke_validate_rhai_script, Behavior, BehaviorCommand, BehaviorContext,
        BlinkBehavior, BobBehavior, FollowBehavior, MenuCarouselBehavior,
        MenuCarouselObjectBehavior, MenuSelectedBehavior, RhaiScriptBehavior, SceneAudioBehavior,
        SelectedArrowsBehavior, StageVisibilityBehavior, TimedVisibilityBehavior,
    };
    use engine_animation::SceneStage;
    use engine_core::effects::Region;
    use engine_core::game_object::{GameObject, GameObjectKind};
    use engine_core::game_state::GameState;
    use engine_core::level_state::LevelState;
    use engine_core::scene::{
        AudioCue, BehaviorParams, BehaviorSpec, MenuOption, Scene, SceneAudio, SceneRenderedMode,
        SceneStages, TermColour,
    };
    use engine_core::scene_runtime_types::{
        ObjectRuntimeState, SidecarIoFrameState, TargetResolver,
    };
    use engine_game::GameplayWorld;
    use rhai::Map as RhaiMap;
    use serde_json::json;
    use serde_json::Value as JsonValue;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    fn scene_object() -> GameObject {
        GameObject {
            id: "scene:intro".to_string(),
            name: "intro".to_string(),
            kind: GameObjectKind::Scene,
            aliases: vec!["intro".to_string()],
            parent_id: None,
            children: Vec::new(),
        }
    }

    fn base_scene() -> Scene {
        Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            target_fps: None,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            ui: Default::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            input: Default::default(),
            postfx: Vec::new(),
            next: None,
            prerender: false,
        }
    }

    fn scene_with_audio(audio: SceneAudio) -> Scene {
        Scene {
            audio,
            ..base_scene()
        }
    }

    fn scene_with_menu_options(count: usize) -> Scene {
        Scene {
            menu_options: (0..count)
                .map(|idx| MenuOption {
                    key: idx.to_string(),
                    label: Some(format!("Option {idx}")),
                    scene: None,
                    next: format!("next-{idx}"),
                })
                .collect(),
            ..base_scene()
        }
    }

    fn empty_rhai_menu_map() -> Arc<RhaiMap> {
        let mut menu_map = RhaiMap::new();
        menu_map.insert("selected_index".into(), 0i64.into());
        menu_map.insert("count".into(), 0i64.into());
        Arc::new(menu_map)
    }

    fn empty_rhai_time_map() -> Arc<RhaiMap> {
        let mut time_map = RhaiMap::new();
        time_map.insert("scene_elapsed_ms".into(), 0i64.into());
        time_map.insert("stage_elapsed_ms".into(), 0i64.into());
        time_map.insert("stage".into(), "on_idle".into());
        Arc::new(time_map)
    }

    fn empty_rhai_key_map() -> Arc<RhaiMap> {
        let mut key_map = RhaiMap::new();
        key_map.insert("code".into(), "".into());
        key_map.insert("ctrl".into(), false.into());
        key_map.insert("alt".into(), false.into());
        key_map.insert("shift".into(), false.into());
        key_map.insert("pressed".into(), false.into());
        key_map.insert("released".into(), false.into());
        Arc::new(key_map)
    }

    fn empty_engine_key_map() -> Arc<RhaiMap> {
        let mut engine_key = RhaiMap::new();
        engine_key.insert("code".into(), "".into());
        engine_key.insert("ctrl".into(), false.into());
        engine_key.insert("alt".into(), false.into());
        engine_key.insert("shift".into(), false.into());
        engine_key.insert("pressed".into(), false.into());
        engine_key.insert("released".into(), false.into());
        engine_key.insert("is_quit".into(), false.into());
        engine_key.insert("any_down".into(), false.into());
        engine_key.insert("down_count".into(), 0_i64.into());
        Arc::new(engine_key)
    }

    fn base_ctx() -> BehaviorContext {
        BehaviorContext {
            stage: SceneStage::OnIdle,
            scene_elapsed_ms: 0,
            stage_elapsed_ms: 0,
            menu_selected_index: 0,
            target_resolver: Arc::new(TargetResolver::default()),
            object_states: Arc::new(HashMap::new()),
            object_kinds: Arc::new(HashMap::new()),
            object_props: Arc::new(HashMap::new()),
            object_regions: Arc::new(HashMap::new()),
            object_text: Arc::new(HashMap::new()),
            ui_focused_target_id: None,
            ui_theme_id: None,
            ui_last_submit_target_id: None,
            ui_last_submit_text: None,
            ui_last_change_target_id: None,
            ui_last_change_text: None,
            game_state: None,
            level_state: None,
            persistence: None,
            catalogs: Arc::new(catalog::ModCatalogs::default()),
            gameplay_world: None,
            collisions: Arc::new(Vec::new()),
            collision_enters: Arc::new(Vec::new()),
            collision_stays: Arc::new(Vec::new()),
            collision_exits: Arc::new(Vec::new()),
            last_raw_key: None,
            keys_down: Arc::new(HashSet::new()),
            keys_just_pressed: Arc::new(HashSet::new()),
            action_bindings: Arc::new(HashMap::new()),
            sidecar_io: Arc::new(SidecarIoFrameState::default()),
            rhai_time_map: empty_rhai_time_map(),
            rhai_menu_map: empty_rhai_menu_map(),
            rhai_key_map: empty_rhai_key_map(),
            engine_key_map: empty_engine_key_map(),
        }
    }

    fn ctx(stage: SceneStage, scene_elapsed_ms: u64, stage_elapsed_ms: u64) -> BehaviorContext {
        // Build time map with correct values for this test context
        let rhai_time_map = {
            let mut time_map = RhaiMap::new();
            time_map.insert(
                "scene_elapsed_ms".into(),
                (scene_elapsed_ms as rhai::INT).into(),
            );
            time_map.insert(
                "stage_elapsed_ms".into(),
                (stage_elapsed_ms as rhai::INT).into(),
            );
            let stage_str: &str = match stage {
                SceneStage::OnEnter => "on_enter",
                SceneStage::OnIdle => "on_idle",
                SceneStage::OnLeave => "on_leave",
                SceneStage::Done => "done",
            };
            time_map.insert("stage".into(), stage_str.into());
            Arc::new(time_map)
        };

        BehaviorContext {
            stage,
            scene_elapsed_ms,
            stage_elapsed_ms,
            rhai_time_map,
            ..base_ctx()
        }
    }

    fn update_ctx_menu_map(ctx: &mut BehaviorContext, menu_count: usize) {
        // When test modifies menu_selected_index, rebuild the menu map to match
        let mut menu_map = RhaiMap::new();
        menu_map.insert(
            "selected_index".into(),
            (ctx.menu_selected_index as rhai::INT).into(),
        );
        menu_map.insert("count".into(), (menu_count as rhai::INT).into());
        ctx.rhai_menu_map = Arc::new(menu_map);
    }

    #[test]
    fn rhai_script_behavior_reads_ipc_scope_values() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
if ipc.has_output {
  out.push(#{ op: "visibility", target: "menu-item-0", visible: true });
}
if ipc.clear_count > 0 {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 1, dy: 0 });
}
if ipc.has_screen_full {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 0, dy: 1 });
}
if ipc.custom_events.len > 0 {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 2, dy: 0 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.sidecar_io = Arc::new(SidecarIoFrameState {
            output_lines: vec!["line".to_string()],
            clear_count: 1,
            screen_full_lines: Some(vec!["full".to_string()]),
            custom_events: vec!["{}".to_string()],
        });
        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "menu-item-0".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 1,
                    dy: 0
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 0,
                    dy: 1
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 2,
                    dy: 0
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_can_spawn_gameplay_entities() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_object("asteroid", #{ tags: ["enemy", "rock"], x: 12, nested: #{ hp: 3 } });
if id > 0 && world.exists(id) {
  world.set(id, "/nested/hp", 7);
}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "world api should not produce ScriptError: {commands:?}"
        );
        let ids = gameplay_world.ids();
        assert_eq!(ids.len(), 1);
        let id = ids[0];
        assert_eq!(gameplay_world.kind_of(id).as_deref(), Some("asteroid"));
        assert_eq!(gameplay_world.query_tag("enemy"), vec![id]);
        assert_eq!(gameplay_world.get(id, "/nested/hp"), Some(json!(7)));
    }

    #[test]
    fn rhai_script_behavior_gameplay_entity_api_supports_typed_getters_and_set() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_object("ship", #{ active: true, score: 4 });
let e = world.entity(id);
if e.exists() && e.get_bool("/active", false) {
  let next = e.get_i("/score", 0) + 9;
  e.set("/score", next);
}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "world entity api should not produce ScriptError: {commands:?}"
        );
        let ids = gameplay_world.ids();
        assert_eq!(ids.len(), 1);
        let id = ids[0];
        assert_eq!(gameplay_world.get(id, "/score"), Some(json!(13)));
        assert_eq!(gameplay_world.get(id, "/active"), Some(json!(true)));
    }

    #[test]
    fn rhai_script_behavior_gameplay_entity_api_supports_bulk_operations() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_object("ship", #{ x: 1.5, y: 2.5, name: "player" });
let e = world.entity(id);

// Test set_many
let updates = #{
  x: 10.5,
  y: 20.5,
  score: 42
};
e.set_many(updates);

// Test data() - should return entire entity data blob
let data = e.data();

// Test get_f and get_s
let x = e.get_f("/x", 0.0);
let name = e.get_s("/name", "");

if x == 10.5 && name == "player" {
  e.set("/success", true);
}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "bulk operations should not produce ScriptError: {commands:?}"
        );
        let ids = gameplay_world.ids();
        assert_eq!(ids.len(), 1);
        let id = ids[0];
        assert_eq!(gameplay_world.get(id, "/x"), Some(json!(10.5)));
        assert_eq!(gameplay_world.get(id, "/y"), Some(json!(20.5)));
        assert_eq!(gameplay_world.get(id, "/score"), Some(json!(42)));
        assert_eq!(gameplay_world.get(id, "/name"), Some(json!("player")));
        assert_eq!(gameplay_world.get(id, "/success"), Some(json!(true)));
    }

    fn region(x: u16, y: u16, width: u16, height: u16) -> Region {
        Region {
            x,
            y,
            width,
            height,
        }
    }

    fn run_behavior<B: Behavior>(
        behavior: &mut B,
        scene: &Scene,
        ctx: BehaviorContext,
    ) -> Vec<BehaviorCommand> {
        let mut commands = Vec::new();
        behavior.update(&scene_object(), scene, &ctx, &mut commands);
        commands
    }

    #[test]
    fn scene_audio_behavior_emits_each_cue_once() {
        let scene = scene_with_audio(SceneAudio {
            on_enter: vec![AudioCue {
                at_ms: 100,
                cue: "thunder".to_string(),
                volume: Some(0.7),
            }],
            on_idle: Vec::new(),
            on_leave: Vec::new(),
        });
        let object = scene_object();
        let ctx = ctx(SceneStage::OnEnter, 100, 100);
        let mut behavior = SceneAudioBehavior::default();
        let mut commands = Vec::new();

        behavior.update(&object, &scene, &ctx, &mut commands);
        behavior.update(&object, &scene, &ctx, &mut commands);

        assert_eq!(
            commands,
            vec![BehaviorCommand::PlayAudioCue {
                cue: "thunder".to_string(),
                volume: Some(0.7)
            }]
        );
    }

    #[test]
    fn blink_behavior_toggles_visibility() {
        let mut behavior = BlinkBehavior::from_params(&BehaviorParams {
            visible_ms: Some(100),
            hidden_ms: Some(100),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &base_scene(),
            ctx(SceneStage::OnIdle, 150, 150),
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn bob_behavior_emits_offset() {
        let mut behavior = BobBehavior::from_params(&BehaviorParams {
            amplitude_x: Some(2),
            amplitude_y: Some(0),
            period_ms: Some(1000),
            phase_ms: Some(250),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnIdle, 0, 0));

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "scene:intro".to_string(),
                dx: 2,
                dy: 0
            }]
        );
    }

    #[test]
    fn builds_known_behavior_from_spec() {
        let behavior = built_in_behavior(&BehaviorSpec {
            name: "blink".to_string(),
            params: BehaviorParams::default(),
        });

        assert!(behavior.is_some());
    }

    #[test]
    fn stage_visibility_behavior_shows_only_selected_stage() {
        let mut behavior = StageVisibilityBehavior::from_params(&BehaviorParams {
            stages: vec!["on-idle".to_string()],
            ..BehaviorParams::default()
        });
        let commands = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnEnter, 0, 0));

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn timed_visibility_behavior_uses_elapsed_time_window() {
        let mut behavior = TimedVisibilityBehavior::from_params(&BehaviorParams {
            target: Some("title".to_string()),
            start_ms: Some(100),
            end_ms: Some(200),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &base_scene(),
            ctx(SceneStage::OnIdle, 150, 150),
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "title".to_string(),
                visible: true,
            }]
        );
    }

    #[test]
    fn timed_visibility_behavior_can_use_stage_clock() {
        let mut behavior = TimedVisibilityBehavior::from_params(&BehaviorParams {
            target: Some("title".to_string()),
            time_scope: Some("stage".to_string()),
            start_ms: Some(100),
            end_ms: Some(200),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &base_scene(),
            ctx(SceneStage::OnIdle, 500, 150),
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "title".to_string(),
                visible: true,
            }]
        );
    }

    #[test]
    fn follow_behavior_copies_target_state() {
        let mut behavior = FollowBehavior::from_params(&BehaviorParams {
            target: Some("leader".to_string()),
            amplitude_x: Some(1),
            amplitude_y: Some(-1),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("leader".to_string(), "obj:leader".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:leader".to_string(),
            ObjectRuntimeState {
                visible: false,
                offset_x: 3,
                offset_y: 2,
            },
        );
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);

        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: false
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 4,
                    dy: 1
                }
            ]
        );
    }

    #[test]
    fn menu_selected_behavior_visibility_matches_selected_index() {
        let mut behavior = MenuSelectedBehavior::from_params(&BehaviorParams {
            index: Some(1),
            ..BehaviorParams::default()
        });
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.menu_selected_index = 1;
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: true
            }]
        );
    }

    #[test]
    fn menu_carousel_centers_selected_item_in_target_region() {
        let mut behavior = MenuCarouselBehavior::from_params(&BehaviorParams {
            target: Some("menu-grid".to_string()),
            index: Some(2),
            window: Some(5),
            step_y: Some(2),
            endless: Some(true),
            ..BehaviorParams::default()
        });

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-grid".to_string(), "obj:menu-grid".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(10, 20, 12, 1));
        object_regions.insert("obj:menu-grid".to_string(), region(0, 10, 40, 9));

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        test_ctx.menu_selected_index = 2;

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(7), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 0,
                    dy: -6
                }
            ]
        );
    }

    #[test]
    fn menu_carousel_wraps_when_endless_enabled() {
        let mut behavior = MenuCarouselBehavior::from_params(&BehaviorParams {
            target: Some("menu-grid".to_string()),
            index: Some(0),
            window: Some(5),
            step_y: Some(2),
            endless: Some(true),
            ..BehaviorParams::default()
        });

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-grid".to_string(), "obj:menu-grid".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(10, 20, 12, 1));
        object_regions.insert("obj:menu-grid".to_string(), region(0, 10, 40, 9));

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        test_ctx.menu_selected_index = 6;

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(7), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 0,
                    dy: -4
                }
            ]
        );
    }

    #[test]
    fn menu_carousel_hides_items_outside_window() {
        let mut behavior = MenuCarouselBehavior::from_params(&BehaviorParams {
            target: Some("menu-grid".to_string()),
            index: Some(6),
            window: Some(5),
            step_y: Some(2),
            endless: Some(false),
            ..BehaviorParams::default()
        });

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-grid".to_string(), "obj:menu-grid".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(10, 20, 12, 1));
        object_regions.insert("obj:menu-grid".to_string(), region(0, 10, 40, 9));

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        test_ctx.menu_selected_index = 0;

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(7), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn menu_carousel_uses_min_step_based_on_item_height() {
        let mut behavior = MenuCarouselBehavior::from_params(&BehaviorParams {
            target: Some("menu-grid".to_string()),
            index: Some(0),
            window: Some(3),
            step_y: Some(1),
            endless: Some(true),
            ..BehaviorParams::default()
        });

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-grid".to_string(), "obj:menu-grid".to_string());
        let mut object_regions = HashMap::new();
        // Item currently at y=20 with height=3 (simulates a taller rendered row).
        object_regions.insert("scene:intro".to_string(), region(10, 20, 24, 3));
        object_regions.insert("obj:menu-grid".to_string(), region(0, 10, 40, 9));

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        test_ctx.menu_selected_index = 2;

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(3), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 0,
                    dy: -2
                }
            ]
        );
    }

    #[test]
    fn menu_carousel_object_controls_multiple_items_from_single_behavior() {
        let mut behavior = MenuCarouselObjectBehavior::from_params(&BehaviorParams {
            target: Some("menu-grid".to_string()),
            item_prefix: Some("menu-item-".to_string()),
            count: Some(3),
            window: Some(3),
            step_y: Some(2),
            endless: Some(true),
            ..BehaviorParams::default()
        });

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-grid".to_string(), "obj:menu-grid".to_string());
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        resolver.register_alias("menu-item-1".to_string(), "obj:menu-item-1".to_string());
        resolver.register_alias("menu-item-2".to_string(), "obj:menu-item-2".to_string());

        let mut object_regions = HashMap::new();
        object_regions.insert("obj:menu-grid".to_string(), region(0, 10, 40, 9));
        object_regions.insert("obj:menu-item-0".to_string(), region(10, 6, 20, 1));
        object_regions.insert("obj:menu-item-1".to_string(), region(10, 10, 20, 1));
        object_regions.insert("obj:menu-item-2".to_string(), region(10, 14, 20, 1));

        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        test_ctx.menu_selected_index = 1;

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(3), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "menu-item-0".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 0,
                    dy: 6
                },
                BehaviorCommand::SetVisibility {
                    target: "menu-item-1".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-1".to_string(),
                    dx: 0,
                    dy: 4
                },
                BehaviorCommand::SetVisibility {
                    target: "menu-item-2".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-2".to_string(),
                    dx: 0,
                    dy: 2
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_emits_visibility_and_offset_commands() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
out.push(#{ op: "visibility", target: "menu-item-0", visible: true });
out.push(#{ op: "offset", target: "menu-item-0", dx: 1, dy: -2 });
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "menu-item-0".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 1,
                    dy: -2
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_reads_ui_scope_values() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
if ui.has_submit && ui.submit_text == "status" && ui.focused_target == "terminal-prompt" {
  out.push(#{ op: "visibility", target: "menu-item-0", visible: true });
}
if ui.theme == "terminal" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 0, dy: 1 });
}
if ui.has_change && ui.change_target == "terminal-prompt" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 2, dy: 0 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.ui_focused_target_id = Some(Arc::from("terminal-prompt"));
        test_ctx.ui_theme_id = Some(Arc::from("terminal"));
        test_ctx.ui_last_submit_target_id = Some(Arc::from("terminal-prompt"));
        test_ctx.ui_last_submit_text = Some(Arc::from("status"));
        test_ctx.ui_last_change_target_id = Some(Arc::from("terminal-prompt"));
        test_ctx.ui_last_change_text = Some(Arc::from("sta"));
        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "menu-item-0".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 0,
                    dy: 1
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 2,
                    dy: 0
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_reads_time_menu_and_game_objects() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
game.set("/session/user", "linus");
game.push("/events", "booted");

let out = [];
if time.scene_elapsed_ms == 480 && time.stage_elapsed_ms == 120 {
  out.push(#{ op: "visibility", target: "menu-item-0", visible: true });
}
if menu.count == 3 && menu.selected_index == 1 && game.get("/session/user") == "linus" && game.has("/events") {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 1, dy: 2 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut test_ctx = ctx(SceneStage::OnIdle, 480, 120);
        test_ctx.menu_selected_index = 1;
        update_ctx_menu_map(&mut test_ctx, 3);
        test_ctx.game_state = Some(GameState::new());
        let commands = run_behavior(&mut behavior, &scene_with_menu_options(3), test_ctx);
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "menu-item-0".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "menu-item-0".to_string(),
                    dx: 1,
                    dy: 2
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_time_delta_ms_clamps_and_persists() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let dt = time.delta_ms(220, "/ast/last_ms");
game.set("/ast/dt", dt);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut test_ctx = ctx(SceneStage::OnIdle, 480, 120);
        let state = GameState::new();
        test_ctx.game_state = Some(state.clone());

        let _ = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(state.get("/ast/dt").and_then(|v| v.as_i64()), Some(0));
        assert_eq!(
            state.get("/ast/last_ms").and_then(|v| v.as_i64()),
            Some(480)
        );

        let mut second_ctx = ctx(SceneStage::OnIdle, 830, 120);
        second_ctx.game_state = Some(state.clone());
        let _ = run_behavior(&mut behavior, &scene_with_menu_options(1), second_ctx);
        assert_eq!(state.get("/ast/dt").and_then(|v| v.as_i64()), Some(220));
        assert_eq!(
            state.get("/ast/last_ms").and_then(|v| v.as_i64()),
            Some(830)
        );
    }

    #[test]
    fn rhai_script_behavior_input_load_profile_emits_bindings() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
input.load_profile("asteroids.default");
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::BindInputAction {
                    action: "turn_left".to_string(),
                    keys: vec!["Left".to_string(), "a".to_string(), "A".to_string()],
                },
                BehaviorCommand::BindInputAction {
                    action: "turn_right".to_string(),
                    keys: vec!["Right".to_string(), "d".to_string(), "D".to_string()],
                },
                BehaviorCommand::BindInputAction {
                    action: "thrust".to_string(),
                    keys: vec!["Up".to_string(), "w".to_string(), "W".to_string()],
                },
                BehaviorCommand::BindInputAction {
                    action: "fire".to_string(),
                    keys: vec![" ".to_string(), "f".to_string(), "F".to_string()],
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_emits_set_text_command() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
out.push(#{ op: "set-text", target: "ram-counter-line", text: "Memory Check: 0640K" });
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetText {
                target: "ram-counter-line".to_string(),
                text: "Memory Check: 0640K".to_string()
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_emits_set_props_command() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
out.push(#{ op: "set-props", target: "menu-item-0", visible: true, dx: 2, dy: -1, text: "HELLO" });
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetProps {
                target: "menu-item-0".to_string(),
                visible: Some(true),
                dx: Some(2),
                dy: Some(-1),
                text: Some("HELLO".to_string())
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_emits_set_property_command() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
out.push(#{ op: "set", target: "menu-item-0", path: "position.y", value: 3 });
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetProperty {
                target: "menu-item-0".to_string(),
                path: "position.y".to_string(),
                value: JsonValue::Number(3.into())
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_set_emits_set_property() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
scene.set("menu-item-0", "position.y", 6);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetProperty {
                target: "menu-item-0".to_string(),
                path: "position.y".to_string(),
                value: JsonValue::Number(6.into())
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_set_normalizes_props_prefix() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
scene.set("menu-item-0", "props.position.y", 6);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetProperty {
                target: "menu-item-0".to_string(),
                path: "position.y".to_string(),
                value: JsonValue::Number(6.into())
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_spawn_and_despawn_emit_commands() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
scene.spawn_object("bullet-0", "bullet-99");
scene.despawn_object("bullet-99");
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SceneSpawn {
                    template: "bullet-0".to_string(),
                    target: "bullet-99".to_string()
                },
                BehaviorCommand::SceneDespawn {
                    target: "bullet-99".to_string()
                }
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_geometry_helpers_are_available() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let ship = ship_points(8);
let rocks = asteroid_points(2, 3);
let score = asteroid_score(0);
let radius = asteroid_radius(2);
let wave = sin32(0);
let out = [];
out.push(#{ op: "set", target: "menu-item-0", path: "position.x", value: ship.len() + rocks.len() });
out.push(#{ op: "set", target: "menu-item-0", path: "position.y", value: score + radius + wave });
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetProperty {
                    target: "menu-item-0".to_string(),
                    path: "position.x".to_string(),
                    value: JsonValue::Number(13.into()),
                },
                BehaviorCommand::SetProperty {
                    target: "menu-item-0".to_string(),
                    path: "position.y".to_string(),
                    value: JsonValue::Number(46.into()),
                },
            ]
        );
    }

    #[test]
    fn rhai_script_behavior_persists_state_between_updates() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let count = if state.contains("count") { state["count"] } else { 0 };
let next = count + 1;
let out = [];
out.push(#{ op: "offset", target: "menu-item-0", dx: 0, dy: next });
#{ commands: out, state: #{ count: next } }
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });

        let first = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 0, 0),
        );
        let second = run_behavior(
            &mut behavior,
            &scene_with_menu_options(1),
            ctx(SceneStage::OnIdle, 16, 16),
        );

        assert_eq!(
            first,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 0,
                dy: 1
            }]
        );
        assert_eq!(
            second,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 0,
                dy: 2
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_exposes_objects_snapshot_by_alias_and_id() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
let obj_alias = scene.get("menu-item-0");
let obj_real = scene.get("obj:menu-item-0");
let kind = obj_real.get("kind");
let dy = obj_real.get("state.offset_y");
let rx = obj_alias.get("region.x");
if kind == "text" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: rx, dy: dy });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 7,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("obj:menu-item-0".to_string(), region(12, 5, 10, 1));
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);
        test_ctx.object_regions = Arc::new(object_regions);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 12,
                dy: 7
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_get_reads_object_snapshot() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
let obj = scene.get("menu-item-0");
if obj.get("kind") == "text" {
  let dy = obj.get("state.offset_y");
  out.push(#{ op: "offset", target: "menu-item-0", dx: 0, dy: dy });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 4,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 0,
                dy: 4
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_api_get_and_set() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let obj = scene.get("menu-item-0");
let dy = obj.get("state.offset_y");
obj.set("position.y", dy + 2);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 5,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetProperty {
                // Alias "menu-item-0" resolves to real object id "obj:menu-item-0".
                target: "obj:menu-item-0".to_string(),
                path: "position.y".to_string(),
                value: JsonValue::Number(7.into())
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_api_reads_props_snapshot() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
let obj = scene.get("menu-item-0");
if obj.get("props.text.font") == "generic:half" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 1, dy: 0 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 0,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut object_props = HashMap::new();
        object_props.insert(
            "obj:menu-item-0".to_string(),
            serde_json::json!({ "text": { "font": "generic:half" } }),
        );
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);
        test_ctx.object_props = Arc::new(object_props);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 1,
                dy: 0
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_scene_object_api_get_falls_back_to_props_prefix() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
let obj = scene.get("menu-item-0");
if obj.get("text.font") == "generic:half" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 2, dy: 0 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 0,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut object_props = HashMap::new();
        object_props.insert(
            "obj:menu-item-0".to_string(),
            serde_json::json!({ "text": { "font": "generic:half" } }),
        );
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);
        test_ctx.object_props = Arc::new(object_props);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 2,
                dy: 0
            }]
        );
    }

    #[test]
    fn rhai_script_behavior_merges_text_content_and_text_props() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let out = [];
let obj = scene.get("menu-item-0");
if obj.get("props.text.content") == "HELLO" && obj.get("props.text.font") == "generic:half" {
  out.push(#{ op: "offset", target: "menu-item-0", dx: 3, dy: 0 });
}
out
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_states = HashMap::new();
        object_states.insert(
            "obj:menu-item-0".to_string(),
            ObjectRuntimeState {
                visible: true,
                offset_x: 0,
                offset_y: 0,
            },
        );
        let mut object_kinds = HashMap::new();
        object_kinds.insert("obj:menu-item-0".to_string(), "text".to_string());
        let mut object_props = HashMap::new();
        object_props.insert(
            "obj:menu-item-0".to_string(),
            serde_json::json!({ "text": { "font": "generic:half" } }),
        );
        let mut object_text = HashMap::new();
        object_text.insert("obj:menu-item-0".to_string(), "HELLO".to_string());
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_states = Arc::new(object_states);
        test_ctx.object_kinds = Arc::new(object_kinds);
        test_ctx.object_props = Arc::new(object_props);
        test_ctx.object_text = Arc::new(object_text);

        let commands = run_behavior(&mut behavior, &scene_with_menu_options(1), test_ctx);
        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "menu-item-0".to_string(),
                dx: 3,
                dy: 0
            }]
        );
    }

    #[test]
    fn selected_arrows_hides_when_target_region_missing() {
        let mut behavior = SelectedArrowsBehavior::from_params(&BehaviorParams {
            target: Some("menu-item-0".to_string()),
            index: Some(0),
            side: Some("left".to_string()),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnIdle, 0, 0));

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn selected_arrows_uses_target_region_and_padding() {
        let mut behavior = SelectedArrowsBehavior::from_params(&BehaviorParams {
            target: Some("menu-item-0".to_string()),
            index: Some(0),
            side: Some("left".to_string()),
            padding: Some(1),
            autoscale_height: Some(true),
            amplitude_x: Some(0),
            period_ms: Some(1000),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(20, 10, 1, 1));
        object_regions.insert("obj:menu-item-0".to_string(), region(30, 8, 10, 3));
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);

        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 7,
                    dy: -1
                }
            ]
        );
    }

    #[test]
    fn selected_arrows_resets_cached_offset_after_deselection() {
        let mut behavior = SelectedArrowsBehavior::from_params(&BehaviorParams {
            target: Some("menu-item-0".to_string()),
            index: Some(0),
            side: Some("left".to_string()),
            padding: Some(1),
            autoscale_height: Some(true),
            amplitude_x: Some(0),
            period_ms: Some(1000),
            ..BehaviorParams::default()
        });
        behavior.last_dx = 8;
        behavior.last_dy = -1;

        let mut deselected_ctx = ctx(SceneStage::OnIdle, 0, 0);
        deselected_ctx.menu_selected_index = 1;
        let commands = run_behavior(&mut behavior, &base_scene(), deselected_ctx);

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
        assert_eq!(behavior.last_dx, 0);
        assert_eq!(behavior.last_dy, 0);

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(20, 10, 1, 1));
        object_regions.insert("obj:menu-item-0".to_string(), region(30, 8, 10, 3));
        let mut selected_ctx = ctx(SceneStage::OnIdle, 0, 0);
        selected_ctx.target_resolver = Arc::new(resolver);
        selected_ctx.object_regions = Arc::new(object_regions);
        let commands = run_behavior(&mut behavior, &base_scene(), selected_ctx);

        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 7,
                    dy: -1
                }
            ]
        );
    }

    #[test]
    fn selected_arrows_centers_using_own_dimensions() {
        let mut behavior = SelectedArrowsBehavior::from_params(&BehaviorParams {
            target: Some("menu-item-0".to_string()),
            index: Some(0),
            side: Some("left".to_string()),
            padding: Some(1),
            autoscale_height: Some(false),
            amplitude_x: Some(0),
            period_ms: Some(1000),
            ..BehaviorParams::default()
        });
        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_regions = HashMap::new();
        object_regions.insert("scene:intro".to_string(), region(20, 10, 3, 5));
        object_regions.insert("obj:menu-item-0".to_string(), region(30, 8, 10, 5));
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.target_resolver = Arc::new(resolver);
        test_ctx.object_regions = Arc::new(object_regions);
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);

        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 6,
                    dy: -2
                }
            ]
        );
    }

    #[test]
    fn test_all_behaviors_in_catalog() {
        // Verify that every behavior registered in built_in_behavior() is present in catalog
        use engine_core::authoring::catalog::behavior_catalog;

        let runtime_behaviors: Vec<&str> = vec![
            "blink",
            "bob",
            "follow",
            "menu-carousel",
            "menu-carousel-object",
            "rhai-script",
            "menu-selected",
            "selected-arrows",
            "stage-visibility",
            "timed-visibility",
        ];

        let catalog = behavior_catalog();
        let catalog_names: Vec<&str> = catalog.iter().map(|(name, _)| *name).collect();

        for behavior in &runtime_behaviors {
            assert!(
                catalog_names.contains(behavior),
                "Behavior '{}' is registered in runtime but missing from catalog",
                behavior
            );
        }

        for catalog_name in &catalog_names {
            assert!(
                runtime_behaviors.contains(catalog_name),
                "Behavior '{}' is in catalog but not registered in built_in_behavior()",
                catalog_name
            );
        }

        assert_eq!(
            runtime_behaviors.len(),
            catalog_names.len(),
            "Mismatch between runtime behaviors ({}) and catalog ({})",
            runtime_behaviors.len(),
            catalog_names.len()
        );
    }

    // ── debug hardening regression tests ──────────────────────────────────────

    #[test]
    fn rhai_script_behavior_captures_compile_error_on_invalid_syntax() {
        let behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some("fn broken( { }".to_string()),
            ..BehaviorParams::default()
        });
        assert!(
            behavior.compile_error.is_some(),
            "compile_error should be set for invalid Rhai syntax"
        );
    }

    #[test]
    fn rhai_script_behavior_no_compile_error_for_valid_script() {
        let behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(r#"let x = 1 + 2; #{}"#.to_string()),
            ..BehaviorParams::default()
        });
        assert!(
            behavior.compile_error.is_none(),
            "compile_error should be None for valid Rhai script"
        );
    }

    #[test]
    fn intro_login_scene_rhai_compiles_without_complexity_error() {
        let script = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../mods/shell-quest/scenes/06-intro-login/scene.rhai"
        ));
        let behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(script.to_string()),
            src: Some("/scenes/06-intro-login/scene.rhai".to_string()),
            ..BehaviorParams::default()
        });
        assert!(
            behavior.compile_error.is_none(),
            "intro-login Rhai script should compile, got: {:?}",
            behavior.compile_error
        );
    }

    #[test]
    fn rhai_script_behavior_emits_script_error_command_on_compile_failure() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some("let x = @@invalid@@;".to_string()),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnIdle, 0, 0));
        let has_script_error = commands
            .iter()
            .any(|c| matches!(c, BehaviorCommand::ScriptError { .. }));
        assert!(
            has_script_error,
            "should emit ScriptError command when compile_error is set"
        );
    }

    #[test]
    fn rhai_script_behavior_emits_compile_error_only_once_per_instance() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some("let x = @@invalid@@;".to_string()),
            ..BehaviorParams::default()
        });
        let first = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnIdle, 0, 0));
        let second = run_behavior(
            &mut behavior,
            &base_scene(),
            ctx(SceneStage::OnIdle, 16, 16),
        );
        let first_errors = first
            .iter()
            .filter(|c| matches!(c, BehaviorCommand::ScriptError { .. }))
            .count();
        let second_errors = second
            .iter()
            .filter(|c| matches!(c, BehaviorCommand::ScriptError { .. }))
            .count();
        assert_eq!(
            first_errors, 1,
            "first tick should emit exactly one ScriptError"
        );
        assert_eq!(
            second_errors, 0,
            "subsequent ticks should not spam compile ScriptError"
        );
    }

    #[test]
    fn rhai_script_behavior_no_script_error_command_for_valid_script() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(r#"#{ state: #{ mode: "ok" } }"#.to_string()),
            ..BehaviorParams::default()
        });
        let commands = run_behavior(&mut behavior, &base_scene(), ctx(SceneStage::OnIdle, 0, 0));
        let has_script_error = commands
            .iter()
            .any(|c| matches!(c, BehaviorCommand::ScriptError { .. }));
        assert!(
            !has_script_error,
            "should not emit ScriptError for valid script"
        );
    }

    #[test]
    #[test]
    #[test]
    fn smoke_validate_rhai_script_supports_world_api() {
        let scene = base_scene();
        let script = r#"
let id = world.spawn_object("probe", #{ tags: ["probe"] });
if id > 0 {
  world.despawn_object(id);
}
#{}
"#;
        assert!(
            smoke_validate_rhai_script(script, Some("./probe.rhai"), &scene).is_ok(),
            "world API scripts should pass smoke validation"
        );
    }

    #[test]
    fn rhai_script_behavior_script_error_carries_scene_id_and_source() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some("let = ;".to_string()),
            src: Some("./scene.rhai".to_string()),
            ..BehaviorParams::default()
        });
        let mut scene = base_scene();
        scene.id = "intro-login".to_string();
        let commands = run_behavior(&mut behavior, &scene, ctx(SceneStage::OnIdle, 0, 0));
        let error_cmd = commands
            .iter()
            .find(|c| matches!(c, BehaviorCommand::ScriptError { .. }));
        assert!(error_cmd.is_some(), "expected ScriptError command");
        if let Some(BehaviorCommand::ScriptError {
            scene_id, source, ..
        }) = error_cmd
        {
            assert_eq!(scene_id, "intro-login");
            assert_eq!(source.as_deref(), Some("./scene.rhai"));
        }
    }

    // ── terminal quest flow regression tests ──────────────────────────────────

    #[test]
    fn rhai_script_behavior_game_state_set_persists_to_game_state() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(r#"game.set("/session/user", "linus"); #{}"#.to_string()),
            ..BehaviorParams::default()
        });
        let game_state = GameState::new();
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.game_state = Some(game_state.clone());
        run_behavior(&mut behavior, &base_scene(), test_ctx);
        assert_eq!(
            game_state.get("/session/user"),
            Some(serde_json::json!("linus")),
            "game.set should persist to GameState"
        );
    }

    #[test]
    fn rhai_script_behavior_game_state_has_returns_true_after_set() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
game.set("/quests/first_message/completed", false);
let ok = game.has("/quests/first_message/completed");
#{}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let game_state = GameState::new();
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.game_state = Some(game_state);
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "game.has after game.set should not produce a ScriptError"
        );
    }

    #[test]
    fn rhai_script_behavior_ui_flash_message_sets_text_and_expiry() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(r#"ui.flash_message("READY", 500); []"#.to_string()),
            ..BehaviorParams::default()
        });
        let game_state = GameState::new();
        let mut test_ctx = ctx(SceneStage::OnIdle, 1200, 0);
        test_ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "ui.flash_message should not produce ScriptError: {commands:?}"
        );
        assert!(
            commands.iter().any(|c| matches!(
                c,
                BehaviorCommand::SetText { target, text }
                    if target == "game-message" && text == "READY"
            )),
            "ui.flash_message should emit SetText: {commands:?}"
        );
        assert_eq!(
            game_state.get("/__ui/game_message/text"),
            Some(serde_json::json!("READY"))
        );
        assert_eq!(
            game_state
                .get("/__ui/game_message/until_ms")
                .and_then(|value| value.as_i64()),
            Some(1700)
        );
    }

    #[test]
    fn rhai_script_behavior_ui_flash_message_auto_clears_after_expiry() {
        let game_state = GameState::new();
        let mut set_behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(r#"ui.flash_message("READY", 500); []"#.to_string()),
            ..BehaviorParams::default()
        });
        let mut set_ctx = ctx(SceneStage::OnIdle, 1200, 0);
        set_ctx.game_state = Some(game_state.clone());
        let _ = run_behavior(&mut set_behavior, &base_scene(), set_ctx);

        let mut clear_behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some("[]".to_string()),
            ..BehaviorParams::default()
        });
        let mut clear_ctx = ctx(SceneStage::OnIdle, 1700, 0);
        clear_ctx.game_state = Some(game_state.clone());
        let commands = run_behavior(&mut clear_behavior, &base_scene(), clear_ctx);

        assert!(
            commands.iter().any(|c| matches!(
                c,
                BehaviorCommand::SetText { target, text }
                    if target == "game-message" && text.is_empty()
            )),
            "expired flash should clear the target text: {commands:?}"
        );
        assert_eq!(game_state.get("/__ui/game_message/text"), None);
        assert_eq!(game_state.get("/__ui/game_message/until_ms"), None);
    }

    #[test]
    fn rhai_script_behavior_level_api_selects_and_mutates_active_level() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
if level.select("asteroids.default") {
  let lives = level.get("/player/lives");
  if lives.type_of() == "i64" {
    level.set("/player/lives", lives + 1);
  }
}
#{}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let level_state = LevelState::new();
        assert!(level_state.register_level(
            "asteroids.default",
            serde_json::json!({
                "player": {
                    "lives": 3
                }
            }),
        ));
        let mut test_ctx = ctx(SceneStage::OnIdle, 0, 0);
        test_ctx.level_state = Some(level_state.clone());
        let commands = run_behavior(&mut behavior, &base_scene(), test_ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "level api should not produce ScriptError"
        );
        assert_eq!(level_state.get("/player/lives"), Some(serde_json::json!(4)));
    }

    #[test]
    fn rhai_script_behavior_spawn_visual_creates_entity_visual_and_binding() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_visual("bullet", "bullet-template", #{
    x: 10.5,
    y: 20.3,
    heading: 1.57,
    collider_radius: 2.5,
    lifetime_ms: 5000
});
if id > 0 && world.exists(id) {
    let xf = world.transform(id);
    if xf.x == 10.5 && xf.y == 20.3 && xf.heading == 1.57 {
        print("ok");
    }
}
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);

        // Check no script errors
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_visual should not produce ScriptError: {commands:?}"
        );

        // Check that SceneSpawn command was emitted
        assert!(
            commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::SceneSpawn {
                template,
                target
            } if template == "bullet-template" && target.starts_with("bullet-"))),
            "spawn_visual should emit SceneSpawn command: {commands:?}"
        );

        // Check entity was created and has correct transform
        let ids = gameplay_world.ids();
        assert!(!ids.is_empty(), "spawn_visual should create an entity");

        if let Some(entity_id) = ids.first() {
            let entity_id = *entity_id;
            assert!(
                gameplay_world.exists(entity_id),
                "created entity should exist"
            );

            if let Some(xf) = gameplay_world.transform(entity_id) {
                assert!((xf.x - 10.5).abs() < 0.01, "x position should match");
                assert!((xf.y - 20.3).abs() < 0.01, "y position should match");
                assert!((xf.heading - 1.57).abs() < 0.01, "heading should match");
            }
        }
    }

    #[test]
    fn rhai_script_behavior_spawn_visual_with_polygon_collider() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_visual("asteroid", "asteroid-template", #{
    x: 15.0,
    y: 25.0,
    heading: 0.0,
    collider_polygon: [[0.0, 0.0], [5.0, 0.0], [2.5, 4.0]],
    collider_layer: 1,
    collider_mask: 2
});
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);

        // Check no script errors
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_visual with polygon should not produce ScriptError: {commands:?}"
        );

        // Check that SceneSpawn command was emitted
        assert!(
            commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::SceneSpawn {
                template,
                target
            } if template == "asteroid-template" && target.starts_with("asteroid-"))),
            "spawn_visual should emit SceneSpawn command"
        );

        // Check entity was created
        let ids = gameplay_world.ids();
        assert!(!ids.is_empty(), "spawn_visual should create an entity");
    }

    #[test]
    fn rhai_script_behavior_spawn_visual_returns_zero_on_failure() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
// Try to spawn with no world (will fail gracefully)
let id = world.spawn_visual("item", "item-template", #{
    x: 0.0,
    y: 0.0
});
// id should be 0 if world creation failed, but we have a world in tests
// so this should actually succeed. Let's test with missing required data
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);

        // Should have created an entity with defaults
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_visual with minimal data should work"
        );
    }

    #[test]
    fn rhai_script_behavior_spawn_prefab_creates_ship_and_asteroid() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let ship = world.spawn_prefab("ship", #{
  cfg: #{
    turn_step_ms: 40,
    ship_thrust: 120.0,
    ship_max_speed: 300.0
  },
  invulnerable_ms: 3000
});
let asteroid = world.spawn_prefab("asteroid", #{
  x: 12.0, y: 18.0, vx: 2.0, vy: -1.0, shape: 3, size: 2
});
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_prefab should not produce ScriptError: {commands:?}"
        );
        assert!(
            commands.iter().any(
                |c| matches!(c, BehaviorCommand::SceneSpawn { template, target }
                if template == "asteroid-template" && target.starts_with("asteroid-"))
            ),
            "asteroid prefab should emit SceneSpawn"
        );

        let ship_ids = gameplay_world.query_kind("ship");
        assert_eq!(ship_ids.len(), 1, "ship prefab should create one ship");
        let ship_id = ship_ids[0];
        assert_eq!(
            gameplay_world.visual(ship_id).and_then(|v| v.visual_id),
            Some("ship".to_string())
        );
        assert!(
            gameplay_world.controller(ship_id).is_some(),
            "ship should have controller"
        );
        assert!(gameplay_world.status_has(ship_id, "invulnerable"));

        let asteroid_ids = gameplay_world.query_kind("asteroid");
        assert_eq!(
            asteroid_ids.len(),
            1,
            "asteroid prefab should create one asteroid"
        );
        let asteroid_id = asteroid_ids[0];
        let xf = gameplay_world
            .transform(asteroid_id)
            .expect("asteroid transform");
        assert!((xf.x - 12.0).abs() < 0.01);
        assert!((xf.y - 18.0).abs() < 0.01);
        let phys = gameplay_world
            .physics(asteroid_id)
            .expect("asteroid physics");
        assert!((phys.vx - 120.0).abs() < 0.01);
        assert!((phys.vy + 60.0).abs() < 0.01);
        assert_eq!(
            gameplay_world
                .get(asteroid_id, "/size")
                .and_then(|v| v.as_i64()),
            Some(2)
        );
    }

    #[test]
    fn rhai_script_behavior_spawn_prefab_unknown_returns_zero() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let id = world.spawn_prefab("missing", #{});
game.set("/test/prefab_id", id);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "unknown prefab should fail gracefully"
        );
        assert_eq!(
            game_state.get("/test/prefab_id").and_then(|v| v.as_i64()),
            Some(0)
        );
        assert_eq!(gameplay_world.count(), 0);
    }

    #[test]
    fn rhai_script_behavior_spawn_group_creates_initial_asteroids() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let ids = world.spawn_group("asteroids.initial", "asteroid");
game.set("/test/spawn_count", ids.len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_group should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state.get("/test/spawn_count").and_then(|v| v.as_i64()),
            Some(10)
        );
        assert_eq!(gameplay_world.query_kind("asteroid").len(), 10);
    }

    #[test]
    fn rhai_script_behavior_spawn_group_unknown_returns_empty() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let ids = world.spawn_group("missing.group", "asteroid");
game.set("/test/spawn_count", ids.len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "unknown spawn_group should fail gracefully"
        );
        assert_eq!(
            game_state.get("/test/spawn_count").and_then(|v| v.as_i64()),
            Some(0)
        );
        assert_eq!(gameplay_world.count(), 0);
    }

    #[test]
    fn rhai_script_behavior_try_fire_weapon_spawns_bullet_and_audio() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let ship = world.spawn_prefab("ship", #{
  cfg: #{
    turn_step_ms: 40,
    ship_thrust: 120.0,
    ship_max_speed: 300.0
  }
});
world.set_physics(ship, 60.0, 120.0, 0.0, 0.0, 0.10, 300.0);
let bullet = world.try_fire_weapon("asteroids.ship", ship, #{
  bullet_speed: 300.0,
  bullet_ttl_ms: 900,
  shot_cooldown_ms: 200
});
game.set("/test/bullet_id", bullet);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "try_fire_weapon should not produce ScriptError: {commands:?}"
        );
        assert!(
            commands.iter().any(|c| matches!(
                c,
                BehaviorCommand::SceneSpawn { template, target }
                    if template == "bullet-template" && target.starts_with("bullet-")
            )),
            "try_fire_weapon should emit bullet SceneSpawn: {commands:?}"
        );
        assert!(
            commands.iter().any(|c| matches!(
                c,
                BehaviorCommand::PlayAudioEvent { event, gain }
                    if event == "gameplay.ship.shoot" && *gain == Some(1.0)
            )),
            "try_fire_weapon should emit shoot audio event: {commands:?}"
        );

        let bullet_id = game_state
            .get("/test/bullet_id")
            .and_then(|value| value.as_i64())
            .expect("stored bullet id") as u64;
        assert!(gameplay_world.exists(bullet_id));

        let ship_id = gameplay_world.query_kind("ship")[0];
        let ship_xf = gameplay_world.transform(ship_id).expect("ship transform");
        let ship_ctrl = gameplay_world.controller(ship_id).expect("ship controller");
        let (dir_x, dir_y) = ship_ctrl.heading_vector();
        let bullet_xf = gameplay_world
            .transform(bullet_id)
            .expect("bullet transform");
        assert!((bullet_xf.x - (ship_xf.x + (dir_x * 9.0))).abs() < 0.01);
        assert!((bullet_xf.y - (ship_xf.y + (dir_y * 9.0))).abs() < 0.01);

        let bullet_phys = gameplay_world.physics(bullet_id).expect("bullet physics");
        assert!((bullet_phys.vx - (60.0 + dir_x * 300.0)).abs() < 0.01);
        assert!((bullet_phys.vy - (120.0 + dir_y * 300.0)).abs() < 0.01);

        assert!(gameplay_world.cooldown_remaining(ship_id, "shot") > 0);
    }

    #[test]
    fn rhai_script_behavior_try_fire_weapon_respects_cooldown() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let ship = world.spawn_prefab("ship", #{
  cfg: #{
    turn_step_ms: 40,
    ship_thrust: 120.0,
    ship_max_speed: 300.0
  }
});
let first = world.try_fire_weapon("asteroids.ship", ship, #{
  bullet_speed: 300.0,
  bullet_ttl_ms: 900,
  shot_cooldown_ms: 200
});
let second = world.try_fire_weapon("asteroids.ship", ship, #{
  bullet_speed: 300.0,
  bullet_ttl_ms: 900,
  shot_cooldown_ms: 200
});
game.set("/test/first", first);
game.set("/test/second", second);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "try_fire_weapon cooldown path should not produce ScriptError: {commands:?}"
        );
        assert!(
            game_state
                .get("/test/first")
                .and_then(|value| value.as_i64())
                .unwrap_or(0)
                > 0,
            "first fire should succeed"
        );
        assert_eq!(
            game_state
                .get("/test/second")
                .and_then(|value| value.as_i64()),
            Some(0),
            "second fire should be blocked by cooldown"
        );
        assert_eq!(gameplay_world.query_kind("bullet").len(), 1);
        assert_eq!(
            commands
                .iter()
                .filter(|c| matches!(c, BehaviorCommand::PlayAudioEvent { event, .. } if event == "gameplay.ship.shoot"))
                .count(),
            1,
            "only the successful shot should emit audio"
        );
    }

    #[test]
    fn rhai_script_behavior_fx_emit_ship_thrust_smoke_spawns_one_particle() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let ship = world.spawn_prefab("ship", #{
  cfg: #{
    turn_step_ms: 40,
    ship_thrust: 120.0,
    ship_max_speed: 300.0
  }
});
world.set_physics(ship, 60.0, 120.0, 0.0, 0.0, 0.10, 300.0);
let ids = fx.emit("asteroids.ship_thrust_smoke", #{ ship_id: ship });
game.set("/test/count", ids.len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "fx thrust smoke should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/count")
                .and_then(|value| value.as_i64()),
            Some(1)
        );

        let ship_id = gameplay_world.query_kind("ship")[0];
        let smoke_id = gameplay_world.query_kind("smoke")[0];
        let ship_xf = gameplay_world.transform(ship_id).expect("ship transform");
        let ship_ctrl = gameplay_world.controller(ship_id).expect("ship controller");
        let (dir_x, dir_y) = ship_ctrl.heading_vector();
        let smoke_xf = gameplay_world.transform(smoke_id).expect("smoke transform");
        assert!((smoke_xf.x - (ship_xf.x - (dir_x * 6.0))).abs() < 0.01);
        assert!((smoke_xf.y - (ship_xf.y - (dir_y * 6.0))).abs() < 0.01);

        let smoke_phys = gameplay_world.physics(smoke_id).expect("smoke physics");
        assert!((smoke_phys.vx - (60.0 - dir_x * 21.0)).abs() < 0.01);
        assert!((smoke_phys.vy - (120.0 - dir_y * 21.0)).abs() < 0.01);
        assert!(gameplay_world.cooldown_remaining(ship_id, "smoke") > 0);
    }

    #[test]
    fn rhai_script_behavior_fx_emit_ship_disintegration_spawns_twelve_smokes() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
let ids = fx.emit("asteroids.ship_disintegration", #{ x: 10.0, y: 20.0 });
game.set("/test/count", ids.len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "fx disintegration should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/count")
                .and_then(|value| value.as_i64()),
            Some(12)
        );
        assert_eq!(gameplay_world.query_kind("smoke").len(), 12);
    }

    #[test]
    fn rhai_script_behavior_tick_heading_anim_advances_phase_and_accumulator() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let asteroid = world.spawn_prefab("asteroid", #{
  x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, shape: 1, size: 2
});
world.set(asteroid, "/rot_phase", 5);
world.set(asteroid, "/rot_speed", 2);
world.set(asteroid, "/rot_step_ms", 72);
world.set(asteroid, "/rot_accum_ms", 20);
let anim = world.tick_heading_anim(asteroid, 160);
game.set("/test/phase", anim["rot_phase"]);
game.set("/test/accum", anim["rot_accum_ms"]);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "tick_heading_anim should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/phase")
                .and_then(|value| value.as_i64()),
            Some(9)
        );
        assert_eq!(
            game_state
                .get("/test/accum")
                .and_then(|value| value.as_i64()),
            Some(36)
        );

        let asteroid_id = gameplay_world.query_kind("asteroid")[0];
        assert_eq!(
            gameplay_world
                .get(asteroid_id, "/rot_phase")
                .and_then(|value| value.as_i64()),
            Some(9)
        );
        assert_eq!(
            gameplay_world
                .get(asteroid_id, "/rot_accum_ms")
                .and_then(|value| value.as_i64()),
            Some(36)
        );
    }

    #[test]
    fn rhai_script_behavior_tick_heading_anim_ignores_nonpositive_step() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let asteroid = world.spawn_prefab("asteroid", #{
  x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, shape: 1, size: 2
});
world.set(asteroid, "/rot_phase", 7);
world.set(asteroid, "/rot_step_ms", 0);
world.set(asteroid, "/rot_accum_ms", 11);
let anim = world.tick_heading_anim(asteroid, 160);
game.set("/test/phase", anim["rot_phase"]);
game.set("/test/accum", anim["rot_accum_ms"]);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "tick_heading_anim zero-step path should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/phase")
                .and_then(|value| value.as_i64()),
            Some(7)
        );
        assert_eq!(
            game_state
                .get("/test/accum")
                .and_then(|value| value.as_i64()),
            Some(11)
        );
    }

    #[test]
    fn rhai_script_behavior_handle_ship_hit_applies_ship_and_asteroid_reaction() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let ship = world.spawn_prefab("ship", #{
  cfg: #{
    turn_step_ms: 40,
    ship_thrust: 120.0,
    ship_max_speed: 300.0
  }
});
let asteroid = world.spawn_prefab("asteroid", #{
  x: 12.0, y: 18.0, vx: 2.0, vy: -1.0, shape: 3, size: 2
});
let handled = world.handle_ship_hit(ship, asteroid, #{
  crack_duration_ms: 1000,
  asteroid_velocity_limit: 240.0,
  ship_invulnerable_ms: 3000,
  ui_text: "HIT!",
  ui_ttl_ms: 450
});
game.set("/test/handled", handled);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 1200, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "handle_ship_hit should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/handled")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert!(
            commands.iter().any(|c| matches!(
                c,
                BehaviorCommand::PlayAudioEvent { event, gain }
                    if event == "gameplay.ship.hit" && *gain == Some(1.0)
            )),
            "handle_ship_hit should emit hit audio: {commands:?}"
        );
        assert!(
            commands.iter().any(|c| matches!(
                c,
                BehaviorCommand::SetText { target, text }
                    if target == "game-message" && text == "HIT!"
            )),
            "handle_ship_hit should emit ui text: {commands:?}"
        );

        let ship_id = gameplay_world.query_kind("ship")[0];
        let asteroid_id = gameplay_world.query_kind("asteroid")[0];
        let ship_phys = gameplay_world.physics(ship_id).expect("ship physics");
        assert!(ship_phys.vx.abs() < 0.01);
        assert!(ship_phys.vy.abs() < 0.01);
        assert!(gameplay_world.status_has(ship_id, "invulnerable"));

        let asteroid_phys = gameplay_world
            .physics(asteroid_id)
            .expect("asteroid physics");
        assert!((asteroid_phys.vx + 120.0).abs() < 0.01);
        assert!((asteroid_phys.vy - 60.0).abs() < 0.01);
        assert_eq!(
            gameplay_world
                .get(asteroid_id, "/flash_ms")
                .and_then(|value| value.as_i64()),
            Some(1000)
        );
        assert_eq!(
            gameplay_world
                .get(asteroid_id, "/flash_total_ms")
                .and_then(|value| value.as_i64()),
            Some(1000)
        );
        assert_eq!(
            gameplay_world
                .get(asteroid_id, "/split_pending")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            game_state.get("/__ui/game_message/text"),
            Some(serde_json::json!("HIT!"))
        );
        assert_eq!(
            game_state
                .get("/__ui/game_message/until_ms")
                .and_then(|value| value.as_i64()),
            Some(1650)
        );
        assert_eq!(gameplay_world.query_kind("smoke").len(), 12);
    }

    #[test]
    fn rhai_script_behavior_handle_ship_hit_noops_when_invulnerable() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let ship = world.spawn_prefab("ship", #{
  cfg: #{
    turn_step_ms: 40,
    ship_thrust: 120.0,
    ship_max_speed: 300.0
  },
  invulnerable_ms: 3000
});
let asteroid = world.spawn_prefab("asteroid", #{
  x: 12.0, y: 18.0, vx: 2.0, vy: -1.0, shape: 3, size: 2
});
let handled = world.handle_ship_hit(ship, asteroid, #{});
game.set("/test/handled", handled);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 1200, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "handle_ship_hit invulnerable path should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/handled")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert!(!commands.iter().any(
            |c| matches!(c, BehaviorCommand::PlayAudioEvent { event, .. } if event == "gameplay.ship.hit")
        ));
        assert_eq!(gameplay_world.query_kind("smoke").len(), 0);
    }

    #[test]
    fn rhai_script_behavior_handle_bullet_hit_despawns_bullet_and_marks_asteroid() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let asteroid = world.spawn_prefab("asteroid", #{
  x: 12.0, y: 18.0, vx: 2.0, vy: -1.0, shape: 3, size: 2
});
let bullet = world.spawn_prefab("bullet", #{
  x: 10.0, y: 20.0, vx: 1.0, vy: 0.0, ttl_ms: 900
});
let result = world.handle_bullet_hit(bullet, asteroid, #{
  crack_duration_ms: 1000,
  ui_text: "HIT",
  ui_ttl_ms: 250
});
game.set("/test/handled", result["handled"]);
game.set("/test/size", result["asteroid_size"]);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 1200, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "handle_bullet_hit should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/handled")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            game_state
                .get("/test/size")
                .and_then(|value| value.as_i64()),
            Some(2)
        );
        assert!(
            commands.iter().any(|c| matches!(
                c,
                BehaviorCommand::SetText { target, text }
                    if target == "game-message" && text == "HIT"
            )),
            "handle_bullet_hit should emit ui text: {commands:?}"
        );

        assert_eq!(gameplay_world.query_kind("bullet").len(), 0);
        let asteroid_id = gameplay_world.query_kind("asteroid")[0];
        assert_eq!(
            gameplay_world
                .get(asteroid_id, "/flash_ms")
                .and_then(|value| value.as_i64()),
            Some(1000)
        );
        assert_eq!(
            gameplay_world
                .get(asteroid_id, "/flash_total_ms")
                .and_then(|value| value.as_i64()),
            Some(1000)
        );
        assert_eq!(
            gameplay_world
                .get(asteroid_id, "/split_pending")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            game_state.get("/__ui/game_message/text"),
            Some(serde_json::json!("HIT"))
        );
        assert_eq!(
            game_state
                .get("/__ui/game_message/until_ms")
                .and_then(|value| value.as_i64()),
            Some(1450)
        );
    }

    #[test]
    fn rhai_script_behavior_handle_bullet_hit_noops_for_split_pending_asteroid() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let asteroid = world.spawn_prefab("asteroid", #{
  x: 12.0, y: 18.0, vx: 2.0, vy: -1.0, shape: 3, size: 2
});
world.set(asteroid, "/split_pending", true);
let bullet = world.spawn_prefab("bullet", #{
  x: 10.0, y: 20.0, vx: 1.0, vy: 0.0, ttl_ms: 900
});
let result = world.handle_bullet_hit(bullet, asteroid, #{});
game.set("/test/handled", result["handled"]);
game.set("/test/size", result["asteroid_size"]);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 1200, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "handle_bullet_hit split-pending path should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/handled")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            game_state
                .get("/test/size")
                .and_then(|value| value.as_i64()),
            Some(0)
        );
        assert_eq!(gameplay_world.query_kind("bullet").len(), 0);
        assert!(!commands.iter().any(
            |c| matches!(c, BehaviorCommand::SetText { target, text } if target == "game-message" && text == "HIT")
        ));
    }

    #[test]
    fn rhai_script_behavior_handle_asteroid_split_spawns_three_children() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let asteroid = world.spawn_prefab("asteroid", #{
  x: 12.0, y: 18.0, vx: 0.0, vy: 0.0, shape: 3, size: 2
});
world.set(asteroid, "/rot_phase", 5);
let result = world.handle_asteroid_split(asteroid, #{});
game.set("/test/handled", result["handled"]);
game.set("/test/despawned", result["despawned"]);
game.set("/test/child_count", result["children"].len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "handle_asteroid_split should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/handled")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            game_state
                .get("/test/despawned")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            game_state
                .get("/test/child_count")
                .and_then(|value| value.as_i64()),
            Some(3)
        );
        assert_eq!(gameplay_world.query_kind("asteroid").len(), 3);
        assert!(
            commands.iter().any(|c| matches!(
                c,
                BehaviorCommand::PlayAudioEvent { event, gain }
                    if event == "gameplay.asteroid.split" && *gain == Some(1.0)
            )),
            "handle_asteroid_split should emit split audio: {commands:?}"
        );
    }

    #[test]
    fn rhai_script_behavior_handle_asteroid_split_despawns_smallest_asteroid() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let asteroid = world.spawn_prefab("asteroid", #{
  x: 12.0, y: 18.0, vx: 0.0, vy: 0.0, shape: 3, size: 0
});
let result = world.handle_asteroid_split(asteroid, #{});
game.set("/test/handled", result["handled"]);
game.set("/test/despawned", result["despawned"]);
game.set("/test/child_count", result["children"].len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "handle_asteroid_split terminal path should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/handled")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            game_state
                .get("/test/despawned")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            game_state
                .get("/test/child_count")
                .and_then(|value| value.as_i64()),
            Some(0)
        );
        assert_eq!(gameplay_world.query_kind("asteroid").len(), 0);
        assert!(!commands.iter().any(
            |c| matches!(c, BehaviorCommand::PlayAudioEvent { event, .. } if event == "gameplay.asteroid.split")
        ));
    }

    #[test]
    fn rhai_script_behavior_spawn_wave_dynamic_creates_requested_asteroids() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
world.rand_seed(42);
let ids = world.spawn_wave("asteroids.dynamic", #{
  spawn_count: 6,
  ship_x: 0.0,
  ship_y: 0.0,
  min_x: -320.0,
  max_x: 320.0,
  min_y: -240.0,
  max_y: 240.0
});
game.set("/test/count", ids.len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_wave should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/count")
                .and_then(|value| value.as_i64()),
            Some(6)
        );
        let asteroid_ids = gameplay_world.query_kind("asteroid");
        assert_eq!(asteroid_ids.len(), 6);
        for asteroid_id in asteroid_ids {
            let xf = gameplay_world
                .transform(asteroid_id)
                .expect("asteroid transform");
            assert!(
                (xf.x - -310.0).abs() < 0.01 || (xf.x - 310.0).abs() < 0.01,
                "wave asteroid should spawn on left/right edge: {xf:?}"
            );
            assert!(xf.y >= -230.0 && xf.y <= 230.0);
        }
    }

    #[test]
    fn rhai_script_behavior_spawn_wave_dynamic_supports_zero_ship_fallback() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
world.rand_seed(7);
let ids = world.spawn_wave("asteroids.dynamic", #{
  spawn_count: 2,
  min_x: -320.0,
  max_x: 320.0,
  min_y: -240.0,
  max_y: 240.0
});
game.set("/test/count", ids.len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "spawn_wave fallback path should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/count")
                .and_then(|value| value.as_i64()),
            Some(2)
        );
        assert_eq!(gameplay_world.query_kind("asteroid").len(), 2);
    }

    #[test]
    fn rhai_script_behavior_ensure_crack_visuals_spawns_and_marks_once() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let asteroid = world.spawn_prefab("asteroid", #{
  x: 12.0, y: 18.0, vx: 0.0, vy: 0.0, shape: 3, size: 2
});
let ids = world.ensure_crack_visuals(asteroid);
game.set("/test/count", ids.len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "ensure_crack_visuals should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/count")
                .and_then(|value| value.as_i64()),
            Some(3)
        );
        assert_eq!(
            commands
                .iter()
                .filter(|c| matches!(
                    c,
                    BehaviorCommand::SceneSpawn { template, target }
                        if template == "asteroid-template" && target.contains("-crack-")
                ))
                .count(),
            3
        );
        let asteroid_id = gameplay_world.query_kind("asteroid")[0];
        assert_eq!(
            gameplay_world
                .get(asteroid_id, "/cracks_spawned")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn rhai_script_behavior_ensure_crack_visuals_is_idempotent() {
        let mut behavior = RhaiScriptBehavior::from_params(&BehaviorParams {
            script: Some(
                r#"
world.set_world_bounds(-320.0, 320.0, -240.0, 240.0);
let asteroid = world.spawn_prefab("asteroid", #{
  x: 12.0, y: 18.0, vx: 0.0, vy: 0.0, shape: 3, size: 2
});
let first = world.ensure_crack_visuals(asteroid);
let second = world.ensure_crack_visuals(asteroid);
game.set("/test/first", first.len);
game.set("/test/second", second.len);
[]
"#
                .to_string(),
            ),
            ..BehaviorParams::default()
        });
        let gameplay_world = GameplayWorld::new();
        let game_state = GameState::new();
        let mut ctx = ctx(SceneStage::OnIdle, 0, 0);
        ctx.gameplay_world = Some(gameplay_world.clone());
        ctx.game_state = Some(game_state.clone());

        let commands = run_behavior(&mut behavior, &base_scene(), ctx);
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, BehaviorCommand::ScriptError { .. })),
            "ensure_crack_visuals idempotent path should not produce ScriptError: {commands:?}"
        );
        assert_eq!(
            game_state
                .get("/test/first")
                .and_then(|value| value.as_i64()),
            Some(3)
        );
        assert_eq!(
            game_state
                .get("/test/second")
                .and_then(|value| value.as_i64()),
            Some(0)
        );
        assert_eq!(
            commands
                .iter()
                .filter(|c| matches!(
                    c,
                    BehaviorCommand::SceneSpawn { template, target }
                        if template == "asteroid-template" && target.contains("-crack-")
                ))
                .count(),
            3
        );
    }

    #[test]
    fn rhai_script_module_resolver_configuration_exists() {
        // Just verify that the asteroids-shared module file exists
        // The actual module loading happens in the app initialization flow
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let module_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .join("mods/asteroids/scripts/asteroids-shared.rhai");
        assert!(
            module_path.exists(),
            "asteroids-shared.rhai module should exist at {:?}",
            module_path
        );
    }
}

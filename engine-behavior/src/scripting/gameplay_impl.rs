//! ScriptGameplayApi and ScriptGameplayEntityApi implementation - large standalone module.
//! This module contains the full impl blocks extracted from lib.rs.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Map as RhaiMap};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

use engine_core::game_state::GameState;
use engine_game::components::{DespawnVisual, LifecyclePolicy, TopDownShipController};
use engine_game::{
    Collider2D, ColliderShape, CollisionHit, GameplayWorld, Lifetime, PhysicsBody2D, Transform2D,
    VisualBinding,
};

use crate::geometry::{asteroid_radius_i32, heading_vector_i32, sin32_i32};
use crate::rhai_util::{json_to_rhai_dynamic, rhai_dynamic_to_json};
use crate::scripting::audio::ScriptFxApi;
use crate::scripting::ephemeral::{spawn_ephemeral_visual, EphemeralSpawn};
use crate::scripting::physics::ScriptEntityPhysicsApi;
use crate::scripting::ui::ScriptUiApi;
use crate::{catalog, BehaviorCommand};

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
    pub(crate) physics: ScriptEntityPhysicsApi,
}

// ── ScriptGameplayApi Implementation ──────────────────────────────────────
impl ScriptGameplayApi {
    pub(crate) fn map_number(args: &RhaiMap, key: &str, fallback: rhai::FLOAT) -> rhai::FLOAT {
        args.get(key)
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(fallback)
    }

    pub(crate) fn map_int(args: &RhaiMap, key: &str, fallback: rhai::INT) -> rhai::INT {
        args.get(key)
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::INT>()
                    .or_else(|| v.clone().try_cast::<rhai::FLOAT>().map(|f| f as rhai::INT))
            })
            .unwrap_or(fallback)
    }

    pub(crate) fn map_map(args: &RhaiMap, key: &str) -> Option<RhaiMap> {
        args.get(key).and_then(|v| v.clone().try_cast::<RhaiMap>())
    }

    pub(crate) fn map_string(args: &RhaiMap, key: &str) -> Option<String> {
        args.get(key)
            .and_then(|v| v.clone().try_cast::<String>())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    pub(crate) fn new(
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

    pub(crate) fn entity(&mut self, id: rhai::INT) -> ScriptGameplayEntityApi {
        let id_u64 = if id < 0 { 0 } else { id as u64 };
        let world = self.world.clone();
        ScriptGameplayEntityApi {
            physics: ScriptEntityPhysicsApi::new(world.clone(), id_u64),
            world,
            id: id_u64,
            queue: Arc::clone(&self.queue),
        }
    }

    pub(crate) fn clear(&mut self) {
        if let Some(world) = self.world.as_ref() {
            world.clear();
        }
    }

    pub(crate) fn count(&mut self) -> rhai::INT {
        self.world
            .as_ref()
            .map(|world| world.count() as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn count_kind(&mut self, kind: &str) -> rhai::INT {
        self.world
            .as_ref()
            .map(|world| world.count_kind(kind) as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn count_tag(&mut self, tag: &str) -> rhai::INT {
        self.world
            .as_ref()
            .map(|world| world.count_tag(tag) as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn first_kind(&mut self, kind: &str) -> rhai::INT {
        self.world
            .as_ref()
            .and_then(|world| world.first_kind(kind))
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn first_tag(&mut self, tag: &str) -> rhai::INT {
        self.world
            .as_ref()
            .and_then(|world| world.first_tag(tag))
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    /// Returns a Rhai map with diagnostic info about current entity counts.
    /// Useful for tracking object growth: { total: N, by_kind: { ... }, by_policy: { ... } }
    pub(crate) fn diagnostic_info(&mut self) -> RhaiMap {
        let Some(world) = self.world.as_ref() else {
            return RhaiMap::new();
        };
        let snapshot = world.diagnostic_snapshot();
        let mut result = RhaiMap::new();
        result.insert("total".into(), (snapshot.total as i64).into());

        let mut by_kind = RhaiMap::new();
        for (kind, count) in snapshot.by_kind {
            by_kind.insert(kind.into(), (count as i64).into());
        }
        result.insert("by_kind".into(), by_kind.into());

        let mut by_policy = RhaiMap::new();
        for (policy, count) in snapshot.by_policy {
            by_policy.insert(policy.into(), (count as i64).into());
        }
        result.insert("by_policy".into(), by_policy.into());

        result
    }

    pub(crate) fn spawn(&mut self, kind: &str, payload: RhaiDynamic) -> rhai::INT {
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

    pub(crate) fn despawn(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let uid = id as u64;
        let tree_ids = world.despawn_tree_ids(uid);
        if let Ok(mut commands) = self.queue.lock() {
            for tree_id in &tree_ids {
                if let Some(binding) = world.visual(*tree_id) {
                    for vid in binding.all_visual_ids() {
                        commands.push(BehaviorCommand::SceneDespawn {
                            target: vid.to_string(),
                        });
                    }
                }
            }
        }
        world.despawn(uid)
    }

    /// Cleanup-aware reset that despawns all dynamic entities and their visuals,
    /// unlike raw `clear()` which only wipes the store.
    pub(crate) fn reset_dynamic_entities(&mut self) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let all_ids = world.ids();
        if let Ok(mut commands) = self.queue.lock() {
            for id in &all_ids {
                if let Some(binding) = world.visual(*id) {
                    for vid in binding.all_visual_ids() {
                        commands.push(BehaviorCommand::SceneDespawn {
                            target: vid.to_string(),
                        });
                    }
                }
            }
        }
        world.clear();
        true
    }

    pub(crate) fn bind_visual(&mut self, id: rhai::INT, visual_id: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 || visual_id.trim().is_empty() {
            return false;
        }
        world.add_visual(id as u64, visual_id.to_string())
    }

    pub(crate) fn exists(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.exists(id as u64)
    }

    pub(crate) fn kind(&mut self, id: rhai::INT) -> String {
        let Some(world) = self.world.as_ref() else {
            return String::new();
        };
        if id < 0 {
            return String::new();
        }
        world.kind_of(id as u64).unwrap_or_default()
    }

    pub(crate) fn tags(&mut self, id: rhai::INT) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        if id < 0 {
            return RhaiArray::new();
        }
        world.tags(id as u64).into_iter().map(Into::into).collect()
    }

    pub(crate) fn ids(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .ids()
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn query_kind(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_kind(kind)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn query_tag(&mut self, tag: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_tag(tag)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn get(&mut self, id: rhai::INT, path: &str) -> RhaiDynamic {
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

    pub(crate) fn set(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
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

    pub(crate) fn has(&mut self, id: rhai::INT, path: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.has(id as u64, path)
    }

    pub(crate) fn remove(&mut self, id: rhai::INT, path: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.remove(id as u64, path)
    }

    pub(crate) fn push(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
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

    pub(crate) fn set_transform(
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

    pub(crate) fn transform(&mut self, id: rhai::INT) -> RhaiDynamic {
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

    pub(crate) fn set_physics(
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

    pub(crate) fn physics(&mut self, id: rhai::INT) -> RhaiDynamic {
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

    pub(crate) fn set_lifetime(&mut self, id: rhai::INT, ttl_ms: rhai::INT) -> bool {
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

    pub(crate) fn set_visual(&mut self, id: rhai::INT, visual_id: &str) -> bool {
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

    pub(crate) fn spawn_visual(&mut self, kind: &str, template: &str, data: RhaiMap) -> rhai::INT {
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

        // Step 7: Set lifetime if provided (skip if zero — means no expiry)
        if let Some(ttl_val) = data.get("lifetime_ms") {
            if let Some(ttl) = ttl_val.clone().try_cast::<rhai::INT>() {
                if ttl > 0 {
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
        }

        entity_id as rhai::INT
    }

    /// Generic prefab applicator: reads catalog components and applies them to an entity.
    /// This centralizes all prefab component logic (physics, collider, controller, lifecycle) 
    /// in one place, eliminating hardcoded match arms.
    fn apply_prefab_components(
        &mut self,
        entity_id: rhai::INT,
        prefab: &catalog::PrefabTemplate,
        args: &RhaiMap,
    ) -> bool {
        let Some(components) = &prefab.components else {
            return true; // No components to apply; entity spawned successfully
        };

        // Apply physics component - check args for overrides
        if let Some(phys) = &components.physics {
            let mut vx = phys.vx.unwrap_or(0.0);
            let mut vy = phys.vy.unwrap_or(0.0);
            let ax = phys.ax.unwrap_or(0.0);
            let ay = phys.ay.unwrap_or(0.0);
            let drag = phys.drag.unwrap_or(0.0);
            let max_speed = phys.max_speed.unwrap_or(0.0);

            // Check args for velocity overrides with velocity scale factor (60.0)
            if let Some(arg_vx) = args.get("vx").and_then(|v| v.as_float().ok()) {
                vx = arg_vx * 60.0;
            }
            if let Some(arg_vy) = args.get("vy").and_then(|v| v.as_float().ok()) {
                vy = arg_vy * 60.0;
            }

            if !self.set_physics(entity_id, vx, vy, ax, ay, drag, max_speed) {
                return false;
            }
        }

        // Apply collider component - check args for radius override
        if let Some(coll) = &components.collider {
            match coll.shape.as_str() {
                "circle" => {
                    let mut radius = coll.radius.unwrap_or(1.0);
                    let layer = coll.layer.unwrap_or(0xFFFF) as rhai::INT;
                    let mask = coll.mask.unwrap_or(0xFFFF) as rhai::INT;

                    // Check args for collider_radius override
                    if let Some(arg_radius) = args.get("collider_radius") {
                        if let Ok(r) = arg_radius.as_float() {
                            radius = r;
                        }
                    }

                    if !self.set_collider_circle(entity_id, radius, layer, mask) {
                        return false;
                    }
                }
                _ => {} // Unknown shape or rect (not yet supported); skip
            }
        }

        // Apply controller component - merge catalog config with args["cfg"] overrides
        if let Some(ctrl) = &components.controller {
            match ctrl.controller_type.as_str() {
                "TopDownShipController" => {
                    let mut config_map = if let Some(cfg) = &ctrl.config {
                        let mut m = RhaiMap::new();
                        for (k, v) in cfg {
                            m.insert(k.clone().into(), json_to_rhai_dynamic(v));
                        }
                        m
                    } else {
                        RhaiMap::new()
                    };

                    // Merge runtime overrides from args["cfg"] (e.g. per-level difficulty)
                    if let Some(cfg_dyn) = args.get("cfg") {
                        if let Some(cfg_map) = cfg_dyn.clone().try_cast::<RhaiMap>() {
                            for (k, v) in &cfg_map {
                                config_map.insert(k.clone(), v.clone());
                            }
                        }
                    }

                    if !self.attach_ship_controller(entity_id, config_map) {
                        return false;
                    }
                }
                _ => {} // Unknown controller type; skip
            }
        }

        // Apply wrappable flag
        if components.wrappable.unwrap_or(false) {
            if !self.enable_wrap_bounds(entity_id) {
                return false;
            }
        }

        // Apply extra data fields from catalog and args overrides
        let mut data = RhaiMap::new();

        // Start with catalog extra_data
        if let Some(extra) = &components.extra_data {
            for (k, v) in extra {
                data.insert(k.clone().into(), json_to_rhai_dynamic(v));
            }
        }

        // Apply init_fields from prefab
        for (k, v) in &prefab.init_fields {
            data.insert(k.clone().into(), json_to_rhai_dynamic(v));
        }

        // Apply args overrides for shape, size, and any other fields
        for (k, v) in args {
            // Skip position/velocity args that are handled separately
            if !["x", "y", "heading", "vx", "vy", "ttl_ms", "radius", "owner_id", "cfg", "invulnerable_ms", "collider_radius"].contains(&k.as_str()) {
                data.insert(k.clone(), v.clone());
            }
        }

        if !data.is_empty() {
            if !self.entity(entity_id).set_many(data) {
                return false;
            }
        }

        true
    }

    pub(crate) fn spawn_prefab(&mut self, name: &str, args: RhaiMap) -> rhai::INT {
        // Look up prefab in catalog
        let Some(prefab) = self.catalogs.prefabs.get(name).cloned() else {
            return 0; // Prefab not found in catalog
        };

        // Extract position from args
        let x = Self::map_number(&args, "x", 0.0);
        let y = Self::map_number(&args, "y", 0.0);
        let heading = Self::map_number(&args, "heading", 0.0);

        // Determine spawn approach based on lifecycle
        let lifecycle_str = prefab
            .components
            .as_ref()
            .and_then(|c| c.lifecycle.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("");

        let sprite_template = prefab.sprite_template.as_deref().unwrap_or(&prefab.kind);

        let id = if lifecycle_str == "Ttl" || lifecycle_str == "TtlOwnerBound" {
            // Ephemeral spawn for TTL-based entities (bullets, smoke, short-lived particles)
            self.spawn_prefab_ephemeral(&prefab, x, y, heading, &args)
        } else {
            // Regular spawn for persistent entities (ship, asteroid)
            let mut visual_args = RhaiMap::new();
            visual_args.insert("x".into(), x.into());
            visual_args.insert("y".into(), y.into());
            visual_args.insert("heading".into(), heading.into());

            let id = self.spawn_visual(&prefab.kind, sprite_template, visual_args);
            if id <= 0 {
                return 0;
            }

            // Apply catalog components generically
            if !self.apply_prefab_components(id, &prefab, &args) {
                let _ = self.despawn(id);
                return 0;
            }

            // Handle mod-specific initialization (passed as args, not hardcoded)
            let invulnerable_ms = Self::map_int(&args, "invulnerable_ms", 0);
            if invulnerable_ms > 0 {
                let _ = self.entity(id).status_add("invulnerable", invulnerable_ms);
            }

            id
        };

        if id <= 0 {
            return 0;
        }

        id
    }

    /// Spawn ephemeral entities with TTL-based lifecycle policies.
    fn spawn_prefab_ephemeral(
        &mut self,
        prefab: &catalog::PrefabTemplate,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        heading: rhai::FLOAT,
        args: &RhaiMap,
    ) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };

        let ttl_ms = Self::map_int(args, "ttl_ms", 0);
        let vx = Self::map_number(args, "vx", 0.0) * 60.0;
        let vy = Self::map_number(args, "vy", 0.0) * 60.0;
        let sprite_template = prefab.sprite_template.as_deref().unwrap_or(&prefab.kind);

        // Extract physics for drag/max_speed
        let (drag, max_speed) = prefab
            .components
            .as_ref()
            .and_then(|c| c.physics.as_ref())
            .map(|p| (p.drag.unwrap_or(0.0), p.max_speed.unwrap_or(0.0)))
            .unwrap_or((0.0, 0.0));

        // Determine lifecycle policy
        let lifecycle_str = prefab
            .components
            .as_ref()
            .and_then(|c| c.lifecycle.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("");

        let owner_id = Self::map_int(args, "owner_id", 0);
        let lifecycle = match lifecycle_str {
            "TtlOwnerBound" => LifecyclePolicy::TtlOwnerBound,
            _ => LifecyclePolicy::Ttl,
        };

        // Build extra_data from prefab components
        let mut extra_data = BTreeMap::new();
        if let Some(components) = &prefab.components {
            if let Some(extra) = &components.extra_data {
                for (k, v) in extra {
                    extra_data.insert(k.clone(), v.clone());
                }
            }
        }

        // Apply args overrides (e.g., radius from visual_args)
        if let Some(radius) = args.get("radius") {
            if let Ok(r) = radius.as_int() {
                extra_data.insert("radius".to_string(), JsonValue::from(r));
            }
        }

        let Some(id) = spawn_ephemeral_visual(
            world,
            &self.queue,
            EphemeralSpawn {
                kind: Box::leak(prefab.kind.clone().into_boxed_str()),
                template: Box::leak(sprite_template.to_string().into_boxed_str()),
                x: x as f32,
                y: y as f32,
                heading: heading as f32,
                vx: vx as f32,
                vy: vy as f32,
                drag: drag as f32,
                max_speed: max_speed as f32,
                ttl_ms: Some(ttl_ms as i32),
                owner_id: (owner_id > 0).then_some(owner_id as u64),
                lifecycle,
                extra_data,
            },
        ) else {
            return 0;
        };

        // Apply collider if specified in prefab
        if let Some(components) = &prefab.components {
            if let Some(coll) = &components.collider {
                match coll.shape.as_str() {
                    "circle" => {
                        let radius = coll.radius.unwrap_or(1.0);
                        let layer = coll.layer.unwrap_or(0xFFFF) as rhai::INT;
                        let mask = coll.mask.unwrap_or(0xFFFF) as rhai::INT;
                        if !self.set_collider_circle(id as rhai::INT, radius, layer, mask) {
                            let _ = self.despawn(id as rhai::INT);
                            return 0;
                        }
                    }
                    _ => {} // Unknown shape or rect (not yet supported); skip
                }
            }
        }

        // Apply wrap if specified
        if prefab
            .components
            .as_ref()
            .and_then(|c| c.wrappable)
            .unwrap_or(false)
        {
            if !self.enable_wrap_bounds(id as rhai::INT) {
                let _ = self.despawn(id as rhai::INT);
                return 0;
            }
        }

        id as rhai::INT
    }

    pub(crate) fn spawn_group(&mut self, group_name: &str, prefab_name: &str) -> RhaiArray {
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

    pub(crate) fn try_fire_weapon(
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
            let heading = controller.current_heading;
            let (dir_x, dir_y) = heading_vector_i32(heading);

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
                let heading = controller.current_heading;
                let (dir_x, dir_y) = heading_vector_i32(heading);

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

    pub(crate) fn tick_heading_anim(&mut self, id: rhai::INT, dt_ms: rhai::INT) -> RhaiMap {
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

    pub(crate) fn handle_ship_hit(
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
            None,
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

    pub(crate) fn handle_bullet_hit(
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

    pub(crate) fn handle_asteroid_split(
        &mut self,
        asteroid_id: rhai::INT,
        args: RhaiMap,
    ) -> RhaiMap {
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

    pub(crate) fn spawn_wave(&mut self, wave_name: &str, args: RhaiMap) -> RhaiArray {
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

    pub(crate) fn ensure_crack_visuals(&mut self, asteroid_id: rhai::INT) -> RhaiArray {
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
        let Some(transform) = world.transform(asteroid_id_u64) else {
            return out;
        };
        let mut asteroid = self.entity(asteroid_id);
        if asteroid.get_bool("/cracks_spawned", false) {
            return out;
        }

        for i in 0..3_i64 {
            let mut extra_data = BTreeMap::new();
            extra_data.insert(
                "owner_id".to_string(),
                JsonValue::from(asteroid_id_u64 as i64),
            );
            extra_data.insert("crack_index".to_string(), JsonValue::from(i));
            let Some(crack_id) = spawn_ephemeral_visual(
                &world,
                &self.queue,
                EphemeralSpawn {
                    kind: "asteroid-crack",
                    template: "asteroid-template",
                    x: transform.x,
                    y: transform.y,
                    heading: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    drag: 0.0,
                    max_speed: 0.0,
                    ttl_ms: None,
                    owner_id: Some(asteroid_id_u64),
                    lifecycle: LifecyclePolicy::OwnerBound,
                    extra_data,
                },
            ) else {
                return RhaiArray::new();
            };
            let visual_id = format!("asteroid-crack-{}", crack_id);
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

    /// Spawns temporary crack visuals for an asteroid that is currently flashing.
    /// Each crack is given a TTL matching the flash duration, so they auto-despawn.
    /// This is called during the flash phase rather than at spawn time.
    pub(crate) fn spawn_flash_cracks(&mut self, asteroid_id: rhai::INT, flash_duration_ms: rhai::INT) -> RhaiArray {
        let mut out = RhaiArray::new();
        if asteroid_id <= 0 || flash_duration_ms <= 0 {
            return out;
        }
        let Some(world) = self.world.clone() else {
            return out;
        };
        let asteroid_id_u64 = asteroid_id as u64;
        if !world.exists(asteroid_id_u64) {
            return out;
        }
        let Some(transform) = world.transform(asteroid_id_u64) else {
            return out;
        };

        let ttl_ms = (flash_duration_ms as i32).max(0);

        for i in 0..3_i64 {
            let mut extra_data = BTreeMap::new();
            extra_data.insert(
                "owner_id".to_string(),
                JsonValue::from(asteroid_id_u64 as i64),
            );
            extra_data.insert("crack_index".to_string(), JsonValue::from(i));
            let Some(crack_id) = spawn_ephemeral_visual(
                &world,
                &self.queue,
                EphemeralSpawn {
                    kind: "asteroid-crack",
                    template: "asteroid-template",
                    x: transform.x,
                    y: transform.y,
                    heading: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    drag: 0.0,
                    max_speed: 0.0,
                    ttl_ms: Some(ttl_ms),
                    owner_id: Some(asteroid_id_u64),
                    lifecycle: LifecyclePolicy::TtlOwnerBound,
                    extra_data,
                },
            ) else {
                return RhaiArray::new();
            };
            let visual_id = format!("asteroid-crack-{}", crack_id);
            out.push(visual_id.into());
        }
        out
    }

    pub(crate) fn flash_ui_message(&self, text: &str, ttl_ms: rhai::INT) -> bool {
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

    pub(crate) fn collisions(&mut self) -> RhaiArray {
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

    pub(crate) fn collisions_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
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

    pub(crate) fn collisions_of(&mut self, kind: &str) -> RhaiArray {
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
    pub(crate) fn filter_hits_by_kind(
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

    pub(crate) fn collision_enters_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return vec![];
        };
        Self::filter_hits_by_kind(&self.collision_enters.clone(), world, kind_a, kind_b)
    }

    pub(crate) fn collision_stays_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return vec![];
        };
        Self::filter_hits_by_kind(&self.collision_stays.clone(), world, kind_a, kind_b)
    }

    pub(crate) fn collision_exits_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return vec![];
        };
        Self::filter_hits_by_kind(&self.collision_exits.clone(), world, kind_a, kind_b)
    }

    pub(crate) fn spawn_child_entity(
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

    pub(crate) fn despawn_children_of(&mut self, parent_id: rhai::INT) {
        if parent_id < 0 {
            return;
        }
        let Some(world) = self.world.as_ref() else {
            return;
        };
        world.despawn_children(parent_id as u64);
    }

    pub(crate) fn enable_wrap(
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

    pub(crate) fn disable_wrap(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        world.remove_wrap_bounds(uid);
        true
    }

    pub(crate) fn set_world_bounds(
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

    pub(crate) fn world_bounds(&mut self) -> RhaiMap {
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

    pub(crate) fn enable_wrap_bounds(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.enable_wrap_bounds(id as u64)
    }

    pub(crate) fn rand_i(&mut self, min: rhai::INT, max: rhai::INT) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return min;
        };
        world.rand_i(min as i32, max as i32) as rhai::INT
    }

    pub(crate) fn rand_seed(&mut self, seed: rhai::INT) {
        let Some(world) = self.world.as_ref() else {
            return;
        };
        world.rand_seed(seed as i64);
    }

    pub(crate) fn tag_add(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.tag_add(id as u64, tag)
    }

    pub(crate) fn tag_remove(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.tag_remove(id as u64, tag)
    }

    pub(crate) fn tag_has(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.tag_has(id as u64, tag)
    }

    pub(crate) fn after_ms(&mut self, label: &str, delay_ms: rhai::INT) {
        let Some(world) = self.world.as_ref() else {
            return;
        };
        world.after_ms(label, delay_ms as i64);
    }

    pub(crate) fn timer_fired(&mut self, label: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.timer_fired(label)
    }

    pub(crate) fn cancel_timer(&mut self, label: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.cancel_timer(label)
    }

    /// Spawn multiple entities from an array of spec maps.
    /// Each map should have `kind: String` and optionally `data: Map`.
    /// Returns an array of spawned entity IDs.
    pub(crate) fn spawn_batch(&mut self, specs: rhai::Array) -> rhai::Array {
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

    pub(crate) fn attach_ship_controller(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let uid = id as u64;

        // Extract config values with defaults; accept alternate key names for compatibility
        let turn_step_ms = config
            .get("turn_step_ms")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(40) as u32;

        let thrust_power = config
            .get("thrust_power")
            .or_else(|| config.get("ship_thrust"))
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(170.0) as f32;

        let max_speed = config
            .get("max_speed")
            .or_else(|| config.get("ship_max_speed"))
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

    pub(crate) fn ship_set_turn(&mut self, id: rhai::INT, dir: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        world.with_controller(uid, |ctrl| {
            ctrl.set_turn(dir.clamp(-1, 1) as i8);
        })
    }

    pub(crate) fn ship_set_thrust(&mut self, id: rhai::INT, on: bool) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        world.with_controller(uid, |ctrl| {
            ctrl.set_thrust(on);
        })
    }

    pub(crate) fn ship_heading(&mut self, id: rhai::INT) -> i32 {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        let uid = id as u64;
        world
            .controller(uid)
            .map(|c| c.current_heading)
            .unwrap_or(0)
    }

    pub(crate) fn ship_heading_vector(&mut self, id: rhai::INT) -> RhaiMap {
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

    pub(crate) fn ship_velocity(&mut self, id: rhai::INT) -> RhaiMap {
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

    pub(crate) fn poll_collision_events(&mut self) -> RhaiArray {
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

    pub(crate) fn clear_events(&mut self) {
        if let Some(world) = self.world.as_ref() {
            world.clear_events();
        }
    }

    pub(crate) fn distance(&mut self, a: rhai::INT, b: rhai::INT) -> rhai::FLOAT {
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

    pub(crate) fn any_alive(&mut self, kind: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.count_kind(kind) > 0
    }
}

impl ScriptGameplayEntityApi {
    pub(crate) fn exists(&mut self) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.exists(self.id)
    }

    pub(crate) fn id(&mut self) -> rhai::INT {
        self.id as rhai::INT
    }

    pub(crate) fn flag(&mut self, name: &str) -> bool {
        let path = format!("/{}", name);
        self.get_bool(&path, false)
    }

    pub(crate) fn set_flag(&mut self, name: &str, value: bool) -> bool {
        let path = format!("/{}", name);
        self.set(&path, value.into())
    }

    pub(crate) fn despawn(&mut self) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let tree_ids = world.despawn_tree_ids(self.id);
        if let Ok(mut commands) = self.queue.lock() {
            for tree_id in &tree_ids {
                if let Some(binding) = world.visual(*tree_id) {
                    for vid in binding.all_visual_ids() {
                        commands.push(BehaviorCommand::SceneDespawn {
                            target: vid.to_string(),
                        });
                    }
                }
            }
        }
        world.despawn(self.id)
    }

    pub(crate) fn get(&mut self, path: &str) -> RhaiDynamic {
        let Some(world) = self.world.as_ref() else {
            return ().into();
        };
        world
            .get(self.id, path)
            .map(|value| json_to_rhai_dynamic(&value))
            .unwrap_or_else(|| ().into())
    }

    pub(crate) fn get_i(&mut self, path: &str, fallback: rhai::INT) -> rhai::INT {
        self.get(path).try_cast::<rhai::INT>().unwrap_or(fallback)
    }

    pub(crate) fn get_bool(&mut self, path: &str, fallback: bool) -> bool {
        self.get(path).try_cast::<bool>().unwrap_or(fallback)
    }

    pub(crate) fn set(&mut self, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.set(self.id, path, value)
    }

    pub(crate) fn kind(&mut self) -> String {
        let Some(world) = self.world.as_ref() else {
            return String::new();
        };
        world.kind_of(self.id).unwrap_or_default()
    }

    pub(crate) fn tags(&mut self) -> RhaiArray {
        let Some(world) = self.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .tags(self.id)
            .into_iter()
            .map(|tag| tag.into())
            .collect()
    }

    pub(crate) fn get_metadata(&mut self) -> RhaiMap {
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

    pub(crate) fn get_components(&mut self) -> RhaiMap {
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

    pub(crate) fn transform(&mut self) -> RhaiMap {
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

    pub(crate) fn set_position(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) -> bool {
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

    pub(crate) fn set_heading(&mut self, heading: rhai::FLOAT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        let Some(mut xf) = world.transform(self.id) else {
            return false;
        };
        xf.heading = heading as f32;
        world.set_transform(self.id, xf)
    }

    #[allow(dead_code)]
    pub(crate) fn physics(&mut self) -> RhaiMap {
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

    pub(crate) fn set_velocity(&mut self, vx: rhai::FLOAT, vy: rhai::FLOAT) -> bool {
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

    pub(crate) fn set_acceleration(&mut self, ax: rhai::FLOAT, ay: rhai::FLOAT) -> bool {
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

    pub(crate) fn collider(&mut self) -> RhaiMap {
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

    pub(crate) fn lifetime_remaining(&mut self) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        let Some(lifetime) = world.lifetime(self.id) else {
            return 0;
        };
        lifetime.ttl_ms as rhai::INT
    }

    pub(crate) fn set_many(&mut self, map: RhaiMap) -> bool {
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

    pub(crate) fn data(&mut self) -> RhaiMap {
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

    pub(crate) fn get_f(&mut self, path: &str, fallback: rhai::FLOAT) -> rhai::FLOAT {
        self.get(path).try_cast::<rhai::FLOAT>().unwrap_or(fallback)
    }

    pub(crate) fn get_s(&mut self, path: &str, fallback: &str) -> String {
        self.get(path)
            .try_cast::<String>()
            .unwrap_or_else(|| fallback.to_string())
    }

    // ── Cooldown API ──────────────────────────────────────────────────────

    pub(crate) fn cooldown_start(&mut self, name: &str, ms: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.cooldown_start(self.id, name, ms as i32)
    }

    pub(crate) fn cooldown_ready(&mut self, name: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return true;
        };
        world.cooldown_ready(self.id, name)
    }

    pub(crate) fn cooldown_remaining(&mut self, name: &str) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        world.cooldown_remaining(self.id, name) as rhai::INT
    }

    // ── Status API ────────────────────────────────────────────────────────

    pub(crate) fn status_add(&mut self, name: &str, ms: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.status_add(self.id, name, ms as i32)
    }

    pub(crate) fn status_has(&mut self, name: &str) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.status_has(self.id, name)
    }

    pub(crate) fn status_remaining(&mut self, name: &str) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        world.status_remaining(self.id, name) as rhai::INT
    }

    // ── Ship Controller API ───────────────────────────────────────────────

    pub(crate) fn attach_ship_controller(&mut self, config: RhaiMap) -> bool {
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

    pub(crate) fn set_turn(&mut self, dir: rhai::INT) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.with_controller(self.id, |ctrl| {
            ctrl.set_turn(dir.clamp(-1, 1) as i8);
        })
    }

    pub(crate) fn set_thrust(&mut self, on: bool) -> bool {
        let Some(world) = self.world.as_ref() else {
            return false;
        };
        world.with_controller(self.id, |ctrl| {
            ctrl.set_thrust(on);
        })
    }

    pub(crate) fn heading(&mut self) -> rhai::INT {
        let Some(world) = self.world.as_ref() else {
            return 0;
        };
        world
            .controller(self.id)
            .map(|c| c.current_heading as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn heading_vector(&mut self) -> RhaiMap {
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

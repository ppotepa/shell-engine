//! ScriptGameplayApi and ScriptGameplayEntityApi implementation - large standalone module.
//! This module contains the full impl blocks extracted from lib.rs.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use rhai::{Array as RhaiArray, Dynamic as RhaiDynamic, Map as RhaiMap};
use serde_json::{Map as JsonMap, Value as JsonValue};

use engine_api::{
    ScriptEntityContext, ScriptWorldContext,
    filter_hits_by_kind, filter_hits_of_kind,
    follow_anchor_from_args, is_ephemeral_lifecycle, map_int, map_number, map_string,
    parse_lifecycle_policy, EmitResolved, EphemeralPrefabResolved,
};
use engine_game::components::{DespawnVisual, LifecyclePolicy, ArcadeController, ParticleColorRamp, ParticlePhysics, ParticleThreadMode, AngularBody, LinearBrake};
use engine_game::{
    Collider2D, ColliderShape, CollisionHit, GameplayWorld, Lifetime, PhysicsBody2D, Transform2D,
    VisualBinding,
};

use crate::rhai_util::{json_to_rhai_dynamic, rhai_dynamic_to_json};
use crate::scripting::ephemeral::{spawn_ephemeral_visual, EphemeralSpawn};
use crate::scripting::physics::ScriptEntityPhysicsApi;
use crate::{catalog, BehaviorCommand, EmitterState};

// ── Struct Definitions ───────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptGameplayApi {
    pub(crate) ctx: ScriptWorldContext,
    pub(crate) catalogs: Arc<catalog::ModCatalogs>,
    pub(crate) emitter_state: Option<EmitterState>,
}

#[derive(Clone)]
pub(crate) struct ScriptGameplayEntityApi {
    pub(crate) ctx: ScriptEntityContext,
    pub(crate) physics: ScriptEntityPhysicsApi,
}

// ── ScriptGameplayApi Implementation ──────────────────────────────────────
impl ScriptGameplayApi {
    pub(crate) fn map_number(args: &RhaiMap, key: &str, fallback: rhai::FLOAT) -> rhai::FLOAT {
        map_number(args, key, fallback)
    }

    pub(crate) fn map_int(args: &RhaiMap, key: &str, fallback: rhai::INT) -> rhai::INT {
        map_int(args, key, fallback)
    }

    pub(crate) fn new(
        world: Option<GameplayWorld>,
        collisions: std::sync::Arc<Vec<CollisionHit>>,
        collision_enters: std::sync::Arc<Vec<CollisionHit>>,
        collision_stays: std::sync::Arc<Vec<CollisionHit>>,
        collision_exits: std::sync::Arc<Vec<CollisionHit>>,
        catalogs: Arc<catalog::ModCatalogs>,
        emitter_state: Option<EmitterState>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
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
            catalogs,
            emitter_state,
        }
    }

    pub(crate) fn entity(&mut self, id: rhai::INT) -> ScriptGameplayEntityApi {
        let id_u64 = if id < 0 { 0 } else { id as u64 };
        let world = self.ctx.world.clone();
        ScriptGameplayEntityApi {
            physics: ScriptEntityPhysicsApi::new(world.clone(), id_u64),
            ctx: ScriptEntityContext::new(world, id_u64, Arc::clone(&self.ctx.queue)),
        }
    }

    pub(crate) fn clear(&mut self) {
        if let Some(world) = self.ctx.world.as_ref() {
            world.clear();
        }
    }

    pub(crate) fn count(&mut self) -> rhai::INT {
        self.ctx.world
            .as_ref()
            .map(|world| world.count() as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn count_kind(&mut self, kind: &str) -> rhai::INT {
        self.ctx.world
            .as_ref()
            .map(|world| world.count_kind(kind) as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn count_tag(&mut self, tag: &str) -> rhai::INT {
        self.ctx.world
            .as_ref()
            .map(|world| world.count_tag(tag) as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn first_kind(&mut self, kind: &str) -> rhai::INT {
        self.ctx.world
            .as_ref()
            .and_then(|world| world.first_kind(kind))
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn first_tag(&mut self, tag: &str) -> rhai::INT {
        self.ctx.world
            .as_ref()
            .and_then(|world| world.first_tag(tag))
            .map(|id| id as rhai::INT)
            .unwrap_or(0)
    }

    /// Returns a Rhai map with diagnostic info about current entity counts.
    /// Useful for tracking object growth: { total: N, by_kind: { ... }, by_policy: { ... } }
    pub(crate) fn diagnostic_info(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.clone() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        let uid = id as u64;
        let tree_ids = world.despawn_tree_ids(uid);
        if let Ok(mut commands) = self.ctx.queue.lock() {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let all_ids = world.ids();
        if let Ok(mut commands) = self.ctx.queue.lock() {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 || visual_id.trim().is_empty() {
            return false;
        }
        world.add_visual(id as u64, visual_id.to_string())
    }

    pub(crate) fn exists(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.exists(id as u64)
    }

    pub(crate) fn kind(&mut self, id: rhai::INT) -> String {
        let Some(world) = self.ctx.world.as_ref() else {
            return String::new();
        };
        if id < 0 {
            return String::new();
        }
        world.kind_of(id as u64).unwrap_or_default()
    }

    pub(crate) fn tags(&mut self, id: rhai::INT) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        if id < 0 {
            return RhaiArray::new();
        }
        world.tags(id as u64).into_iter().map(Into::into).collect()
    }

    pub(crate) fn ids(&mut self) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .ids()
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn query_kind(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_kind(kind)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn query_tag(&mut self, tag: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .query_tag(tag)
            .into_iter()
            .map(|id| (id as rhai::INT).into())
            .collect()
    }

    pub(crate) fn get(&mut self, id: rhai::INT, path: &str) -> RhaiDynamic {
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.has(id as u64, path)
    }

    pub(crate) fn remove(&mut self, id: rhai::INT, path: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.remove(id as u64, path)
    }

    pub(crate) fn push(&mut self, id: rhai::INT, path: &str, value: RhaiDynamic) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        if id < 0 {
            return false;
        }
        world.set_lifetime(
            id as u64,
            Lifetime {
                ttl_ms: ttl_ms as i32,
                original_ttl_ms: ttl_ms as i32,
                on_expire: DespawnVisual::None,
            },
        )
    }

    pub(crate) fn set_visual(&mut self, id: rhai::INT, visual_id: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.clone() else {
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
            let mut commands = match self.ctx.queue.lock() {
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
                            let px = point_arr[0]
                                .clone()
                                .try_cast::<rhai::FLOAT>()
                                .or_else(|| {
                                    point_arr[0]
                                        .clone()
                                        .try_cast::<rhai::INT>()
                                        .map(|v| v as rhai::FLOAT)
                                });
                            let py = point_arr[1]
                                .clone()
                                .try_cast::<rhai::FLOAT>()
                                .or_else(|| {
                                    point_arr[1]
                                        .clone()
                                        .try_cast::<rhai::INT>()
                                        .map(|v| v as rhai::FLOAT)
                                });
                            if let (Some(px), Some(py)) = (px, py) {
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
                            original_ttl_ms: ttl as i32,
                            on_expire: DespawnVisual::None,
                        },
                    ) {
                        world.despawn(entity_id);
                        return 0;
                    }
                }
            }
        }

        // Step 8: Create physics body if velocity/physics fields are present
        let has_physics = data.get("vx").is_some()
            || data.get("vy").is_some()
            || data.get("drag").is_some()
            || data.get("max_speed").is_some();
        if has_physics {
            let extract_f = |key: &str| -> f32 {
                data.get(key)
                    .and_then(|v| {
                        v.clone()
                            .try_cast::<rhai::FLOAT>()
                            .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
                    })
                    .unwrap_or(0.0) as f32
            };
            let body = PhysicsBody2D {
                vx: extract_f("vx"),
                vy: extract_f("vy"),
                ax: extract_f("ax"),
                ay: extract_f("ay"),
                drag: extract_f("drag"),
                max_speed: extract_f("max_speed"),
            };
            if !world.set_physics(entity_id, body) {
                world.despawn(entity_id);
                return 0;
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

            if let Some(arg_vx) = args.get("vx").and_then(|v| v.as_float().ok()) {
                vx = arg_vx;
            }
            if let Some(arg_vy) = args.get("vy").and_then(|v| v.as_float().ok()) {
                vy = arg_vy;
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
                "ArcadeController" => {
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

                    if !self.attach_controller(entity_id, config_map) {
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

        let id = if is_ephemeral_lifecycle(lifecycle_str) {
            // Ephemeral spawn for TTL-based entities (bullets, smoke, short-lived particles)
            self.spawn_prefab_ephemeral(&prefab, x, y, heading, &args)
        } else {
            // Regular spawn for persistent entities
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
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };

        let vx = Self::map_number(args, "vx", 0.0);
        let vy = Self::map_number(args, "vy", 0.0);
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

        let resolved = self.resolve_ephemeral_prefab(prefab, args, lifecycle_str, vx, vy, drag, max_speed);

        let Some(id) = spawn_ephemeral_visual(
            world,
            &self.ctx.queue,
            EphemeralSpawn {
                kind: prefab.kind.clone(),
                template: sprite_template.to_string(),
                x: x as f32,
                y: y as f32,
                heading: heading as f32,
                vx: resolved.vx,
                vy: resolved.vy,
                drag: resolved.drag,
                max_speed: resolved.max_speed,
                ttl_ms: Some(resolved.ttl_ms),
                owner_id: resolved.owner_id,
                lifecycle: resolved.lifecycle,
                follow_anchor: resolved.follow_anchor,
                extra_data: resolved.extra_data,
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
        RhaiArray::new()
    }

    pub(crate) fn collisions(&mut self) -> RhaiArray {
        self.ctx.collisions
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
        let Some(world) = self.ctx.world.as_ref() else {
            return vec![];
        };
        filter_hits_by_kind(&self.ctx.collisions, world, kind_a, kind_b)
    }

    pub(crate) fn collisions_of(&mut self, kind: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return vec![];
        };
        filter_hits_of_kind(&self.ctx.collisions, world, kind)
    }

    pub(crate) fn collision_enters_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return vec![];
        };
        filter_hits_by_kind(&self.ctx.collision_enters, world, kind_a, kind_b)
    }

    pub(crate) fn collision_stays_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return vec![];
        };
        filter_hits_by_kind(&self.ctx.collision_stays, world, kind_a, kind_b)
    }

    pub(crate) fn collision_exits_between(&mut self, kind_a: &str, kind_b: &str) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return vec![];
        };
        filter_hits_by_kind(&self.ctx.collision_exits, world, kind_a, kind_b)
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
            .ctx
            .world
            .as_ref()
            .map(|w| w.exists(parent_uid))
            .unwrap_or(false);
        if !parent_exists {
            return 0;
        }
        let child_id = self.spawn_visual(kind, template, data);
        if child_id > 0 {
            if let Some(world) = self.ctx.world.as_ref() {
                world.register_child(parent_uid, child_id as u64);
            }
        }
        child_id
    }

    pub(crate) fn despawn_children_of(&mut self, parent_id: rhai::INT) {
        if parent_id < 0 {
            return;
        }
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        let bounds =
            engine_game::WrapBounds::new(min_x as f32, max_x as f32, min_y as f32, max_y as f32);
        world.set_wrap_bounds(uid, bounds)
    }

    pub(crate) fn disable_wrap(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let uid = id as u64;
        world.remove_wrap_bounds(uid);
        true
    }

    pub(crate) fn set_world_bounds(
        &mut self,
        min_x: rhai::FLOAT,
        min_y: rhai::FLOAT,
        max_x: rhai::FLOAT,
        max_y: rhai::FLOAT,
    ) {
        let Some(world) = self.ctx.world.as_ref() else {
            return;
        };
        world.set_world_bounds(min_x as f32, max_x as f32, min_y as f32, max_y as f32);
    }

    pub(crate) fn world_bounds(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
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

    /// Attach an [`AngularBody`] to an entity.
    ///
    /// `config` map keys (all optional, snake_case field names):
    /// `accel`, `max`, `deadband`, `auto_brake`.
    pub(crate) fn angular_body_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else { return false; };
        let get_f = |map: &RhaiMap, key: &str, default: f32| -> f32 {
            map.get(key).and_then(|v| v.as_float().ok()).unwrap_or(default as rhai::FLOAT) as f32
        };
        let get_b = |map: &RhaiMap, key: &str, default: bool| -> bool {
            map.get(key).and_then(|v| v.as_bool().ok()).unwrap_or(default)
        };
        let body = AngularBody {
            accel:      get_f(&config, "accel",      5.5),
            max:        get_f(&config, "max",         7.0),
            deadband:   get_f(&config, "deadband",    0.10),
            auto_brake: get_b(&config, "auto_brake",  true),
            ..Default::default()
        };
        world.attach_angular_body(id as u64, body)
    }

    /// Set normalised turn input (−1.0…+1.0) for this frame.
    ///
    /// The `angular_body_system` reads this value and applies torque to the
    /// entity's angular velocity automatically — no manual tick needed.
    pub(crate) fn set_angular_input(&mut self, id: rhai::INT, input: rhai::FLOAT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else { return false; };
        world.set_angular_input(id as u64, input as f32)
    }

    /// Read the current angular velocity (rad/s) of an entity's [`AngularBody`].
    ///
    /// Returns `0.0` if the entity has no `AngularBody`.
    pub(crate) fn angular_vel(&mut self, id: rhai::INT) -> rhai::FLOAT {
        let Some(world) = self.ctx.world.as_ref() else { return 0.0; };
        world.angular_vel(id as u64).unwrap_or(0.0) as rhai::FLOAT
    }

    /// Attach a [`LinearBrake`] component to an entity.
    ///
    /// Config map keys: `decel` (f32), `deadband` (f32), `auto_brake` (bool).
    pub(crate) fn linear_brake_attach(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else { return false; };
        let get_f = |map: &RhaiMap, key: &str, default: f32| -> f32 {
            map.get(key).and_then(|v| v.as_float().ok()).unwrap_or(default as rhai::FLOAT) as f32
        };
        let get_b = |map: &RhaiMap, key: &str, default: bool| -> bool {
            map.get(key).and_then(|v| v.as_bool().ok()).unwrap_or(default)
        };
        let brake = LinearBrake {
            decel:      get_f(&config, "decel",      45.0),
            deadband:   get_f(&config, "deadband",    2.5),
            auto_brake: get_b(&config, "auto_brake",  true),
            active: false,
        };
        world.attach_linear_brake(id as u64, brake)
    }

    /// Suppress auto-braking for this frame (call when entity is thrusting).
    pub(crate) fn set_linear_brake_active(&mut self, id: rhai::INT, active: bool) -> bool {
        let Some(world) = self.ctx.world.as_ref() else { return false; };
        world.set_linear_brake_active(id as u64, active)
    }

    pub(crate) fn enable_wrap_bounds(&mut self, id: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.enable_wrap_bounds(id as u64)
    }

    pub(crate) fn rand_i(&mut self, min: rhai::INT, max: rhai::INT) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return min;
        };
        world.rand_i(min as i32, max as i32) as rhai::INT
    }

    pub(crate) fn rand_seed(&mut self, seed: rhai::INT) {
        let Some(world) = self.ctx.world.as_ref() else {
            return;
        };
        world.rand_seed(seed as i64);
    }

    pub(crate) fn tag_add(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.tag_add(id as u64, tag)
    }

    pub(crate) fn tag_remove(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.tag_remove(id as u64, tag)
    }

    pub(crate) fn tag_has(&mut self, id: rhai::INT, tag: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.tag_has(id as u64, tag)
    }

    pub(crate) fn after_ms(&mut self, label: &str, delay_ms: rhai::INT) {
        let Some(world) = self.ctx.world.as_ref() else {
            return;
        };
        world.after_ms(label, delay_ms as i64);
    }

    pub(crate) fn timer_fired(&mut self, label: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.timer_fired(label)
    }

    pub(crate) fn cancel_timer(&mut self, label: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.cancel_timer(label)
    }

    /// Spawn multiple entities from an array of spec maps.
    /// Each map should have `kind: String` and optionally `data: Map`.
    /// Returns an array of spawned entity IDs.
    pub(crate) fn spawn_batch(&mut self, specs: rhai::Array) -> rhai::Array {
        let Some(world) = self.ctx.world.as_ref() else {
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

    pub(crate) fn emit(
        &mut self,
        emitter_name: &str,
        owner_id: rhai::INT,
        args: RhaiMap,
    ) -> rhai::INT {
        let Some(world) = self.ctx.world.clone() else {
            return 0;
        };
        let Some(config) = self.catalogs.emitters.get(emitter_name).cloned() else {
            return 0;
        };
        let owner_uid = if owner_id > 0 { owner_id as u64 } else { 0 };
        if owner_uid == 0 || !world.exists(owner_uid) {
            return 0;
        }

        let owner = self.entity(owner_uid as rhai::INT);
        let xf = owner.clone().transform();
        let phys = owner.clone().physics();
        let heading = xf
            .get("heading")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);
        let x = xf
            .get("x")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);
        let y = xf
            .get("y")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);

        let heading_vec = owner.clone().heading_vector();
        let hx = heading_vec
            .get("x")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);
        let hy = heading_vec
            .get("y")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(-1.0);
        let base_vx = phys
            .get("vx")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);
        let base_vy = phys
            .get("vy")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
            .unwrap_or(0.0);

        let cooldown_name = config
            .cooldown_name
            .clone()
            .unwrap_or_else(|| emitter_name.to_string());
        let cooldown_ms = config.cooldown_ms.unwrap_or(0).max(0) as f64;
        let min_cooldown_ms = config
            .min_cooldown_ms
            .unwrap_or(config.cooldown_ms.unwrap_or(0))
            .max(0) as f64;
        let ramp_ms = config.ramp_ms.unwrap_or(0).max(0) as f64;
        let thrust_ms = Self::map_int(&args, "thrust_ms", 0).max(0) as f64;
        let effective_cooldown = if ramp_ms > 0.0 && cooldown_ms > min_cooldown_ms {
            let t = (thrust_ms / ramp_ms).clamp(0.0, 1.0);
            cooldown_ms + (min_cooldown_ms - cooldown_ms) * t
        } else {
            cooldown_ms.max(min_cooldown_ms)
        };
        if effective_cooldown > 0.0 && !owner.clone().cooldown_ready(&cooldown_name) {
            return 0;
        }

        if let Some(max_count) = config.max_count.filter(|value| *value > 0) {
            if let Some(state) = &self.emitter_state {
                while state.active_count(emitter_name, Some(owner_uid)) >= max_count as usize {
                    let Some(oldest) = state.evict_oldest(emitter_name, Some(owner_uid)) else {
                        break;
                    };
                    // Queue visual despawn before removing the gameplay entity so
                    // the scene layer/sprite is cleaned up alongside the entity.
                    if let Some(binding) = world.visual(oldest) {
                        if let Ok(mut q) = self.ctx.queue.lock() {
                            for vid in binding.all_visual_ids() {
                                q.push(BehaviorCommand::SceneDespawn {
                                    target: vid.to_string(),
                                });
                            }
                        }
                    }
                    let _ = world.despawn(oldest);
                }
            }
        }

        let (spawn_offset, config_side_offset) = Self::resolve_emit_anchor_offsets(&config, &args);
        let backward_speed = config.backward_speed.unwrap_or(0.0);
        let velocity_scale = config.velocity_scale.unwrap_or(1.0);
        // side_offset: perpendicular right offset from heading direction.
        // Right-perp of (hx, hy) is (hy, -hx).  Negative values = left side.
        let side_offset = config_side_offset
            + Self::map_number(&args, "side_offset", 0.0);
        let resolved = self.resolve_emit(&config, &args, spawn_offset, hx, hy);

        // Base emission axis is provided in world-space by resolve_emit.
        // Additional spread is applied around that base.
        let dir_angle = resolved.base_dir_y.atan2(resolved.base_dir_x) + resolved.spread;
        let dir_x = dir_angle.cos();
        let dir_y = dir_angle.sin();

        let Some(id) = spawn_ephemeral_visual(
            &world,
            &self.ctx.queue,
            EphemeralSpawn {
                kind: resolved.kind.clone(),
                template: resolved.template.clone(),
                x: (x - hx * spawn_offset + hy * side_offset) as f32,
                y: (y - hy * spawn_offset - hx * side_offset) as f32,
                heading: heading as f32,
                vx: (base_vx * backward_speed + dir_x * resolved.speed * velocity_scale) as f32,
                vy: (base_vy * backward_speed + dir_y * resolved.speed * velocity_scale) as f32,
                drag: 0.0,
                max_speed: 0.0,
                ttl_ms: Some(resolved.ttl_ms),
                owner_id: resolved.lifecycle.is_owner_bound().then_some(owner_uid),
                lifecycle: resolved.lifecycle,
                follow_anchor: resolved.follow_anchor,
                extra_data: resolved.extra_data,
            },
        ) else {
            return 0;
        };

        if let Some(binding) = world.visual(id) {
            if let Some(visual_id) = binding.visual_id {
                if !resolved.fg.trim().is_empty() {
                    if let Ok(mut queue) = self.ctx.queue.lock() {
                        queue.push(BehaviorCommand::SetProperty {
                            target: visual_id.clone(),
                            path: "style.fg".to_string(),
                            value: JsonValue::from(resolved.fg.clone()),
                        });
                    }
                }
                if resolved.radius > 1 {
                    let points = vec![[0, 0], [resolved.radius as i32, 0]];
                    if let Ok(mut queue) = self.ctx.queue.lock() {
                        queue.push(BehaviorCommand::SetProperty {
                            target: visual_id,
                            path: "vector.points".to_string(),
                            value: JsonValue::Array(
                                points
                                    .into_iter()
                                    .map(|[px, py]| {
                                        JsonValue::Array(vec![JsonValue::from(px), JsonValue::from(py)])
                                    })
                                    .collect(),
                            ),
                        });
                    }
                }
            }
        }

        // Attach particle physics configuration if emitter specifies thread_mode/collision/gravity
        if config.thread_mode.is_some() || config.collision.is_some() || config.gravity_scale.is_some() {
            let thread_mode = config.thread_mode.as_deref().map(ParticleThreadMode::from_str).unwrap_or_default();
            let collision = config.collision.unwrap_or(false);
            let gravity_scale = config.gravity_scale.unwrap_or(0.0) as f32;
            let bounce = config.bounce.unwrap_or(0.0) as f32;
            let mass = config.mass.unwrap_or(1.0) as f32;
            let collision_mask = config.collision_mask.clone().unwrap_or_default();
            
            let particle_physics = ParticlePhysics {
                thread_mode,
                collision,
                collision_mask,
                gravity_scale,
                bounce,
                mass,
            };
            let _ = world.attach_particle_physics(id, particle_physics);
        }

        // Resolve color ramp: args override > catalog default
        let ramp_colors: Option<Vec<String>> = args
            .get("color_ramp")
            .and_then(|v| v.clone().try_cast::<rhai::Array>())
            .map(|arr| arr.into_iter().filter_map(|c| c.try_cast::<String>()).collect())
            .or_else(|| config.color_ramp.clone());
        if let Some(colors) = ramp_colors {
            if !colors.is_empty() {
                let radius_max = args
                    .get("radius_max")
                    .and_then(|v| v.clone().try_cast::<rhai::INT>())
                    .unwrap_or_else(|| config.radius_max.unwrap_or(resolved.radius as i64))
                    as i32;
                let radius_min = args
                    .get("radius_min")
                    .and_then(|v| v.clone().try_cast::<rhai::INT>())
                    .unwrap_or_else(|| config.radius_min.unwrap_or(0))
                    as i32;
                let _ = world.attach_particle_ramp(id, ParticleColorRamp { colors, radius_max, radius_min });
            }
        }

        if effective_cooldown > 0.0 {
            let _ = self
                .entity(owner_uid as rhai::INT)
                .cooldown_start(&cooldown_name, effective_cooldown.round() as rhai::INT);
        }
        if let Some(state) = &self.emitter_state {
            state.track_spawn(emitter_name, Some(owner_uid), id);
        }
        id as rhai::INT
    }

    fn resolve_ephemeral_prefab(
        &self,
        prefab: &catalog::PrefabTemplate,
        args: &RhaiMap,
        lifecycle_str: &str,
        vx: rhai::FLOAT,
        vy: rhai::FLOAT,
        drag: f64,
        max_speed: f64,
    ) -> EphemeralPrefabResolved {
        let owner_id = Self::map_int(args, "owner_id", 0);
        let lifecycle = parse_lifecycle_policy(lifecycle_str, LifecyclePolicy::Ttl);
        let follow_anchor = lifecycle
            .follows_owner()
            .then(|| follow_anchor_from_args(args, 0.0, 0.0, true));

        let mut extra_data = BTreeMap::new();
        if let Some(components) = &prefab.components {
            if let Some(extra) = &components.extra_data {
                for (k, v) in extra {
                    extra_data.insert(k.clone(), v.clone());
                }
            }
        }
        if let Some(radius) = args.get("radius") {
            if let Ok(r) = radius.as_int() {
                extra_data.insert("radius".to_string(), JsonValue::from(r));
            }
        }

        EphemeralPrefabResolved {
            ttl_ms: Self::map_int(args, "ttl_ms", 0) as i32,
            vx: vx as f32,
            vy: vy as f32,
            drag: drag as f32,
            max_speed: max_speed as f32,
            owner_id: (owner_id > 0).then_some(owner_id as u64),
            lifecycle,
            follow_anchor,
            extra_data,
        }
    }

    fn resolve_emit(
        &self,
        config: &catalog::EmitterConfig,
        args: &RhaiMap,
        spawn_offset: f64,
        heading_x: f64,
        heading_y: f64,
    ) -> EmitResolved {
        let owner_bound = args
            .get("owner_bound")
            .and_then(|v| v.clone().try_cast::<bool>())
            .unwrap_or(false);
        let lifecycle_name = args
            .get("lifecycle")
            .and_then(|v| v.clone().try_cast::<String>())
            .or_else(|| config.lifecycle.clone())
            .unwrap_or_else(|| {
                if owner_bound {
                    "TtlOwnerBound".to_string()
                } else {
                    "Ttl".to_string()
                }
            });
        let lifecycle = parse_lifecycle_policy(
            &lifecycle_name,
            if owner_bound {
                LifecyclePolicy::TtlOwnerBound
            } else {
                LifecyclePolicy::Ttl
            },
        );
        let follow_anchor = lifecycle.follows_owner().then(|| {
            follow_anchor_from_args(
                args,
                config.follow_local_x.unwrap_or(-spawn_offset),
                config.follow_local_y.unwrap_or(0.0),
                config.follow_inherit_heading.unwrap_or(true),
            )
        });

        let radius = Self::map_int(args, "radius", config.radius.unwrap_or(1)).max(1);
        let fg = map_string(args, "fg").unwrap_or_default();
        let mut extra_data = BTreeMap::new();
        extra_data.insert("radius".to_string(), JsonValue::from(radius));
        if !fg.trim().is_empty() {
            extra_data.insert("fg".to_string(), JsonValue::from(fg.clone()));
        }

        let (base_dir_x, base_dir_y) = Self::resolve_emit_base_dir(config, args, heading_x, heading_y);
        let base_emission_angle = config.emission_angle.unwrap_or(0.0);
        EmitResolved {
            speed: Self::map_number(args, "speed", 0.0),
            base_dir_x,
            base_dir_y,
            spread: base_emission_angle + Self::map_number(args, "spread", 0.0),
            ttl_ms: Self::map_int(args, "ttl_ms", config.ttl_ms.unwrap_or(250)).max(1) as i32,
            radius,
            template: map_string(args, "template").unwrap_or_else(|| "debris".to_string()),
            kind: map_string(args, "kind").unwrap_or_else(|| "fx".to_string()),
            fg,
            lifecycle,
            follow_anchor,
            extra_data,
        }
    }

    /// Resolve base emission axis in world-space.
    /// Priority:
    /// 1) args.emission_local_x/y
    /// 2) config.emission_local_x/y
    /// 3) default owner backward axis (-heading)
    fn resolve_emit_base_dir(
        config: &catalog::EmitterConfig,
        args: &RhaiMap,
        heading_x: f64,
        heading_y: f64,
    ) -> (f64, f64) {
        let arg_local_x = args
            .get("emission_local_x")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>());
        let arg_local_y = args
            .get("emission_local_y")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>());
        if let (Some(local_x), Some(local_y)) = (arg_local_x, arg_local_y) {
            if let Some(dir) = Self::local_vec_to_world_unit(local_x, local_y, heading_x, heading_y) {
                return dir;
            }
        }

        if let (Some(local_x), Some(local_y)) = (config.emission_local_x, config.emission_local_y) {
            if let Some(dir) = Self::local_vec_to_world_unit(local_x, local_y, heading_x, heading_y) {
                return dir;
            }
        }

        (-heading_x, -heading_y)
    }

    /// Convert an owner-local direction vector into normalized world-space direction.
    /// Local frame: +x right, +y down.
    fn local_vec_to_world_unit(
        local_x: f64,
        local_y: f64,
        heading_x: f64,
        heading_y: f64,
    ) -> Option<(f64, f64)> {
        let len = (local_x * local_x + local_y * local_y).sqrt();
        if len <= f64::EPSILON {
            return None;
        }
        let nx = local_x / len;
        let ny = local_y / len;
        // Build owner-local basis from heading:
        // forward=(hx,hy), right=(-hy,hx), down=backward=( -hx,-hy )
        // local(+x right,+y down): world = right*lx + down*ly
        let right_x = -heading_y;
        let right_y = heading_x;
        let down_x = -heading_x;
        let down_y = -heading_y;
        let wx = right_x * nx + down_x * ny;
        let wy = right_y * nx + down_y * ny;
        let wlen = (wx * wx + wy * wy).sqrt();
        if wlen <= f64::EPSILON {
            None
        } else {
            Some((wx / wlen, wy / wlen))
        }
    }

    /// Resolve emitter anchor as legacy (spawn_offset/side_offset) from either:
    /// 1) args.local_x/local_y
    /// 2) config.local_x/local_y
    /// 3) config.edge_{from,to}_* + edge_t interpolation
    /// 4) legacy config.spawn_offset/config.side_offset fallback
    fn resolve_emit_anchor_offsets(config: &catalog::EmitterConfig, args: &RhaiMap) -> (f64, f64) {
        let arg_local_x = args.get("local_x").and_then(|v| v.clone().try_cast::<rhai::FLOAT>());
        let arg_local_y = args.get("local_y").and_then(|v| v.clone().try_cast::<rhai::FLOAT>());
        if let (Some(local_x), Some(local_y)) = (arg_local_x, arg_local_y) {
            return (local_y, -local_x);
        }

        if let (Some(local_x), Some(local_y)) = (config.local_x, config.local_y) {
            return (local_y, -local_x);
        }

        if let (Some(from_x), Some(from_y), Some(to_x), Some(to_y)) = (
            config.edge_from_x,
            config.edge_from_y,
            config.edge_to_x,
            config.edge_to_y,
        ) {
            let t = config.edge_t.unwrap_or(0.5).clamp(0.0, 1.0);
            let local_x = from_x + (to_x - from_x) * t;
            let local_y = from_y + (to_y - from_y) * t;
            return (local_y, -local_x);
        }

        (
            config.spawn_offset.unwrap_or(0.0),
            config.side_offset.unwrap_or(0.0),
        )
    }

    pub(crate) fn attach_controller(&mut self, id: rhai::INT, config: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let uid = id as u64;

        // Extract config values; all fields are required
        let Some(turn_step_ms_val) = config
            .get("turn_step_ms")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
        else {
            eprintln!("[attach_controller] missing required field: turn_step_ms");
            return false;
        };

        let Some(thrust_power_val) = config
            .get("thrust_power")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
        else {
            eprintln!("[attach_controller] missing required field: thrust_power");
            return false;
        };

        let Some(max_speed_val) = config
            .get("max_speed")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
        else {
            eprintln!("[attach_controller] missing required field: max_speed");
            return false;
        };

        let Some(heading_bits_val) = config
            .get("heading_bits")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
        else {
            eprintln!("[attach_controller] missing required field: heading_bits");
            return false;
        };

        let turn_step_ms = turn_step_ms_val as u32;
        let thrust_power = thrust_power_val as f32;
        let max_speed = max_speed_val as f32;
        let heading_bits = heading_bits_val as u8;

        let mut controller =
            ArcadeController::new(turn_step_ms, thrust_power, max_speed, heading_bits);
        if let Some(xf) = world.transform(uid) {
            controller.set_heading_radians(xf.heading);
        }
        world.attach_controller(uid, controller)
    }

    pub(crate) fn poll_collision_events(&mut self) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
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
        if let Some(world) = self.ctx.world.as_ref() {
            world.clear_events();
        }
    }

    pub(crate) fn distance(&mut self, a: rhai::INT, b: rhai::INT) -> rhai::FLOAT {
        let Some(world) = self.ctx.world.as_ref() else {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.count_kind(kind) > 0
    }
}

impl ScriptGameplayEntityApi {
    pub(crate) fn exists(&mut self) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.exists(self.ctx.id)
    }

    pub(crate) fn id(&mut self) -> rhai::INT {
        self.ctx.id as rhai::INT
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
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let tree_ids = world.despawn_tree_ids(self.ctx.id);
        if let Ok(mut commands) = self.ctx.queue.lock() {
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
        world.despawn(self.ctx.id)
    }

    pub(crate) fn get(&mut self, path: &str) -> RhaiDynamic {
        let Some(world) = self.ctx.world.as_ref() else {
            return ().into();
        };
        world
            .get(self.ctx.id, path)
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
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(value) = rhai_dynamic_to_json(&value) else {
            return false;
        };
        world.set(self.ctx.id, path, value)
    }

    pub(crate) fn kind(&mut self) -> String {
        let Some(world) = self.ctx.world.as_ref() else {
            return String::new();
        };
        world.kind_of(self.ctx.id).unwrap_or_default()
    }

    /// Returns the remaining lifetime as a fraction of the original TTL (1.0 = fresh, 0.0 = about to expire).
    pub(crate) fn lifetime_fraction(&mut self) -> rhai::FLOAT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0.0;
        };
        let Some(lt) = world.lifetime(self.ctx.id) else {
            return 0.0;
        };
        if lt.original_ttl_ms <= 0 {
            return 0.0;
        }
        (lt.ttl_ms as rhai::FLOAT / lt.original_ttl_ms as rhai::FLOAT).clamp(0.0, 1.0)
    }

    /// Queues a style.fg colour update on this entity's bound visual.
    pub(crate) fn set_fg(&mut self, color: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(binding) = world.visual(self.ctx.id) else {
            return false;
        };
        let Some(visual_id) = binding.visual_id else {
            return false;
        };
        let Ok(mut queue) = self.ctx.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::SetProperty {
            target: visual_id,
            path: "style.fg".to_string(),
            value: JsonValue::from(color),
        });
        true
    }

    /// Queues a vector.points update to resize this entity's bound visual to [radius].
    /// r=1 leaves the template dot as-is; r=0 sets a zero-length point (invisible).
    pub(crate) fn set_radius(&mut self, r: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(binding) = world.visual(self.ctx.id) else {
            return false;
        };
        let Some(visual_id) = binding.visual_id else {
            return false;
        };
        let Ok(mut queue) = self.ctx.queue.lock() else {
            return false;
        };
        let r = r.max(0) as i32;
        queue.push(BehaviorCommand::SetProperty {
            target: visual_id,
            path: "vector.points".to_string(),
            value: JsonValue::Array(vec![
                JsonValue::Array(vec![JsonValue::from(0), JsonValue::from(0)]),
                JsonValue::Array(vec![JsonValue::from(r), JsonValue::from(0)]),
            ]),
        });
        true
    }

    pub(crate) fn tags(&mut self) -> RhaiArray {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiArray::new();
        };
        world
            .tags(self.ctx.id)
            .into_iter()
            .map(|tag| tag.into())
            .collect()
    }

    pub(crate) fn get_metadata(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(entity) = world.get_entity(self.ctx.id) else {
            return RhaiMap::new();
        };

        let mut metadata = RhaiMap::new();
        metadata.insert("id".into(), (self.ctx.id as rhai::INT).into());
        metadata.insert("kind".into(), entity.kind.into());

        let tags: RhaiArray = entity.tags.iter().map(|t| t.clone().into()).collect();
        metadata.insert("tags".into(), tags.into());

        // Include all components
        if let Some(transform) = world.transform(self.ctx.id) {
            let mut xf = RhaiMap::new();
            xf.insert("x".into(), (transform.x as rhai::FLOAT).into());
            xf.insert("y".into(), (transform.y as rhai::FLOAT).into());
            xf.insert("heading".into(), (transform.heading as rhai::FLOAT).into());
            metadata.insert("transform".into(), xf.into());
        }

        if let Some(physics) = world.physics(self.ctx.id) {
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

        if let Some(collider) = world.collider(self.ctx.id) {
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

        if let Some(lifetime) = world.lifetime(self.ctx.id) {
            let mut life = RhaiMap::new();
            life.insert("ttl_ms".into(), (lifetime.ttl_ms as rhai::INT).into());
            metadata.insert("lifetime".into(), life.into());
        }

        if let Some(visual) = world.visual(self.ctx.id) {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };

        let mut components = RhaiMap::new();

        if let Some(transform) = world.transform(self.ctx.id) {
            let mut xf = RhaiMap::new();
            xf.insert("x".into(), (transform.x as rhai::FLOAT).into());
            xf.insert("y".into(), (transform.y as rhai::FLOAT).into());
            xf.insert("heading".into(), (transform.heading as rhai::FLOAT).into());
            components.insert("transform".into(), xf.into());
        }

        if let Some(physics) = world.physics(self.ctx.id) {
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

        if let Some(collider) = world.collider(self.ctx.id) {
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

        if let Some(lifetime) = world.lifetime(self.ctx.id) {
            let mut life = RhaiMap::new();
            life.insert("ttl_ms".into(), (lifetime.ttl_ms as rhai::INT).into());
            components.insert("lifetime".into(), life.into());
        }

        if let Some(visual) = world.visual(self.ctx.id) {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(xf) = world.transform(self.ctx.id) else {
            return RhaiMap::new();
        };

        let mut result = RhaiMap::new();
        result.insert("x".into(), (xf.x as rhai::FLOAT).into());
        result.insert("y".into(), (xf.y as rhai::FLOAT).into());
        result.insert("heading".into(), (xf.heading as rhai::FLOAT).into());
        result
    }

    pub(crate) fn set_position(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(mut xf) = world.transform(self.ctx.id) else {
            return false;
        };
        xf.x = x as f32;
        xf.y = y as f32;
        world.set_transform(self.ctx.id, xf)
    }

    pub(crate) fn set_heading(&mut self, heading: rhai::FLOAT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(mut xf) = world.transform(self.ctx.id) else {
            return false;
        };
        xf.heading = heading as f32;
        if !world.set_transform(self.ctx.id, xf) {
            return false;
        }
        let _ = world.with_controller(self.ctx.id, |ctrl| {
            ctrl.set_heading_radians(heading as f32);
        });
        true
    }

    #[allow(dead_code)]
    pub(crate) fn physics(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(phys) = world.physics(self.ctx.id) else {
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

    pub(crate) fn set_acceleration(&mut self, ax: rhai::FLOAT, ay: rhai::FLOAT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        let Some(mut phys) = world.physics(self.ctx.id) else {
            return false;
        };
        phys.ax = ax as f32;
        phys.ay = ay as f32;
        world.set_physics(self.ctx.id, phys)
    }

    pub(crate) fn collider(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(coll) = world.collider(self.ctx.id) else {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        let Some(lifetime) = world.lifetime(self.ctx.id) else {
            return 0;
        };
        lifetime.ttl_ms as rhai::INT
    }

    pub(crate) fn set_many(&mut self, map: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        for (key, value) in map {
            let Some(json_value) = rhai_dynamic_to_json(&value) else {
                return false;
            };
            if !world.set(self.ctx.id, &format!("/{}", key), json_value) {
                return false;
            }
        }
        true
    }

    pub(crate) fn data(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        let Some(entity) = world.get_entity(self.ctx.id) else {
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
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.cooldown_start(self.ctx.id, name, ms as i32)
    }

    pub(crate) fn cooldown_ready(&mut self, name: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return true;
        };
        world.cooldown_ready(self.ctx.id, name)
    }

    pub(crate) fn cooldown_remaining(&mut self, name: &str) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        world.cooldown_remaining(self.ctx.id, name) as rhai::INT
    }

    // ── Status API ────────────────────────────────────────────────────────

    pub(crate) fn status_add(&mut self, name: &str, ms: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.status_add(self.ctx.id, name, ms as i32)
    }

    pub(crate) fn status_has(&mut self, name: &str) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.status_has(self.ctx.id, name)
    }

    pub(crate) fn status_remaining(&mut self, name: &str) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        world.status_remaining(self.ctx.id, name) as rhai::INT
    }

    // ── Arcade Controller API ─────────────────────────────────────────────

    pub(crate) fn attach_controller(&mut self, config: RhaiMap) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        // Extract config values; all fields are required
        let Some(turn_step_ms_val) = config
            .get("turn_step_ms")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
        else {
            eprintln!("[attach_controller] missing required field: turn_step_ms");
            return false;
        };

        let Some(thrust_power_val) = config
            .get("thrust_power")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
        else {
            eprintln!("[attach_controller] missing required field: thrust_power");
            return false;
        };

        let Some(max_speed_val) = config
            .get("max_speed")
            .and_then(|v| v.clone().try_cast::<rhai::FLOAT>())
        else {
            eprintln!("[attach_controller] missing required field: max_speed");
            return false;
        };

        let Some(heading_bits_val) = config
            .get("heading_bits")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
        else {
            eprintln!("[attach_controller] missing required field: heading_bits");
            return false;
        };

        let controller = ArcadeController::new(
            turn_step_ms_val as u32,
            thrust_power_val as f32,
            max_speed_val as f32,
            heading_bits_val as u8,
        );
        world.attach_controller(self.ctx.id, controller)
    }

    pub(crate) fn set_turn(&mut self, dir: rhai::INT) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.with_controller(self.ctx.id, |ctrl| {
            ctrl.set_turn(dir.clamp(-1, 1) as i8);
        })
    }

    pub(crate) fn set_thrust(&mut self, on: bool) -> bool {
        let Some(world) = self.ctx.world.as_ref() else {
            return false;
        };
        world.with_controller(self.ctx.id, |ctrl| {
            ctrl.set_thrust(on);
        })
    }

    pub(crate) fn heading(&mut self) -> rhai::INT {
        let Some(world) = self.ctx.world.as_ref() else {
            return 0;
        };
        world
            .controller(self.ctx.id)
            .map(|c| c.current_heading as rhai::INT)
            .unwrap_or(0)
    }

    pub(crate) fn heading_vector(&mut self) -> RhaiMap {
        let Some(world) = self.ctx.world.as_ref() else {
            return RhaiMap::new();
        };
        match world.controller(self.ctx.id) {
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

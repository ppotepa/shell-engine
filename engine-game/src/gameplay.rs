//! Shared gameplay world state for dynamic gameplay entities.
//!
//! This crate intentionally keeps the data model generic. Engine systems and
//! Rhai scripts can use it to spawn, query, mutate, and despawn gameplay
//! entities without binding the runtime to one specific game.

use serde_json::{json, Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::components::{
    AngularBody, AngularMotor3D, ArcadeController, Assembly3D, AtmosphereAffected2D,
    AttachmentBundle3D, BootstrapAssembly3D, BootstrapPreset3D, CharacterMotor3D, Collider2D,
    ComponentBundle3D, ControlBundle3D, ControlIntent3D, EntityTimers, FlightMotor3D,
    FollowAnchor2D, FollowAnchor3D, GameplayEvent, GravityAffected2D, LifecyclePolicy, Lifetime,
    LinearBrake, LinearMotor3D, MotorBundle3D, Ownership, ParticleColorRamp, ParticlePhysics,
    PhysicsBody2D, PhysicsBody3D, ReferenceFrameBinding3D, ReferenceFrameState3D, SpatialBundle3D,
    SpatialKind, ThrusterRamp, Transform2D, Transform3D, VehicleRuntimePrimitives,
    VehicleStateCache, VisualBinding, WrapBounds,
};
use engine_vehicle::{BrakePhase, VehicleProfile, VehicleTelemetry};

/// Snapshot of a spawned gameplay entity.
#[derive(Clone, Debug, PartialEq)]
pub struct GameplayEntity {
    pub id: u64,
    pub kind: String,
    pub tags: BTreeSet<String>,
    pub data: JsonValue,
}

#[derive(Clone, Debug)]
struct GameplayStore {
    next_id: u64,
    entities: BTreeMap<u64, GameplayEntity>,
    transforms: BTreeMap<u64, Transform2D>,
    transforms_3d: BTreeMap<u64, Transform3D>,
    physics: BTreeMap<u64, PhysicsBody2D>,
    physics_3d: BTreeMap<u64, PhysicsBody3D>,
    control_intents_3d: BTreeMap<u64, ControlIntent3D>,
    reference_frame_bindings_3d: BTreeMap<u64, ReferenceFrameBinding3D>,
    reference_frame_state_3d: BTreeMap<u64, ReferenceFrameState3D>,
    gravity: BTreeMap<u64, GravityAffected2D>,
    atmosphere: BTreeMap<u64, AtmosphereAffected2D>,
    colliders: BTreeMap<u64, Collider2D>,
    lifetimes: BTreeMap<u64, Lifetime>,
    lifecycles: BTreeMap<u64, LifecyclePolicy>,
    ownership: BTreeMap<u64, Ownership>,
    follow_anchors: BTreeMap<u64, FollowAnchor2D>,
    follow_anchors_3d: BTreeMap<u64, FollowAnchor3D>,
    visuals: BTreeMap<u64, VisualBinding>,
    timers: BTreeMap<u64, EntityTimers>,
    wrap_bounds: BTreeMap<u64, WrapBounds>,
    controllers: BTreeMap<u64, ArcadeController>,
    particle_physics: BTreeMap<u64, ParticlePhysics>,
    particle_ramps: BTreeMap<u64, ParticleColorRamp>,
    angular_bodies: BTreeMap<u64, AngularBody>,
    linear_brakes: BTreeMap<u64, LinearBrake>,
    thruster_ramps: BTreeMap<u64, ThrusterRamp>,
    linear_motors_3d: BTreeMap<u64, LinearMotor3D>,
    angular_motors_3d: BTreeMap<u64, AngularMotor3D>,
    character_motors_3d: BTreeMap<u64, CharacterMotor3D>,
    flight_motors_3d: BTreeMap<u64, FlightMotor3D>,
    vehicle_state: VehicleStateCache,
    /// Parent → child entity IDs. Children are auto-despawned when parent despawns.
    children: BTreeMap<u64, Vec<u64>>,
    /// Gameplay events accumulated this frame (cleared each frame start).
    events: Vec<GameplayEvent>,
    rng_seed: u64,
    world_bounds: Option<WrapBounds>,
    /// Named world-level one-shot timers. Value is remaining milliseconds.
    world_timers: std::collections::HashMap<String, i64>,
    /// Labels of world timers that fired this tick (cleared at start of next tick).
    fired_world_timers: Vec<String>,
    /// O(1) kind count: kind string → live entity count.
    kind_counts: std::collections::HashMap<String, usize>,
    /// O(1) tag count: tag string → live entity count.
    tag_counts: std::collections::HashMap<String, usize>,
}

impl Default for GameplayStore {
    fn default() -> Self {
        Self {
            next_id: 0,
            entities: BTreeMap::new(),
            transforms: BTreeMap::new(),
            transforms_3d: BTreeMap::new(),
            physics: BTreeMap::new(),
            physics_3d: BTreeMap::new(),
            control_intents_3d: BTreeMap::new(),
            reference_frame_bindings_3d: BTreeMap::new(),
            reference_frame_state_3d: BTreeMap::new(),
            gravity: BTreeMap::new(),
            atmosphere: BTreeMap::new(),
            colliders: BTreeMap::new(),
            lifetimes: BTreeMap::new(),
            lifecycles: BTreeMap::new(),
            ownership: BTreeMap::new(),
            follow_anchors: BTreeMap::new(),
            follow_anchors_3d: BTreeMap::new(),
            visuals: BTreeMap::new(),
            timers: BTreeMap::new(),
            wrap_bounds: BTreeMap::new(),
            controllers: BTreeMap::new(),
            particle_physics: BTreeMap::new(),
            particle_ramps: BTreeMap::new(),
            angular_bodies: BTreeMap::new(),
            linear_brakes: BTreeMap::new(),
            thruster_ramps: BTreeMap::new(),
            linear_motors_3d: BTreeMap::new(),
            angular_motors_3d: BTreeMap::new(),
            character_motors_3d: BTreeMap::new(),
            flight_motors_3d: BTreeMap::new(),
            vehicle_state: VehicleStateCache::default(),
            children: BTreeMap::new(),
            events: Vec::new(),
            rng_seed: 1337,
            world_bounds: None,
            world_timers: std::collections::HashMap::new(),
            fired_world_timers: Vec::new(),
            kind_counts: std::collections::HashMap::new(),
            tag_counts: std::collections::HashMap::new(),
        }
    }
}

/// Thread-safe gameplay entity store.
///
/// The store is generic on purpose:
/// - `kind` is a lightweight gameplay classification.
/// - `tags` are optional role labels.
/// - `data` carries all gameplay-specific state.
#[derive(Clone, Debug)]
pub struct GameplayWorld {
    store: Arc<Mutex<GameplayStore>>,
}

/// Ignition ramp factor: 0 until `delay_ms` has passed, then linearly 0→1 over `ramp_ms`.
#[inline]
fn igf(elapsed_ms: f32, delay_ms: f32, ramp_ms: f32) -> f32 {
    if elapsed_ms < delay_ms {
        return 0.0;
    }
    ((elapsed_ms - delay_ms) / ramp_ms).min(1.0)
}

fn vehicle_runtime_primitives(store: &GameplayStore, id: u64) -> VehicleRuntimePrimitives<'_> {
    VehicleRuntimePrimitives {
        transform: store.transforms.get(&id),
        physics: store.physics.get(&id),
        controller: store.controllers.get(&id),
        angular_body: store.angular_bodies.get(&id),
        linear_brake: store.linear_brakes.get(&id),
        thruster_ramp: store.thruster_ramps.get(&id),
    }
}

fn snapshot_vehicle_profile_from_store(store: &GameplayStore, id: u64) -> Option<VehicleProfile> {
    if !store.entities.contains_key(&id) {
        return None;
    }

    match (
        store.vehicle_state.profiles.get(&id).cloned(),
        vehicle_runtime_primitives(store, id).profile_input(),
    ) {
        (Some(mut profile), Some(input)) => {
            profile.sync_from_runtime(input);
            Some(profile)
        }
        (Some(profile), None) => Some(profile),
        (None, Some(input)) => Some(VehicleProfile::from_runtime(input)),
        (None, None) => None,
    }
}

fn snapshot_vehicle_telemetry_from_store(
    store: &GameplayStore,
    id: u64,
) -> Option<VehicleTelemetry> {
    if !store.entities.contains_key(&id) {
        return None;
    }

    if let Some(input) = vehicle_runtime_primitives(store, id).telemetry_input() {
        Some(VehicleTelemetry::from_runtime(input))
    } else {
        store.vehicle_state.telemetry.get(&id).cloned()
    }
}

impl GameplayWorld {
    /// Creates an empty gameplay world.
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(GameplayStore::default())),
        }
    }

    /// Removes all gameplay entities and resets the id counter.
    pub fn clear(&self) {
        if let Ok(mut store) = self.store.lock() {
            *store = GameplayStore::default();
        }
    }

    fn apply_assembly3d_to_store(store: &mut GameplayStore, id: u64, assembly: Assembly3D) {
        if let Some(transform) = assembly.spatial.transform {
            store.transforms_3d.insert(id, transform);
        }
        if let Some(physics) = assembly.spatial.physics {
            store.physics_3d.insert(id, physics);
        }
        if let Some(intent) = assembly.control.control_intent {
            store.control_intents_3d.insert(id, intent);
        }
        if let Some(reference_frame) = assembly.control.reference_frame {
            store
                .reference_frame_bindings_3d
                .insert(id, reference_frame);
        }
        if let Some(reference_frame_state) = assembly.control.reference_frame_state {
            store
                .reference_frame_state_3d
                .insert(id, reference_frame_state);
        }
        if let Some(follow_anchor) = assembly.attachments.follow_anchor {
            store.follow_anchors_3d.insert(id, follow_anchor);
        }
        if let Some(linear_motor) = assembly.motors.linear_motor {
            store.linear_motors_3d.insert(id, linear_motor);
        }
        if let Some(angular_motor) = assembly.motors.angular_motor {
            store.angular_motors_3d.insert(id, angular_motor);
        }
        if let Some(character_motor) = assembly.motors.character_motor {
            store.character_motors_3d.insert(id, character_motor);
        }
        if let Some(flight_motor) = assembly.motors.flight_motor {
            store.flight_motors_3d.insert(id, flight_motor);
        }
    }

    fn apply_bundle3d_to_store(store: &mut GameplayStore, id: u64, bundle: ComponentBundle3D) {
        Self::apply_assembly3d_to_store(store, id, bundle.into_assembly());
    }

    fn snapshot_assembly3d_from_store(store: &GameplayStore, id: u64) -> Option<Assembly3D> {
        if !store.entities.contains_key(&id) {
            return None;
        }
        let assembly = Assembly3D {
            spatial: SpatialBundle3D {
                transform: store.transforms_3d.get(&id).copied(),
                physics: store.physics_3d.get(&id).copied(),
            },
            control: ControlBundle3D {
                control_intent: store.control_intents_3d.get(&id).copied(),
                reference_frame: store.reference_frame_bindings_3d.get(&id).cloned(),
                reference_frame_state: store.reference_frame_state_3d.get(&id).copied(),
            },
            attachments: AttachmentBundle3D {
                follow_anchor: store.follow_anchors_3d.get(&id).copied(),
            },
            motors: MotorBundle3D {
                linear_motor: store.linear_motors_3d.get(&id).copied(),
                angular_motor: store.angular_motors_3d.get(&id).copied(),
                character_motor: store.character_motors_3d.get(&id).copied(),
                flight_motor: store.flight_motors_3d.get(&id).copied(),
            },
        };
        (!assembly.is_empty()).then_some(assembly)
    }

    fn snapshot_bundle3d_from_store(store: &GameplayStore, id: u64) -> Option<ComponentBundle3D> {
        Self::snapshot_assembly3d_from_store(store, id).map(Assembly3D::into_bundle)
    }

    fn detach_assembly3d_from_store(store: &mut GameplayStore, id: u64) -> Option<Assembly3D> {
        let assembly = Assembly3D {
            spatial: SpatialBundle3D {
                transform: store.transforms_3d.remove(&id),
                physics: store.physics_3d.remove(&id),
            },
            control: ControlBundle3D {
                control_intent: store.control_intents_3d.remove(&id),
                reference_frame: store.reference_frame_bindings_3d.remove(&id),
                reference_frame_state: store.reference_frame_state_3d.remove(&id),
            },
            attachments: AttachmentBundle3D {
                follow_anchor: store.follow_anchors_3d.remove(&id),
            },
            motors: MotorBundle3D {
                linear_motor: store.linear_motors_3d.remove(&id),
                angular_motor: store.angular_motors_3d.remove(&id),
                character_motor: store.character_motors_3d.remove(&id),
                flight_motor: store.flight_motors_3d.remove(&id),
            },
        };
        (!assembly.is_empty()).then_some(assembly)
    }

    fn detach_bundle3d_from_store(store: &mut GameplayStore, id: u64) -> Option<ComponentBundle3D> {
        Self::detach_assembly3d_from_store(store, id).map(Assembly3D::into_bundle)
    }

    fn ids_with_any_3d_from_store(store: &GameplayStore) -> BTreeSet<u64> {
        let mut ids = BTreeSet::new();
        ids.extend(store.transforms_3d.keys().copied());
        ids.extend(store.physics_3d.keys().copied());
        ids.extend(store.control_intents_3d.keys().copied());
        ids.extend(store.reference_frame_bindings_3d.keys().copied());
        ids.extend(store.reference_frame_state_3d.keys().copied());
        ids.extend(store.follow_anchors_3d.keys().copied());
        ids.extend(store.linear_motors_3d.keys().copied());
        ids.extend(store.angular_motors_3d.keys().copied());
        ids.extend(store.character_motors_3d.keys().copied());
        ids.extend(store.flight_motors_3d.keys().copied());
        ids
    }

    fn with_existing_entity<R, F>(&self, id: u64, f: F) -> Option<R>
    where
        F: FnOnce(&mut GameplayStore) -> R,
    {
        let Ok(mut store) = self.store.lock() else {
            return None;
        };
        if !store.entities.contains_key(&id) {
            return None;
        }
        Some(f(&mut store))
    }

    /// Collects all visual IDs that need cleanup, then clears all gameplay entities.
    ///
    /// Returns a list of scene object IDs (visual IDs) that should be despawned
    /// via engine commands. This ensures visuals are cleaned up before gameplay
    /// state is wiped, maintaining consistency between presentation and gameplay layers.
    pub fn collect_visuals_for_cleanup(&self) -> Vec<String> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        let mut visuals = Vec::new();
        for binding in store.visuals.values() {
            for vid in binding.all_visual_ids() {
                visuals.push(vid.to_string());
            }
        }
        visuals
    }

    /// Returns the number of active entities.
    pub fn count(&self) -> usize {
        let Ok(store) = self.store.lock() else {
            return 0;
        };
        store.entities.len()
    }

    pub fn attach_component_bundle3d(&self, id: u64, bundle: ComponentBundle3D) -> bool {
        self.with_existing_entity(id, |store| {
            Self::apply_bundle3d_to_store(store, id, bundle);
            true
        })
        .unwrap_or(false)
    }

    pub fn attach_assembly3d(&self, id: u64, assembly: Assembly3D) -> bool {
        self.with_existing_entity(id, |store| {
            Self::apply_assembly3d_to_store(store, id, assembly);
            true
        })
        .unwrap_or(false)
    }

    pub fn bootstrap_preset3d(&self, id: u64, preset: BootstrapPreset3D) -> bool {
        self.bootstrap_assembly3d(id, preset.into_bootstrap_assembly())
    }

    pub fn bootstrap_assembly3d(&self, id: u64, preset: BootstrapAssembly3D) -> bool {
        self.with_existing_entity(id, |store| {
            let BootstrapAssembly3D {
                assembly,
                controlled,
                owner_id,
                inherit_owner_lifecycle,
                lifecycle,
            } = preset;
            if let Some(owner_id) = owner_id {
                if !store.entities.contains_key(&owner_id) {
                    return false;
                }
            }
            Self::apply_assembly3d_to_store(store, id, assembly);

            if controlled {
                store.vehicle_state.controlled_entity = Some(id);
            }

            if let Some(owner_id) = owner_id {
                store.ownership.insert(id, Ownership { owner_id });
                let children = store.children.entry(owner_id).or_default();
                if !children.contains(&id) {
                    children.push(id);
                }
                if lifecycle.is_none() && inherit_owner_lifecycle {
                    let inherited = store
                        .lifecycles
                        .get(&owner_id)
                        .copied()
                        .unwrap_or(LifecyclePolicy::FollowOwner);
                    store.lifecycles.insert(id, inherited);
                }
            }

            if let Some(policy) = lifecycle {
                store.lifecycles.insert(id, policy);
            }

            true
        })
        .unwrap_or(false)
    }

    pub fn component_bundle3d(&self, id: u64) -> Option<ComponentBundle3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        Self::snapshot_bundle3d_from_store(&store, id)
    }

    pub fn assembly3d(&self, id: u64) -> Option<Assembly3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        Self::snapshot_assembly3d_from_store(&store, id)
    }

    pub fn batch_read_component_bundles3d(&self, ids: &[u64]) -> Vec<(u64, ComponentBundle3D)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        ids.iter()
            .filter_map(|&id| {
                Self::snapshot_bundle3d_from_store(&store, id).map(|bundle| (id, bundle))
            })
            .collect()
    }

    pub fn batch_read_assemblies3d(&self, ids: &[u64]) -> Vec<(u64, Assembly3D)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        ids.iter()
            .filter_map(|&id| {
                Self::snapshot_assembly3d_from_store(&store, id).map(|assembly| (id, assembly))
            })
            .collect()
    }

    pub fn batch_read_all_component_bundles3d(&self) -> Vec<(u64, ComponentBundle3D)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        Self::ids_with_any_3d_from_store(&store)
            .into_iter()
            .filter_map(|id| {
                Self::snapshot_bundle3d_from_store(&store, id).map(|bundle| (id, bundle))
            })
            .collect()
    }

    pub fn batch_read_all_assemblies3d(&self) -> Vec<(u64, Assembly3D)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        Self::ids_with_any_3d_from_store(&store)
            .into_iter()
            .filter_map(|id| {
                Self::snapshot_assembly3d_from_store(&store, id).map(|assembly| (id, assembly))
            })
            .collect()
    }

    pub fn detach_component_bundle3d(&self, id: u64) -> Option<ComponentBundle3D> {
        let Ok(mut store) = self.store.lock() else {
            return None;
        };
        let detached = Self::detach_bundle3d_from_store(&mut store, id);
        if store.vehicle_state.controlled_entity == Some(id) {
            store.vehicle_state.controlled_entity = None;
        }
        detached
    }

    pub fn detach_assembly3d(&self, id: u64) -> Option<Assembly3D> {
        let Ok(mut store) = self.store.lock() else {
            return None;
        };
        let detached = Self::detach_assembly3d_from_store(&mut store, id);
        if store.vehicle_state.controlled_entity == Some(id) {
            store.vehicle_state.controlled_entity = None;
        }
        detached
    }

    /// Returns the total number of bound visual IDs across all entities.
    pub fn total_visual_count(&self) -> usize {
        let Ok(store) = self.store.lock() else {
            return 0;
        };
        store
            .visuals
            .values()
            .map(|b| b.all_visual_ids().len())
            .sum()
    }

    /// Creates a diagnostic snapshot of current entity counts by kind and lifecycle policy.
    pub fn diagnostic_snapshot(&self) -> crate::diagnostics::EntityCountSnapshot {
        let Ok(store) = self.store.lock() else {
            return crate::diagnostics::EntityCountSnapshot::default();
        };

        let mut by_kind = std::collections::BTreeMap::new();
        let mut by_policy = std::collections::BTreeMap::new();

        for entity in store.entities.values() {
            *by_kind.entry(entity.kind.clone()).or_insert(0) += 1;
        }

        for policy in store.lifecycles.values() {
            let policy_name = match policy {
                LifecyclePolicy::Persistent => "Persistent",
                LifecyclePolicy::Manual => "Manual",
                LifecyclePolicy::Ttl => "Ttl",
                LifecyclePolicy::OwnerBound => "OwnerBound",
                LifecyclePolicy::TtlOwnerBound => "TtlOwnerBound",
                LifecyclePolicy::FollowOwner => "FollowOwner",
                LifecyclePolicy::TtlFollowOwner => "TtlFollowOwner",
            };
            *by_policy.entry(policy_name.to_string()).or_insert(0) += 1;
        }

        crate::diagnostics::EntityCountSnapshot {
            total: store.entities.len(),
            by_kind,
            by_policy,
            timestamp_ms: 0,
        }
    }

    /// Spawns a new entity with the given kind and payload.
    ///
    /// If `payload` is an object with a top-level `tags: [...]` array, those
    /// tags are extracted into the entity tag set and removed from the stored
    /// payload.
    pub fn spawn(&self, kind: &str, payload: JsonValue) -> Option<u64> {
        let kind = kind.trim();
        if kind.is_empty() {
            return None;
        }

        let mut store = self.store.lock().ok()?;
        store.next_id = store.next_id.wrapping_add(1);
        if store.next_id == 0 {
            store.next_id = 1;
        }
        let id = store.next_id;
        let (tags, data) = split_payload(payload);
        // Maintain O(1) counts.
        *store.kind_counts.entry(kind.to_string()).or_insert(0) += 1;
        for tag in &tags {
            *store.tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
        store.entities.insert(
            id,
            GameplayEntity {
                id,
                kind: kind.to_string(),
                tags,
                data,
            },
        );
        Some(id)
    }

    /// Removes an entity by id. Any children registered via `register_child` are
    /// also despawned recursively.
    pub fn despawn(&self, id: u64) -> bool {
        let (removed, child_ids) = {
            let Ok(mut store) = self.store.lock() else {
                return false;
            };
            let removed = if let Some(entity) = store.entities.remove(&id) {
                // Decrement O(1) counts.
                if let Some(c) = store.kind_counts.get_mut(&entity.kind) {
                    *c = c.saturating_sub(1);
                }
                for tag in &entity.tags {
                    if let Some(c) = store.tag_counts.get_mut(tag) {
                        *c = c.saturating_sub(1);
                    }
                }
                true
            } else {
                false
            };
            store.transforms.remove(&id);
            store.transforms_3d.remove(&id);
            store.physics.remove(&id);
            store.physics_3d.remove(&id);
            store.control_intents_3d.remove(&id);
            store.reference_frame_bindings_3d.remove(&id);
            store.reference_frame_state_3d.remove(&id);
            store.gravity.remove(&id);
            store.atmosphere.remove(&id);
            store.colliders.remove(&id);
            store.lifetimes.remove(&id);
            store.lifecycles.remove(&id);
            store.ownership.remove(&id);
            store.follow_anchors.remove(&id);
            store.follow_anchors_3d.remove(&id);
            store.visuals.remove(&id);
            store.timers.remove(&id);
            store.wrap_bounds.remove(&id);
            store.controllers.remove(&id);
            store.particle_physics.remove(&id);
            store.particle_ramps.remove(&id);
            store.angular_bodies.remove(&id);
            store.linear_brakes.remove(&id);
            store.thruster_ramps.remove(&id);
            store.linear_motors_3d.remove(&id);
            store.angular_motors_3d.remove(&id);
            store.character_motors_3d.remove(&id);
            store.flight_motors_3d.remove(&id);
            store.vehicle_state.clear_entity(id);
            for children in store.children.values_mut() {
                children.retain(|child_id| *child_id != id);
            }
            let child_ids = store.children.remove(&id).unwrap_or_default();
            (removed, child_ids)
        };
        // Recursively despawn children after releasing the lock.
        for child_id in child_ids {
            self.despawn(child_id);
        }
        removed
    }

    /// Registers `child` as a child of `parent`. When `parent` is despawned,
    /// `child` is automatically despawned too.
    pub fn register_child(&self, parent: u64, child: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&parent) || !store.entities.contains_key(&child) {
            return false;
        }
        store
            .ownership
            .insert(child, Ownership { owner_id: parent });
        let children = store.children.entry(parent).or_default();
        if !children.contains(&child) {
            children.push(child);
        }
        true
    }

    /// Despawns all children registered under `parent` without despawning the parent itself.
    pub fn despawn_children(&self, parent: u64) {
        let child_ids = {
            let Ok(mut store) = self.store.lock() else {
                return;
            };
            store.children.remove(&parent).unwrap_or_default()
        };
        for child_id in child_ids {
            self.despawn(child_id);
        }
    }

    /// Returns the root entity plus any recursively registered children in despawn order.
    pub fn despawn_tree_ids(&self, root: u64) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        if !store.entities.contains_key(&root) {
            return Vec::new();
        }
        let mut out = Vec::new();
        let mut stack = vec![root];
        let mut visited = BTreeSet::new();

        while let Some(id) = stack.pop() {
            if !visited.insert(id) {
                continue;
            }
            out.push(id);
            if let Some(children) = store.children.get(&id) {
                for child_id in children.iter().rev() {
                    stack.push(*child_id);
                }
            }
        }

        out
    }

    /// Returns `true` if the entity exists.
    pub fn exists(&self, id: u64) -> bool {
        let Ok(store) = self.store.lock() else {
            return false;
        };
        store.entities.contains_key(&id)
    }

    /// Returns the kind of an entity.
    pub fn kind_of(&self, id: u64) -> Option<String> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.entities.get(&id).map(|entity| entity.kind.clone())
    }

    /// Returns the tags of an entity.
    pub fn tags(&self, id: u64) -> Vec<String> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .entities
            .get(&id)
            .map(|entity| entity.tags.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns the ids of all entities, ordered by creation order.
    pub fn ids(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.entities.keys().copied().collect()
    }

    /// Returns the ids of all entities with the given kind.
    pub fn query_kind(&self, kind: &str) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .entities
            .iter()
            .filter(|(_, entity)| entity.kind == kind)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Returns the number of entities with the given kind. O(1).
    pub fn count_kind(&self, kind: &str) -> usize {
        self.store
            .lock()
            .ok()
            .and_then(|s| s.kind_counts.get(kind).copied())
            .unwrap_or(0)
    }

    /// Returns the first entity id with the given kind, if any.
    pub fn first_kind(&self, kind: &str) -> Option<u64> {
        self.query_kind(kind).into_iter().next()
    }

    /// Returns the ids of all entities containing the given tag.
    pub fn query_tag(&self, tag: &str) -> Vec<u64> {
        let tag = tag.trim();
        if tag.is_empty() {
            return Vec::new();
        }
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .entities
            .iter()
            .filter(|(_, entity)| entity.tags.contains(tag))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Returns the number of entities containing the given tag. O(1).
    pub fn count_tag(&self, tag: &str) -> usize {
        self.store
            .lock()
            .ok()
            .and_then(|s| s.tag_counts.get(tag).copied())
            .unwrap_or(0)
    }

    /// Returns the first entity id containing the given tag, if any.
    pub fn first_tag(&self, tag: &str) -> Option<u64> {
        self.query_tag(tag).into_iter().next()
    }

    /// Returns all entity ids within a circular radius of a point.
    pub fn query_circle(&self, x: f32, y: f32, radius: f32) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        let radius_sq = radius * radius;
        store
            .transforms
            .iter()
            .filter(|(_, transform)| {
                let dx = transform.x - x;
                let dy = transform.y - y;
                dx * dx + dy * dy <= radius_sq
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// Returns all entity ids within an axis-aligned bounding box.
    pub fn query_rect(&self, x: f32, y: f32, w: f32, h: f32) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        let min_x = x;
        let max_x = x + w;
        let min_y = y;
        let max_y = y + h;
        store
            .transforms
            .iter()
            .filter(|(_, transform)| {
                transform.x >= min_x
                    && transform.x <= max_x
                    && transform.y >= min_y
                    && transform.y <= max_y
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// Returns the closest entity id within max_dist, or None.
    pub fn query_nearest(&self, x: f32, y: f32, max_dist: f32) -> Option<u64> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        let max_dist_sq = max_dist * max_dist;
        store
            .transforms
            .iter()
            .filter_map(|(id, transform)| {
                let dx = transform.x - x;
                let dy = transform.y - y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= max_dist_sq {
                    Some((*id, dist_sq))
                } else {
                    None
                }
            })
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| id)
    }

    /// Returns the closest entity of a specific kind within max_dist, or None.
    pub fn query_nearest_kind(&self, kind: &str, x: f32, y: f32, max_dist: f32) -> Option<u64> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        let max_dist_sq = max_dist * max_dist;
        store
            .entities
            .iter()
            .filter(|(_, entity)| entity.kind == kind)
            .filter_map(|(id, _)| {
                store.transforms.get(id).map(|transform| {
                    let dx = transform.x - x;
                    let dy = transform.y - y;
                    let dist_sq = dx * dx + dy * dy;
                    (*id, dist_sq)
                })
            })
            .filter(|(_, dist_sq)| *dist_sq <= max_dist_sq)
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| id)
    }

    /// Returns a clone of an entity snapshot.
    pub fn get_entity(&self, id: u64) -> Option<GameplayEntity> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.entities.get(&id).cloned()
    }

    /// Returns the entire data JSON blob of an entity, or None if the entity doesn't exist.
    pub fn data(&self, id: u64) -> Option<JsonValue> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.entities.get(&id).map(|entity| entity.data.clone())
    }

    /// Bulk writes multiple properties into an entity using a map of key-value pairs.
    /// Each key is treated as a JSON pointer path (prefixed with /).
    pub fn set_many(&self, id: u64, map: &std::collections::BTreeMap<String, JsonValue>) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get_mut(&id) else {
            return false;
        };
        for (key, value) in map {
            if !set_path(&mut entity.data, &format!("/{}", key), value.clone()) {
                return false;
            }
        }
        true
    }

    // --- Component accessors -------------------------------------------------

    pub fn set_transform(&self, id: u64, xf: Transform2D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.transforms.insert(id, xf);
        true
    }

    pub fn transform(&self, id: u64) -> Option<Transform2D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.transforms.get(&id).copied()
    }

    pub fn set_transform3d(&self, id: u64, xf: Transform3D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.transforms_3d.insert(id, xf);
        true
    }

    pub fn transform3d(&self, id: u64) -> Option<Transform3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.transforms_3d.get(&id).copied()
    }

    pub fn with_transform3d<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut Transform3D),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(xf) = store.transforms_3d.get_mut(&id) else {
            return false;
        };
        f(xf);
        true
    }

    pub fn remove_transform3d(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.transforms_3d.remove(&id).is_some()
    }

    pub fn set_physics(&self, id: u64, body: PhysicsBody2D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.physics.insert(id, body);
        true
    }

    pub fn physics(&self, id: u64) -> Option<PhysicsBody2D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.physics.get(&id).copied()
    }

    pub fn set_physics3d(&self, id: u64, body: PhysicsBody3D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.physics_3d.insert(id, body);
        true
    }

    pub fn physics3d(&self, id: u64) -> Option<PhysicsBody3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.physics_3d.get(&id).copied()
    }

    pub fn with_physics3d<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut PhysicsBody3D),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(body) = store.physics_3d.get_mut(&id) else {
            return false;
        };
        f(body);
        true
    }

    pub fn remove_physics3d(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.physics_3d.remove(&id).is_some()
    }

    pub fn attach_control_intent3d(&self, id: u64, intent: ControlIntent3D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.control_intents_3d.insert(id, intent);
        true
    }

    pub fn control_intent3d(&self, id: u64) -> Option<ControlIntent3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.control_intents_3d.get(&id).copied()
    }

    pub fn with_control_intent3d<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut ControlIntent3D),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(intent) = store.control_intents_3d.get_mut(&id) else {
            return false;
        };
        f(intent);
        true
    }

    pub fn remove_control_intent3d(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.control_intents_3d.remove(&id).is_some()
    }

    pub fn attach_reference_frame3d(&self, id: u64, binding: ReferenceFrameBinding3D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.reference_frame_bindings_3d.insert(id, binding);
        true
    }

    pub fn reference_frame3d(&self, id: u64) -> Option<ReferenceFrameBinding3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.reference_frame_bindings_3d.get(&id).cloned()
    }

    pub fn with_reference_frame3d<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut ReferenceFrameBinding3D),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(binding) = store.reference_frame_bindings_3d.get_mut(&id) else {
            return false;
        };
        f(binding);
        true
    }

    pub fn set_reference_frame_state3d(&self, id: u64, state: ReferenceFrameState3D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.reference_frame_state_3d.insert(id, state);
        true
    }

    pub fn reference_frame_state3d(&self, id: u64) -> Option<ReferenceFrameState3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.reference_frame_state_3d.get(&id).copied()
    }

    pub fn with_reference_frame_state3d<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut ReferenceFrameState3D),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(state) = store.reference_frame_state_3d.get_mut(&id) else {
            return false;
        };
        f(state);
        true
    }

    pub fn remove_reference_frame3d(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let removed_binding = store.reference_frame_bindings_3d.remove(&id).is_some();
        let removed_state = store.reference_frame_state_3d.remove(&id).is_some();
        removed_binding || removed_state
    }

    pub fn attach_gravity(&self, id: u64, gravity: GravityAffected2D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.gravity.insert(id, gravity);
        true
    }

    pub fn gravity(&self, id: u64) -> Option<GravityAffected2D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.gravity.get(&id).cloned()
    }

    pub fn ids_with_gravity(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.gravity.keys().copied().collect()
    }

    pub fn attach_atmosphere(&self, id: u64, atmosphere: AtmosphereAffected2D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.atmosphere.insert(id, atmosphere);
        true
    }

    pub fn atmosphere(&self, id: u64) -> Option<AtmosphereAffected2D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.atmosphere.get(&id).cloned()
    }

    pub fn ids_with_atmosphere(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.atmosphere.keys().copied().collect()
    }

    pub fn set_atmosphere_state(&self, id: u64, atmosphere: AtmosphereAffected2D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.atmosphere.insert(id, atmosphere);
        true
    }

    /// Apply an instant velocity change (impulse) to an entity's physics body.
    /// If the entity has no physics body, this does nothing and returns false.
    pub fn apply_impulse(&self, id: u64, vx: f32, vy: f32) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if let Some(body) = store.physics.get_mut(&id) {
            body.vx += vx;
            body.vy += vy;
            true
        } else {
            false
        }
    }

    /// Get the magnitude (scalar speed) of an entity's velocity.
    /// Returns 0.0 if the entity has no physics body.
    pub fn velocity_magnitude(&self, id: u64) -> f32 {
        let Ok(store) = self.store.lock() else {
            return 0.0;
        };
        if let Some(body) = store.physics.get(&id) {
            (body.vx * body.vx + body.vy * body.vy).sqrt()
        } else {
            0.0
        }
    }

    /// Get the angle (in radians) of an entity's velocity vector.
    /// Returns 0.0 if the entity has no physics body or zero velocity.
    pub fn velocity_angle(&self, id: u64) -> f32 {
        let Ok(store) = self.store.lock() else {
            return 0.0;
        };
        if let Some(body) = store.physics.get(&id) {
            body.vy.atan2(body.vx)
        } else {
            0.0
        }
    }

    /// Set velocity from polar coordinates (speed and angle).
    /// Angle is in radians. Returns false if entity has no physics body.
    pub fn set_velocity_polar(&self, id: u64, speed: f32, angle: f32) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if let Some(body) = store.physics.get_mut(&id) {
            body.vx = angle.cos() * speed;
            body.vy = angle.sin() * speed;
            true
        } else {
            false
        }
    }

    pub fn set_collider(&self, id: u64, collider: Collider2D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.colliders.insert(id, collider);
        true
    }

    pub fn collider(&self, id: u64) -> Option<Collider2D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.colliders.get(&id).cloned()
    }

    pub fn set_lifetime(&self, id: u64, lifetime: Lifetime) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.lifetimes.insert(id, lifetime);
        true
    }

    pub fn lifetime(&self, id: u64) -> Option<Lifetime> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.lifetimes.get(&id).copied()
    }

    pub fn set_lifecycle(&self, id: u64, policy: LifecyclePolicy) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.lifecycles.insert(id, policy);
        true
    }

    pub fn lifecycle(&self, id: u64) -> Option<LifecyclePolicy> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.lifecycles.get(&id).copied()
    }

    pub fn ids_with_lifecycle(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.lifecycles.keys().copied().collect()
    }

    pub fn ownership(&self, id: u64) -> Option<Ownership> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.ownership.get(&id).copied()
    }

    pub fn set_follow_anchor(&self, id: u64, follow: FollowAnchor2D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.follow_anchors.insert(id, follow);
        true
    }

    pub fn follow_anchor(&self, id: u64) -> Option<FollowAnchor2D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.follow_anchors.get(&id).copied()
    }

    pub fn set_follow_anchor3d(&self, id: u64, follow: FollowAnchor3D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.follow_anchors_3d.insert(id, follow);
        true
    }

    pub fn follow_anchor3d(&self, id: u64) -> Option<FollowAnchor3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.follow_anchors_3d.get(&id).copied()
    }

    pub fn ids_with_follow_anchor(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.follow_anchors.keys().copied().collect()
    }

    pub fn remove_follow_anchor(&self, id: u64) {
        if let Ok(mut store) = self.store.lock() {
            store.follow_anchors.remove(&id);
        }
    }

    pub fn remove_follow_anchor3d(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.follow_anchors_3d.remove(&id).is_some()
    }

    pub fn set_visual(&self, id: u64, binding: VisualBinding) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.visuals.insert(id, binding);
        true
    }

    pub fn visual(&self, id: u64) -> Option<VisualBinding> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.visuals.get(&id).cloned()
    }

    pub fn add_visual(&self, id: u64, visual_id: String) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store
            .visuals
            .entry(id)
            .or_default()
            .additional_visuals
            .push(visual_id);
        true
    }

    pub fn ids_with_physics(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.physics.keys().copied().collect()
    }

    pub fn ids_with_transform3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.transforms_3d.keys().copied().collect()
    }

    pub fn ids_with_physics3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.physics_3d.keys().copied().collect()
    }

    pub fn ids_with_control_intent3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.control_intents_3d.keys().copied().collect()
    }

    pub fn ids_with_reference_frame3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.reference_frame_bindings_3d.keys().copied().collect()
    }

    pub fn ids_with_reference_frame_state3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.reference_frame_state_3d.keys().copied().collect()
    }

    pub fn ids_with_follow_anchor3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.follow_anchors_3d.keys().copied().collect()
    }

    pub fn ids_with_any_3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        Self::ids_with_any_3d_from_store(&store)
            .into_iter()
            .collect()
    }

    pub fn ids_with_lifetime(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.lifetimes.keys().copied().collect()
    }

    pub fn ids_with_colliders(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.colliders.keys().copied().collect()
    }

    pub fn ids_with_visual_binding(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.visuals.keys().copied().collect()
    }

    pub fn remove_lifetime(&self, id: u64) {
        if let Ok(mut store) = self.store.lock() {
            store.lifetimes.remove(&id);
        }
    }

    // ── Timers (cooldowns + statuses) ────────────────────────────────────

    /// Start or reset a named cooldown for `id`. Counts down to 0 (ready).
    pub fn cooldown_start(&self, id: u64, name: &str, ms: i32) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store
            .timers
            .entry(id)
            .or_default()
            .cooldowns
            .insert(name.to_string(), ms.max(0));
        true
    }

    /// Returns `true` if the named cooldown has expired (or was never started).
    pub fn cooldown_ready(&self, id: u64, name: &str) -> bool {
        let Ok(store) = self.store.lock() else {
            return true;
        };
        store
            .timers
            .get(&id)
            .and_then(|t| t.cooldowns.get(name))
            .map(|&ms| ms <= 0)
            .unwrap_or(true)
    }

    /// Returns remaining ms for a cooldown, or 0 if ready/absent.
    pub fn cooldown_remaining(&self, id: u64, name: &str) -> i32 {
        let Ok(store) = self.store.lock() else {
            return 0;
        };
        store
            .timers
            .get(&id)
            .and_then(|t| t.cooldowns.get(name))
            .copied()
            .unwrap_or(0)
            .max(0)
    }

    /// Add or refresh a named status effect for `id`. Active while remaining > 0.
    pub fn status_add(&self, id: u64, name: &str, ms: i32) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store
            .timers
            .entry(id)
            .or_default()
            .statuses
            .insert(name.to_string(), ms.max(1));
        true
    }

    /// Returns `true` if the named status is active (remaining > 0).
    pub fn status_has(&self, id: u64, name: &str) -> bool {
        let Ok(store) = self.store.lock() else {
            return false;
        };
        store
            .timers
            .get(&id)
            .and_then(|t| t.statuses.get(name))
            .map(|&ms| ms > 0)
            .unwrap_or(false)
    }

    /// Returns remaining ms for a status, or 0 if inactive/absent.
    pub fn status_remaining(&self, id: u64, name: &str) -> i32 {
        let Ok(store) = self.store.lock() else {
            return 0;
        };
        store
            .timers
            .get(&id)
            .and_then(|t| t.statuses.get(name))
            .copied()
            .unwrap_or(0)
            .max(0)
    }

    pub fn ids_with_timers(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.timers.keys().copied().collect()
    }

    /// Tick all timers by `dt_ms`. Cooldowns clamp at 0; expired statuses are removed.
    pub fn tick_timers(&self, dt_ms: u64) {
        let dt = dt_ms as i32;
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        store.timers.retain(|_, timers| {
            for v in timers.cooldowns.values_mut() {
                *v = (*v - dt).max(0);
            }
            timers.statuses.retain(|_, v| {
                *v -= dt;
                *v > 0
            });
            // Keep the entry as long as there are any cooldowns (even at 0)
            !timers.cooldowns.is_empty() || !timers.statuses.is_empty()
        });
    }

    /// Tick world-level one-shot timers by `dt_ms`. Expired labels are moved to
    /// `fired_world_timers` and consumed by [`timer_fired`].
    pub fn tick_world_timers(&self, dt_ms: u64) {
        let dt = dt_ms as i64;
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        store.fired_world_timers.clear();
        let mut fired = Vec::new();
        store.world_timers.retain(|label, remaining| {
            *remaining -= dt;
            if *remaining <= 0 {
                fired.push(label.clone());
                false
            } else {
                true
            }
        });
        store.fired_world_timers = fired;
    }

    /// Schedule a named one-shot timer that fires after `delay_ms` milliseconds.
    /// When it fires, `world.timer_fired(label)` returns `true` exactly once.
    /// Calling `after_ms` again with the same label resets the countdown.
    pub fn after_ms(&self, label: &str, delay_ms: i64) {
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        store
            .world_timers
            .insert(label.to_string(), delay_ms.max(1));
    }

    /// Returns `true` exactly once when the named timer scheduled with `after_ms` fires.
    pub fn timer_fired(&self, label: &str) -> bool {
        let Ok(store) = self.store.lock() else {
            return false;
        };
        store.fired_world_timers.contains(&label.to_string())
    }

    /// Cancel a pending world timer. Returns `true` if it was pending.
    pub fn cancel_timer(&self, label: &str) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.world_timers.remove(label).is_some()
    }

    // ── WrapBounds ────────────────────────────────────────────────────────

    /// Enable toroidal position wrap for `id` within `bounds`.
    pub fn set_wrap_bounds(&self, id: u64, bounds: WrapBounds) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.wrap_bounds.insert(id, bounds);
        true
    }

    pub fn wrap_bounds_for(&self, id: u64) -> Option<WrapBounds> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.wrap_bounds.get(&id).copied()
    }

    pub fn remove_wrap_bounds(&self, id: u64) {
        if let Ok(mut store) = self.store.lock() {
            store.wrap_bounds.remove(&id);
        }
    }

    pub fn ids_with_wrap(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.wrap_bounds.keys().copied().collect()
    }

    /// Apply toroidal wrap for all entities with WrapBounds after physics integration.
    pub fn apply_wrap(&self) {
        let ids = self.ids_with_wrap();
        for id in ids {
            let Some(bounds) = self.wrap_bounds_for(id) else {
                continue;
            };
            let Some(mut xf) = self.transform(id) else {
                continue;
            };
            let nx = bounds.wrap_x(xf.x);
            let ny = bounds.wrap_y(xf.y);
            if nx != xf.x || ny != xf.y {
                xf.x = nx;
                xf.y = ny;
                let _ = self.set_transform(id, xf);
            }
        }
    }

    /// Apply owner-follow attachments after owner motion has been resolved.
    pub fn apply_follow_anchors(&self) {
        let ids = self.ids_with_follow_anchor();
        for id in ids {
            let Some(follow) = self.follow_anchor(id) else {
                continue;
            };
            let Some(ownership) = self.ownership(id) else {
                continue;
            };
            let Some(owner_xf) = self.transform(ownership.owner_id) else {
                continue;
            };
            let current_heading = self.transform(id).map(|xf| xf.heading).unwrap_or(0.0);
            let (sin_h, cos_h) = owner_xf.heading.sin_cos();
            let xf = Transform2D {
                x: owner_xf.x + cos_h * follow.local_x - sin_h * follow.local_y,
                y: owner_xf.y + sin_h * follow.local_x + cos_h * follow.local_y,
                z: owner_xf.z,
                heading: if follow.inherit_heading {
                    owner_xf.heading
                } else {
                    current_heading
                },
            };
            let _ = self.set_transform(id, xf);
        }
    }

    pub fn spatial_kind(&self, id: u64) -> Option<SpatialKind> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        if !store.entities.contains_key(&id) {
            return None;
        }
        if store.transforms_3d.contains_key(&id)
            || store.physics_3d.contains_key(&id)
            || store.control_intents_3d.contains_key(&id)
            || store.reference_frame_bindings_3d.contains_key(&id)
            || store.reference_frame_state_3d.contains_key(&id)
            || store.follow_anchors_3d.contains_key(&id)
            || store.linear_motors_3d.contains_key(&id)
            || store.angular_motors_3d.contains_key(&id)
            || store.character_motors_3d.contains_key(&id)
            || store.flight_motors_3d.contains_key(&id)
        {
            Some(SpatialKind::ThreeD)
        } else {
            Some(SpatialKind::TwoD)
        }
    }

    /// Attach an ArcadeController to an entity.
    pub fn attach_controller(&self, id: u64, controller: ArcadeController) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.controllers.insert(id, controller);
        true
    }

    /// Retrieve the arcade controller for an entity.
    pub fn controller(&self, id: u64) -> Option<ArcadeController> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.controllers.get(&id).cloned()
    }

    /// Mutate an arcade controller. Returns false if entity has no controller.
    pub fn with_controller<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut ArcadeController),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if let Some(ctrl) = store.controllers.get_mut(&id) {
            f(ctrl);
            true
        } else {
            false
        }
    }

    /// Get all entity IDs with controllers.
    pub fn ids_with_controller(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.controllers.keys().copied().collect()
    }

    /// Remove a ship controller from an entity.
    pub fn remove_controller(&self, id: u64) {
        if let Ok(mut store) = self.store.lock() {
            store.controllers.remove(&id);
        }
    }

    pub fn attach_linear_motor3d(&self, id: u64, motor: LinearMotor3D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.linear_motors_3d.insert(id, motor);
        true
    }

    pub fn linear_motor3d(&self, id: u64) -> Option<LinearMotor3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.linear_motors_3d.get(&id).copied()
    }

    pub fn ids_with_linear_motor3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.linear_motors_3d.keys().copied().collect()
    }

    pub fn with_linear_motor3d<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut LinearMotor3D),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(motor) = store.linear_motors_3d.get_mut(&id) else {
            return false;
        };
        f(motor);
        true
    }

    pub fn remove_linear_motor3d(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.linear_motors_3d.remove(&id).is_some()
    }

    pub fn attach_angular_motor3d(&self, id: u64, motor: AngularMotor3D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.angular_motors_3d.insert(id, motor);
        true
    }

    pub fn angular_motor3d(&self, id: u64) -> Option<AngularMotor3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.angular_motors_3d.get(&id).copied()
    }

    pub fn ids_with_angular_motor3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.angular_motors_3d.keys().copied().collect()
    }

    pub fn with_angular_motor3d<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut AngularMotor3D),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(motor) = store.angular_motors_3d.get_mut(&id) else {
            return false;
        };
        f(motor);
        true
    }

    pub fn remove_angular_motor3d(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.angular_motors_3d.remove(&id).is_some()
    }

    pub fn attach_character_motor3d(&self, id: u64, motor: CharacterMotor3D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.character_motors_3d.insert(id, motor);
        true
    }

    pub fn character_motor3d(&self, id: u64) -> Option<CharacterMotor3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.character_motors_3d.get(&id).copied()
    }

    pub fn ids_with_character_motor3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.character_motors_3d.keys().copied().collect()
    }

    pub fn with_character_motor3d<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut CharacterMotor3D),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(motor) = store.character_motors_3d.get_mut(&id) else {
            return false;
        };
        f(motor);
        true
    }

    pub fn remove_character_motor3d(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.character_motors_3d.remove(&id).is_some()
    }

    pub fn attach_flight_motor3d(&self, id: u64, motor: FlightMotor3D) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.flight_motors_3d.insert(id, motor);
        true
    }

    pub fn flight_motor3d(&self, id: u64) -> Option<FlightMotor3D> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.flight_motors_3d.get(&id).copied()
    }

    pub fn ids_with_flight_motor3d(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.flight_motors_3d.keys().copied().collect()
    }

    pub fn with_flight_motor3d<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut FlightMotor3D),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(motor) = store.flight_motors_3d.get_mut(&id) else {
            return false;
        };
        f(motor);
        true
    }

    pub fn remove_flight_motor3d(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.flight_motors_3d.remove(&id).is_some()
    }

    /// Set the currently controlled gameplay entity.
    ///
    /// Returns `false` if the entity does not exist.
    pub fn set_controlled_entity(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.vehicle_state.controlled_entity = Some(id);
        true
    }

    /// Read the currently controlled gameplay entity.
    pub fn controlled_entity(&self) -> Option<u64> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.vehicle_state.controlled_entity
    }

    /// Clear the currently controlled gameplay entity.
    ///
    /// Returns `true` when a controlled entity was present.
    pub fn clear_controlled_entity(&self) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.vehicle_state.controlled_entity.take().is_some()
    }

    // ── Vehicle snapshot seam ────────────────────────────────────────────

    /// Returns all entity IDs that participate in the vehicle seam either via
    /// cached neutral vehicle state or via the underlying generic motion
    /// components.
    pub fn ids_with_vehicle_state(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        let mut ids = BTreeSet::new();
        ids.extend(store.vehicle_state.profiles.keys().copied());
        ids.extend(store.vehicle_state.telemetry.keys().copied());
        ids.extend(store.controllers.keys().copied());
        ids.extend(store.angular_bodies.keys().copied());
        ids.extend(store.linear_brakes.keys().copied());
        ids.extend(store.thruster_ramps.keys().copied());
        ids.into_iter().collect()
    }

    /// Attach or replace a cached [`VehicleProfile`] DTO for an entity.
    pub fn attach_vehicle_profile(&self, id: u64, profile: VehicleProfile) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.vehicle_state.profiles.insert(id, profile);
        true
    }

    /// Read the cached [`VehicleProfile`] for an entity.
    pub fn vehicle_profile(&self, id: u64) -> Option<VehicleProfile> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.vehicle_state.profiles.get(&id).cloned()
    }

    /// Mutate the cached [`VehicleProfile`] for an entity.
    pub fn with_vehicle_profile<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut VehicleProfile),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(profile) = store.vehicle_state.profiles.get_mut(&id) else {
            return false;
        };
        f(profile);
        true
    }

    /// Returns all entity IDs with a cached [`VehicleProfile`].
    pub fn ids_with_vehicle_profile(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.vehicle_state.profiles.keys().copied().collect()
    }

    /// Detach the cached [`VehicleProfile`] from an entity.
    pub fn detach_vehicle_profile(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.vehicle_state.profiles.remove(&id).is_some()
    }

    /// Build a vehicle profile snapshot from the attached generic motion components.
    ///
    /// If a cached profile already exists, `profile_id` and `label` are preserved
    /// and the motion-derived fields are refreshed from runtime state.
    pub fn snapshot_vehicle_profile(&self, id: u64) -> Option<VehicleProfile> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        snapshot_vehicle_profile_from_store(&store, id)
    }

    /// Refresh the cached [`VehicleProfile`] from the current runtime motion state.
    pub fn sync_vehicle_profile(&self, id: u64) -> Option<VehicleProfile> {
        let Ok(mut store) = self.store.lock() else {
            return None;
        };
        let profile = snapshot_vehicle_profile_from_store(&store, id)?;
        store.vehicle_state.profiles.insert(id, profile.clone());
        Some(profile)
    }

    /// Attach or replace cached [`VehicleTelemetry`] DTO for an entity.
    pub fn attach_vehicle_telemetry(&self, id: u64, telemetry: VehicleTelemetry) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.vehicle_state.telemetry.insert(id, telemetry);
        true
    }

    /// Read cached [`VehicleTelemetry`] for an entity.
    pub fn vehicle_telemetry(&self, id: u64) -> Option<VehicleTelemetry> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.vehicle_state.telemetry.get(&id).cloned()
    }

    /// Mutate cached [`VehicleTelemetry`] for an entity.
    pub fn with_vehicle_telemetry<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut VehicleTelemetry),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(telemetry) = store.vehicle_state.telemetry.get_mut(&id) else {
            return false;
        };
        f(telemetry);
        true
    }

    /// Returns all entity IDs with cached [`VehicleTelemetry`].
    pub fn ids_with_vehicle_telemetry(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.vehicle_state.telemetry.keys().copied().collect()
    }

    /// Detach cached [`VehicleTelemetry`] from an entity.
    pub fn detach_vehicle_telemetry(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.vehicle_state.telemetry.remove(&id).is_some()
    }

    /// Build a runtime telemetry snapshot directly from attached generic motion components.
    ///
    /// If no runtime inputs are attached, falls back to any cached telemetry.
    pub fn snapshot_vehicle_telemetry(&self, id: u64) -> Option<VehicleTelemetry> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        snapshot_vehicle_telemetry_from_store(&store, id)
    }

    /// Refresh cached [`VehicleTelemetry`] from the current runtime motion state.
    pub fn sync_vehicle_telemetry(&self, id: u64) -> Option<VehicleTelemetry> {
        let Ok(mut store) = self.store.lock() else {
            return None;
        };
        let telemetry = snapshot_vehicle_telemetry_from_store(&store, id)?;
        store.vehicle_state.telemetry.insert(id, telemetry.clone());
        Some(telemetry)
    }

    /// Refresh both cached vehicle surfaces in one lock acquisition.
    pub fn sync_vehicle_runtime_state(
        &self,
        id: u64,
    ) -> Option<(VehicleProfile, VehicleTelemetry)> {
        let Ok(mut store) = self.store.lock() else {
            return None;
        };
        let profile = snapshot_vehicle_profile_from_store(&store, id)?;
        let telemetry = snapshot_vehicle_telemetry_from_store(&store, id)?;
        store.vehicle_state.profiles.insert(id, profile.clone());
        store.vehicle_state.telemetry.insert(id, telemetry.clone());
        Some((profile, telemetry))
    }

    // === PARTICLE PHYSICS API ===

    /// Attach particle physics configuration to an entity.
    pub fn attach_particle_physics(&self, id: u64, physics: ParticlePhysics) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.particle_physics.insert(id, physics);
        true
    }

    /// Retrieve particle physics configuration for an entity.
    pub fn particle_physics(&self, id: u64) -> Option<ParticlePhysics> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.particle_physics.get(&id).cloned()
    }

    /// Get all entity IDs that have particle physics (for worker thread processing).
    pub fn ids_with_particle_physics(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.particle_physics.keys().copied().collect()
    }

    /// Get all entity IDs that should be processed on worker thread.
    pub fn ids_for_worker_physics(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .particle_physics
            .iter()
            .filter(|(_, pp)| pp.thread_mode.uses_worker_thread())
            .map(|(id, _)| *id)
            .collect()
    }

    // =========================================================================
    // PARTICLE COLOR RAMP API
    // =========================================================================

    /// Attach a color/radius ramp to a particle entity.
    pub fn attach_particle_ramp(&self, id: u64, ramp: ParticleColorRamp) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.particle_ramps.insert(id, ramp);
        true
    }

    /// Retrieve the color ramp for a particle entity.
    pub fn particle_ramp(&self, id: u64) -> Option<ParticleColorRamp> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.particle_ramps.get(&id).cloned()
    }

    /// Get all entity IDs that have a particle color ramp.
    pub fn ids_with_particle_ramp(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.particle_ramps.keys().copied().collect()
    }

    /// Batch-read ramp data for all particles in a single lock. Returns
    /// `(entity_id, visual_id, ramp, ttl_ms, original_ttl_ms)` tuples.
    pub fn batch_read_particle_ramps(&self) -> Vec<(u64, String, ParticleColorRamp, i32, i32)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        let mut out = Vec::with_capacity(store.particle_ramps.len());
        for (&id, ramp) in &store.particle_ramps {
            if ramp.colors.is_empty() {
                continue;
            }
            let Some(lifetime) = store.lifetimes.get(&id) else {
                continue;
            };
            let Some(binding) = store.visuals.get(&id) else {
                continue;
            };
            let Some(ref visual_id) = binding.visual_id else {
                continue;
            };
            out.push((
                id,
                visual_id.clone(),
                ramp.clone(),
                lifetime.ttl_ms,
                lifetime.original_ttl_ms,
            ));
        }
        out
    }

    // =========================================================================
    // ANGULAR BODY - Generic inertia-based rotation
    // =========================================================================

    /// Attach or replace the [`AngularBody`] component for an entity.
    ///
    /// All fields default to `AngularBody::default()` if the entity exists;
    /// pass a pre-built struct to customise config.
    pub fn attach_angular_body(&self, id: u64, body: AngularBody) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.angular_bodies.insert(id, body);
        true
    }

    pub fn angular_body(&self, id: u64) -> Option<AngularBody> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.angular_bodies.get(&id).cloned()
    }

    pub fn with_angular_body<F>(&self, id: u64, f: F) -> bool
    where
        F: FnOnce(&mut AngularBody),
    {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(ab) = store.angular_bodies.get_mut(&id) else {
            return false;
        };
        f(ab);
        true
    }

    pub fn ids_with_angular_body(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.angular_bodies.keys().copied().collect()
    }

    /// Set the per-frame normalised turn input for an [`AngularBody`] entity.
    ///
    /// Call this every frame before the `angular_body_system` tick (which happens
    /// inside the engine game-loop automatically).
    pub fn set_angular_input(&self, id: u64, input: f32) -> bool {
        self.with_angular_body(id, |ab| ab.input = input)
    }

    /// Read the current angular velocity (rad/s) of an [`AngularBody`] entity.
    pub fn angular_vel(&self, id: u64) -> Option<f32> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.angular_bodies.get(&id).map(|ab| ab.angular_vel)
    }

    /// Advance all [`AngularBody`] components by one physics tick.
    ///
    /// Called automatically by `angular_body_system` — do not call from scripts.
    pub fn tick_angular_bodies(&self, dt_ms: u64) {
        let dt = dt_ms as f32 / 1000.0;
        let ids: Vec<u64> = {
            let Ok(store) = self.store.lock() else {
                return;
            };
            store.angular_bodies.keys().copied().collect()
        };
        for id in ids {
            let Ok(mut store) = self.store.lock() else {
                return;
            };
            let Some(ab) = store.angular_bodies.get_mut(&id) else {
                continue;
            };

            if ab.input != 0.0 {
                let torque = ab.input * ab.accel * dt;
                ab.angular_vel = (ab.angular_vel + torque).clamp(-ab.max, ab.max);
            } else if ab.auto_brake && ab.angular_vel.abs() > ab.deadband {
                let brake = -ab.angular_vel.signum() * ab.angular_vel.abs() * 4.5 * dt;
                ab.angular_vel += brake;
                if ab.angular_vel.abs() < ab.deadband {
                    ab.angular_vel = 0.0;
                }
            } else if ab.auto_brake {
                ab.angular_vel = 0.0;
            }

            let angular_vel = ab.angular_vel;
            // Integrate angular velocity into heading
            if let Some(mut xf) = store.transforms.get(&id).copied() {
                xf.heading += angular_vel * dt;
                store.transforms.insert(id, xf);
            }
        }
    }

    // ── LinearBrake ───────────────────────────────────────────────────────

    /// Attach or replace the [`LinearBrake`] component for an entity.
    pub fn attach_linear_brake(&self, id: u64, brake: LinearBrake) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.linear_brakes.insert(id, brake);
        true
    }

    pub fn linear_brake(&self, id: u64) -> Option<LinearBrake> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.linear_brakes.get(&id).cloned()
    }

    pub fn ids_with_linear_brake(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.linear_brakes.keys().copied().collect()
    }

    /// Set the per-frame `active` flag — suppresses braking when true.
    pub fn set_linear_brake_active(&self, id: u64, active: bool) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(lb) = store.linear_brakes.get_mut(&id) else {
            return false;
        };
        lb.active = active;
        true
    }

    /// Apply linear braking to all entities with a [`LinearBrake`] component.
    ///
    /// Called by `linear_brake_system` each physics tick after arcade controller.
    pub fn tick_linear_brakes(&self, dt_ms: u64) {
        let dt = (dt_ms as f32) / 1000.0;
        if dt <= 0.0 {
            return;
        }
        let ids = self.ids_with_linear_brake();
        for id in ids {
            let Ok(mut store) = self.store.lock() else {
                continue;
            };
            let Some(lb) = store.linear_brakes.get_mut(&id) else {
                continue;
            };
            if !lb.auto_brake || lb.active {
                lb.active = false; // reset for next frame
                continue;
            }
            lb.active = false; // reset for next frame
            let decel = lb.decel;
            let deadband = lb.deadband;
            drop(store); // release lock before physics read
            let Ok(mut store) = self.store.lock() else {
                continue;
            };
            let Some(body) = store.physics.get_mut(&id) else {
                continue;
            };
            let speed = (body.vx * body.vx + body.vy * body.vy).sqrt();
            if speed <= deadband {
                body.vx = 0.0;
                body.vy = 0.0;
                continue;
            }
            let impulse = (decel * dt).min(speed);
            body.vx -= (body.vx / speed) * impulse;
            body.vy -= (body.vy / speed) * impulse;
        }
    }

    // ── ThrusterRamp ──────────────────────────────────────────────────────

    /// Attach or replace the [`ThrusterRamp`] component for an entity.
    pub fn attach_thruster_ramp(&self, id: u64, ramp: ThrusterRamp) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.thruster_ramps.insert(id, ramp);
        true
    }

    /// Read the current [`ThrusterRamp`] outputs for an entity (cloned snapshot).
    pub fn thruster_ramp(&self, id: u64) -> Option<ThrusterRamp> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.thruster_ramps.get(&id).cloned()
    }

    /// Returns all entity IDs that have a [`ThrusterRamp`] component.
    pub fn ids_with_thruster_ramp(&self) -> Vec<u64> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store.thruster_ramps.keys().copied().collect()
    }

    /// Detach the [`ThrusterRamp`] component from an entity.
    pub fn detach_thruster_ramp(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        store.thruster_ramps.remove(&id).is_some()
    }

    /// Advance thruster ramp state for all entities that have one.
    ///
    /// Reads `ArcadeController`, `AngularBody`, `LinearBrake`, and `PhysicsBody2D`
    /// for each entity. Writes updated factor outputs back to `ThrusterRamp`.
    /// Called by `thruster_ramp_system` each gameplay tick after `linear_brake_system`.
    pub fn tick_thruster_ramps(&self, dt_ms: u64) {
        let dt = dt_ms as f32;
        if dt <= 0.0 {
            return;
        }

        let ids: Vec<u64> = {
            let Ok(store) = self.store.lock() else {
                return;
            };
            store.thruster_ramps.keys().copied().collect()
        };

        for id in ids {
            // ── Read all inputs in one lock ───────────────────────────────
            let (is_thrusting, rotation_input_active, angular_vel, speed) = {
                let Ok(store) = self.store.lock() else {
                    continue;
                };
                let thrusting = store
                    .controllers
                    .get(&id)
                    .map(|c| c.is_thrusting)
                    .unwrap_or(false);
                let rot_input = store
                    .angular_bodies
                    .get(&id)
                    .map(|ab| ab.input != 0.0)
                    .unwrap_or(false);
                let ang_vel = store
                    .angular_bodies
                    .get(&id)
                    .map(|ab| ab.angular_vel)
                    .unwrap_or(0.0);
                let (vx, vy) = store
                    .physics
                    .get(&id)
                    .map(|p| (p.vx, p.vy))
                    .unwrap_or((0.0, 0.0));
                let speed = (vx * vx + vy * vy).sqrt();
                (thrusting, rot_input, ang_vel, speed)
            };

            let linear_input_active = is_thrusting || rotation_input_active;

            // ── Read ramp config + state ──────────────────────────────────
            let ramp_snap = {
                let Ok(store) = self.store.lock() else {
                    continue;
                };
                store.thruster_ramps.get(&id).cloned()
            };
            let Some(mut ramp) = ramp_snap else {
                continue;
            };

            // Derived state
            let still_rotating = angular_vel.abs() > ramp.rot_deadband;
            let still_moving = speed > ramp.move_deadband;

            // ── No-input accumulator ──────────────────────────────────────
            if linear_input_active {
                ramp.no_input_ms = 0.0;
            } else {
                ramp.no_input_ms += dt;
            }

            // ── Thrust ignition ramp ──────────────────────────────────────
            if is_thrusting {
                ramp.thrust_ignition_ms += dt;
                ramp.thrust_factor = igf(
                    ramp.thrust_ignition_ms,
                    ramp.thrust_delay_ms,
                    ramp.thrust_ramp_ms,
                );
            } else {
                ramp.thrust_ignition_ms = 0.0;
                ramp.thrust_factor = 0.0;
            }

            // ── Rotation factor (derived from angular vel, no ramp needed) ─
            ramp.rot_factor = (angular_vel.abs() / ramp.rot_factor_max_vel).min(1.0);

            // ── Brake ignition ramp ───────────────────────────────────────
            let linear_brake_active =
                ramp.no_input_ms >= ramp.no_input_threshold_ms && still_moving && !still_rotating;

            if still_rotating && !rotation_input_active {
                ramp.brake_ignition_ms += dt;
                ramp.brake_factor = igf(
                    ramp.brake_ignition_ms,
                    ramp.thrust_delay_ms * 0.3,
                    ramp.thrust_ramp_ms * 0.5,
                );
                ramp.brake_phase = BrakePhase::Rotation;
            } else if linear_brake_active {
                // Reset ignition timer on phase entry (Rotation→Linear transition)
                if ramp.brake_phase != BrakePhase::Linear {
                    ramp.brake_ignition_ms = 0.0;
                }
                ramp.brake_ignition_ms += dt;
                ramp.brake_factor = igf(
                    ramp.brake_ignition_ms,
                    ramp.thrust_delay_ms * 0.5,
                    ramp.thrust_ramp_ms * 0.8,
                );
                ramp.brake_phase = BrakePhase::Linear;
            } else if !linear_input_active && !still_rotating && !still_moving {
                ramp.brake_ignition_ms = 0.0;
                ramp.brake_phase = BrakePhase::Stopped;
            } else if is_thrusting && !rotation_input_active && !still_rotating {
                ramp.brake_phase = BrakePhase::Thrusting;
            } else if rotation_input_active {
                ramp.brake_ignition_ms = 0.0;
                ramp.brake_phase = BrakePhase::Idle;
            }

            // ── Final stabilisation burst ─────────────────────────────────
            ramp.final_burst_fired = false;
            ramp.final_burst_wave = 0;

            let burst_trigger_zone = linear_brake_active
                && speed < ramp.burst_speed_threshold
                && speed > ramp.move_deadband;

            if burst_trigger_zone {
                if !ramp.final_burst_triggered {
                    ramp.final_burst_triggered = true;
                    ramp.final_burst_waves = 0;
                    ramp.final_burst_timer_ms = 0.0;
                }
                if ramp.final_burst_waves < ramp.burst_wave_count {
                    ramp.final_burst_timer_ms += dt;
                    if ramp.final_burst_timer_ms >= ramp.burst_wave_interval_ms {
                        ramp.final_burst_fired = true;
                        ramp.final_burst_wave = ramp.final_burst_waves;
                        ramp.final_burst_waves += 1;
                        ramp.final_burst_timer_ms = 0.0;
                    }
                }
            } else if !still_moving && !still_rotating {
                ramp.final_burst_triggered = false;
                ramp.final_burst_waves = 0;
                ramp.final_burst_timer_ms = 0.0;
            }
            if linear_input_active {
                ramp.final_burst_triggered = false;
                ramp.final_burst_waves = 0;
                ramp.final_burst_timer_ms = 0.0;
            }

            // ── Write outputs back ────────────────────────────────────────
            let Ok(mut store) = self.store.lock() else {
                continue;
            };
            store.thruster_ramps.insert(id, ramp);
        }
    }

    pub fn batch_read_physics(
        &self,
        ids: &[u64],
    ) -> Vec<(u64, Transform2D, PhysicsBody2D, Option<ParticlePhysics>)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        ids.iter()
            .filter_map(|&id| {
                let xf = store.transforms.get(&id)?;
                let body = store.physics.get(&id)?;
                let pp = store.particle_physics.get(&id).cloned();
                Some((id, *xf, *body, pp))
            })
            .collect()
    }

    pub fn batch_read_physics3d(&self, ids: &[u64]) -> Vec<(u64, Transform3D, PhysicsBody3D)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        ids.iter()
            .filter_map(|&id| {
                let xf = store.transforms_3d.get(&id)?;
                let body = store.physics_3d.get(&id)?;
                Some((id, *xf, *body))
            })
            .collect()
    }

    pub fn batch_read_all_physics3d(&self) -> Vec<(u64, Transform3D, PhysicsBody3D)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .physics_3d
            .keys()
            .filter_map(|&id| {
                let xf = store.transforms_3d.get(&id)?;
                let body = store.physics_3d.get(&id)?;
                Some((id, *xf, *body))
            })
            .collect()
    }

    pub fn batch_write_physics3d(&self, results: &[(u64, Transform3D, PhysicsBody3D)]) {
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        for (id, xf, body) in results {
            if store.entities.contains_key(id) {
                store.transforms_3d.insert(*id, *xf);
                store.physics_3d.insert(*id, *body);
            }
        }
    }

    pub fn batch_read_control_intents3d(&self) -> Vec<(u64, ControlIntent3D)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .control_intents_3d
            .iter()
            .map(|(&id, intent)| (id, *intent))
            .collect()
    }

    pub fn batch_read_reference_frames3d(
        &self,
    ) -> Vec<(u64, ReferenceFrameBinding3D, Option<ReferenceFrameState3D>)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .reference_frame_bindings_3d
            .iter()
            .map(|(&id, binding)| {
                (
                    id,
                    binding.clone(),
                    store.reference_frame_state_3d.get(&id).copied(),
                )
            })
            .collect()
    }

    pub fn batch_read_motor_stack3d(
        &self,
    ) -> Vec<(
        u64,
        Option<LinearMotor3D>,
        Option<AngularMotor3D>,
        Option<CharacterMotor3D>,
        Option<FlightMotor3D>,
    )> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        let mut ids = BTreeSet::new();
        ids.extend(store.linear_motors_3d.keys().copied());
        ids.extend(store.angular_motors_3d.keys().copied());
        ids.extend(store.character_motors_3d.keys().copied());
        ids.extend(store.flight_motors_3d.keys().copied());
        ids.into_iter()
            .map(|id| {
                (
                    id,
                    store.linear_motors_3d.get(&id).copied(),
                    store.angular_motors_3d.get(&id).copied(),
                    store.character_motors_3d.get(&id).copied(),
                    store.flight_motors_3d.get(&id).copied(),
                )
            })
            .collect()
    }

    /// Batch read ALL physics entities in a single lock acquisition.
    pub fn batch_read_all_physics(
        &self,
    ) -> Vec<(u64, Transform2D, PhysicsBody2D, Option<ParticlePhysics>)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .physics
            .keys()
            .filter_map(|&id| {
                let xf = store.transforms.get(&id)?;
                let body = store.physics.get(&id)?;
                let pp = store.particle_physics.get(&id).cloned();
                Some((id, *xf, *body, pp))
            })
            .collect()
    }

    /// Batch read physics for entities processed inline (thread_mode=Light or no particle physics).
    /// Worker-thread particles (Physics/Gravity) are excluded — handled by async path.
    pub fn batch_read_inline_physics(
        &self,
    ) -> Vec<(u64, Transform2D, PhysicsBody2D, Option<ParticlePhysics>)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .physics
            .keys()
            .filter_map(|&id| {
                let pp = store.particle_physics.get(&id).cloned();
                if pp
                    .as_ref()
                    .map(|p| p.thread_mode.uses_worker_thread())
                    .unwrap_or(false)
                {
                    return None; // skip — will be processed by async particle system
                }
                let xf = store.transforms.get(&id)?;
                let body = store.physics.get(&id)?;
                Some((id, *xf, *body, pp))
            })
            .collect()
    }

    /// Batch read physics for worker-thread particles only (thread_mode=Physics|Gravity).
    pub fn batch_read_worker_physics(
        &self,
    ) -> Vec<(u64, Transform2D, PhysicsBody2D, ParticlePhysics)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        store
            .particle_physics
            .iter()
            .filter(|(_, pp)| pp.thread_mode.uses_worker_thread())
            .filter_map(|(&id, pp)| {
                let xf = store.transforms.get(&id)?;
                let body = store.physics.get(&id)?;
                Some((id, *xf, *body, pp.clone()))
            })
            .collect()
    }

    /// Batch write physics results in a single lock acquisition.
    pub fn batch_write_physics(&self, results: &[(u64, Transform2D, PhysicsBody2D)]) {
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        for (id, xf, body) in results {
            if store.entities.contains_key(id) {
                store.transforms.insert(*id, *xf);
                store.physics.insert(*id, *body);
            }
        }
    }

    /// Batch read lifecycle data for parallel TTL computation.
    /// Returns (id, ttl_ms, lifecycle_policy, owner_id).
    pub fn batch_read_lifecycle(
        &self,
        ids: &[u64],
    ) -> Vec<(u64, i32, LifecyclePolicy, Option<u64>)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        ids.iter()
            .filter_map(|&id| {
                let policy = store.lifecycles.get(&id)?;
                let lt = store.lifetimes.get(&id)?;
                let owner = store.ownership.get(&id).map(|o| o.owner_id);
                Some((id, lt.ttl_ms, *policy, owner))
            })
            .collect()
    }

    /// Batch write TTL updates in a single lock acquisition.
    pub fn batch_write_ttl(&self, updates: &[(u64, i32)]) {
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        for (id, new_ttl) in updates {
            if let Some(lt) = store.lifetimes.get_mut(id) {
                lt.ttl_ms = *new_ttl;
            }
        }
    }

    /// Batch read transforms only (for rendering/collision).
    pub fn batch_read_transforms(&self, ids: &[u64]) -> Vec<(u64, Transform2D)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        ids.iter()
            .filter_map(|&id| store.transforms.get(&id).map(|xf| (id, *xf)))
            .collect()
    }

    /// Batch-read transform + visual_id for all entities with visual bindings.
    /// Returns `(visual_id, x, y, z, heading)` tuples in a single lock.
    pub fn batch_read_visual_sync(&self) -> Vec<(String, f32, f32, f32, f32)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };
        let mut out = Vec::with_capacity(store.visuals.len());
        for (&id, binding) in &store.visuals {
            let Some(ref visual_id) = binding.visual_id else {
                continue;
            };
            let Some(xf) = store.transforms.get(&id) else {
                continue;
            };
            out.push((visual_id.clone(), xf.x, xf.y, xf.z, xf.heading));
        }
        out
    }

    /// Batch write transforms only.
    pub fn batch_write_transforms(&self, updates: &[(u64, Transform2D)]) {
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        for (id, xf) in updates {
            if store.entities.contains_key(id) {
                store.transforms.insert(*id, *xf);
            }
        }
    }

    /// Emit a gameplay event to be collected and polled by scripts.
    pub fn emit_event(&self, event: GameplayEvent) {
        if let Ok(mut store) = self.store.lock() {
            store.events.push(event);
        }
    }

    /// Get all events of a specific type without clearing the buffer.
    ///
    /// Note: This returns copies of events without clearing. Call clear_events() separately.
    pub fn poll_events(&self, event_type: &str) -> Vec<(u64, u64)> {
        let Ok(store) = self.store.lock() else {
            return Vec::new();
        };

        let mut results = Vec::new();
        if event_type == "collision_enter" {
            for event in &store.events {
                let GameplayEvent::CollisionEnter { a, b } = event;
                results.push((*a, *b));
            }
        }
        results
    }

    /// Clear all accumulated events (call at start of frame before polling).
    pub fn clear_events(&self) {
        if let Ok(mut store) = self.store.lock() {
            store.events.clear();
        }
    }

    /// Reads a value from an entity payload using JSON pointer notation.
    pub fn get(&self, id: u64, path: &str) -> Option<JsonValue> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        let entity = store.entities.get(&id)?;
        get_path(&entity.data, path)
    }

    /// Writes a value into an entity payload using JSON pointer notation.
    pub fn set(&self, id: u64, path: &str, value: JsonValue) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get_mut(&id) else {
            return false;
        };
        set_path(&mut entity.data, path, value)
    }

    /// Checks if a value exists at `path` in the entity payload.
    pub fn has(&self, id: u64, path: &str) -> bool {
        let Ok(store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get(&id) else {
            return false;
        };
        get_path(&entity.data, path).is_some()
    }

    /// Removes a value at `path` in the entity payload.
    pub fn remove(&self, id: u64, path: &str) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get_mut(&id) else {
            return false;
        };
        remove_path(&mut entity.data, path)
    }

    /// Pushes a value into an array at `path` in the entity payload.
    pub fn push(&self, id: u64, path: &str, value: JsonValue) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get_mut(&id) else {
            return false;
        };
        push_path(&mut entity.data, path, value)
    }

    // ── RNG ──────────────────────────────────────────────────────────────

    /// LCG random int in [min, max] inclusive. Advances internal seed.
    pub fn rand_i(&self, min: i32, max: i32) -> i32 {
        let Ok(mut store) = self.store.lock() else {
            return min;
        };
        store.rng_seed = store.rng_seed.wrapping_mul(1103515245).wrapping_add(12345) & 0x7fff_ffff;
        let range = (max - min).unsigned_abs() as u64 + 1;
        min + (store.rng_seed % range) as i32
    }

    /// Reset the RNG seed.
    pub fn rand_seed(&self, seed: i64) {
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        store.rng_seed = seed as u64 & 0x7fff_ffff;
    }

    // ── World-level wrap bounds ───────────────────────────────────────────

    /// Store global world bounds (used by enable_wrap_bounds).
    pub fn set_world_bounds(&self, min_x: f32, max_x: f32, min_y: f32, max_y: f32) {
        let Ok(mut store) = self.store.lock() else {
            return;
        };
        store.world_bounds = Some(WrapBounds {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z: 0.0,
            max_z: 0.0,
        });
    }

    /// Read global world bounds.
    pub fn world_bounds(&self) -> Option<WrapBounds> {
        let Ok(store) = self.store.lock() else {
            return None;
        };
        store.world_bounds
    }

    /// Enable wrap on entity using stored world bounds. No-op if world bounds not set.
    pub fn enable_wrap_bounds(&self, id: u64) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(bounds) = store.world_bounds else {
            return false;
        };
        if !store.entities.contains_key(&id) {
            return false;
        }
        store.wrap_bounds.insert(id, bounds);
        true
    }

    // ── Entity tag mutation ───────────────────────────────────────────────

    /// Add a runtime tag to an entity. Returns false if entity does not exist.
    pub fn tag_add(&self, id: u64, tag: &str) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get_mut(&id) else {
            return false;
        };
        if entity.tags.insert(tag.to_string()) {
            // Newly inserted — update O(1) counter.
            *store.tag_counts.entry(tag.to_string()).or_insert(0) += 1;
        }
        true
    }

    /// Remove a runtime tag from an entity.
    pub fn tag_remove(&self, id: u64, tag: &str) -> bool {
        let Ok(mut store) = self.store.lock() else {
            return false;
        };
        let Some(entity) = store.entities.get_mut(&id) else {
            return false;
        };
        let removed = entity.tags.remove(tag);
        if removed {
            if let Some(c) = store.tag_counts.get_mut(tag) {
                *c = c.saturating_sub(1);
            }
        }
        removed
    }

    /// Check if an entity has a specific runtime tag.
    pub fn tag_has(&self, id: u64, tag: &str) -> bool {
        let Ok(store) = self.store.lock() else {
            return false;
        };
        store
            .entities
            .get(&id)
            .map(|e| e.tags.contains(tag))
            .unwrap_or(false)
    }
}

impl Default for GameplayWorld {
    fn default() -> Self {
        Self::new()
    }
}

fn split_payload(payload: JsonValue) -> (BTreeSet<String>, JsonValue) {
    let mut tags = BTreeSet::new();
    let data = match payload {
        JsonValue::Object(mut map) => {
            if let Some(JsonValue::Array(values)) = map.remove("tags") {
                for value in values {
                    if let Some(tag) = value.as_str().map(str::trim).filter(|tag| !tag.is_empty()) {
                        tags.insert(tag.to_string());
                    }
                }
            }

            JsonValue::Object(map)
        }
        other => return (tags, other),
    };
    (tags, data)
}

fn get_path(payload: &JsonValue, path: &str) -> Option<JsonValue> {
    if path.is_empty() || path == "/" {
        return Some(payload.clone());
    }
    // JSON Pointer (RFC 6901) requires a leading '/'.
    // set_path accepts bare keys ("hp") by stripping any leading slash before
    // splitting, so we must normalise the same way here.
    let pointer = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    payload.pointer(&pointer).cloned()
}

fn set_path(payload: &mut JsonValue, path: &str, value: JsonValue) -> bool {
    if path.is_empty() || path == "/" {
        *payload = value;
        return true;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let (parent_parts, key) = match parts.split_last() {
        Some((k, p)) => (p, *k),
        None => return false,
    };
    let mut current = payload;
    for &part in parent_parts {
        if !current.is_object() {
            *current = JsonValue::Object(JsonMap::new());
        }
        let Some(obj) = current.as_object_mut() else {
            return false;
        };
        if !obj.contains_key(part) {
            obj.insert(part.to_string(), json!({}));
        }
        let Some(next) = obj.get_mut(part) else {
            return false;
        };
        current = next;
    }
    if !current.is_object() {
        *current = JsonValue::Object(JsonMap::new());
    }
    let Some(obj) = current.as_object_mut() else {
        return false;
    };
    obj.insert(key.to_string(), value);
    true
}

fn remove_path(payload: &mut JsonValue, path: &str) -> bool {
    if path.is_empty() || path == "/" {
        return false;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let (parent_parts, key) = match parts.split_last() {
        Some((k, p)) => (p, *k),
        None => return false,
    };
    let mut current = payload;
    for &part in parent_parts {
        let Some(obj) = current.as_object_mut() else {
            return false;
        };
        let Some(next) = obj.get_mut(part) else {
            return false;
        };
        current = next;
    }
    let Some(obj) = current.as_object_mut() else {
        return false;
    };
    obj.remove(key).is_some()
}

fn push_path(payload: &mut JsonValue, path: &str, value: JsonValue) -> bool {
    if path.is_empty() || path == "/" {
        return false;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let (parent_parts, key) = match parts.split_last() {
        Some((k, p)) => (p, *k),
        None => return false,
    };
    let mut current = payload;
    for &part in parent_parts {
        if !current.is_object() {
            *current = JsonValue::Object(JsonMap::new());
        }
        let Some(obj) = current.as_object_mut() else {
            return false;
        };
        if !obj.contains_key(part) {
            obj.insert(part.to_string(), json!({}));
        }
        let Some(next) = obj.get_mut(part) else {
            return false;
        };
        current = next;
    }
    if !current.is_object() {
        *current = JsonValue::Object(JsonMap::new());
    }
    let Some(obj) = current.as_object_mut() else {
        return false;
    };
    let entry = obj.entry(key.to_string()).or_insert_with(|| json!([]));
    if let Some(arr) = entry.as_array_mut() {
        arr.push(value);
    } else {
        let prev = entry.clone();
        *entry = json!([prev, value]);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::GameplayWorld;
    use crate::components::{
        AngularBody, AngularMotor3D, ArcadeController, Assembly3D, BootstrapAssembly3D,
        BootstrapPreset3D, CharacterMotor3D, ComponentBundle3D, ControlBundle3D, ControlIntent3D,
        FlightMotor3D, FollowAnchor2D, FollowAnchor3D, LifecyclePolicy, LinearBrake, LinearMotor3D,
        MotorBundle3D, PhysicsBody2D, PhysicsBody3D, ReferenceFrameBinding3D, ReferenceFrameMode,
        ReferenceFrameState3D, SpatialBundle3D, SpatialKind, ThrusterRamp, Transform2D,
        Transform3D,
    };
    use engine_vehicle::{BrakePhase, VehicleProfile};
    use serde_json::json;

    #[test]
    fn spawns_queries_and_mutates_entities() {
        let world = GameplayWorld::new();
        let id = world
            .spawn(
                "projectile",
                json!({
                    "tags": ["projectile", "controlled"],
                    "x": 10,
                    "y": 20
                }),
            )
            .expect("spawn should return an id");
        assert!(world.exists(id));
        assert_eq!(world.kind_of(id).as_deref(), Some("projectile"));
        assert_eq!(world.query_kind("projectile"), vec![id]);
        assert_eq!(world.query_tag("controlled"), vec![id]);
        assert_eq!(world.get(id, "/x"), Some(json!(10)));
        assert!(world.set(id, "/velocity/x", json!(4)));
        assert_eq!(world.get(id, "/velocity/x"), Some(json!(4)));
        assert!(world.remove(id, "/velocity/x"));
        assert!(!world.has(id, "/velocity/x"));
        assert!(world.despawn(id));
        assert!(!world.exists(id));
    }

    #[test]
    fn clear_resets_world() {
        let world = GameplayWorld::new();
        assert!(world.spawn("enemy", json!({"x": 1})).is_some());
        assert_eq!(world.count(), 1);
        world.clear();
        assert_eq!(world.count(), 0);
        assert!(world.ids().is_empty());
        assert_eq!(world.query_kind("enemy"), Vec::<u64>::new());
    }

    #[test]
    fn controlled_entity_round_trip_and_clears_on_despawn() {
        let world = GameplayWorld::new();
        let pilot = world.spawn("pilot", json!({})).expect("pilot");
        let backup = world.spawn("backup", json!({})).expect("backup");

        assert!(!world.set_controlled_entity(0));
        assert_eq!(world.controlled_entity(), None);
        assert!(world.set_controlled_entity(pilot));
        assert_eq!(world.controlled_entity(), Some(pilot));
        assert!(world.clear_controlled_entity());
        assert_eq!(world.controlled_entity(), None);
        assert!(!world.clear_controlled_entity());

        assert!(world.set_controlled_entity(backup));
        assert_eq!(world.controlled_entity(), Some(backup));
        assert!(world.despawn(backup));
        assert_eq!(world.controlled_entity(), None);
    }

    #[test]
    fn register_child_records_ownership_and_parent_despawn_removes_child() {
        let world = GameplayWorld::new();
        let parent = world.spawn("parent", json!({})).expect("parent");
        let child = world.spawn("child", json!({})).expect("child");

        assert!(world.set_lifecycle(child, LifecyclePolicy::OwnerBound));
        assert!(world.register_child(parent, child));
        assert_eq!(
            world.ownership(child).map(|ownership| ownership.owner_id),
            Some(parent)
        );

        assert!(world.despawn(parent));
        assert!(!world.exists(parent));
        assert!(!world.exists(child));
    }

    #[test]
    fn apply_follow_anchors_tracks_owner_transform_and_heading() {
        let world = GameplayWorld::new();
        let owner = world.spawn("owner", json!({})).expect("owner");
        let child = world.spawn("child", json!({})).expect("child");
        assert!(world.register_child(owner, child));
        assert!(world.set_transform(
            owner,
            Transform2D {
                x: 10.0,
                y: 20.0,
                z: 0.0,
                heading: std::f32::consts::FRAC_PI_2,
            },
        ));
        assert!(world.set_transform(
            child,
            Transform2D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                heading: 0.0,
            },
        ));
        assert!(world.set_lifecycle(child, LifecyclePolicy::TtlFollowOwner));
        assert!(world.set_follow_anchor(
            child,
            FollowAnchor2D {
                local_x: -4.0,
                local_y: 2.0,
                inherit_heading: true,
            },
        ));

        world.apply_follow_anchors();

        let child_xf = world.transform(child).expect("child transform");
        assert!((child_xf.x - 8.0).abs() < 0.001);
        assert!((child_xf.y - 16.0).abs() < 0.001);
        assert!((child_xf.heading - std::f32::consts::FRAC_PI_2).abs() < 0.001);
    }

    #[test]
    fn vehicle_profile_snapshot_syncs_runtime_fields_and_preserves_metadata() {
        let world = GameplayWorld::new();
        let ship = world.spawn("vehicle", json!({})).expect("ship");

        let mut controller = ArcadeController::new(90, 12.5, 18.0, 16);
        controller.set_thrust(true);
        assert!(world.attach_controller(ship, controller));
        assert!(world.attach_angular_body(
            ship,
            AngularBody {
                accel: 2.5,
                max: 4.0,
                deadband: 0.2,
                auto_brake: false,
                input: 0.0,
                angular_vel: 0.0,
            }
        ));
        assert!(world.attach_linear_brake(
            ship,
            LinearBrake {
                decel: 9.5,
                deadband: 1.25,
                auto_brake: true,
                active: false,
            }
        ));
        assert!(world.attach_thruster_ramp(ship, ThrusterRamp::default()));
        assert!(world.attach_vehicle_profile(
            ship,
            VehicleProfile {
                profile_id: "sim-lite".to_string(),
                label: Some("Sim Lite".to_string()),
                ..VehicleProfile::default()
            }
        ));

        let snapshot = world
            .snapshot_vehicle_profile(ship)
            .expect("vehicle profile snapshot");
        assert_eq!(snapshot.profile_id, "sim-lite");
        assert_eq!(snapshot.label.as_deref(), Some("Sim Lite"));
        assert_eq!(snapshot.heading_bits, Some(16));
        assert_eq!(snapshot.turn_step_ms, Some(90));
        assert_eq!(snapshot.thrust_power, 12.5);
        assert_eq!(snapshot.max_speed, 18.0);
        assert_eq!(snapshot.angular_accel, 2.5);
        assert_eq!(snapshot.angular_max, 4.0);
        assert_eq!(snapshot.angular_deadband, 0.2);
        assert!(!snapshot.angular_auto_brake);
        assert_eq!(snapshot.linear_brake_decel, 9.5);
        assert_eq!(snapshot.linear_brake_deadband, 1.25);
        assert!(snapshot.linear_auto_brake);
        assert!(snapshot.thruster_ramp_enabled);

        let synced = world.sync_vehicle_profile(ship).expect("synced profile");
        assert_eq!(synced, snapshot);
        assert_eq!(world.vehicle_profile(ship), Some(snapshot.clone()));
        assert_eq!(world.ids_with_vehicle_profile(), vec![ship]);
        assert_eq!(world.ids_with_vehicle_state(), vec![ship]);
        assert!(world.detach_vehicle_profile(ship));
        assert_eq!(world.vehicle_profile(ship), None);
    }

    #[test]
    fn vehicle_telemetry_snapshot_and_sync_follow_runtime_state() {
        let world = GameplayWorld::new();
        let ship = world.spawn("vehicle", json!({})).expect("ship");

        assert!(world.set_transform(
            ship,
            Transform2D {
                x: 12.0,
                y: -3.0,
                z: 0.0,
                heading: std::f32::consts::FRAC_PI_2,
            },
        ));
        assert!(world.set_physics(
            ship,
            PhysicsBody2D {
                vx: 3.0,
                vy: 4.0,
                vz: 0.0,
                ax: 1.0,
                ay: 2.0,
                az: 0.0,
                drag: 0.0,
                max_speed: 20.0,
                mass: 1.0,
                restitution: 0.7,
            },
        ));

        let mut controller = ArcadeController::new(60, 8.0, 20.0, 32);
        controller.set_turn(1);
        controller.set_thrust(true);
        assert!(world.attach_controller(ship, controller));
        assert!(world.attach_angular_body(
            ship,
            AngularBody {
                accel: 3.0,
                max: 6.0,
                deadband: 0.1,
                auto_brake: true,
                input: -0.25,
                angular_vel: 1.5,
            }
        ));
        assert!(world.attach_linear_brake(
            ship,
            LinearBrake {
                decel: 12.0,
                deadband: 0.5,
                auto_brake: true,
                active: false,
            }
        ));
        assert!(world.attach_thruster_ramp(
            ship,
            ThrusterRamp {
                thrust_factor: 0.8,
                rot_factor: 0.25,
                brake_factor: 0.6,
                brake_phase: BrakePhase::Linear,
                final_burst_fired: true,
                final_burst_wave: 2,
                ..ThrusterRamp::default()
            }
        ));

        let snapshot = world
            .snapshot_vehicle_telemetry(ship)
            .expect("vehicle telemetry snapshot");
        assert!((snapshot.facing.forward_x - 1.0).abs() < 0.001);
        assert!(snapshot.facing.forward_y.abs() < 0.001);
        assert_eq!(snapshot.motion.velocity_x, 3.0);
        assert_eq!(snapshot.motion.velocity_y, 4.0);
        assert!((snapshot.motion.speed - 5.0).abs() < 0.001);
        assert!((snapshot.motion.forward_speed - 3.0).abs() < 0.001);
        assert!((snapshot.motion.lateral_speed - 4.0).abs() < 0.001);
        assert!((snapshot.motion.forward_accel - 1.0).abs() < 0.001);
        assert!((snapshot.motion.lateral_accel - 2.0).abs() < 0.001);
        assert_eq!(snapshot.turn_input, -0.25);
        assert_eq!(snapshot.thrust_input, 1.0);
        assert!(snapshot.is_thrusting);
        assert!(snapshot.is_braking);
        assert_eq!(snapshot.angular_vel, 1.5);
        assert_eq!(snapshot.thrust_factor, 0.8);
        assert_eq!(snapshot.rot_factor, 0.25);
        assert_eq!(snapshot.brake_factor, 0.6);
        assert_eq!(snapshot.brake_phase, BrakePhase::Linear);
        assert!(snapshot.final_burst_fired);
        assert_eq!(snapshot.final_burst_wave, 2);

        let synced = world
            .sync_vehicle_runtime_state(ship)
            .expect("synced vehicle runtime state");
        assert_eq!(synced.1, snapshot);
        assert_eq!(world.vehicle_telemetry(ship), Some(snapshot));
        assert_eq!(world.ids_with_vehicle_telemetry(), vec![ship]);

        assert!(world.despawn(ship));
        assert_eq!(world.vehicle_telemetry(ship), None);
        assert_eq!(world.snapshot_vehicle_telemetry(ship), None);
    }

    #[test]
    fn stores_and_reads_true_3d_components_without_affecting_legacy_2d() {
        let world = GameplayWorld::new();
        let entity = world.spawn("pilot", json!({})).expect("entity");

        let transform = Transform3D {
            position: [1.0, 2.0, 3.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        };
        let physics = PhysicsBody3D {
            linear_velocity: [4.0, 5.0, 6.0],
            angular_velocity: [0.1, 0.2, 0.3],
            ..PhysicsBody3D::default()
        };
        let intent = ControlIntent3D {
            move_local: [1.0, 0.0, -1.0],
            throttle: 0.75,
            boost: true,
            ..ControlIntent3D::default()
        };
        let frame = ReferenceFrameBinding3D {
            mode: ReferenceFrameMode::LocalHorizon,
            body_id: Some("terra".to_string()),
            inherit_linear_velocity: true,
            ..ReferenceFrameBinding3D::default()
        };
        let frame_state = ReferenceFrameState3D {
            altitude_km: 42.0,
            ..ReferenceFrameState3D::default()
        };
        let anchor = FollowAnchor3D {
            local_offset: [0.0, 1.0, -3.0],
            inherit_orientation: true,
        };

        assert!(world.set_transform3d(entity, transform));
        assert!(world.set_physics3d(entity, physics));
        assert!(world.attach_control_intent3d(entity, intent));
        assert!(world.attach_reference_frame3d(entity, frame.clone()));
        assert!(world.set_reference_frame_state3d(entity, frame_state));
        assert!(world.set_follow_anchor3d(entity, anchor));
        assert!(world.attach_linear_motor3d(entity, LinearMotor3D::default()));
        assert!(world.attach_angular_motor3d(entity, AngularMotor3D::default()));
        assert!(world.attach_character_motor3d(entity, CharacterMotor3D::default()));
        assert!(world.attach_flight_motor3d(entity, FlightMotor3D::default()));

        assert_eq!(world.transform3d(entity), Some(transform));
        assert_eq!(world.physics3d(entity), Some(physics));
        assert_eq!(world.control_intent3d(entity), Some(intent));
        assert_eq!(world.reference_frame3d(entity), Some(frame));
        assert_eq!(world.reference_frame_state3d(entity), Some(frame_state));
        assert_eq!(world.follow_anchor3d(entity), Some(anchor));
        assert_eq!(world.spatial_kind(entity), Some(SpatialKind::ThreeD));
        assert_eq!(world.transform(entity), None);
        assert_eq!(world.physics(entity), None);
    }

    #[test]
    fn three_d_batch_accessors_round_trip_and_cleanup_on_despawn() {
        let world = GameplayWorld::new();
        let entity = world.spawn("ship", json!({})).expect("entity");

        let transform = Transform3D {
            position: [10.0, 20.0, 30.0],
            ..Transform3D::default()
        };
        let physics = PhysicsBody3D {
            linear_velocity: [2.0, 3.0, 4.0],
            ..PhysicsBody3D::default()
        };

        assert!(world.set_transform3d(entity, transform));
        assert!(world.set_physics3d(entity, physics));

        assert_eq!(world.ids_with_transform3d(), vec![entity]);
        assert_eq!(world.ids_with_physics3d(), vec![entity]);
        assert_eq!(
            world.batch_read_physics3d(&[entity]),
            vec![(entity, transform, physics)]
        );
        assert_eq!(
            world.batch_read_all_physics3d(),
            vec![(entity, transform, physics)]
        );

        let next = (
            entity,
            Transform3D {
                position: [11.0, 22.0, 33.0],
                ..transform
            },
            PhysicsBody3D {
                linear_velocity: [5.0, 6.0, 7.0],
                ..physics
            },
        );
        world.batch_write_physics3d(&[next]);

        assert_eq!(world.transform3d(entity), Some(next.1));
        assert_eq!(world.physics3d(entity), Some(next.2));

        assert!(world.despawn(entity));
        assert_eq!(world.transform3d(entity), None);
        assert_eq!(world.physics3d(entity), None);
        assert_eq!(world.ids_with_transform3d(), Vec::<u64>::new());
        assert_eq!(world.ids_with_physics3d(), Vec::<u64>::new());
    }

    #[test]
    fn three_d_defaults_and_accessors_reject_missing_entities() {
        let world = GameplayWorld::new();
        assert_eq!(Transform3D::default().orientation, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(PhysicsBody3D::default().mass, 1.0);
        assert_eq!(
            ReferenceFrameBinding3D::default().mode,
            ReferenceFrameMode::World
        );
        assert_eq!(ReferenceFrameState3D::default().basis_up, [0.0, 1.0, 0.0]);
        assert!(LinearMotor3D::default().boost_scale > 0.0);

        assert!(!world.set_transform3d(999, Transform3D::default()));
        assert!(!world.set_physics3d(999, PhysicsBody3D::default()));
        assert!(!world.attach_control_intent3d(999, ControlIntent3D::default()));
        assert!(!world.attach_reference_frame3d(999, ReferenceFrameBinding3D::default()));
        assert!(!world.set_reference_frame_state3d(999, ReferenceFrameState3D::default()));
        assert!(!world.set_follow_anchor3d(999, FollowAnchor3D::default()));
        assert!(!world.attach_linear_motor3d(999, LinearMotor3D::default()));
        assert!(!world.attach_angular_motor3d(999, AngularMotor3D::default()));
        assert!(!world.attach_character_motor3d(999, CharacterMotor3D::default()));
        assert!(!world.attach_flight_motor3d(999, FlightMotor3D::default()));
        assert_eq!(world.spatial_kind(999), None);
    }

    #[test]
    fn three_d_helpers_batch_and_mutation_cover_full_runtime_stack() {
        let world = GameplayWorld::new();
        let entity = world.spawn("drone", json!({})).expect("entity");

        assert!(world.set_transform3d(entity, Transform3D::default()));
        assert!(world.set_physics3d(entity, PhysicsBody3D::default()));
        assert!(world.attach_control_intent3d(entity, ControlIntent3D::default()));
        assert!(world.attach_reference_frame3d(entity, ReferenceFrameBinding3D::default()));
        assert!(world.set_reference_frame_state3d(entity, ReferenceFrameState3D::default()));
        assert!(world.set_follow_anchor3d(entity, FollowAnchor3D::default()));
        assert!(world.attach_linear_motor3d(entity, LinearMotor3D::default()));
        assert!(world.attach_angular_motor3d(entity, AngularMotor3D::default()));
        assert!(world.attach_character_motor3d(entity, CharacterMotor3D::default()));
        assert!(world.attach_flight_motor3d(entity, FlightMotor3D::default()));

        assert!(world.with_transform3d(entity, |xf| xf.position = [7.0, 8.0, 9.0]));
        assert!(world.with_physics3d(entity, |body| body.linear_velocity = [1.0, 2.0, 3.0]));
        assert!(world.with_control_intent3d(entity, |intent| intent.throttle = 0.5));
        assert!(world.with_reference_frame3d(entity, |frame| {
            frame.mode = ReferenceFrameMode::Orbital;
        }));
        assert!(world.with_reference_frame_state3d(entity, |state| {
            state.altitude_km = 123.0;
        }));
        assert!(world.with_linear_motor3d(entity, |motor| motor.accel = 4.0));
        assert!(world.with_angular_motor3d(entity, |motor| motor.yaw_rate = 2.0));
        assert!(world.with_character_motor3d(entity, |motor| motor.jump_speed = 6.0));
        assert!(world.with_flight_motor3d(entity, |motor| {
            motor.horizon_lock_strength = 0.25;
        }));

        assert_eq!(world.ids_with_any_3d(), vec![entity]);
        assert_eq!(
            world.batch_read_control_intents3d(),
            vec![(
                entity,
                ControlIntent3D {
                    throttle: 0.5,
                    ..ControlIntent3D::default()
                }
            )]
        );
        assert_eq!(
            world.batch_read_reference_frames3d(),
            vec![(
                entity,
                ReferenceFrameBinding3D {
                    mode: ReferenceFrameMode::Orbital,
                    ..ReferenceFrameBinding3D::default()
                },
                Some(ReferenceFrameState3D {
                    altitude_km: 123.0,
                    ..ReferenceFrameState3D::default()
                })
            )]
        );
        assert_eq!(
            world.batch_read_motor_stack3d(),
            vec![(
                entity,
                Some(LinearMotor3D {
                    accel: 4.0,
                    ..LinearMotor3D::default()
                }),
                Some(AngularMotor3D {
                    yaw_rate: 2.0,
                    ..AngularMotor3D::default()
                }),
                Some(CharacterMotor3D {
                    jump_speed: 6.0,
                    ..CharacterMotor3D::default()
                }),
                Some(FlightMotor3D {
                    horizon_lock_strength: 0.25,
                    ..FlightMotor3D::default()
                }),
            )]
        );

        assert!(world.remove_control_intent3d(entity));
        assert!(world.remove_reference_frame3d(entity));
        assert!(world.remove_follow_anchor3d(entity));
        assert!(world.remove_linear_motor3d(entity));
        assert!(world.remove_angular_motor3d(entity));
        assert!(world.remove_character_motor3d(entity));
        assert!(world.remove_flight_motor3d(entity));
        assert!(world.remove_physics3d(entity));
        assert!(world.remove_transform3d(entity));
        assert_eq!(world.ids_with_any_3d(), Vec::<u64>::new());
        assert_eq!(world.spatial_kind(entity), Some(SpatialKind::TwoD));
    }

    #[test]
    fn component_bundle3d_round_trips_and_detaches_as_one_unit() {
        let world = GameplayWorld::new();
        let entity = world.spawn("probe", json!({})).expect("entity");
        let bundle = ComponentBundle3D {
            transform: Some(Transform3D {
                position: [3.0, 4.0, 5.0],
                ..Transform3D::default()
            }),
            physics: Some(PhysicsBody3D {
                linear_velocity: [1.0, 2.0, 3.0],
                ..PhysicsBody3D::default()
            }),
            control_intent: Some(ControlIntent3D {
                throttle: 1.0,
                ..ControlIntent3D::default()
            }),
            linear_motor: Some(LinearMotor3D {
                accel: 9.0,
                ..LinearMotor3D::default()
            }),
            ..ComponentBundle3D::default()
        };

        assert!(world.attach_component_bundle3d(entity, bundle.clone()));
        assert_eq!(world.component_bundle3d(entity), Some(bundle.clone()));
        assert_eq!(
            world.batch_read_component_bundles3d(&[entity]),
            vec![(entity, bundle.clone())]
        );
        assert_eq!(
            world.batch_read_all_component_bundles3d(),
            vec![(entity, bundle.clone())]
        );

        let detached = world
            .detach_component_bundle3d(entity)
            .expect("detached bundle should exist");
        assert_eq!(detached, bundle);
        assert_eq!(world.component_bundle3d(entity), None);
        assert_eq!(world.ids_with_any_3d(), Vec::<u64>::new());
    }

    #[test]
    fn assembly3d_overlay_and_round_trip_lower_into_component_storage() {
        let world = GameplayWorld::new();
        let entity = world.spawn("probe", json!({})).expect("entity");

        let base = Assembly3D::default()
            .with_spatial(SpatialBundle3D {
                transform: Some(Transform3D {
                    position: [1.0, 2.0, 3.0],
                    ..Transform3D::default()
                }),
                physics: None,
            })
            .with_control(ControlBundle3D {
                control_intent: Some(ControlIntent3D {
                    throttle: 0.25,
                    ..ControlIntent3D::default()
                }),
                reference_frame: None,
                reference_frame_state: None,
            });
        let overlay = Assembly3D::default()
            .with_spatial(SpatialBundle3D {
                transform: None,
                physics: Some(PhysicsBody3D {
                    linear_velocity: [8.0, 0.0, 0.0],
                    ..PhysicsBody3D::default()
                }),
            })
            .with_motors(MotorBundle3D {
                linear_motor: Some(LinearMotor3D {
                    accel: 11.0,
                    ..LinearMotor3D::default()
                }),
                angular_motor: None,
                character_motor: None,
                flight_motor: Some(FlightMotor3D {
                    horizon_lock_strength: 0.5,
                    ..FlightMotor3D::default()
                }),
            });
        let assembly = base.overlay(overlay);

        assert!(world.attach_assembly3d(entity, assembly.clone()));
        assert_eq!(world.assembly3d(entity), Some(assembly.clone()));
        assert_eq!(
            world.batch_read_assemblies3d(&[entity]),
            vec![(entity, assembly.clone())]
        );
        assert_eq!(
            world.batch_read_all_assemblies3d(),
            vec![(entity, assembly.clone())]
        );

        let bundle: ComponentBundle3D = assembly.clone().into_bundle();
        assert_eq!(world.component_bundle3d(entity), Some(bundle.clone()));
        assert_eq!(bundle.into_assembly(), assembly.clone());

        let detached = world
            .detach_assembly3d(entity)
            .expect("detached assembly should exist");
        assert_eq!(detached, assembly);
        assert_eq!(world.assembly3d(entity), None);
        assert_eq!(world.ids_with_any_3d(), Vec::<u64>::new());
    }

    #[test]
    fn bootstrap_preset3d_applies_bundle_sets_controlled_and_owner_links() {
        let world = GameplayWorld::new();
        let owner = world.spawn("owner", json!({})).expect("owner");
        let entity = world.spawn("pilot", json!({})).expect("entity");
        assert!(world.set_lifecycle(owner, LifecyclePolicy::FollowOwner));

        let preset = BootstrapPreset3D {
            components: ComponentBundle3D {
                transform: Some(Transform3D {
                    position: [10.0, 0.0, 0.0],
                    ..Transform3D::default()
                }),
                reference_frame: Some(ReferenceFrameBinding3D {
                    mode: ReferenceFrameMode::ParentEntity,
                    entity_id: Some(owner),
                    ..ReferenceFrameBinding3D::default()
                }),
                follow_anchor: Some(FollowAnchor3D {
                    local_offset: [0.0, 2.0, -4.0],
                    inherit_orientation: true,
                }),
                flight_motor: Some(FlightMotor3D::default()),
                ..ComponentBundle3D::default()
            },
            controlled: true,
            owner_id: Some(owner),
            inherit_owner_lifecycle: true,
            lifecycle: None,
        };

        assert!(world.bootstrap_preset3d(entity, preset));
        assert_eq!(world.controlled_entity(), Some(entity));
        assert_eq!(
            world.ownership(entity).map(|ownership| ownership.owner_id),
            Some(owner)
        );
        assert_eq!(world.lifecycle(entity), Some(LifecyclePolicy::FollowOwner));
        assert!(world.follow_anchor3d(entity).is_some());
        assert!(world.flight_motor3d(entity).is_some());

        let detached = world
            .detach_component_bundle3d(entity)
            .expect("detached bundle should exist");
        assert!(detached.reference_frame.is_some());
        assert!(detached.follow_anchor.is_some());
        assert!(detached.flight_motor.is_some());
        assert_eq!(world.controlled_entity(), None);
    }

    #[test]
    fn bootstrap_assembly3d_applies_grouped_defaults_and_owner_links() {
        let world = GameplayWorld::new();
        let owner = world.spawn("carrier", json!({})).expect("owner");
        let entity = world.spawn("camera", json!({})).expect("entity");
        assert!(world.set_lifecycle(owner, LifecyclePolicy::FollowOwner));

        let preset = BootstrapAssembly3D {
            assembly: Assembly3D::default()
                .with_spatial(SpatialBundle3D {
                    transform: Some(Transform3D {
                        position: [0.0, 3.0, -6.0],
                        ..Transform3D::default()
                    }),
                    physics: Some(PhysicsBody3D::default()),
                })
                .with_control(ControlBundle3D {
                    control_intent: Some(ControlIntent3D {
                        look_local: [0.0, 1.0, 0.0],
                        ..ControlIntent3D::default()
                    }),
                    reference_frame: Some(ReferenceFrameBinding3D {
                        mode: ReferenceFrameMode::ParentEntity,
                        entity_id: Some(owner),
                        ..ReferenceFrameBinding3D::default()
                    }),
                    reference_frame_state: Some(ReferenceFrameState3D {
                        altitude_km: 1.5,
                        ..ReferenceFrameState3D::default()
                    }),
                })
                .with_motors(MotorBundle3D {
                    linear_motor: None,
                    angular_motor: Some(AngularMotor3D {
                        yaw_rate: 1.5,
                        ..AngularMotor3D::default()
                    }),
                    character_motor: None,
                    flight_motor: None,
                }),
            controlled: true,
            owner_id: Some(owner),
            inherit_owner_lifecycle: true,
            lifecycle: None,
        };

        assert!(world.bootstrap_assembly3d(entity, preset));
        assert_eq!(world.controlled_entity(), Some(entity));
        assert_eq!(
            world.ownership(entity).map(|ownership| ownership.owner_id),
            Some(owner)
        );
        assert_eq!(world.lifecycle(entity), Some(LifecyclePolicy::FollowOwner));
        assert_eq!(
            world.assembly3d(entity).expect("assembly"),
            Assembly3D {
                spatial: SpatialBundle3D {
                    transform: Some(Transform3D {
                        position: [0.0, 3.0, -6.0],
                        ..Transform3D::default()
                    }),
                    physics: Some(PhysicsBody3D::default()),
                },
                control: ControlBundle3D {
                    control_intent: Some(ControlIntent3D {
                        look_local: [0.0, 1.0, 0.0],
                        ..ControlIntent3D::default()
                    }),
                    reference_frame: Some(ReferenceFrameBinding3D {
                        mode: ReferenceFrameMode::ParentEntity,
                        entity_id: Some(owner),
                        ..ReferenceFrameBinding3D::default()
                    }),
                    reference_frame_state: Some(ReferenceFrameState3D {
                        altitude_km: 1.5,
                        ..ReferenceFrameState3D::default()
                    }),
                },
                attachments: Default::default(),
                motors: MotorBundle3D {
                    linear_motor: None,
                    angular_motor: Some(AngularMotor3D {
                        yaw_rate: 1.5,
                        ..AngularMotor3D::default()
                    }),
                    character_motor: None,
                    flight_motor: None,
                },
            }
        );
    }

    #[test]
    fn bootstrap_assembly3d_applies_explicit_lifecycle_even_without_spatial_components() {
        let world = GameplayWorld::new();
        let owner = world.spawn("carrier", json!({})).expect("owner");
        let entity = world.spawn("turret", json!({})).expect("entity");
        assert!(world.set_lifecycle(owner, LifecyclePolicy::FollowOwner));

        let preset = BootstrapAssembly3D {
            assembly: Assembly3D::default(),
            controlled: false,
            owner_id: Some(owner),
            inherit_owner_lifecycle: true,
            lifecycle: Some(LifecyclePolicy::TtlFollowOwner),
        };

        assert!(world.bootstrap_assembly3d(entity, preset.clone()));
        assert_eq!(
            world.ownership(entity).map(|ownership| ownership.owner_id),
            Some(owner)
        );
        assert_eq!(
            world.lifecycle(entity),
            Some(LifecyclePolicy::TtlFollowOwner)
        );

        let round_trip = preset.into_bootstrap_preset().into_bootstrap_assembly();
        assert_eq!(round_trip.lifecycle, Some(LifecyclePolicy::TtlFollowOwner));
    }
}

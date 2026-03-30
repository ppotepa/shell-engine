//! Entity prefab system: templates for spawning entities with preconfigured components.
//!
//! A prefab defines a reusable entity template with:
//! - Entity kind (e.g., "asteroid", "projectile")
//! - Initial component values (transform, physics, collider, etc.)
//! - Visual bindings and other metadata
//!
//! Prefabs can be registered at scene load time and instantiated on-demand,
//! making it easy to spawn variants (e.g., small/medium/large asteroids).

use serde_json::Value as JsonValue;
use crate::components::{
    Collider2D, EntityTimers, Lifetime, PhysicsBody2D, Transform2D,
    VisualBinding, WrapBounds, TopDownShipController, Health, SplitOnDestroy,
};

/// A template for spawning entities with preconfigured components.
#[derive(Clone, Debug)]
pub struct PrefabSpec {
    /// Entity kind/type (e.g., "asteroid", "projectile").
    pub kind: String,
    /// Optional initial transform.
    pub transform: Option<Transform2D>,
    /// Optional physics body.
    pub physics: Option<PhysicsBody2D>,
    /// Optional collider.
    pub collider: Option<Collider2D>,
    /// Optional lifetime.
    pub lifetime: Option<Lifetime>,
    /// Optional visual binding.
    pub visual: Option<VisualBinding>,
    /// Optional wrapping bounds.
    pub wrap_bounds: Option<WrapBounds>,
    /// Optional ship controller.
    pub ship_controller: Option<TopDownShipController>,
    /// Optional health.
    pub health: Option<Health>,
    /// Optional split-on-destroy config.
    pub split_on_destroy: Option<SplitOnDestroy>,
    /// Optional entity timers (cooldowns/statuses).
    pub timers: Option<EntityTimers>,
    /// Optional JSON payload for gameplay-specific data.
    pub payload: Option<JsonValue>,
}

impl PrefabSpec {
    /// Create a new prefab with just a kind.
    pub fn new(kind: &str) -> Self {
        Self {
            kind: kind.to_string(),
            transform: None,
            physics: None,
            collider: None,
            lifetime: None,
            visual: None,
            wrap_bounds: None,
            ship_controller: None,
            health: None,
            split_on_destroy: None,
            timers: None,
            payload: None,
        }
    }

    /// Set the transform component.
    pub fn with_transform(mut self, xf: Transform2D) -> Self {
        self.transform = Some(xf);
        self
    }

    /// Set the physics body component.
    pub fn with_physics(mut self, body: PhysicsBody2D) -> Self {
        self.physics = Some(body);
        self
    }

    /// Set the collider component.
    pub fn with_collider(mut self, collider: Collider2D) -> Self {
        self.collider = Some(collider);
        self
    }

    /// Set the visual binding component.
    pub fn with_visual(mut self, visual: VisualBinding) -> Self {
        self.visual = Some(visual);
        self
    }

    /// Set the health component.
    pub fn with_health(mut self, health: Health) -> Self {
        self.health = Some(health);
        self
    }

    /// Set the ship controller component.
    pub fn with_ship_controller(mut self, controller: TopDownShipController) -> Self {
        self.ship_controller = Some(controller);
        self
    }

    /// Set the split-on-destroy component.
    pub fn with_split_on_destroy(mut self, split: SplitOnDestroy) -> Self {
        self.split_on_destroy = Some(split);
        self
    }

    /// Set optional lifetime.
    pub fn with_lifetime(mut self, lifetime: Lifetime) -> Self {
        self.lifetime = Some(lifetime);
        self
    }

    /// Set optional wrap bounds.
    pub fn with_wrap_bounds(mut self, bounds: WrapBounds) -> Self {
        self.wrap_bounds = Some(bounds);
        self
    }

    /// Set optional entity timers.
    pub fn with_timers(mut self, timers: EntityTimers) -> Self {
        self.timers = Some(timers);
        self
    }

    /// Set optional JSON payload.
    pub fn with_payload(mut self, payload: JsonValue) -> Self {
        self.payload = Some(payload);
        self
    }
}

impl Default for PrefabSpec {
    fn default() -> Self {
        Self::new("entity")
    }
}

/// Instantiation parameters for overriding prefab values at spawn time.
///
/// Allows scripts to spawn from a prefab while overriding specific fields
/// (e.g., position, velocity, size modifiers).
#[derive(Clone, Debug, Default)]
pub struct SpawnParams {
    /// Override position (x, y).
    pub position: Option<(f32, f32)>,
    /// Override velocity (vx, vy).
    pub velocity: Option<(f32, f32)>,
    /// Rotation in radians.
    pub heading: Option<f32>,
    /// Size modifier (e.g., -1 for one size smaller).
    pub size_delta: Option<i32>,
}

impl SpawnParams {
    /// Create empty spawn parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set position override.
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Some((x, y));
        self
    }

    /// Set velocity override.
    pub fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.velocity = Some((vx, vy));
        self
    }

    /// Set heading override.
    pub fn with_heading(mut self, heading: f32) -> Self {
        self.heading = Some(heading);
        self
    }

    /// Set size delta override.
    pub fn with_size_delta(mut self, delta: i32) -> Self {
        self.size_delta = Some(delta);
        self
    }
}

//! Typed gameplay components used by engine systems and scripts.

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transform2D {
    pub x: f32,
    pub y: f32,
    /// Heading in radians. Scripts using 32-step headings can convert as needed.
    pub heading: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsBody2D {
    pub vx: f32,
    pub vy: f32,
    pub ax: f32,
    pub ay: f32,
    /// Linear drag factor per second (0.0 = none, 1.0 = full stop).
    pub drag: f32,
    /// Maximum linear speed magnitude; 0.0 disables the clamp.
    pub max_speed: f32,
}

impl Default for PhysicsBody2D {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            ax: 0.0,
            ay: 0.0,
            drag: 0.0,
            max_speed: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColliderShape {
    Circle { radius: f32 },
    Polygon { points: Vec<[f32; 2]> },
}

impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::Circle { radius: 1.0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Collider2D {
    pub shape: ColliderShape,
    pub layer: u32,
    pub mask: u32,
}

impl Default for Collider2D {
    fn default() -> Self {
        Self {
            shape: ColliderShape::default(),
            layer: 0xFFFF_FFFF,
            mask: 0xFFFF_FFFF,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct VisualBinding {
    pub visual_id: Option<String>,
    pub additional_visuals: Vec<String>,
}

impl VisualBinding {
    /// Returns all bound visual IDs (primary + additional).
    pub fn all_visual_ids(&self) -> Vec<&str> {
        let mut ids = Vec::new();
        if let Some(ref vid) = self.visual_id {
            ids.push(vid.as_str());
        }
        for vid in &self.additional_visuals {
            ids.push(vid.as_str());
        }
        ids
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DespawnVisual {
    None,
    DespawnWithEntity,
}

impl Default for DespawnVisual {
    fn default() -> Self {
        DespawnVisual::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Lifetime {
    pub ttl_ms: i32,
    pub on_expire: DespawnVisual,
}

/// Per-entity named timers for cooldowns and timed status effects.
///
/// **Cooldowns** count down to 0 and stay there (ready when 0 or absent).
/// **Statuses** count down to 0 and are removed (active while > 0).
#[derive(Clone, Debug, Default)]
pub struct EntityTimers {
    pub cooldowns: HashMap<String, i32>,
    pub statuses: HashMap<String, i32>,
}

/// World-wrap bounds. When set on an entity, the physics system clamps
/// its transform to the toroidal region after each integration step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WrapBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl WrapBounds {
    pub fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        Self { min_x, max_x, min_y, max_y }
    }

    /// Wrap a single value in [min, max] toroidally.
    #[inline]
    pub fn wrap_x(&self, x: f32) -> f32 {
        let range = self.max_x - self.min_x;
        if range <= 0.0 { return x; }
        if x < self.min_x { self.max_x }
        else if x > self.max_x { self.min_x }
        else { x }
    }

    #[inline]
    pub fn wrap_y(&self, y: f32) -> f32 {
        let range = self.max_y - self.min_y;
        if range <= 0.0 { return y; }
        if y < self.min_y { self.max_y }
        else if y > self.max_y { self.min_y }
        else { y }
    }
}

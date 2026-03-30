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
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    /// Wrap a single value in [min, max] toroidally.
    #[inline]
    pub fn wrap_x(&self, x: f32) -> f32 {
        let range = self.max_x - self.min_x;
        if range <= 0.0 {
            return x;
        }
        if x < self.min_x {
            self.max_x
        } else if x > self.max_x {
            self.min_x
        } else {
            x
        }
    }

    #[inline]
    pub fn wrap_y(&self, y: f32) -> f32 {
        let range = self.max_y - self.min_y;
        if range <= 0.0 {
            return y;
        }
        if y < self.min_y {
            self.max_y
        } else if y > self.max_y {
            self.min_y
        } else {
            y
        }
    }
}

/// Arcade-style 2D ship controller (Asteroids/Robotron style).
///
/// Manages heading on a discrete 32-step (or configurable) circle,
/// turn accumulation for frame-rate independent rotation, and thrust input.
/// The system integrates heading changes and applies thrust acceleration to a
/// paired PhysicsBody2D each frame.
#[derive(Clone, Debug)]
pub struct TopDownShipController {
    /// Current heading on the circle (0 to heading_bits-1).
    pub current_heading: i32,
    /// Number of steps in the heading circle. Common values: 8, 16, 32.
    pub heading_bits: u8,
    /// Milliseconds between rotation steps for discrete heading updates.
    pub turn_step_ms: u32,
    /// Accumulated time since last heading change (internal).
    pub turn_accumulator: u32,
    /// Current turn input: -1 (left), 0 (stopped), +1 (right).
    pub turn_direction: i8,

    /// Is the ship currently thrusting (input state).
    pub is_thrusting: bool,
    /// Acceleration magnitude when thrusting (in velocity_scale units).
    pub thrust_power: f32,
    /// Maximum speed magnitude when clamped (in velocity_scale units). 0 = unclamped.
    pub max_speed: f32,
}

impl TopDownShipController {
    /// Create a new controller with given configuration.
    pub fn new(turn_step_ms: u32, thrust_power: f32, max_speed: f32, heading_bits: u8) -> Self {
        Self {
            current_heading: 0,
            heading_bits,
            turn_step_ms: turn_step_ms.max(1),
            turn_accumulator: 0,
            turn_direction: 0,
            is_thrusting: false,
            thrust_power,
            max_speed,
        }
    }

    /// Set the turn direction (-1, 0, or +1).
    pub fn set_turn(&mut self, dir: i8) {
        self.turn_direction = dir.max(-1).min(1);
    }

    /// Set thrusting state.
    pub fn set_thrust(&mut self, on: bool) {
        self.is_thrusting = on;
    }

    /// Get heading as a unit vector (x, y).
    /// Returns (sin, -cos) from 32-step heading, normalized to approximate unit vectors.
    /// heading 0 = UP (0, -1), heading 8 = RIGHT (1, 0), heading 16 = DOWN (0, 1), heading 24 = LEFT (-1, 0)
    pub fn heading_vector(&self) -> (f32, f32) {
        let h = self.current_heading % (self.heading_bits as i32);
        
        // sin32 returns 0-2711 for indices 0-31 (one quadrant of sine curve)
        // We need full sine wave: apply sign based on which half of the circle we're in
        let sin_raw = sin32(h) as f32;
        let cos_raw = sin32((h + (self.heading_bits as i32) / 4) % (self.heading_bits as i32)) as f32;
        
        // Apply sign for lower half of circle (heading 16-31)
        let sin_val = if h < 16 { sin_raw } else { -sin_raw };
        let cos_val = if h < 8 || h >= 24 { cos_raw } else { -cos_raw };
        
        // Normalize by max value (2711) to get approximate unit vector
        (sin_val / 2711.0, -cos_val / 2711.0)
    }

    /// Convert heading to radians for Transform2D.
    pub fn heading_radians(&self) -> f32 {
        (self.current_heading as f32) * (std::f32::consts::TAU / (self.heading_bits as f32))
    }
}

impl Default for TopDownShipController {
    fn default() -> Self {
        Self::new(40, 170.0, 4.5, 32)
    }
}

/// Precomputed sin32 lookup table for fast heading-based direction calculation.
/// sin32(i) gives the sine of (i / 32) * 2π, scaled to i16 range.
/// Used for 2D direction vectors in discrete heading systems.
#[inline]
fn sin32(i: i32) -> i32 {
    const SIN_TABLE: [i32; 32] = [
        0, 201, 401, 601, 797, 989, 1176, 1356, 1530, 1696, 1853, 1999, 2135, 2259, 2370, 2467,
        2549, 2616, 2665, 2697, 2711, 2707, 2685, 2644, 2585, 2508, 2413, 2300, 2169, 2021, 1856,
        1674,
    ];
    SIN_TABLE[((i % 32).abs()) as usize]
}

/// Gameplay events emitted during frame processing.
///
/// Events accumulate during a frame and can be polled by scripts via the world API.
/// Events are cleared at the start of the next frame.
#[derive(Clone, Debug, PartialEq)]
pub enum GameplayEvent {
    /// Two entities collided this frame (a, b).
    /// Emitted for both (a, b) and (b, a) directions for script convenience.
    CollisionEnter { a: u64, b: u64 },
}

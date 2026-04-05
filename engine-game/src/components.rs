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
    pub original_ttl_ms: i32,
    pub on_expire: DespawnVisual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LifecyclePolicy {
    #[default]
    Persistent,
    Manual,
    Ttl,
    OwnerBound,
    TtlOwnerBound,
    FollowOwner,
    TtlFollowOwner,
}

impl LifecyclePolicy {
    pub fn uses_ttl(self) -> bool {
        matches!(self, Self::Ttl | Self::TtlOwnerBound | Self::TtlFollowOwner)
    }

    pub fn is_owner_bound(self) -> bool {
        matches!(
            self,
            Self::OwnerBound | Self::TtlOwnerBound | Self::FollowOwner | Self::TtlFollowOwner
        )
    }

    pub fn follows_owner(self) -> bool {
        matches!(self, Self::FollowOwner | Self::TtlFollowOwner)
    }

    pub fn is_transient(self) -> bool {
        !matches!(self, Self::Persistent)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DespawnReason {
    #[default]
    Manual,
    Expired,
    OwnerDestroyed,
    Collision,
    SceneReset,
    InvalidLifecycle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ownership {
    pub owner_id: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FollowAnchor2D {
    pub local_x: f32,
    pub local_y: f32,
    pub inherit_heading: bool,
}

impl Default for FollowAnchor2D {
    fn default() -> Self {
        Self {
            local_x: 0.0,
            local_y: 0.0,
            inherit_heading: true,
        }
    }
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

/// Arcade-style top-down controller for 2D entities.
///
/// Manages heading on a discrete configurable-step circle,
/// turn accumulation for frame-rate independent rotation, and thrust input.
/// The system integrates heading changes and applies thrust acceleration to a
/// paired PhysicsBody2D each frame.
#[derive(Clone, Debug)]
pub struct ArcadeController {
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

    /// Whether the entity is currently thrusting (input state).
    pub is_thrusting: bool,
    /// Acceleration magnitude when thrusting (in velocity_scale units).
    pub thrust_power: f32,
    /// Maximum speed magnitude when clamped (in velocity_scale units). 0 = unclamped.
    pub max_speed: f32,
}

impl ArcadeController {
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
    /// heading 0 = UP (0, -1), heading_bits/4 = RIGHT (1, 0), etc.
    /// Uses heading_radians() so the direction exactly matches the visual transform.
    pub fn heading_vector(&self) -> (f32, f32) {
        let r = self.heading_radians();
        (r.sin(), -r.cos())
    }

    /// Convert heading to radians for Transform2D.
    pub fn heading_radians(&self) -> f32 {
        (self.current_heading as f32) * (std::f32::consts::TAU / (self.heading_bits as f32))
    }

    /// Snap controller heading to the nearest discrete step for the given radians.
    pub fn set_heading_radians(&mut self, radians: f32) {
        let heading_bits = self.heading_bits.max(1) as f32;
        let turns = radians.rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
        let step = (turns * heading_bits).round() as i32;
        self.current_heading = step.rem_euclid(self.heading_bits as i32);
    }
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

/// Thread processing mode for particles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ParticleThreadMode {
    /// Lightweight particle - processed on main thread (default).
    #[default]
    Light,
    /// Full physics particle - processed on worker thread.
    Physics,
    /// Gravity-affected particle - processed on worker thread.
    Gravity,
}

impl ParticleThreadMode {
    /// Parse from string (for YAML config).
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "physics" => Self::Physics,
            "gravity" => Self::Gravity,
            _ => Self::Light,
        }
    }
    
    /// Check if particle should be processed on worker thread.
    pub fn uses_worker_thread(self) -> bool {
        matches!(self, Self::Physics | Self::Gravity)
    }
}

/// Particle color/radius animation driven by remaining lifetime.
///
/// Attached to ephemeral particle entities by the emitter system.
/// The `particle_ramp_system` reads this each frame and pushes
/// `SetProperty` commands to update `style.fg` and `style.radius`.
///
/// Color sampling: `idx = floor((1.0 - life_ratio) * N)`, clamped to N-1.
/// Radius: `lerp(radius_min, radius_max, life_ratio).round()`.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleColorRamp {
    /// Color sequence: index 0 = freshest (life=1.0), last = oldest (life→0).
    pub colors: Vec<String>,
    /// Radius at full life (life=1.0).
    pub radius_max: i32,
    /// Radius at end of life (life→0). 0 = fade out, ≥1 = stays visible.
    pub radius_min: i32,
}
/// Applied when `thread_mode` is not "light".
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ParticlePhysics {
    /// Processing mode (light/physics/gravity).
    pub thread_mode: ParticleThreadMode,
    /// Enable collision detection.
    pub collision: bool,
    /// Tags this particle can collide with.
    pub collision_mask: Vec<String>,
    /// Gravity scale (0.0 = no gravity, 1.0 = world gravity).
    pub gravity_scale: f32,
    /// Bounce coefficient (0.0 = absorb, 1.0 = elastic).
    pub bounce: f32,
    /// Particle mass for physics calculations.
    pub mass: f32,
}

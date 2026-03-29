//! Typed gameplay components used by engine systems and scripts.

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

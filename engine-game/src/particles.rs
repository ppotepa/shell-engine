//! Particle effect system: spawning transient visual effects.
//!
//! Particle effects are short-lived entities with visual bindings and lifetimes,
//! used for explosions, smoke, sparks, hit-flashes, etc.
//!
//! Built-in effects can be registered and spawned via Rhai with single function calls.

use crate::components::{Lifetime, PhysicsBody2D, VisualBinding};
use crate::prefabs::PrefabSpec;

/// Built-in particle effect types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleEffectType {
    /// Explosion: rapid outward burst, bright flash, fades quickly.
    Explosion,
    /// Smoke: slower dispersing effect, darker, lingers longer.
    Smoke,
    /// Sparks: small particles, fast, short-lived.
    Sparks,
    /// Hit-flash: single frame effect at impact point.
    HitFlash,
    /// Blood: slower droplets, gravity-affected.
    Blood,
}

impl ParticleEffectType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Explosion => "explosion",
            Self::Smoke => "smoke",
            Self::Sparks => "sparks",
            Self::HitFlash => "hit_flash",
            Self::Blood => "blood",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "explosion" => Some(Self::Explosion),
            "smoke" => Some(Self::Smoke),
            "sparks" => Some(Self::Sparks),
            "hit_flash" => Some(Self::HitFlash),
            "blood" => Some(Self::Blood),
            _ => None,
        }
    }
}

/// Helper to create built-in particle effect prefabs.
pub struct ParticleFxFactory;

impl ParticleFxFactory {
    /// Create a prefab for an explosion effect.
    pub fn explosion() -> PrefabSpec {
        PrefabSpec::new("particle")
            .with_visual(VisualBinding {
                visual_id: Some("particle_explosion".to_string()),
                additional_visuals: Vec::new(),
            })
            .with_lifetime(Lifetime {
                ttl_ms: 200,
                on_expire: crate::components::DespawnVisual::DespawnWithEntity,
            })
            .with_physics(PhysicsBody2D {
                vx: 0.0,
                vy: 0.0,
                ax: 0.0,
                ay: 0.0,
                drag: 0.1,
                max_speed: 0.0,
            })
    }

    /// Create a prefab for a smoke effect.
    pub fn smoke() -> PrefabSpec {
        PrefabSpec::new("particle")
            .with_visual(VisualBinding {
                visual_id: Some("particle_smoke".to_string()),
                additional_visuals: Vec::new(),
            })
            .with_lifetime(Lifetime {
                ttl_ms: 600,
                on_expire: crate::components::DespawnVisual::DespawnWithEntity,
            })
            .with_physics(PhysicsBody2D {
                vx: 0.0,
                vy: 0.0,
                ax: 0.0,
                ay: 0.0,
                drag: 0.05,
                max_speed: 0.0,
            })
    }

    /// Create a prefab for a sparks effect.
    pub fn sparks() -> PrefabSpec {
        PrefabSpec::new("particle")
            .with_visual(VisualBinding {
                visual_id: Some("particle_sparks".to_string()),
                additional_visuals: Vec::new(),
            })
            .with_lifetime(Lifetime {
                ttl_ms: 150,
                on_expire: crate::components::DespawnVisual::DespawnWithEntity,
            })
            .with_physics(PhysicsBody2D {
                vx: 0.0,
                vy: 0.0,
                ax: 0.0,
                ay: 0.0,
                drag: 0.2,
                max_speed: 0.0,
            })
    }

    /// Create a prefab for a hit-flash effect.
    pub fn hit_flash() -> PrefabSpec {
        PrefabSpec::new("particle")
            .with_visual(VisualBinding {
                visual_id: Some("particle_hit_flash".to_string()),
                additional_visuals: Vec::new(),
            })
            .with_lifetime(Lifetime {
                ttl_ms: 50,
                on_expire: crate::components::DespawnVisual::DespawnWithEntity,
            })
    }

    /// Create a prefab for a blood effect.
    pub fn blood() -> PrefabSpec {
        PrefabSpec::new("particle")
            .with_visual(VisualBinding {
                visual_id: Some("particle_blood".to_string()),
                additional_visuals: Vec::new(),
            })
            .with_lifetime(Lifetime {
                ttl_ms: 800,
                on_expire: crate::components::DespawnVisual::DespawnWithEntity,
            })
            .with_physics(PhysicsBody2D {
                vx: 0.0,
                vy: 0.0,
                ax: 0.0,
                ay: 0.5, // gravity effect
                drag: 0.08,
                max_speed: 0.0,
            })
    }

    /// Get the prefab spec for a built-in effect type.
    pub fn get_prefab(effect_type: ParticleEffectType) -> PrefabSpec {
        match effect_type {
            ParticleEffectType::Explosion => Self::explosion(),
            ParticleEffectType::Smoke => Self::smoke(),
            ParticleEffectType::Sparks => Self::sparks(),
            ParticleEffectType::HitFlash => Self::hit_flash(),
            ParticleEffectType::Blood => Self::blood(),
        }
    }
}

use crate::buffer::Buffer;
use crate::effects::metadata::{EffectMetadata, META_UNKNOWN};
use crate::scene::{EffectParams, EffectTargetKind};

/// A region of the screen the effect operates on.
#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Region {
    #[inline]
    pub fn full(buffer: &Buffer) -> Self {
        Self {
            x: 0,
            y: 0,
            width: buffer.width,
            height: buffer.height,
        }
    }

    #[inline]
    pub fn row(y: u16, x: u16, width: u16) -> Self {
        Self {
            x,
            y,
            width,
            height: 1,
        }
    }
}

/// Core effect abstraction.
pub trait Effect: Send + Sync {
    /// Declares which authored targets this effect can safely operate on.
    fn compatible_targets(&self) -> EffectTargetMask {
        self.metadata().compatible_targets
    }

    /// Returns compile-time metadata for this effect.
    fn metadata(&self) -> &'static EffectMetadata {
        &META_UNKNOWN
    }

    /// Apply this effect to `buffer` within `region`.
    /// `progress` is 0.0–1.0 normalized time within the effect duration.
    fn apply(&self, progress: f32, params: &EffectParams, region: Region, buffer: &mut Buffer);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectTargetMask(u8);

impl EffectTargetMask {
    pub const SCENE: Self = Self(1 << 0);
    pub const LAYER: Self = Self(1 << 1);
    pub const SPRITE: Self = Self(1 << 2);
    pub const SPRITE_TEXT: Self = Self(1 << 3);
    pub const SPRITE_BITMAP: Self = Self(1 << 4);
    pub const ANY: Self = Self(
        Self::SCENE.0
            | Self::LAYER.0
            | Self::SPRITE.0
            | Self::SPRITE_TEXT.0
            | Self::SPRITE_BITMAP.0,
    );

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn supports(self, kind: EffectTargetKind) -> bool {
        match kind {
            EffectTargetKind::Any => true,
            EffectTargetKind::Scene => self.contains(Self::SCENE),
            EffectTargetKind::Layer => self.contains(Self::LAYER),
            EffectTargetKind::Sprite => self.contains(Self::SPRITE),
            EffectTargetKind::SpriteText => {
                self.contains(Self::SPRITE_TEXT) || self.contains(Self::SPRITE)
            }
            EffectTargetKind::SpriteBitmap => {
                self.contains(Self::SPRITE_BITMAP) || self.contains(Self::SPRITE)
            }
        }
    }

    const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

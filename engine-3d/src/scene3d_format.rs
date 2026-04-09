//! Parser for `.scene3d.yml` — describes a complete 3D scene with objects, lights,
//! materials, camera, and named frames (static or animated clips).
//!
//! The parsed [`Scene3DDefinition`] is consumed by [`Scene3DPrerenderStep`] to
//! pre-render every named frame into the [`Scene3DAtlas`] before a scene loads.

use engine_core::scene::SceneRenderedMode;
use serde::Deserialize;
use std::collections::HashMap;

// ── Top-level definition ─────────────────────────────────────────────────────

/// Parsed representation of a `.scene3d.yml` file.
#[derive(Debug, Deserialize)]
pub struct Scene3DDefinition {
    pub id: String,
    pub viewport: ViewportDef,
    #[serde(default)]
    pub camera: CameraDef,
    #[serde(default)]
    pub lights: Vec<LightDef>,
    #[serde(default)]
    pub materials: HashMap<String, MaterialDef>,
    #[serde(default)]
    pub objects: Vec<ObjectDef>,
    /// Named frames. Key = frame_id used in `Sprite::Scene3D.frame`.
    pub frames: HashMap<String, FrameDef>,

    // ── Cross-file references ───────────────────────────────────────────────
    /// Path to a shared materials YAML (`HashMap<name, MaterialDef>`).
    /// Loaded and merged before rendering; local materials override ref materials.
    #[serde(default, rename = "materials-ref")]
    pub materials_ref: Option<String>,
    /// Path to a shared camera YAML (`CameraDef`). Replaces the local `camera` block.
    #[serde(default, rename = "camera-ref")]
    pub camera_ref: Option<String>,
    /// Path to a shared lights YAML (`Vec<LightDef>`). Applied when local `lights` is empty.
    #[serde(default, rename = "lights-ref")]
    pub lights_ref: Option<String>,
}

// ── Viewport ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ViewportDef {
    pub width: u16,
    pub height: u16,
    /// Override renderer mode for this 3D scene.
    #[serde(default)]
    pub rendered_mode: Option<SceneRenderedMode>,
}

// ── Camera ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CameraDef {
    #[serde(default = "default_camera_distance")]
    pub distance: f32,
    #[serde(default = "default_fov")]
    pub fov_degrees: f32,
    #[serde(default = "default_near_clip")]
    pub near_clip: f32,
    /// Camera position override (if not using orbit distance).
    #[serde(default)]
    pub position: Option<[f32; 3]>,
    /// Point the camera looks at.
    #[serde(default)]
    pub look_at: Option<[f32; 3]>,
}

impl Default for CameraDef {
    fn default() -> Self {
        Self {
            distance: default_camera_distance(),
            fov_degrees: default_fov(),
            near_clip: default_near_clip(),
            position: None,
            look_at: None,
        }
    }
}

fn default_camera_distance() -> f32 {
    3.0
}
fn default_fov() -> f32 {
    60.0
}
fn default_near_clip() -> f32 {
    0.001
}

// ── Lights ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LightDef {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: LightKind,
    /// Directional light: unit direction vector [x, y, z].
    #[serde(default)]
    pub direction: Option<[f32; 3]>,
    /// Point light: world-space position [x, y, z].
    #[serde(default)]
    pub position: Option<[f32; 3]>,
    #[serde(default = "default_intensity")]
    pub intensity: f32,
    /// Flicker: quantise time to N snapshots per second (0 = off).
    #[serde(default)]
    pub snap_hz: f32,
    #[serde(default)]
    pub colour: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LightKind {
    Directional,
    Point,
    Ambient,
}

fn default_intensity() -> f32 {
    1.0
}

// ── Materials ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct MaterialDef {
    #[serde(default)]
    pub surface_mode: SurfaceMode,
    /// Number of cel-shading brightness levels (1 = flat, 4 = classic).
    #[serde(default = "default_cel_levels")]
    pub cel_levels: u8,
    #[serde(default)]
    pub fg_colour: Option<String>,
    #[serde(default)]
    pub bg_colour: Option<String>,
    #[serde(default)]
    pub wireframe_char: Option<char>,
    /// Opacity 0.0–1.0 (blended over the layer buffer).
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    /// Blend factor between raw lighting and tonal cel-shade colours (0.0 = lighting only, 1.0 = full tonal).
    #[serde(default)]
    pub tone_mix: f32,
    /// Cel-shading shadow band colour (hex, e.g. "#000033").
    #[serde(default)]
    pub shadow_colour: Option<String>,
    /// Cel-shading midtone band colour (hex, e.g. "#333366").
    #[serde(default)]
    pub midtone_colour: Option<String>,
    /// Cel-shading highlight band colour (hex, e.g. "#6666ff").
    #[serde(default)]
    pub highlight_colour: Option<String>,
    /// Whether to cull back-facing polygons.
    #[serde(default)]
    pub backface_cull: bool,
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceMode {
    #[default]
    Material,
    Wireframe,
    Unlit,
}

fn default_cel_levels() -> u8 {
    4
}
fn default_opacity() -> f32 {
    1.0
}

// ── Objects ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ObjectDef {
    pub id: String,
    /// Path to `.obj` file relative to asset root.
    pub mesh: String,
    /// Key into `Scene3DDefinition.materials`.
    pub material: String,
    #[serde(default)]
    pub transform: TransformDef,
}

/// Structured 3-axis rotation (degrees). All axes default to 0.0.
#[derive(Debug, Deserialize, Default, Clone, Copy)]
pub struct RotationDef {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub z: f32,
}

/// Scale: either a uniform scalar or per-axis vector.
/// ```yaml
/// scale: 4.65          # uniform
/// scale: { x: 1.0, y: 2.0, z: 1.0 }  # per-axis
/// ```
#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum ScaleDef {
    Uniform(f32),
    PerAxis { x: f32, y: f32, z: f32 },
}

impl ScaleDef {
    /// Resolve to a uniform scale. Per-axis uses the maximum component.
    /// (The renderer supports uniform scale only; per-axis is future-proofing.)
    pub fn uniform(&self) -> f32 {
        match *self {
            ScaleDef::Uniform(s) => s,
            ScaleDef::PerAxis { x, y, z } => x.max(y).max(z),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct TransformDef {
    #[serde(default)]
    pub translation: Option<[f32; 3]>,
    /// Structured 3-axis rotation (degrees). Takes priority over legacy fields when present.
    #[serde(default)]
    pub rotation: Option<RotationDef>,
    /// Legacy: Y-axis rotation (degrees). Used when `rotation` is absent.
    #[serde(default)]
    pub rotation_y: f32,
    /// Legacy: X-axis rotation / pitch (degrees). Used when `rotation` is absent.
    #[serde(default)]
    pub pitch: f32,
    /// Legacy: Z-axis rotation / roll (degrees). Used when `rotation` is absent.
    #[serde(default)]
    pub roll: f32,
    /// Scale — either a scalar or `{x, y, z}` map.
    #[serde(default)]
    pub scale: Option<ScaleDef>,
}

impl TransformDef {
    /// Resolved rotation as `(pitch_x, yaw_y, roll_z)` in degrees.
    /// Prefers the structured `rotation` block; falls back to legacy `pitch`/`rotation_y`/`roll`.
    pub fn resolved_rotation(&self) -> (f32, f32, f32) {
        if let Some(r) = self.rotation {
            (r.x, r.y, r.z)
        } else {
            (self.pitch, self.rotation_y, self.roll)
        }
    }

    /// Resolved uniform scale (1.0 if not specified).
    pub fn resolved_scale(&self) -> f32 {
        self.scale.map(|s| s.uniform()).unwrap_or(1.0)
    }
}

// ── Frames ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FrameDef {
    /// Animated clip — generates N frames named `{frame_id}-{n}` (0-indexed).
    Clip(ClipFrameDef),
    /// Static frame — renders once with the listed objects visible.
    Static(StaticFrameDef),
}

#[derive(Debug, Deserialize)]
pub struct StaticFrameDef {
    /// Object IDs to show; others are hidden.
    pub show: Vec<String>,
    /// Per-object property overrides for this frame.
    #[serde(default)]
    pub overrides: Vec<ObjectOverride>,
}

#[derive(Debug, Deserialize)]
pub struct ClipFrameDef {
    /// Object IDs visible throughout the clip (objects can still fade via tweens).
    pub show: Vec<String>,
    pub clip: ClipDef,
}

#[derive(Debug, Deserialize)]
pub struct ClipDef {
    pub duration_ms: u64,
    /// Number of discrete frames to pre-render.
    pub keyframes: u32,
    #[serde(default)]
    pub tweens: Vec<TweenDef>,
}

/// Interpolated property value across the clip timeline.
#[derive(Debug, Deserialize)]
pub struct TweenDef {
    pub object: String,
    /// Property name understood by the prerender step:
    /// `clip_y_min`, `clip_y_max`, `yaw_offset`, `opacity`, `translation_x/y/z`.
    pub property: String,
    pub from: f32,
    pub to: f32,
    /// Easing: `linear` (default), `ease_in`, `ease_out`, `ease_in_out`.
    #[serde(default)]
    pub easing: Easing,
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Easing {
    /// Map t ∈ [0, 1] according to the easing curve.
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
        }
    }
}

/// Property override applied to a specific object in a static frame.
#[derive(Debug, Deserialize)]
pub struct ObjectOverride {
    pub object: String,
    pub property: String,
    pub value: f32,
}

// ── Loader ───────────────────────────────────────────────────────────────────

/// Load and parse a `.scene3d.yml` file from disk.
pub fn load_scene3d(
    path: &str,
) -> Result<Scene3DDefinition, Box<dyn std::error::Error + Send + Sync>> {
    let text = std::fs::read_to_string(path)?;
    let def: Scene3DDefinition = serde_yaml::from_str(&text)?;
    Ok(def)
}

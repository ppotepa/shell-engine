use super::{color::TermColour, BehaviorSpec, LayerStages, SceneRenderedMode};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

/// Controls uppercase conversion of text sprite content before glyph lookup.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TextTransform {
    /// Preserve authored case (default). Glyphs are looked up as written.
    #[default]
    None,
    /// Force all characters to uppercase before glyph lookup.
    /// Use for retro/block-title rendering with the generic bitmap font.
    Uppercase,
}

/// Glow halo effect for a text sprite.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Glow {
    /// Glow colour. If None, uses a dimmed version of the sprite's fg_colour.
    pub colour: Option<TermColour>,
    /// Radius in cells. 1 = 8-neighbor halo. Default 1.
    #[serde(default = "default_glow_radius")]
    pub radius: u16,
}

fn default_glow_radius() -> u16 {
    1
}

fn default_true() -> bool {
    true
}

fn default_grid_line() -> u16 {
    1
}

fn default_grid_span() -> u16 {
    1
}

fn default_panel_padding() -> u16 {
    1
}

fn default_panel_border_width() -> u16 {
    1
}

fn default_panel_radius() -> u16 {
    1
}

fn default_panel_shadow_x() -> i32 {
    1
}

fn default_panel_shadow_y() -> i32 {
    1
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
/// Declares the main axis used by a flex container sprite.
pub enum FlexDirection {
    /// Stacks children top-to-bottom.
    #[default]
    Column,
    /// Stacks children left-to-right.
    Row,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteSizePreset {
    Small,
    Medium,
    Large,
}

impl SpriteSizePreset {
    pub const fn generic_mode(self) -> &'static str {
        match self {
            Self::Small => "1",
            Self::Medium => "2",
            Self::Large => "3",
        }
    }

    pub const fn image_scale_ratio(self) -> (u16, u16) {
        match self {
            Self::Small => (1, 3),
            Self::Medium => (1, 2),
            Self::Large => (2, 3),
        }
    }

    pub const fn obj_dimensions(self) -> (u16, u16) {
        match self {
            Self::Small => (32, 12),
            Self::Medium => (64, 24),
            Self::Large => (96, 36),
        }
    }
}

impl TryFrom<u8> for SpriteSizePreset {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Small),
            2 => Ok(Self::Medium),
            3 => Ok(Self::Large),
            other => Err(format!("unsupported sprite size preset: {other}")),
        }
    }
}

impl<'de> Deserialize<'de> for SpriteSizePreset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = u8::deserialize(deserializer)?;
        Self::try_from(raw).map_err(serde::de::Error::custom)
    }
}

/// A drawable object within a layer. Has its own local bitmap position and lifecycle.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Sprite {
    /// Terminal-native text sprite.
    /// If `font` is None: characters written directly to buffer (fast path).
    /// If `font` is Some: text is rasterized using a bitmap font definition (rasterizer path).
    Text {
        /// Optional stable identifier for script-side lookups (e.g. intro.rs logic hooks).
        #[serde(default)]
        id: Option<String>,
        content: String,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        size: Option<SpriteSizePreset>,
        font: Option<String>,
        /// Optional per-sprite renderer mode override.
        #[serde(default, rename = "force-renderer-mode")]
        force_renderer_mode: Option<SceneRenderedMode>,
        /// Optional per-sprite font mode override (e.g. ascii/raster/half/quad/braille).
        #[serde(default, rename = "force-font-mode")]
        force_font_mode: Option<String>,
        align_x: Option<HorizontalAlign>,
        align_y: Option<VerticalAlign>,
        fg_colour: Option<TermColour>,
        bg_colour: Option<TermColour>,
        /// Delay from scene start (ms) before sprite becomes visible.
        #[serde(default)]
        appear_at_ms: Option<u64>,
        /// Optional absolute scene time (ms) after which sprite is hidden.
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        /// Optional reveal duration (ms). When set, text appears left->right.
        #[serde(default)]
        reveal_ms: Option<u64>,
        /// Hide the sprite immediately when scene enters on_leave.
        #[serde(default)]
        hide_on_leave: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
        /// Optional glow halo rendered behind the sprite.
        #[serde(default)]
        glow: Option<Glow>,
        /// Text case transformation applied before glyph lookup.
        /// Default: `none` (preserve authored case).
        #[serde(default)]
        text_transform: TextTransform,
    },
    /// Bitmap image sprite rendered on terminal grid in selected mode.
    Image {
        #[serde(default)]
        id: Option<String>,
        source: String,
        /// Optional spritesheet column count. When >1, image is treated as a sheet.
        #[serde(default, rename = "spritesheet-columns")]
        spritesheet_columns: Option<u16>,
        /// Optional spritesheet row count. When >1, image is treated as a sheet.
        #[serde(default, rename = "spritesheet-rows")]
        spritesheet_rows: Option<u16>,
        /// Optional 0-based frame index selected from spritesheet cells.
        #[serde(default, rename = "frame-index")]
        frame_index: Option<u16>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        size: Option<SpriteSizePreset>,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        /// When true, scales the image to exactly fill its resolved draw area.
        #[serde(default, rename = "stretch-to-area")]
        stretch_to_area: bool,
        #[serde(default, rename = "force-renderer-mode")]
        force_renderer_mode: Option<SceneRenderedMode>,
        align_x: Option<HorizontalAlign>,
        align_y: Option<VerticalAlign>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
    },
    /// Wavefront OBJ mesh rendered as terminal wireframe/material.
    Obj {
        #[serde(default)]
        id: Option<String>,
        source: String,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        size: Option<SpriteSizePreset>,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        #[serde(default, rename = "force-renderer-mode")]
        force_renderer_mode: Option<SceneRenderedMode>,
        #[serde(default, rename = "surface-mode")]
        surface_mode: Option<String>,
        #[serde(default, rename = "backface-cull")]
        backface_cull: Option<bool>,
        /// Vertical clip region top edge (0.0 = top of sprite, 1.0 = bottom). Default 0.0.
        #[serde(default, rename = "clip-y-min")]
        clip_y_min: Option<f32>,
        /// Vertical clip region bottom edge (0.0 = top of sprite, 1.0 = bottom). Default 1.0.
        #[serde(default, rename = "clip-y-max")]
        clip_y_max: Option<f32>,
        #[serde(default)]
        scale: Option<f32>,
        #[serde(default, rename = "yaw-deg")]
        yaw_deg: Option<f32>,
        #[serde(default, rename = "pitch-deg")]
        pitch_deg: Option<f32>,
        #[serde(default, rename = "roll-deg")]
        roll_deg: Option<f32>,
        /// Static initial rotation of the mesh around the X axis (degrees).
        #[serde(default, rename = "rotation-x")]
        rotation_x: Option<f32>,
        /// Static initial rotation of the mesh around the Y axis (degrees).
        #[serde(default, rename = "rotation-y")]
        rotation_y: Option<f32>,
        /// Static initial rotation of the mesh around the Z axis (degrees).
        #[serde(default, rename = "rotation-z")]
        rotation_z: Option<f32>,
        #[serde(default, rename = "rotate-y-deg-per-sec")]
        rotate_y_deg_per_sec: Option<f32>,
        #[serde(default, rename = "camera-distance")]
        camera_distance: Option<f32>,
        #[serde(default, rename = "fov-degrees")]
        fov_degrees: Option<f32>,
        #[serde(default, rename = "near-clip")]
        near_clip: Option<f32>,
        #[serde(default, rename = "light-direction-x")]
        light_direction_x: Option<f32>,
        #[serde(default, rename = "light-direction-y")]
        light_direction_y: Option<f32>,
        #[serde(default, rename = "light-direction-z")]
        light_direction_z: Option<f32>,
        #[serde(default, rename = "light-2-direction-x")]
        light_2_direction_x: Option<f32>,
        #[serde(default, rename = "light-2-direction-y")]
        light_2_direction_y: Option<f32>,
        #[serde(default, rename = "light-2-direction-z")]
        light_2_direction_z: Option<f32>,
        #[serde(default, rename = "light-2-intensity")]
        light_2_intensity: Option<f32>,
        #[serde(default, rename = "light-point-x")]
        light_point_x: Option<f32>,
        #[serde(default, rename = "light-point-y")]
        light_point_y: Option<f32>,
        #[serde(default, rename = "light-point-z")]
        light_point_z: Option<f32>,
        #[serde(default, rename = "light-point-intensity")]
        light_point_intensity: Option<f32>,
        #[serde(default, rename = "light-point-colour")]
        light_point_colour: Option<TermColour>,
        #[serde(default, rename = "light-point-flicker-depth")]
        light_point_flicker_depth: Option<f32>,
        #[serde(default, rename = "light-point-flicker-hz")]
        light_point_flicker_hz: Option<f32>,
        /// Orbit angular speed (Hz) for point light 1 around the Y axis (smooth).
        #[serde(default, rename = "light-point-orbit-hz")]
        light_point_orbit_hz: Option<f32>,
        /// Teleport snap rate (Hz) for point light 1: instant jump to pseudo-random position at this frequency.
        #[serde(default, rename = "light-point-snap-hz")]
        light_point_snap_hz: Option<f32>,
        #[serde(default, rename = "light-point-2-x")]
        light_point_2_x: Option<f32>,
        #[serde(default, rename = "light-point-2-y")]
        light_point_2_y: Option<f32>,
        #[serde(default, rename = "light-point-2-z")]
        light_point_2_z: Option<f32>,
        #[serde(default, rename = "light-point-2-intensity")]
        light_point_2_intensity: Option<f32>,
        #[serde(default, rename = "light-point-2-colour")]
        light_point_2_colour: Option<TermColour>,
        #[serde(default, rename = "light-point-2-flicker-depth")]
        light_point_2_flicker_depth: Option<f32>,
        #[serde(default, rename = "light-point-2-flicker-hz")]
        light_point_2_flicker_hz: Option<f32>,
        /// Orbit angular speed (Hz) for point light 2 around the Y axis (smooth).
        #[serde(default, rename = "light-point-2-orbit-hz")]
        light_point_2_orbit_hz: Option<f32>,
        /// Teleport snap rate (Hz) for point light 2: instant jump to pseudo-random position at this frequency.
        #[serde(default, rename = "light-point-2-snap-hz")]
        light_point_2_snap_hz: Option<f32>,
        #[serde(default, rename = "cel-levels")]
        cel_levels: Option<u8>,
        #[serde(default, rename = "shadow-colour")]
        shadow_colour: Option<TermColour>,
        #[serde(default, rename = "midtone-colour")]
        midtone_colour: Option<TermColour>,
        #[serde(default, rename = "highlight-colour")]
        highlight_colour: Option<TermColour>,
        #[serde(default, rename = "tone-mix")]
        tone_mix: Option<f32>,
        #[serde(default, rename = "draw-char")]
        draw_char: Option<String>,
        align_x: Option<HorizontalAlign>,
        align_y: Option<VerticalAlign>,
        fg_colour: Option<TermColour>,
        bg_colour: Option<TermColour>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
        /// When `true` (default) and the scene has `prerender: true`, this sprite will be
        /// pre-rendered once at its initial pose during scene transition.
        /// Set to `false` to opt out (e.g. for sprites that animate continuously).
        #[serde(default = "default_true")]
        prerender: bool,
    },
    /// UI panel container rendered as a themed box with optional border, corner radius and shadow.
    #[serde(rename = "panel")]
    Panel {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default, rename = "width-percent")]
        width_percent: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        #[serde(default = "default_panel_padding")]
        padding: u16,
        #[serde(default = "default_panel_border_width", rename = "border-width")]
        border_width: u16,
        #[serde(default = "default_panel_radius", rename = "corner-radius")]
        corner_radius: u16,
        #[serde(default = "default_panel_shadow_x", rename = "shadow-x")]
        shadow_x: i32,
        #[serde(default = "default_panel_shadow_y", rename = "shadow-y")]
        shadow_y: i32,
        #[serde(default, rename = "force-renderer-mode")]
        force_renderer_mode: Option<SceneRenderedMode>,
        align_x: Option<HorizontalAlign>,
        align_y: Option<VerticalAlign>,
        fg_colour: Option<TermColour>,
        bg_colour: Option<TermColour>,
        #[serde(rename = "border-colour")]
        border_colour: Option<TermColour>,
        #[serde(rename = "shadow-colour")]
        shadow_colour: Option<TermColour>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
        #[serde(default)]
        children: Vec<Sprite>,
    },
    /// Grid layout container. Children are renderable sprites arranged in rows/columns.
    Grid {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        #[serde(default, rename = "gap-x")]
        gap_x: u16,
        #[serde(default, rename = "gap-y")]
        gap_y: u16,
        #[serde(default, rename = "force-renderer-mode")]
        force_renderer_mode: Option<SceneRenderedMode>,
        align_x: Option<HorizontalAlign>,
        align_y: Option<VerticalAlign>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
        columns: Vec<String>,
        rows: Vec<String>,
        #[serde(default)]
        children: Vec<Sprite>,
    },
    /// Flex layout container. Children are auto-stacked vertically (column) or horizontally (row).
    #[serde(rename = "flex")]
    Flex {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        #[serde(default)]
        gap: u16,
        #[serde(default)]
        direction: FlexDirection,
        #[serde(default, rename = "force-renderer-mode")]
        force_renderer_mode: Option<SceneRenderedMode>,
        align_x: Option<HorizontalAlign>,
        align_y: Option<VerticalAlign>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
        #[serde(default)]
        children: Vec<Sprite>,
    },
    /// A pre-rendered 3D scene blitted from the Scene3D atlas.
    /// The atlas is populated during scene preparation (before first frame) from a `.scene3d.yml`.
    Scene3D {
        #[serde(default)]
        id: Option<String>,
        /// Path to `.scene3d.yml` relative to the mod root.
        src: String,
        /// Which named frame from the atlas to display. Changed at runtime via `scene.set`.
        #[serde(default)]
        frame: String,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
    },
}

impl Sprite {
    pub fn id(&self) -> Option<&str> {
        match self {
            Sprite::Text { id, .. }
            | Sprite::Image { id, .. }
            | Sprite::Obj { id, .. }
            | Sprite::Panel { id, .. }
            | Sprite::Grid { id, .. }
            | Sprite::Flex { id, .. }
            | Sprite::Scene3D { id, .. } => id.as_deref(),
        }
    }

    pub fn z_index(&self) -> i32 {
        match self {
            Sprite::Text { z_index, .. }
            | Sprite::Image { z_index, .. }
            | Sprite::Obj { z_index, .. }
            | Sprite::Panel { z_index, .. }
            | Sprite::Grid { z_index, .. }
            | Sprite::Flex { z_index, .. }
            | Sprite::Scene3D { z_index, .. } => *z_index,
        }
    }

    pub fn stages(&self) -> &LayerStages {
        match self {
            Sprite::Text { stages, .. }
            | Sprite::Image { stages, .. }
            | Sprite::Obj { stages, .. }
            | Sprite::Panel { stages, .. }
            | Sprite::Grid { stages, .. }
            | Sprite::Flex { stages, .. }
            | Sprite::Scene3D { stages, .. } => stages,
        }
    }

    pub fn grid_position(&self) -> (u16, u16, u16, u16) {
        let (row, col, row_span, col_span) = match self {
            Sprite::Text {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Image {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Obj {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Panel {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Grid {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Flex {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Scene3D {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            } => (*grid_row, *grid_col, *row_span, *col_span),
        };
        (row.max(1), col.max(1), row_span.max(1), col_span.max(1))
    }

    pub fn walk_recursive<'a, F>(&'a self, visit: &mut F)
    where
        F: FnMut(&'a Sprite),
    {
        visit(self);
        match self {
            Sprite::Panel { children, .. }
            | Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. } => {
                for child in children {
                    child.walk_recursive(visit);
                }
            }
            _ => {}
        }
    }

    pub fn behaviors(&self) -> &[BehaviorSpec] {
        match self {
            Sprite::Text { behaviors, .. }
            | Sprite::Image { behaviors, .. }
            | Sprite::Obj { behaviors, .. }
            | Sprite::Panel { behaviors, .. }
            | Sprite::Grid { behaviors, .. }
            | Sprite::Flex { behaviors, .. }
            | Sprite::Scene3D { behaviors, .. } => behaviors,
        }
    }

    pub fn hide_on_leave(&self) -> bool {
        match self {
            Sprite::Text { hide_on_leave, .. }
            | Sprite::Image { hide_on_leave, .. }
            | Sprite::Obj { hide_on_leave, .. }
            | Sprite::Panel { hide_on_leave, .. }
            | Sprite::Grid { hide_on_leave, .. }
            | Sprite::Flex { hide_on_leave, .. }
            | Sprite::Scene3D { hide_on_leave, .. } => *hide_on_leave,
        }
    }

    pub fn appear_at_ms(&self) -> Option<u64> {
        match self {
            Sprite::Text { appear_at_ms, .. }
            | Sprite::Image { appear_at_ms, .. }
            | Sprite::Obj { appear_at_ms, .. }
            | Sprite::Panel { appear_at_ms, .. }
            | Sprite::Grid { appear_at_ms, .. }
            | Sprite::Flex { appear_at_ms, .. }
            | Sprite::Scene3D { appear_at_ms, .. } => *appear_at_ms,
        }
    }

    pub fn disappear_at_ms(&self) -> Option<u64> {
        match self {
            Sprite::Text { disappear_at_ms, .. }
            | Sprite::Image { disappear_at_ms, .. }
            | Sprite::Obj { disappear_at_ms, .. }
            | Sprite::Panel { disappear_at_ms, .. }
            | Sprite::Grid { disappear_at_ms, .. }
            | Sprite::Flex { disappear_at_ms, .. }
            | Sprite::Scene3D { disappear_at_ms, .. } => *disappear_at_ms,
        }
    }

    pub fn animations(&self) -> &[crate::scene::Animation] {
        match self {
            Sprite::Text { animations, .. }
            | Sprite::Image { animations, .. }
            | Sprite::Obj { animations, .. }
            | Sprite::Panel { animations, .. }
            | Sprite::Grid { animations, .. }
            | Sprite::Flex { animations, .. }
            | Sprite::Scene3D { animations, .. } => animations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Sprite, SpriteSizePreset};
    use crate::scene::SceneRenderedMode;

    #[test]
    fn supports_negative_sprite_offsets() {
        let raw = r#"
type: text
content: "TEST"
x: -2
y: -8
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("sprite should parse");
        match sprite {
            Sprite::Text { x, y, .. } => {
                assert_eq!(x, -2);
                assert_eq!(y, -8);
            }
            Sprite::Image { .. }
            | Sprite::Obj { .. }
            | Sprite::Panel { .. }
            | Sprite::Grid { .. }
            | Sprite::Flex { .. }
            | Sprite::Scene3D { .. } => {
                panic!("expected text sprite")
            }
        }
    }

    #[test]
    fn parses_force_renderer_and_font_modes() {
        let raw = r#"
type: text
content: "TEST"
font: "generic:2"
force-renderer-mode: quadblock
force-font-mode: braille
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("sprite should parse");
        match sprite {
            Sprite::Text {
                force_renderer_mode,
                force_font_mode,
                behaviors,
                ..
            } => {
                assert_eq!(force_renderer_mode, Some(SceneRenderedMode::QuadBlock));
                assert_eq!(force_font_mode.as_deref(), Some("braille"));
                assert!(behaviors.is_empty());
            }
            Sprite::Image { .. }
            | Sprite::Obj { .. }
            | Sprite::Panel { .. }
            | Sprite::Grid { .. }
            | Sprite::Flex { .. }
            | Sprite::Scene3D { .. } => {
                panic!("expected text sprite")
            }
        }
    }

    #[test]
    fn parses_image_sprite() {
        let raw = r#"
type: image
source: "/assets/images/tux.png"
size: 3
force-renderer-mode: halfblock
stretch-to-area: true
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("image sprite should parse");
        match sprite {
            Sprite::Image {
                source,
                spritesheet_columns,
                spritesheet_rows,
                frame_index,
                size,
                width,
                height,
                stretch_to_area,
                force_renderer_mode,
                ..
            } => {
                assert_eq!(source, "/assets/images/tux.png");
                assert_eq!(spritesheet_columns, None);
                assert_eq!(spritesheet_rows, None);
                assert_eq!(frame_index, None);
                assert_eq!(size, Some(SpriteSizePreset::Large));
                assert_eq!(width, None);
                assert_eq!(height, None);
                assert!(stretch_to_area);
                assert_eq!(force_renderer_mode, Some(SceneRenderedMode::HalfBlock));
            }
            Sprite::Text { .. }
            | Sprite::Obj { .. }
            | Sprite::Panel { .. }
            | Sprite::Grid { .. }
            | Sprite::Flex { .. }
            | Sprite::Scene3D { .. } => {
                panic!("expected image sprite")
            }
        }
    }

    #[test]
    fn parses_image_spritesheet_fields() {
        let raw = r#"
type: image
source: "/assets/images/difficulty1.png"
spritesheet-columns: 5
spritesheet-rows: 1
frame-index: 3
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("image sprite should parse");
        match sprite {
            Sprite::Image {
                spritesheet_columns,
                spritesheet_rows,
                frame_index,
                ..
            } => {
                assert_eq!(spritesheet_columns, Some(5));
                assert_eq!(spritesheet_rows, Some(1));
                assert_eq!(frame_index, Some(3));
            }
            _ => panic!("expected image sprite"),
        }
    }

    #[test]
    fn rejects_unsupported_size_preset() {
        let raw = r#"
type: image
source: "/assets/images/tux.png"
size: 4
"#;
        let error = serde_yaml::from_str::<Sprite>(raw).expect_err("size should be rejected");
        assert!(
            error.to_string().contains("unsupported sprite size preset"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn parses_grid_sprite_with_children() {
        let raw = r#"
type: grid
columns: ["1fr","1fr","1fr"]
rows: ["auto","1fr"]
children:
  - type: text
    content: "A"
    grid-col: 1
    grid-row: 1
  - type: image
    source: "/assets/images/tux.png"
    grid-col: 2
    grid-row: 2
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("grid sprite should parse");
        match sprite {
            Sprite::Grid {
                columns,
                rows,
                children,
                ..
            } => {
                assert_eq!(columns.len(), 3);
                assert_eq!(rows.len(), 2);
                assert_eq!(children.len(), 2);
            }
            Sprite::Text { .. }
            | Sprite::Image { .. }
            | Sprite::Obj { .. }
            | Sprite::Panel { .. }
            | Sprite::Flex { .. }
            | Sprite::Scene3D { .. } => {
                panic!("expected grid sprite")
            }
        }
    }

    #[test]
    fn parses_sprite_behaviors() {
        let raw = r#"
type: text
id: title
content: "TEST"
behaviors:
  - name: blink
    params:
      visible_ms: 100
      hidden_ms: 150
  - name: bob
    params:
      amplitude_y: 2
      period_ms: 1000
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("sprite should parse");
        match sprite {
            Sprite::Text { behaviors, .. } => {
                assert_eq!(behaviors.len(), 2);
                assert_eq!(behaviors[0].name, "blink");
                assert_eq!(behaviors[1].params.amplitude_y, Some(2));
            }
            Sprite::Image { .. }
            | Sprite::Obj { .. }
            | Sprite::Panel { .. }
            | Sprite::Grid { .. }
            | Sprite::Flex { .. }
            | Sprite::Scene3D { .. } => {
                panic!("expected text sprite")
            }
        }
    }
}

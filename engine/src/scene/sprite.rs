use super::{color::TermColour, LayerStages, SceneRenderedMode};
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

fn default_grid_line() -> u16 {
    1
}

fn default_grid_span() -> u16 {
    1
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
        /// Optional glow halo rendered behind the sprite.
        #[serde(default)]
        glow: Option<Glow>,
    },
    /// PNG image sprite rendered on terminal grid in selected mode.
    Image {
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
        width: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
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
        columns: Vec<String>,
        rows: Vec<String>,
        #[serde(default)]
        children: Vec<Sprite>,
    },
}

impl Sprite {
    pub fn z_index(&self) -> i32 {
        match self {
            Sprite::Text { z_index, .. }
            | Sprite::Image { z_index, .. }
            | Sprite::Grid { z_index, .. } => *z_index,
        }
    }

    pub fn stages(&self) -> &LayerStages {
        match self {
            Sprite::Text { stages, .. }
            | Sprite::Image { stages, .. }
            | Sprite::Grid { stages, .. } => stages,
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
            | Sprite::Grid {
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
        if let Sprite::Grid { children, .. } = self {
            for child in children {
                child.walk_recursive(visit);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sprite;
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
            Sprite::Image { .. } | Sprite::Grid { .. } => panic!("expected text sprite"),
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
                ..
            } => {
                assert_eq!(force_renderer_mode, Some(SceneRenderedMode::QuadBlock));
                assert_eq!(force_font_mode.as_deref(), Some("braille"));
            }
            Sprite::Image { .. } | Sprite::Grid { .. } => panic!("expected text sprite"),
        }
    }

    #[test]
    fn parses_image_sprite() {
        let raw = r#"
type: image
source: "/assets/images/tux.png"
width: 64
height: 48
force-renderer-mode: halfblock
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("image sprite should parse");
        match sprite {
            Sprite::Image {
                source,
                width,
                height,
                force_renderer_mode,
                ..
            } => {
                assert_eq!(source, "/assets/images/tux.png");
                assert_eq!(width, Some(64));
                assert_eq!(height, Some(48));
                assert_eq!(force_renderer_mode, Some(SceneRenderedMode::HalfBlock));
            }
            Sprite::Text { .. } | Sprite::Grid { .. } => panic!("expected image sprite"),
        }
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
            Sprite::Text { .. } | Sprite::Image { .. } => panic!("expected grid sprite"),
        }
    }
}

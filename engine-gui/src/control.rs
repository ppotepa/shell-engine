//! Trait-based GUI control model — each widget type implements [`GuiControl`].
//!
//! This replaces the monolithic `GuiWidgetDef` enum approach with polymorphic
//! dispatch. The system loop calls `on_mouse_down`, `on_drag`, `on_mouse_up`
//! without knowing the widget type — each control owns its behavior.

use crate::state::GuiWidgetState;

// ── Helper types ─────────────────────────────────────────────────────────────

/// Hit-test rectangle in screen coordinates.
#[derive(Debug, Clone, Copy)]
pub struct WidgetRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl WidgetRect {
    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        mx >= self.x as f32
            && mx < (self.x + self.w) as f32
            && my >= self.y as f32
            && my < (self.y + self.h) as f32
    }
}

/// Instruction for the engine to position a handle sprite each frame.
#[derive(Debug, Clone)]
pub struct VisualSync {
    /// Sprite alias (resolved by the caller via TargetResolver).
    pub sprite_alias: String,
    /// Pixel offset to apply as `offset_x` on the sprite's ObjectRuntimeState.
    pub offset_x: i32,
}

// ── Core trait ───────────────────────────────────────────────────────────────

/// Core trait for all GUI controls.
///
/// Each widget type implements its own input handling and visual sync logic.
/// The [`GuiSystem`](crate::system::GuiSystem) dispatches events through these
/// methods without knowing the concrete type.
pub trait GuiControl: Send + Sync + std::fmt::Debug {
    /// Unique widget identifier (authored in YAML `id:` field).
    fn id(&self) -> &str;

    /// Associated visual sprite id.
    fn sprite(&self) -> &str;

    /// Hit-test bounding rect. Return `None` for non-interactive widgets.
    fn bounds(&self) -> Option<WidgetRect>;

    /// Default value for this widget when first created.
    fn initial_value(&self) -> f64;

    /// Called when mouse button is pressed inside this widget's bounds.
    fn on_mouse_down(&self, state: &mut GuiWidgetState, x: f32, y: f32);

    /// Called each frame while the mouse is held and moving (drag).
    fn on_drag(&self, state: &mut GuiWidgetState, x: f32, y: f32);

    /// Called when mouse button is released.
    /// `still_hovered` is true if the cursor is still inside bounds.
    fn on_mouse_up(&self, state: &mut GuiWidgetState, still_hovered: bool);

    /// Optional per-frame visual positioning for engine-managed sprites.
    /// Returns a [`VisualSync`] describing which sprite to move and by how much.
    fn visual_sync(&self, value: f64) -> Option<VisualSync>;
}

// ── Slider ───────────────────────────────────────────────────────────────────

/// Horizontal drag slider mapped to a `[min, max]` range.
#[derive(Debug, Clone)]
pub struct SliderControl {
    pub id: String,
    pub sprite: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub min: f64,
    pub max: f64,
    pub value: f64,
    pub hit_padding: i32,
    pub handle: String,
}

impl SliderControl {
    fn apply_drag(&self, state: &mut GuiWidgetState, mouse_x: f32) {
        let t = ((mouse_x - self.x as f32) / (self.w).max(1) as f32).clamp(0.0, 1.0) as f64;
        state.value = self.min + t * (self.max - self.min);
    }
}

impl GuiControl for SliderControl {
    fn id(&self) -> &str { &self.id }
    fn sprite(&self) -> &str { &self.sprite }

    fn bounds(&self) -> Option<WidgetRect> {
        let p = self.hit_padding;
        Some(WidgetRect {
            x: self.x - p,
            y: self.y - p,
            w: self.w + 2 * p,
            h: self.h + 2 * p,
        })
    }

    fn initial_value(&self) -> f64 {
        self.value.max(self.min)
    }

    fn on_mouse_down(&self, state: &mut GuiWidgetState, x: f32, _y: f32) {
        state.pressed = true;
        self.apply_drag(state, x);
    }

    fn on_drag(&self, state: &mut GuiWidgetState, x: f32, _y: f32) {
        self.apply_drag(state, x);
    }

    fn on_mouse_up(&self, state: &mut GuiWidgetState, _still_hovered: bool) {
        state.pressed = false;
    }

    fn visual_sync(&self, value: f64) -> Option<VisualSync> {
        if self.handle.is_empty() {
            return None;
        }
        let range = self.max - self.min;
        let frac = if range.abs() > f64::EPSILON {
            ((value - self.min) / range).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Some(VisualSync {
            sprite_alias: self.handle.clone(),
            offset_x: (frac * self.w as f64).round() as i32,
        })
    }
}

// ── Button ───────────────────────────────────────────────────────────────────

/// Clickable button — fires `clicked` once per press-release inside bounds.
#[derive(Debug, Clone)]
pub struct ButtonControl {
    pub id: String,
    pub sprite: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl GuiControl for ButtonControl {
    fn id(&self) -> &str { &self.id }
    fn sprite(&self) -> &str { &self.sprite }

    fn bounds(&self) -> Option<WidgetRect> {
        Some(WidgetRect { x: self.x, y: self.y, w: self.w, h: self.h })
    }

    fn initial_value(&self) -> f64 { 0.0 }

    fn on_mouse_down(&self, state: &mut GuiWidgetState, _x: f32, _y: f32) {
        state.pressed = true;
    }

    fn on_drag(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}

    fn on_mouse_up(&self, state: &mut GuiWidgetState, still_hovered: bool) {
        state.pressed = false;
        if still_hovered {
            state.clicked = true;
        }
    }

    fn visual_sync(&self, _value: f64) -> Option<VisualSync> { None }
}

// ── Toggle ───────────────────────────────────────────────────────────────────

/// Boolean toggle — flips value on each click.
#[derive(Debug, Clone)]
pub struct ToggleControl {
    pub id: String,
    pub sprite: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub initial_on: bool,
}

impl GuiControl for ToggleControl {
    fn id(&self) -> &str { &self.id }
    fn sprite(&self) -> &str { &self.sprite }

    fn bounds(&self) -> Option<WidgetRect> {
        Some(WidgetRect { x: self.x, y: self.y, w: self.w, h: self.h })
    }

    fn initial_value(&self) -> f64 {
        if self.initial_on { 1.0 } else { 0.0 }
    }

    fn on_mouse_down(&self, state: &mut GuiWidgetState, _x: f32, _y: f32) {
        state.pressed = true;
    }

    fn on_drag(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}

    fn on_mouse_up(&self, state: &mut GuiWidgetState, still_hovered: bool) {
        state.pressed = false;
        if still_hovered {
            state.clicked = true;
            state.value = if state.value > 0.5 { 0.0 } else { 1.0 };
            state.changed = true;
        }
    }

    fn visual_sync(&self, _value: f64) -> Option<VisualSync> { None }
}

// ── Panel ────────────────────────────────────────────────────────────────────

/// Visibility group — non-interactive, controls panel layer show/hide.
#[derive(Debug, Clone)]
pub struct PanelControl {
    pub id: String,
    pub sprite: String,
    pub visible: bool,
}

impl GuiControl for PanelControl {
    fn id(&self) -> &str { &self.id }
    fn sprite(&self) -> &str { &self.sprite }
    fn bounds(&self) -> Option<WidgetRect> { None }

    fn initial_value(&self) -> f64 {
        if self.visible { 1.0 } else { 0.0 }
    }

    fn on_mouse_down(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}
    fn on_drag(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}
    fn on_mouse_up(&self, _state: &mut GuiWidgetState, _still_hovered: bool) {}
    fn visual_sync(&self, _value: f64) -> Option<VisualSync> { None }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_drag_computes_value() {
        let slider = SliderControl {
            id: "s".into(), sprite: "track".into(),
            x: 10, y: 10, w: 100, h: 12,
            min: 0.0, max: 255.0, value: 0.0,
            hit_padding: 0, handle: "h".into(),
        };
        let mut ws = GuiWidgetState::default();
        slider.on_mouse_down(&mut ws, 60.0, 15.0);
        assert!(ws.pressed);
        // (60-10)/100 = 0.5 → 127.5
        assert!((ws.value - 127.5).abs() < 0.1);
    }

    #[test]
    fn slider_visual_sync_returns_offset() {
        let slider = SliderControl {
            id: "s".into(), sprite: "track".into(),
            x: 50, y: 10, w: 190, h: 12,
            min: 0.0, max: 255.0, value: 0.0,
            hit_padding: 0, handle: "h".into(),
        };
        let sync = slider.visual_sync(127.5).unwrap();
        assert_eq!(sync.sprite_alias, "h");
        // 127.5/255 * 190 ≈ 95
        assert!((sync.offset_x - 95).abs() <= 1);
    }

    #[test]
    fn button_click_fires_on_release_inside() {
        let btn = ButtonControl {
            id: "b".into(), sprite: "b".into(),
            x: 0, y: 0, w: 50, h: 20,
        };
        let mut ws = GuiWidgetState::default();
        btn.on_mouse_down(&mut ws, 10.0, 5.0);
        assert!(ws.pressed);
        assert!(!ws.clicked);
        btn.on_mouse_up(&mut ws, true);
        assert!(!ws.pressed);
        assert!(ws.clicked);
    }

    #[test]
    fn button_no_click_on_release_outside() {
        let btn = ButtonControl {
            id: "b".into(), sprite: "b".into(),
            x: 0, y: 0, w: 50, h: 20,
        };
        let mut ws = GuiWidgetState::default();
        btn.on_mouse_down(&mut ws, 10.0, 5.0);
        btn.on_mouse_up(&mut ws, false);
        assert!(!ws.clicked);
    }

    #[test]
    fn toggle_flips_value() {
        let toggle = ToggleControl {
            id: "t".into(), sprite: "t".into(),
            x: 0, y: 0, w: 100, h: 16, initial_on: false,
        };
        let mut ws = GuiWidgetState { value: 0.0, ..Default::default() };
        toggle.on_mouse_up(&mut ws, true);
        assert!((ws.value - 1.0).abs() < f64::EPSILON);
        assert!(ws.changed);

        ws.changed = false;
        toggle.on_mouse_up(&mut ws, true);
        assert!(ws.value.abs() < f64::EPSILON);
    }

    #[test]
    fn widget_rect_hit_test() {
        let r = WidgetRect { x: 10, y: 20, w: 100, h: 50 };
        assert!(r.hit_test(10.0, 20.0));
        assert!(r.hit_test(109.0, 69.0));
        assert!(!r.hit_test(110.0, 20.0));
        assert!(!r.hit_test(5.0, 20.0));
    }
}

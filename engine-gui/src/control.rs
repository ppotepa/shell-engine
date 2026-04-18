//! Trait-based GUI control model — each widget type implements [`GuiControl`].
//!
//! This replaces the monolithic `GuiWidgetDef` enum approach with polymorphic
//! dispatch. The system loop calls `on_mouse_down`, `on_drag`, `on_mouse_up`
//! without knowing the widget type — each control owns its behavior.

use crate::state::GuiWidgetState;
use engine_events::{KeyCode, KeyEvent};

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

#[derive(Debug, Clone)]
pub enum VisualSyncAction {
    OffsetX { sprite_alias: String, offset_x: i32 },
    SetVisible { sprite_alias: String, visible: bool },
    SetText { sprite_alias: String, text: String },
}

/// Instruction bundle for engine-managed widget visuals.
#[derive(Debug, Clone, Default)]
pub struct VisualSync {
    pub actions: Vec<VisualSyncAction>,
}

impl VisualSync {
    pub fn single(action: VisualSyncAction) -> Self {
        Self {
            actions: vec![action],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChoiceOption {
    pub value: String,
    pub label: String,
}

impl ChoiceOption {
    pub fn new(value: String, label: String) -> Self {
        Self { value, label }
    }
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
    fn bounds(&self, state: &GuiWidgetState) -> Option<WidgetRect>;

    /// Initial state for this widget when first created.
    fn initial_state(&self) -> GuiWidgetState;

    /// Called when mouse button is pressed inside this widget's bounds.
    fn on_mouse_down(&self, state: &mut GuiWidgetState, x: f32, y: f32);

    /// Called each frame while the mouse is held and moving (drag).
    fn on_drag(&self, state: &mut GuiWidgetState, x: f32, y: f32);

    /// Called when mouse button is released.
    /// `still_hovered` is true if the cursor is still inside bounds.
    fn on_mouse_up(&self, state: &mut GuiWidgetState, x: f32, y: f32, still_hovered: bool);

    /// Whether this widget accepts keyboard focus.
    fn wants_keyboard_focus(&self) -> bool {
        false
    }

    /// Called for key-down events while this widget has keyboard focus.
    fn on_key_down(&self, _state: &mut GuiWidgetState, _key: &KeyEvent) {}

    /// Optional per-frame visual positioning for engine-managed sprites.
    /// Returns a [`VisualSync`] describing which sprite to move and by how much.
    fn visual_sync(&self, state: &GuiWidgetState) -> Option<VisualSync>;

    /// Optional runtime-layout bound update (used when controls follow sprite region).
    fn set_bounds(&mut self, _rect: WidgetRect) {}
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
    pub follow_layout: bool,
}

impl SliderControl {
    fn apply_drag(&self, state: &mut GuiWidgetState, mouse_x: f32) {
        let t = ((mouse_x - self.x as f32) / (self.w).max(1) as f32).clamp(0.0, 1.0) as f64;
        state.value = self.min + t * (self.max - self.min);
    }
}

impl GuiControl for SliderControl {
    fn id(&self) -> &str {
        &self.id
    }
    fn sprite(&self) -> &str {
        &self.sprite
    }

    fn bounds(&self, _state: &GuiWidgetState) -> Option<WidgetRect> {
        let p = self.hit_padding;
        Some(WidgetRect {
            x: self.x - p,
            y: self.y - p,
            w: self.w + 2 * p,
            h: self.h + 2 * p,
        })
    }

    fn initial_state(&self) -> GuiWidgetState {
        GuiWidgetState {
            value: self.value.max(self.min),
            ..Default::default()
        }
    }

    fn on_mouse_down(&self, state: &mut GuiWidgetState, x: f32, _y: f32) {
        state.pressed = true;
        self.apply_drag(state, x);
    }

    fn on_drag(&self, state: &mut GuiWidgetState, x: f32, _y: f32) {
        self.apply_drag(state, x);
    }

    fn on_mouse_up(&self, state: &mut GuiWidgetState, _x: f32, _y: f32, _still_hovered: bool) {
        state.pressed = false;
    }

    fn visual_sync(&self, state: &GuiWidgetState) -> Option<VisualSync> {
        if self.handle.is_empty() {
            return None;
        }
        let range = self.max - self.min;
        let frac = if range.abs() > f64::EPSILON {
            ((state.value - self.min) / range).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Some(VisualSync::single(VisualSyncAction::OffsetX {
            sprite_alias: self.handle.clone(),
            offset_x: (frac * self.w as f64).round() as i32,
        }))
    }

    fn set_bounds(&mut self, rect: WidgetRect) {
        if !self.follow_layout {
            return;
        }
        self.x = rect.x;
        self.y = rect.y;
        self.w = rect.w.max(1);
        self.h = rect.h.max(1);
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
    pub follow_layout: bool,
}

impl GuiControl for ButtonControl {
    fn id(&self) -> &str {
        &self.id
    }
    fn sprite(&self) -> &str {
        &self.sprite
    }

    fn bounds(&self, _state: &GuiWidgetState) -> Option<WidgetRect> {
        Some(WidgetRect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
        })
    }

    fn initial_state(&self) -> GuiWidgetState {
        GuiWidgetState::default()
    }

    fn on_mouse_down(&self, state: &mut GuiWidgetState, _x: f32, _y: f32) {
        state.pressed = true;
    }

    fn on_drag(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}

    fn on_mouse_up(&self, state: &mut GuiWidgetState, _x: f32, _y: f32, still_hovered: bool) {
        state.pressed = false;
        if still_hovered {
            state.clicked = true;
        }
    }

    fn visual_sync(&self, _state: &GuiWidgetState) -> Option<VisualSync> {
        None
    }

    fn set_bounds(&mut self, rect: WidgetRect) {
        if !self.follow_layout {
            return;
        }
        self.x = rect.x;
        self.y = rect.y;
        self.w = rect.w.max(1);
        self.h = rect.h.max(1);
    }
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
    pub follow_layout: bool,
}

impl GuiControl for ToggleControl {
    fn id(&self) -> &str {
        &self.id
    }
    fn sprite(&self) -> &str {
        &self.sprite
    }

    fn bounds(&self, _state: &GuiWidgetState) -> Option<WidgetRect> {
        Some(WidgetRect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
        })
    }

    fn initial_state(&self) -> GuiWidgetState {
        GuiWidgetState {
            value: if self.initial_on { 1.0 } else { 0.0 },
            ..Default::default()
        }
    }

    fn on_mouse_down(&self, state: &mut GuiWidgetState, _x: f32, _y: f32) {
        state.pressed = true;
    }

    fn on_drag(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}

    fn on_mouse_up(&self, state: &mut GuiWidgetState, _x: f32, _y: f32, still_hovered: bool) {
        state.pressed = false;
        if still_hovered {
            state.clicked = true;
            state.value = if state.value > 0.5 { 0.0 } else { 1.0 };
            state.changed = true;
        }
    }

    fn visual_sync(&self, _state: &GuiWidgetState) -> Option<VisualSync> {
        None
    }

    fn set_bounds(&mut self, rect: WidgetRect) {
        if !self.follow_layout {
            return;
        }
        self.x = rect.x;
        self.y = rect.y;
        self.w = rect.w.max(1);
        self.h = rect.h.max(1);
    }
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
    fn id(&self) -> &str {
        &self.id
    }
    fn sprite(&self) -> &str {
        &self.sprite
    }
    fn bounds(&self, _state: &GuiWidgetState) -> Option<WidgetRect> {
        None
    }

    fn initial_state(&self) -> GuiWidgetState {
        GuiWidgetState {
            value: if self.visible { 1.0 } else { 0.0 },
            ..Default::default()
        }
    }

    fn on_mouse_down(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}
    fn on_drag(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}
    fn on_mouse_up(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32, _still_hovered: bool) {}
    fn visual_sync(&self, _state: &GuiWidgetState) -> Option<VisualSync> {
        None
    }
}

/// Single-choice segmented control.
#[derive(Debug, Clone)]
pub struct RadioGroupControl {
    pub id: String,
    pub sprite: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub options: Vec<ChoiceOption>,
    pub selected: usize,
    pub selected_sprites: Vec<String>,
    pub follow_layout: bool,
}

impl RadioGroupControl {
    fn index_at(&self, x: f32, y: f32) -> Option<usize> {
        if self.options.is_empty() || self.w <= 0 || self.h <= 0 {
            return None;
        }
        let rect = WidgetRect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
        };
        if !rect.hit_test(x, y) {
            return None;
        }
        let rel_x = (x - self.x as f32).max(0.0);
        let segment_w = (self.w as f32 / self.options.len().max(1) as f32).max(1.0);
        let idx = (rel_x / segment_w).floor() as usize;
        Some(idx.min(self.options.len().saturating_sub(1)))
    }
}

impl GuiControl for RadioGroupControl {
    fn id(&self) -> &str {
        &self.id
    }
    fn sprite(&self) -> &str {
        &self.sprite
    }
    fn bounds(&self, _state: &GuiWidgetState) -> Option<WidgetRect> {
        Some(WidgetRect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
        })
    }
    fn initial_state(&self) -> GuiWidgetState {
        let selected = self.selected.min(self.options.len().saturating_sub(1));
        GuiWidgetState {
            value: selected as f64,
            selected_index: Some(selected),
            ..Default::default()
        }
    }
    fn on_mouse_down(&self, state: &mut GuiWidgetState, x: f32, y: f32) {
        state.pressed = true;
        if let Some(idx) = self.index_at(x, y) {
            state.value = idx as f64;
            state.selected_index = Some(idx);
        }
    }
    fn on_drag(&self, state: &mut GuiWidgetState, x: f32, y: f32) {
        if let Some(idx) = self.index_at(x, y) {
            state.value = idx as f64;
            state.selected_index = Some(idx);
        }
    }
    fn on_mouse_up(&self, state: &mut GuiWidgetState, _x: f32, _y: f32, still_hovered: bool) {
        state.pressed = false;
        if still_hovered {
            state.clicked = true;
        }
    }
    fn visual_sync(&self, state: &GuiWidgetState) -> Option<VisualSync> {
        if self.selected_sprites.is_empty() {
            return None;
        }
        let selected = state.selected_index.unwrap_or(0);
        Some(VisualSync {
            actions: self
                .selected_sprites
                .iter()
                .enumerate()
                .filter(|(_, alias)| !alias.is_empty())
                .map(|(idx, alias)| VisualSyncAction::SetVisible {
                    sprite_alias: alias.clone(),
                    visible: idx == selected,
                })
                .collect(),
        })
    }
    fn set_bounds(&mut self, rect: WidgetRect) {
        if !self.follow_layout {
            return;
        }
        self.x = rect.x;
        self.y = rect.y;
        self.w = rect.w.max(1);
        self.h = rect.h.max(1);
    }
}

/// Editable single-line text field.
#[derive(Debug, Clone)]
pub struct TextInputControl {
    pub id: String,
    pub sprite: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub text_sprite: String,
    pub placeholder: String,
    pub value: String,
    pub max_length: usize,
    pub follow_layout: bool,
}

impl GuiControl for TextInputControl {
    fn id(&self) -> &str {
        &self.id
    }
    fn sprite(&self) -> &str {
        &self.sprite
    }
    fn bounds(&self, _state: &GuiWidgetState) -> Option<WidgetRect> {
        Some(WidgetRect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
        })
    }
    fn initial_state(&self) -> GuiWidgetState {
        GuiWidgetState {
            text: self.value.clone(),
            ..Default::default()
        }
    }
    fn on_mouse_down(&self, state: &mut GuiWidgetState, _x: f32, _y: f32) {
        state.pressed = true;
    }
    fn on_drag(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}
    fn on_mouse_up(&self, state: &mut GuiWidgetState, _x: f32, _y: f32, still_hovered: bool) {
        state.pressed = false;
        if still_hovered {
            state.clicked = true;
        }
    }
    fn wants_keyboard_focus(&self) -> bool {
        true
    }
    fn on_key_down(&self, state: &mut GuiWidgetState, key: &KeyEvent) {
        match key.code {
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(engine_events::KeyModifiers::CONTROL)
                    && !key.modifiers.contains(engine_events::KeyModifiers::ALT)
                    && state.text.chars().count() < self.max_length.max(1)
                    && !ch.is_control()
                {
                    state.text.push(ch);
                }
            }
            KeyCode::Backspace => {
                state.text.pop();
            }
            KeyCode::Enter => {
                state.submitted = true;
            }
            KeyCode::Esc => {}
            _ => {}
        }
    }
    fn visual_sync(&self, state: &GuiWidgetState) -> Option<VisualSync> {
        if self.text_sprite.is_empty() {
            return None;
        }
        let text = if state.text.is_empty() {
            self.placeholder.clone()
        } else {
            state.text.clone()
        };
        Some(VisualSync::single(VisualSyncAction::SetText {
            sprite_alias: self.text_sprite.clone(),
            text,
        }))
    }
    fn set_bounds(&mut self, rect: WidgetRect) {
        if !self.follow_layout {
            return;
        }
        self.x = rect.x;
        self.y = rect.y;
        self.w = rect.w.max(1);
        self.h = rect.h.max(1);
    }
}

/// Editable single-line numeric field.
#[derive(Debug, Clone)]
pub struct NumberInputControl {
    pub id: String,
    pub sprite: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub text_sprite: String,
    pub placeholder: String,
    pub value: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub max_length: usize,
    pub follow_layout: bool,
}

impl NumberInputControl {
    fn clamp_text_value(&self, state: &mut GuiWidgetState) {
        let trimmed = state.text.trim();
        if trimmed.is_empty() || matches!(trimmed, "-" | "." | "-.") {
            state.value = 0.0;
            return;
        }
        if let Ok(mut value) = trimmed.parse::<f64>() {
            if let Some(min) = self.min {
                value = value.max(min);
            }
            if let Some(max) = self.max {
                value = value.min(max);
            }
            if let Some(step) = self.step.filter(|s| *s > f64::EPSILON) {
                let base = self.min.unwrap_or(0.0);
                value = ((value - base) / step).round() * step + base;
                if let Some(min) = self.min {
                    value = value.max(min);
                }
                if let Some(max) = self.max {
                    value = value.min(max);
                }
            }
            state.value = value;
            state.text = value.to_string();
        }
    }
}

impl GuiControl for NumberInputControl {
    fn id(&self) -> &str {
        &self.id
    }
    fn sprite(&self) -> &str {
        &self.sprite
    }
    fn bounds(&self, _state: &GuiWidgetState) -> Option<WidgetRect> {
        Some(WidgetRect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
        })
    }
    fn initial_state(&self) -> GuiWidgetState {
        let mut state = GuiWidgetState {
            text: self.value.clone(),
            ..Default::default()
        };
        self.clamp_text_value(&mut state);
        state
    }
    fn on_mouse_down(&self, state: &mut GuiWidgetState, _x: f32, _y: f32) {
        state.pressed = true;
    }
    fn on_drag(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}
    fn on_mouse_up(&self, state: &mut GuiWidgetState, _x: f32, _y: f32, still_hovered: bool) {
        state.pressed = false;
        if still_hovered {
            state.clicked = true;
        }
    }
    fn wants_keyboard_focus(&self) -> bool {
        true
    }
    fn on_key_down(&self, state: &mut GuiWidgetState, key: &KeyEvent) {
        match key.code {
            KeyCode::Char(ch) => {
                if !key.modifiers.contains(engine_events::KeyModifiers::CONTROL)
                    && !key.modifiers.contains(engine_events::KeyModifiers::ALT)
                    && state.text.chars().count() < self.max_length.max(1)
                    && !ch.is_control()
                {
                    let allow = ch.is_ascii_digit()
                        || (ch == '.' && !state.text.contains('.'))
                        || (ch == '-' && state.text.is_empty());
                    if allow {
                        state.text.push(ch);
                        if let Ok(value) = state.text.parse::<f64>() {
                            state.value = value;
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                state.text.pop();
                if let Ok(value) = state.text.parse::<f64>() {
                    state.value = value;
                } else if state.text.is_empty() {
                    state.value = 0.0;
                }
            }
            KeyCode::Enter => {
                self.clamp_text_value(state);
                state.submitted = true;
            }
            KeyCode::Esc => {}
            _ => {}
        }
    }
    fn visual_sync(&self, state: &GuiWidgetState) -> Option<VisualSync> {
        if self.text_sprite.is_empty() {
            return None;
        }
        let text = if state.text.is_empty() {
            self.placeholder.clone()
        } else {
            state.text.clone()
        };
        Some(VisualSync::single(VisualSyncAction::SetText {
            sprite_alias: self.text_sprite.clone(),
            text,
        }))
    }
    fn set_bounds(&mut self, rect: WidgetRect) {
        if !self.follow_layout {
            return;
        }
        self.x = rect.x;
        self.y = rect.y;
        self.w = rect.w.max(1);
        self.h = rect.h.max(1);
    }
}

/// Compact single-choice popup selector.
#[derive(Debug, Clone)]
pub struct DropdownControl {
    pub id: String,
    pub sprite: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub options: Vec<ChoiceOption>,
    pub selected: usize,
    pub popup_sprite: String,
    pub label_sprite: String,
    pub option_sprites: Vec<String>,
    pub popup_above: bool,
    pub follow_layout: bool,
}

impl DropdownControl {
    fn popup_bounds(&self) -> WidgetRect {
        let popup_h = self.h.max(1) * self.options.len().max(1) as i32;
        WidgetRect {
            x: self.x,
            y: if self.popup_above {
                self.y - popup_h
            } else {
                self.y + self.h
            },
            w: self.w,
            h: popup_h,
        }
    }
}

impl GuiControl for DropdownControl {
    fn id(&self) -> &str {
        &self.id
    }
    fn sprite(&self) -> &str {
        &self.sprite
    }
    fn bounds(&self, state: &GuiWidgetState) -> Option<WidgetRect> {
        if state.open {
            let popup = self.popup_bounds();
            let top = self.y.min(popup.y);
            let bottom = (self.y + self.h).max(popup.y + popup.h);
            Some(WidgetRect {
                x: self.x,
                y: top,
                w: self.w,
                h: bottom - top,
            })
        } else {
            Some(WidgetRect {
                x: self.x,
                y: self.y,
                w: self.w,
                h: self.h,
            })
        }
    }
    fn initial_state(&self) -> GuiWidgetState {
        let selected = self.selected.min(self.options.len().saturating_sub(1));
        GuiWidgetState {
            value: selected as f64,
            selected_index: Some(selected),
            open: false,
            ..Default::default()
        }
    }
    fn on_mouse_down(&self, state: &mut GuiWidgetState, _x: f32, _y: f32) {
        state.pressed = true;
    }
    fn on_drag(&self, _state: &mut GuiWidgetState, _x: f32, _y: f32) {}
    fn on_mouse_up(&self, state: &mut GuiWidgetState, x: f32, y: f32, still_hovered: bool) {
        state.pressed = false;
        if !still_hovered {
            state.open = false;
            return;
        }
        if state.open {
            if let Some(idx) = self.options.get(0).and_then(|_| {
                let popup = self.popup_bounds();
                if popup.hit_test(x, y) {
                    let rel_y = (y - popup.y as f32).max(0.0);
                    let row_h = self.h.max(1) as f32;
                    let idx = (rel_y / row_h).floor() as usize;
                    Some(idx.min(self.options.len().saturating_sub(1)))
                } else {
                    None
                }
            }) {
                state.value = idx as f64;
                state.selected_index = Some(idx);
                state.changed = true;
            }
            state.open = false;
            state.clicked = true;
        } else {
            state.open = true;
            state.clicked = true;
        }
    }
    fn visual_sync(&self, state: &GuiWidgetState) -> Option<VisualSync> {
        let mut actions = Vec::new();
        if !self.popup_sprite.is_empty() {
            actions.push(VisualSyncAction::SetVisible {
                sprite_alias: self.popup_sprite.clone(),
                visible: state.open,
            });
        }
        if !self.label_sprite.is_empty() {
            let label = state
                .selected_index
                .and_then(|idx| self.options.get(idx))
                .map(|opt| opt.label.clone())
                .unwrap_or_default();
            actions.push(VisualSyncAction::SetText {
                sprite_alias: self.label_sprite.clone(),
                text: label,
            });
        }
        for (idx, sprite_alias) in self.option_sprites.iter().enumerate() {
            if sprite_alias.is_empty() {
                continue;
            }
            let text = self
                .options
                .get(idx)
                .map(|opt| {
                    let prefix = if state.selected_index == Some(idx) {
                        ">"
                    } else {
                        " "
                    };
                    format!("{prefix} {}", opt.label)
                })
                .unwrap_or_default();
            actions.push(VisualSyncAction::SetVisible {
                sprite_alias: sprite_alias.clone(),
                visible: state.open && idx < self.options.len(),
            });
            actions.push(VisualSyncAction::SetText {
                sprite_alias: sprite_alias.clone(),
                text,
            });
        }
        if actions.is_empty() {
            None
        } else {
            Some(VisualSync { actions })
        }
    }
    fn set_bounds(&mut self, rect: WidgetRect) {
        if !self.follow_layout {
            return;
        }
        self.x = rect.x;
        self.y = rect.y;
        self.w = rect.w.max(1);
        self.h = rect.h.max(1);
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_drag_computes_value() {
        let slider = SliderControl {
            id: "s".into(),
            sprite: "track".into(),
            x: 10,
            y: 10,
            w: 100,
            h: 12,
            min: 0.0,
            max: 255.0,
            value: 0.0,
            hit_padding: 0,
            handle: "h".into(),
            follow_layout: true,
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
            id: "s".into(),
            sprite: "track".into(),
            x: 50,
            y: 10,
            w: 190,
            h: 12,
            min: 0.0,
            max: 255.0,
            value: 0.0,
            hit_padding: 0,
            handle: "h".into(),
            follow_layout: true,
        };
        let state = GuiWidgetState {
            value: 127.5,
            ..Default::default()
        };
        let sync = slider.visual_sync(&state).unwrap();
        let VisualSyncAction::OffsetX {
            sprite_alias,
            offset_x,
        } = &sync.actions[0]
        else {
            panic!("expected offset sync");
        };
        assert_eq!(sprite_alias, "h");
        // 127.5/255 * 190 ≈ 95
        assert!((*offset_x - 95).abs() <= 1);
    }

    #[test]
    fn slider_set_bounds_updates_geometry() {
        let mut slider = SliderControl {
            id: "s".into(),
            sprite: "track".into(),
            x: 50,
            y: 10,
            w: 190,
            h: 12,
            min: 0.0,
            max: 255.0,
            value: 0.0,
            hit_padding: 0,
            handle: "h".into(),
            follow_layout: true,
        };
        slider.set_bounds(WidgetRect {
            x: 100,
            y: 20,
            w: 150,
            h: 10,
        });
        assert_eq!(slider.x, 100);
        assert_eq!(slider.y, 20);
        assert_eq!(slider.w, 150);
        assert_eq!(slider.h, 10);
    }

    #[test]
    fn button_click_fires_on_release_inside() {
        let btn = ButtonControl {
            id: "b".into(),
            sprite: "b".into(),
            x: 0,
            y: 0,
            w: 50,
            h: 20,
            follow_layout: true,
        };
        let mut ws = GuiWidgetState::default();
        btn.on_mouse_down(&mut ws, 10.0, 5.0);
        assert!(ws.pressed);
        assert!(!ws.clicked);
        btn.on_mouse_up(&mut ws, 10.0, 5.0, true);
        assert!(!ws.pressed);
        assert!(ws.clicked);
    }

    #[test]
    fn button_no_click_on_release_outside() {
        let btn = ButtonControl {
            id: "b".into(),
            sprite: "b".into(),
            x: 0,
            y: 0,
            w: 50,
            h: 20,
            follow_layout: true,
        };
        let mut ws = GuiWidgetState::default();
        btn.on_mouse_down(&mut ws, 10.0, 5.0);
        btn.on_mouse_up(&mut ws, 100.0, 50.0, false);
        assert!(!ws.clicked);
    }

    #[test]
    fn toggle_flips_value() {
        let toggle = ToggleControl {
            id: "t".into(),
            sprite: "t".into(),
            x: 0,
            y: 0,
            w: 100,
            h: 16,
            initial_on: false,
            follow_layout: true,
        };
        let mut ws = GuiWidgetState {
            value: 0.0,
            ..Default::default()
        };
        toggle.on_mouse_up(&mut ws, 10.0, 5.0, true);
        assert!((ws.value - 1.0).abs() < f64::EPSILON);
        assert!(ws.changed);

        ws.changed = false;
        toggle.on_mouse_up(&mut ws, 10.0, 5.0, true);
        assert!(ws.value.abs() < f64::EPSILON);
    }

    #[test]
    fn radio_group_selects_segment_and_updates_visuals() {
        let radio = RadioGroupControl {
            id: "mode".into(),
            sprite: "group".into(),
            x: 0,
            y: 0,
            w: 120,
            h: 20,
            options: vec![
                ChoiceOption::new("a".into(), "A".into()),
                ChoiceOption::new("b".into(), "B".into()),
                ChoiceOption::new("c".into(), "C".into()),
            ],
            selected: 0,
            selected_sprites: vec!["sel-a".into(), "sel-b".into(), "sel-c".into()],
            follow_layout: true,
        };
        let mut ws = radio.initial_state();
        radio.on_mouse_down(&mut ws, 85.0, 10.0);
        assert_eq!(ws.selected_index, Some(2));
        let sync = radio.visual_sync(&ws).expect("sync");
        assert_eq!(sync.actions.len(), 3);
    }

    #[test]
    fn dropdown_visual_sync_uses_selected_label_and_open_state() {
        let dropdown = DropdownControl {
            id: "preset".into(),
            sprite: "trigger".into(),
            x: 10,
            y: 20,
            w: 100,
            h: 16,
            options: vec![
                ChoiceOption::new("earth".into(), "Earth".into()),
                ChoiceOption::new("mars".into(), "Mars".into()),
            ],
            selected: 1,
            popup_sprite: "popup".into(),
            label_sprite: "label".into(),
            option_sprites: vec!["opt-0".into(), "opt-1".into()],
            popup_above: false,
            follow_layout: true,
        };
        let mut ws = dropdown.initial_state();
        ws.open = true;
        let sync = dropdown.visual_sync(&ws).expect("sync");
        assert_eq!(sync.actions.len(), 6);
    }

    #[test]
    fn widget_rect_hit_test() {
        let r = WidgetRect {
            x: 10,
            y: 20,
            w: 100,
            h: 50,
        };
        assert!(r.hit_test(10.0, 20.0));
        assert!(r.hit_test(109.0, 69.0));
        assert!(!r.hit_test(110.0, 20.0));
        assert!(!r.hit_test(5.0, 20.0));
    }
}

use super::*;

impl SceneRuntime {
    pub fn ui_theme_id(&self) -> Option<&str> {
        self.ui_state.theme_id.as_deref()
    }

    pub fn ui_theme_style(&self) -> Option<UiThemeStyle> {
        self.ui_state.theme_style
    }

    /// Store the raw key event for the current frame so scripts can read it via `key.*`.
    pub fn set_last_raw_key(&mut self, key: RawKeyEvent) {
        self.ui_state.last_raw_key = Some(key);
    }

    /// Clear raw key state at the start of each frame.
    pub fn clear_last_raw_key(&mut self) {
        self.ui_state.last_raw_key = None;
    }

    /// Track a key-down in the held-key set for script-side polling (`input.down(...)`).
    pub fn set_key_down(&mut self, key: &RawKeyEvent) {
        let normalized = normalize_key_code(&key.code);
        if !normalized.is_empty() {
            self.ui_state.keys_down.insert(normalized);
        }
    }

    /// Track a key-up in the held-key set for script-side polling (`input.down(...)`).
    pub fn set_key_up(&mut self, key: &RawKeyEvent) {
        let normalized = normalize_key_code(&key.code);
        if !normalized.is_empty() {
            self.ui_state.keys_down.remove(&normalized);
        }
    }

    /// Clear all held keys (used on focus-loss to avoid stuck movement input).
    pub fn clear_keys_down(&mut self) {
        self.ui_state.keys_down.clear();
        if let Some(state) = self.free_look_camera.as_mut() {
            state.held_keys.clear();
            state.last_mouse_pos = None;
        }
    }

    /// Returns a clone of the current held-key set for behavior context.
    pub fn keys_down_snapshot(&self) -> std::collections::HashSet<String> {
        self.ui_state.keys_down.clone()
    }

    pub fn frame_scroll_y(&self) -> f32 {
        self.ui_state.scroll_y
    }

    pub fn frame_ctrl_scroll_y(&self) -> f32 {
        self.ui_state.ctrl_scroll_y
    }

    pub fn set_frame_scroll_state(&mut self, scroll_y: f32, ctrl_scroll_y: f32) {
        self.ui_state.scroll_y = if scroll_y.is_finite() { scroll_y } else { 0.0 };
        self.ui_state.ctrl_scroll_y = if ctrl_scroll_y.is_finite() {
            ctrl_scroll_y
        } else {
            0.0
        };
    }

    pub fn accumulate_frame_scroll_state(&mut self, scroll_y: f32, ctrl_scroll_y: f32) {
        if scroll_y.is_finite() {
            self.ui_state.scroll_y += scroll_y;
        }
        if ctrl_scroll_y.is_finite() {
            self.ui_state.ctrl_scroll_y += ctrl_scroll_y;
        }
    }

    pub fn focused_ui_target_id(&self) -> Option<&str> {
        if self.ui_state.focus_order.is_empty() {
            return None;
        }
        self.ui_state
            .focus_order
            .get(self.ui_state.focused_index)
            .map(String::as_str)
    }

    pub fn handle_ui_focus_keys(&mut self, key_presses: &[KeyEvent]) -> bool {
        if key_presses.is_empty() || self.ui_state.focus_order.len() <= 1 {
            return false;
        }
        let mut changed = false;
        for key in key_presses {
            match key.code {
                KeyCode::BackTab => {
                    self.focus_prev();
                    changed = true;
                }
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        self.focus_prev();
                    } else {
                        self.focus_next();
                    }
                    changed = true;
                }
                _ => {}
            }
        }
        changed
    }

    pub fn ui_last_submit_snapshot(&self) -> Option<(u64, String, String)> {
        self.ui_state.last_submit.as_ref().map(|ev| {
            (
                self.ui_state.submit_seq,
                ev.target_id.clone(),
                ev.text.clone(),
            )
        })
    }

    pub fn ui_last_change_snapshot(&self) -> Option<(u64, String, String)> {
        self.ui_state.last_change.as_ref().map(|ev| {
            (
                self.ui_state.change_seq,
                ev.target_id.clone(),
                ev.text.clone(),
            )
        })
    }

    pub fn last_raw_key_snapshot(&self) -> Option<RawKeyEvent> {
        self.ui_state.last_raw_key.clone()
    }

    pub fn reset_frame_state(&mut self) {
        for state in self.object_states.values_mut() {
            *state = ObjectRuntimeState::default();
        }
        self.ui_state.sidecar_io = SidecarIoFrameState::default();
    }

    pub(crate) fn initialize_ui_state(&mut self) {
        let focus_order = normalize_focus_order(&self.scene.ui.focus_order);
        self.ui_state.focus_order = focus_order;
        self.ui_state.focused_index = 0;
        let resolved_theme = resolve_ui_theme_or_default(self.scene.ui.theme.as_deref());
        self.ui_state.theme_id = Some(resolved_theme.id.to_string());
        self.ui_state.theme_style = Some(resolved_theme);
        self.ui_state.last_submit = None;
        self.ui_state.last_change = None;
    }

    fn focus_next(&mut self) {
        let total = self.ui_state.focus_order.len();
        if total <= 1 {
            return;
        }
        self.ui_state.focused_index = (self.ui_state.focused_index + 1) % total;
    }

    fn focus_prev(&mut self) {
        let total = self.ui_state.focus_order.len();
        if total <= 1 {
            return;
        }
        self.ui_state.focused_index = if self.ui_state.focused_index == 0 {
            total - 1
        } else {
            self.ui_state.focused_index - 1
        };
    }

    #[allow(dead_code)]
    pub(crate) fn is_ui_target_focused(&self, target_id: &str) -> bool {
        self.focused_ui_target_id()
            .map(|focused| focused == target_id)
            .unwrap_or(true)
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_text_layout(&self, sprite_id: &str) -> Option<TextLayoutSpec> {
        self.scene
            .layers
            .iter()
            .find_map(|layer| find_text_layout_recursive(&layer.sprites, sprite_id))
    }
}

fn normalize_focus_order(input: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for value in input {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if out.iter().all(|existing| existing != trimmed) {
            out.push(trimmed.to_string());
        }
    }
    out
}

fn normalize_key_code(code: &str) -> String {
    if code == " " {
        return " ".to_string();
    }
    let trimmed = code.trim();
    if trimmed.len() == 1 {
        return trimmed.to_ascii_lowercase();
    }
    trimmed.to_string()
}

#[allow(dead_code)]
pub(crate) fn find_panel_layout_recursive(
    sprites: &[Sprite],
    panel_id: &str,
    scene_width: u16,
) -> Option<PanelLayoutSpec> {
    for sprite in sprites {
        match sprite {
            Sprite::Panel {
                id: Some(id),
                width,
                width_percent,
                height,
                border_width,
                padding,
                children,
                ..
            } => {
                if id == panel_id {
                    let computed_width = if let Some(explicit) = *width {
                        explicit
                    } else if let Some(percent) = *width_percent {
                        ((u32::from(scene_width) * u32::from(percent.clamp(1, 100))) / 100).max(1)
                            as u16
                    } else {
                        scene_width
                    };
                    return Some(PanelLayoutSpec {
                        width: computed_width.max(1),
                        border_width: *border_width,
                        padding: *padding,
                        height: height.unwrap_or(3).max(1),
                    });
                }
                if let Some(layout) = find_panel_layout_recursive(children, panel_id, scene_width) {
                    return Some(layout);
                }
            }
            Sprite::Panel { children, .. }
            | Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. } => {
                if let Some(layout) = find_panel_layout_recursive(children, panel_id, scene_width) {
                    return Some(layout);
                }
            }
            Sprite::Text { .. }
            | Sprite::Image { .. }
            | Sprite::Obj { .. }
            | Sprite::Planet { .. }
            | Sprite::Scene3D { .. }
            | Sprite::Vector { .. } => {}
        }
    }
    None
}

#[allow(dead_code)]
pub(crate) fn set_panel_height_recursive(
    sprites: &mut [Sprite],
    panel_id: &str,
    next_height: u16,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Panel {
                id: Some(id),
                height,
                children,
                ..
            } => {
                if id == panel_id {
                    *height = Some(next_height.max(1));
                    *updated = true;
                }
                set_panel_height_recursive(children, panel_id, next_height, updated);
            }
            Sprite::Panel { children, .. }
            | Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. } => {
                set_panel_height_recursive(children, panel_id, next_height, updated)
            }
            Sprite::Text { .. }
            | Sprite::Image { .. }
            | Sprite::Obj { .. }
            | Sprite::Planet { .. }
            | Sprite::Scene3D { .. }
            | Sprite::Vector { .. } => {}
        }
    }
}

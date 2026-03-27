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
        self.ui_state.last_raw_key = None;
        self.ui_state.sidecar_io = SidecarIoFrameState::default();
    }

    pub(crate) fn initialize_ui_state(&mut self) {
        let mut focus_order = normalize_focus_order(&self.scene.ui.focus_order);
        if focus_order.is_empty() {
            if let Some(prompt_id) = self
                .terminal_shell_state
                .as_ref()
                .map(|state| state.controls.prompt_sprite_id.clone())
            {
                focus_order.push(prompt_id);
            }
        }
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

    pub(crate) fn is_ui_target_focused(&self, target_id: &str) -> bool {
        self.focused_ui_target_id()
            .map(|focused| focused == target_id)
            .unwrap_or(true)
    }

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
            | Sprite::Scene3D { .. } => {}
        }
    }
    None
}

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
            | Sprite::Scene3D { .. } => {}
        }
    }
}

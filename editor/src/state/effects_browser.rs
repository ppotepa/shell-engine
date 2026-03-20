//! Effects browser command dispatch.

use crate::domain::effect_params::{self, EffectParamSpec, EffectParamValue};
use crate::domain::effects_preview_scene;
use crate::input::commands::Command;

use super::{focus::FocusPane, now_millis, AppState, EffectsCodeTab, SidebarItem};

impl AppState {
    pub(super) fn restart_effect_preview_clock(&mut self) {
        self.effects.effects_preview_started_at_ms = now_millis();
    }

    /// Returns the parameter specifications for the currently selected effect.
    pub fn effect_param_specs(&self) -> &'static [EffectParamSpec] {
        self.selected_builtin_effect()
            .map(effect_params::effect_param_specs)
            .unwrap_or(&[])
    }

    /// Returns the spec for the currently focused effect parameter, if any.
    pub fn selected_effect_param_spec(&self) -> Option<&'static EffectParamSpec> {
        self.effect_param_specs()
            .get(self.effects.effect_param_cursor)
    }

    /// Returns the current value for the given parameter, preferring user overrides.
    pub fn effect_param_value(&self, spec: &EffectParamSpec) -> EffectParamValue {
        if let Some(value) = self.effects.effect_param_overrides.get(spec.name) {
            return *value;
        }

        self.selected_builtin_effect()
            .map(effect_params::default_effect_params)
            .and_then(|params| effect_params::effect_param_value(&params, spec.name))
            .unwrap_or_else(|| spec.default_value())
    }

    pub(super) fn sync_effect_param_cursor(&mut self) {
        let len = self.effect_param_specs().len();
        if len == 0 {
            self.effects.effect_param_cursor = 0;
        } else {
            self.effects.effect_param_cursor = self.effects.effect_param_cursor.min(len - 1);
        }
    }

    fn reset_selected_effect_preview(&mut self) {
        self.effects.effect_param_cursor = 0;
        self.effects.effect_param_overrides.clear();
        self.effects.effects_code_scroll = 0;
        self.effects.effects_code_tab = EffectsCodeTab::Info;
        self.sync_effect_preview_scene_yaml();
        self.restart_effect_preview_clock();
    }

    pub(super) fn move_effect_selection(&mut self, next_cursor: usize) {
        if next_cursor != self.effects.effect_cursor {
            self.effects.effect_cursor = next_cursor;
            self.reset_selected_effect_preview();
        } else {
            self.restart_effect_preview_clock();
        }
    }

    pub(super) fn move_effect_param_cursor(&mut self, delta: isize) {
        let len = self.effect_param_specs().len();
        if len == 0 {
            self.effects.effect_param_cursor = 0;
            return;
        }

        let next = (self.effects.effect_param_cursor as isize + delta).clamp(0, (len - 1) as isize);
        self.effects.effect_param_cursor = next as usize;
    }

    pub(super) fn adjust_selected_effect_param(&mut self, delta_dir: f32) {
        let Some(spec) = self.selected_effect_param_spec().copied() else {
            return;
        };

        let current = self.effect_param_value(&spec).as_float();
        let next = spec.adjust(current, delta_dir);
        self.effects
            .effect_param_overrides
            .insert(spec.name.to_string(), next);
        self.sync_effect_preview_scene_yaml();
        self.restart_effect_preview_clock();
        self.status = format!("{}: {}", spec.label, spec.render_value(next.as_float()));
    }

    pub(super) fn activate_effects_browser(&mut self) {
        self.reset_scene_fullscreen_state();
        self.sidebar.active = SidebarItem::Search;
        self.sidebar.visible = true;
        self.effects.effects_live_preview = true;
        self.sync_effect_preview_scene_yaml();
        self.restart_effect_preview_clock();
        self.sync_effect_param_cursor();
        self.status = "Effects Browser: LIVE preview ON | Tab focus | j/k effect | Enter controls"
            .to_string();
    }

    pub(super) fn toggle_effects_preview(&mut self) {
        if self.sidebar.active != SidebarItem::Search {
            return;
        }

        self.effects.effects_live_preview = !self.effects.effects_live_preview;
        self.sync_effect_preview_scene_yaml();
        self.restart_effect_preview_clock();
        self.sync_effect_param_cursor();

        self.status = if self.effects.effects_live_preview {
            "Effects Browser: LIVE preview ON | Tab focus | ↑/↓ param | ←/→ adjust".to_string()
        } else {
            "Effects Browser: LIVE preview OFF | F enables live buffer preview".to_string()
        };
    }

    pub(super) fn sync_effect_preview_scene_yaml(&mut self) {
        let Some(effect_name) = self.selected_builtin_effect() else {
            self.effects.effects_preview_scene_yaml.clear();
            return;
        };

        let mut params = effect_params::default_effect_params(effect_name);
        effect_params::apply_overrides(
            effect_name,
            &self.effects.effect_param_overrides,
            &mut params,
        );
        self.effects.effects_preview_scene_yaml =
            effects_preview_scene::build_preview_scene_yaml_default(effect_name, &params);
    }

    pub(super) fn handle_effects_browser_command(&mut self, cmd: Command) -> bool {
        if self.sidebar.active != SidebarItem::Search {
            return false;
        }
        match cmd {
            Command::Up => {
                if self.effects.effects_live_preview && self.focus == FocusPane::Inspector {
                    self.move_effect_param_cursor(-1);
                } else if self.effects.effects_live_preview && self.focus == FocusPane::Browser {
                    self.effects.effects_code_scroll =
                        self.effects.effects_code_scroll.saturating_sub(1);
                } else {
                    self.move_effect_selection(self.effects.effect_cursor.saturating_sub(1));
                }
                true
            }
            Command::Down => {
                if self.effects.effects_live_preview && self.focus == FocusPane::Inspector {
                    self.move_effect_param_cursor(1);
                } else if self.effects.effects_live_preview && self.focus == FocusPane::Browser {
                    self.effects.effects_code_scroll =
                        self.effects.effects_code_scroll.saturating_add(1);
                } else {
                    let max = self.effects.builtin_effects.len().saturating_sub(1);
                    self.move_effect_selection((self.effects.effect_cursor + 1).min(max));
                }
                true
            }
            Command::Left => {
                if self.effects.effects_live_preview && self.focus == FocusPane::Inspector {
                    self.adjust_selected_effect_param(-1.0);
                }
                true
            }
            Command::Right => {
                if self.effects.effects_live_preview && self.focus == FocusPane::Inspector {
                    self.adjust_selected_effect_param(1.0);
                }
                true
            }
            Command::EnterFile => {
                if !self.effects.effects_live_preview {
                    self.effects.effects_live_preview = true;
                }
                self.focus = FocusPane::Inspector;
                self.sync_effect_param_cursor();
                self.restart_effect_preview_clock();
                self.status =
                    "Effects Browser: controls focused | ↑/↓ param | ←/→ adjust | F toggle"
                        .to_string();
                true
            }
            Command::ToggleEffectsPreview | Command::SceneFullscreenHoldStart => {
                self.toggle_effects_preview();
                true
            }
            Command::NextCodeTab => {
                self.effects.effects_code_scroll = 0;
                self.effects.effects_code_tab = self.effects.effects_code_tab.next();
                true
            }
            Command::PrevCodeTab => {
                self.effects.effects_code_scroll = 0;
                self.effects.effects_code_tab = self.effects.effects_code_tab.prev();
                true
            }
            _ => false,
        }
    }
}

//! Lifecycle control handlers for terminal shell, object viewer, and size tester.
//!
//! Provides input routing for:
//! - Terminal shell key presses and navigation
//! - Object viewer controls (rotation, zoom, mode switching)
//! - Terminal size testing and presets
//!
//! These controls are runtime-specific and not embedded in the scene model.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalShellRoute {
    Absent,
    Passive,
    ConsumedInput,
    BackRequested(Option<String>),
}

impl SceneRuntime {
    pub fn terminal_size_presets(&self) -> Option<Vec<(u16, u16)>> {
        let cfg = self.scene.input.terminal_size_tester.clone()?;
        let mut out = Vec::new();
        for preset in cfg.presets {
            if let Some(engine_runtime::RenderSize::Fixed { width, height }) =
                engine_runtime::parse_render_size(&preset)
            {
                out.push((width, height));
            }
        }
        if out.is_empty() {
            out.extend([(80, 24), (100, 30), (120, 36), (160, 48)]);
        }
        Some(out)
    }

    pub fn handle_terminal_shell_lifecycle_keys(
        &mut self,
        key_presses: &[KeyEvent],
    ) -> TerminalShellRoute {
        if !self.has_terminal_shell() {
            return TerminalShellRoute::Absent;
        }
        if self.terminal_shell_back_requested(key_presses) {
            return TerminalShellRoute::BackRequested(self.scene().next.clone());
        }
        if self.handle_terminal_shell_keys(key_presses) {
            return TerminalShellRoute::ConsumedInput;
        }
        TerminalShellRoute::Passive
    }

    pub fn apply_obj_viewer_key_presses(&mut self, key_presses: &[KeyEvent]) -> bool {
        let Some(sprite_id) = self
            .scene
            .input
            .obj_viewer
            .as_ref()
            .map(|cfg| cfg.sprite_id.clone())
        else {
            return false;
        };

        if key_presses
            .iter()
            .any(|key| matches!(key.code, KeyCode::Enter))
        {
            return false;
        }

        let orbit_active = self.is_obj_orbit_active(&sprite_id);
        let mut zoom_delta = 0.0f32;
        let mut mode_switch: Option<SceneRenderedMode> = None;
        let mut toggle_wireframe = false;
        let mut toggle_orbit = false;
        let mut pan_dx = 0.0f32;
        let mut pan_dy = 0.0f32;

        for key in key_presses {
            match key.code {
                KeyCode::Char('a') | KeyCode::Char('A') => zoom_delta += 0.1,
                KeyCode::Char('z') | KeyCode::Char('Z') => zoom_delta -= 0.1,
                KeyCode::Char('1') | KeyCode::Char('6') => {
                    mode_switch = Some(SceneRenderedMode::Cell)
                }
                KeyCode::Char('2') | KeyCode::Char('7') => {
                    mode_switch = Some(SceneRenderedMode::HalfBlock)
                }
                KeyCode::Char('3') | KeyCode::Char('8') => {
                    mode_switch = Some(SceneRenderedMode::QuadBlock)
                }
                KeyCode::Char('4') => mode_switch = Some(SceneRenderedMode::Braille),
                KeyCode::Char('5') => toggle_wireframe = true,
                KeyCode::Char('o') | KeyCode::Char('O') => toggle_orbit = true,
                KeyCode::Left if !orbit_active => pan_dx -= 0.04,
                KeyCode::Right if !orbit_active => pan_dx += 0.04,
                KeyCode::Up if !orbit_active => pan_dy += 0.04,
                KeyCode::Down if !orbit_active => pan_dy -= 0.04,
                _ => {}
            }
        }

        if zoom_delta != 0.0 {
            let _ = self.adjust_obj_scale(&sprite_id, zoom_delta);
        }
        if let Some(mode) = mode_switch {
            self.set_scene_rendered_mode(mode);
        }
        if toggle_wireframe {
            let _ = self.toggle_obj_surface_mode(&sprite_id);
        }
        if toggle_orbit {
            let _ = self.toggle_obj_orbit(&sprite_id);
            self.set_obj_last_mouse_pos(&sprite_id, None);
        }
        if pan_dx != 0.0 || pan_dy != 0.0 {
            self.apply_obj_camera_pan(&sprite_id, pan_dx, pan_dy);
        }

        true
    }

    pub fn apply_obj_viewer_mouse_moves(&mut self, mouse_moves: &[(u16, u16)]) {
        let Some(sprite_id) = self
            .scene
            .input
            .obj_viewer
            .as_ref()
            .map(|cfg| cfg.sprite_id.clone())
        else {
            return;
        };

        if self.is_obj_orbit_active(&sprite_id) {
            if let Some(last) = mouse_moves.last() {
                self.set_obj_last_mouse_pos(&sprite_id, Some(*last));
            }
            return;
        }

        let Some((mut prev_col, mut prev_row)) = self.obj_last_mouse_pos(&sprite_id) else {
            if let Some(last) = mouse_moves.last() {
                self.set_obj_last_mouse_pos(&sprite_id, Some(*last));
            }
            return;
        };

        let mut total_dyaw = 0.0f32;
        let mut total_dpitch = 0.0f32;
        for &(col, row) in mouse_moves {
            let dc = col as f32 - prev_col as f32;
            let dr = row as f32 - prev_row as f32;
            total_dyaw += dc * 1.8;
            total_dpitch += dr * 2.8;
            prev_col = col;
            prev_row = row;
        }

        self.set_obj_last_mouse_pos(&sprite_id, Some((prev_col, prev_row)));
        if total_dyaw != 0.0 || total_dpitch != 0.0 {
            self.apply_obj_camera_look(&sprite_id, total_dyaw, total_dpitch);
        }
    }
}

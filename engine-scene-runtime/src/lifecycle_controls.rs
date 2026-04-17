//! Lifecycle control handlers for object viewer and other runtime helpers.
//!
//! Provides input routing for:
//! - Object viewer controls (rotation, zoom)
//!
//! These controls are runtime-specific and not embedded in the scene model.

use super::*;

impl SceneRuntime {
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
        let mut toggle_wireframe = false;
        let mut toggle_orbit = false;
        let mut pan_dx = 0.0f32;
        let mut pan_dy = 0.0f32;

        for key in key_presses {
            match key.code {
                KeyCode::Char('a') | KeyCode::Char('A') => zoom_delta += 0.1,
                KeyCode::Char('z') | KeyCode::Char('Z') => zoom_delta -= 0.1,
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

    pub fn apply_obj_viewer_mouse_moves(&mut self, mouse_moves: &[(f32, f32)]) {
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

        let Some((mut prev_x, mut prev_y)) = self.obj_last_mouse_pos(&sprite_id) else {
            if let Some(last) = mouse_moves.last() {
                self.set_obj_last_mouse_pos(&sprite_id, Some(*last));
            }
            return;
        };

        let mut total_dyaw = 0.0f32;
        let mut total_dpitch = 0.0f32;
        for &(x, y) in mouse_moves {
            let dc = x - prev_x;
            let dr = y - prev_y;
            total_dyaw += dc * 1.8;
            total_dpitch += dr * 2.8;
            prev_x = x;
            prev_y = y;
        }

        self.set_obj_last_mouse_pos(&sprite_id, Some((prev_x, prev_y)));
        if total_dyaw != 0.0 || total_dpitch != 0.0 {
            self.apply_obj_camera_look(&sprite_id, total_dyaw, total_dpitch);
        }
    }

    /// Feed input events to the GUI system and update `gui_state`.
    /// Handle sprite positions are synced by the behavior system after reset_frame_state.
    pub fn update_gui(&mut self, events: Vec<engine_events::InputEvent>) {
        engine_gui::GuiSystem::update(&self.gui_widgets, &mut self.gui_state, &events);
        self.cached_gui_state = None;
    }

    /// Positions every widget's managed sprites at the correct offset for the current value.
    /// Each control's [`visual_sync`](engine_gui::GuiControl::visual_sync) decides what to sync.
    pub fn sync_widget_visuals(&mut self) {
        let resolver = std::sync::Arc::clone(&self.resolver_cache);
        for widget in &self.gui_widgets {
            let value = self.gui_state.value(widget.id());
            if let Some(sync) = widget.visual_sync(value) {
                let object_id = resolver
                    .resolve_alias(&sync.sprite_alias)
                    .unwrap_or(&sync.sprite_alias);
                if let Some(obj_state) = self.object_states.get_mut(object_id) {
                    obj_state.offset_x = sync.offset_x;
                }
            }
        }
    }

    /// Return a shared, cheaply-clonable snapshot of the current GUI state.
    pub fn gui_state_arc(&mut self) -> std::sync::Arc<engine_gui::GuiRuntimeState> {
        if let Some(cached) = &self.cached_gui_state {
            return std::sync::Arc::clone(cached);
        }
        let arc = std::sync::Arc::new(self.gui_state.clone());
        self.cached_gui_state = Some(std::sync::Arc::clone(&arc));
        arc
    }
}

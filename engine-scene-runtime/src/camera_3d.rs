use super::*;

const FREE_LOOK_CAPTURED_KEYS: &[&str] = &[
    "w", "a", "s", "d", "q", "e", " ", "Up", "Down", "Left", "Right",
];
const FREE_LOOK_VERTICAL_LOOK_SCALE: f32 = 1.8;
const FREE_LOOK_HORIZONTAL_LOOK_SCALE: f32 = 1.8;
const FREE_LOOK_PITCH_LIMIT_DEG: f32 = 85.0;

impl SceneRuntime {
    /// Returns `true` if the orbit camera is currently active.
    pub fn orbit_camera_active(&self) -> bool {
        self.orbit_camera.as_ref().is_some_and(|s| s.active)
    }

    /// Handle key events for the orbit camera.
    ///
    /// Orbit mode is always active when free-look is not engaged.
    /// `+`/`=` zooms in; `-` zooms out.
    pub fn apply_orbit_camera_key_events(
        &mut self,
        key_presses: &[KeyEvent],
        key_releases: &[KeyEvent],
    ) -> bool {
        // Yield to free-look camera when it is engaged.
        if self.orbit_camera.is_none() || self.free_look_camera_engaged() {
            return false;
        }

        let _ = key_releases;

        let active = self.orbit_camera.as_ref().is_some_and(|s| s.active);
        if active {
            let (mut dist, dist_min, dist_max, step, target) = {
                let s = self.orbit_camera.as_ref().unwrap();
                (s.distance, s.distance_min, s.distance_max, s.distance_step, s.target.clone())
            };
            let mut changed = false;
            for key in key_presses {
                match key.code {
                    KeyCode::Char('=') | KeyCode::Char('+') => {
                        dist = (dist - step).max(dist_min);
                        changed = true;
                    }
                    KeyCode::Char('-') => {
                        dist = (dist + step).min(dist_max);
                        changed = true;
                    }
                    _ => {}
                }
            }
            if changed {
                self.orbit_camera.as_mut().unwrap().distance = dist;
                let v = serde_json::Value::from(dist as f64);
                let _ = self.set_obj_sprite_property(&target, "obj.camera-distance", &v);
            }
        }

        false
    }

    /// Feed mouse moves into the orbit camera when left-dragging on empty canvas.
    /// Skipped when free-look camera is engaged (mouse is used for look-around then).
    pub fn apply_orbit_camera_mouse_moves(&mut self, mouse_moves: &[(f32, f32)]) {
        if self.free_look_camera_engaged() {
            return;
        }
        if mouse_moves.is_empty() {
            return;
        }
        // Read drag state before mutably borrowing orbit_camera.
        let is_dragging = {
            use engine_events::MouseButton;
            self.gui_state.drag_button == Some(MouseButton::Left)
                && self.gui_state.drag_widget.is_none()
        };

        let Some(state) = self.orbit_camera.as_mut() else {
            return;
        };
        if !state.active {
            return;
        }

        let Some((mut prev_x, mut prev_y)) = state.last_mouse_pos else {
            state.last_mouse_pos = mouse_moves.last().copied();
            return;
        };

        if !is_dragging {
            state.last_mouse_pos = mouse_moves.last().copied();
            return;
        }

        let sensitivity = state.drag_sensitivity;
        let mut total_dyaw = 0.0f32;
        let mut total_dpitch = 0.0f32;
        for &(x, y) in mouse_moves {
            total_dyaw += (x - prev_x) * sensitivity;
            total_dpitch += (y - prev_y) * sensitivity;
            prev_x = x;
            prev_y = y;
        }
        state.last_mouse_pos = Some((prev_x, prev_y));
        state.yaw += total_dyaw;
        let pitch_min = state.pitch_min;
        let pitch_max = state.pitch_max;
        state.pitch = (state.pitch + total_dpitch).clamp(pitch_min, pitch_max);
    }

    /// Apply orbit camera state to its target sprite each frame.
    ///
    /// Writes `obj.yaw`, `obj.pitch`, and `obj.camera-distance` to position the camera
    /// around the target sprite. Does not override auto-rotation — Rhai controls that.
    pub fn step_orbit_camera(&mut self) -> bool {
        let Some(state) = self.orbit_camera.as_ref() else {
            return false;
        };
        if !state.active {
            return false;
        }
        let (target, yaw, pitch, dist) =
            (state.target.clone(), state.yaw, state.pitch, state.distance);

        let v_yaw = serde_json::Value::from(yaw as f64);
        let v_pitch = serde_json::Value::from(pitch as f64);
        let v_dist = serde_json::Value::from(dist as f64);

        let _ = self.set_obj_sprite_property(&target, "obj.yaw", &v_yaw);
        let _ = self.set_obj_sprite_property(&target, "obj.pitch", &v_pitch);
        let _ = self.set_obj_sprite_property(&target, "obj.camera-distance", &v_dist);
        true
    }

    pub fn adjust_obj_scale(&mut self, sprite_id: &str, delta: f32) -> bool {
        if delta == 0.0 {
            return false;
        }
        let mut updated = false;
        for layer in &mut self.scene.layers {
            for_each_obj_mut(&mut layer.sprites, &mut |sprite| {
                if let Sprite::Obj { id, scale, .. } = sprite {
                    if id.as_deref() == Some(sprite_id) {
                        *scale = Some((scale.unwrap_or(1.0) + delta).clamp(0.1, 8.0));
                        updated = true;
                    }
                }
            });
        }
        updated
    }

    pub fn toggle_obj_surface_mode(&mut self, sprite_id: &str) -> bool {
        let mut updated = false;
        for layer in &mut self.scene.layers {
            for_each_obj_mut(&mut layer.sprites, &mut |sprite| {
                if let Sprite::Obj {
                    id, surface_mode, ..
                } = sprite
                {
                    if id.as_deref() == Some(sprite_id) {
                        let is_wireframe = surface_mode
                            .as_deref()
                            .map(str::trim)
                            .is_some_and(|m| m.eq_ignore_ascii_case("wireframe"));
                        *surface_mode = Some(
                            if is_wireframe {
                                "material"
                            } else {
                                "wireframe"
                            }
                            .to_string(),
                        );
                        updated = true;
                    }
                }
            });
        }
        updated
    }

    pub fn toggle_obj_orbit(&mut self, sprite_id: &str) -> bool {
        let default_speed = self
            .obj_orbit_default_speed
            .get(sprite_id)
            .copied()
            .unwrap_or(20.0);
        let mut updated = false;
        for layer in &mut self.scene.layers {
            for_each_obj_mut(&mut layer.sprites, &mut |sprite| {
                if let Sprite::Obj {
                    id,
                    rotate_y_deg_per_sec,
                    ..
                } = sprite
                {
                    if id.as_deref() == Some(sprite_id) {
                        let current = rotate_y_deg_per_sec.unwrap_or(default_speed);
                        *rotate_y_deg_per_sec = Some(if current.abs() < f32::EPSILON {
                            default_speed
                        } else {
                            0.0
                        });
                        updated = true;
                    }
                }
            });
        }
        updated
    }

    /// Returns true if the OBJ sprite's orbit (auto-rotation) is currently active.
    pub fn is_obj_orbit_active(&self, sprite_id: &str) -> bool {
        for layer in &self.scene.layers {
            if let Some(active) = obj_orbit_active_in_sprites(&layer.sprites, sprite_id) {
                return active;
            }
        }
        false
    }

    /// Accumulate free-camera pan (view-space units) for a sprite.
    pub fn apply_obj_camera_pan(&mut self, sprite_id: &str, dx: f32, dy: f32) {
        let state = self
            .obj_camera_states
            .entry(sprite_id.to_string())
            .or_default();
        state.pan_x += dx;
        state.pan_y += dy;
        self.cached_obj_camera_states = None; // Invalidate cache
    }

    /// Accumulate free-camera look rotation (degrees) for a sprite.
    pub fn apply_obj_camera_look(&mut self, sprite_id: &str, dyaw: f32, dpitch: f32) {
        let state = self
            .obj_camera_states
            .entry(sprite_id.to_string())
            .or_default();
        state.look_yaw += dyaw;
        state.look_pitch = (state.look_pitch + dpitch).clamp(-85.0, 85.0);
        self.cached_obj_camera_states = None; // Invalidate cache
    }

    pub fn obj_camera_state(&self, sprite_id: &str) -> ObjCameraState {
        self.obj_camera_states
            .get(sprite_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_obj_last_mouse_pos(&mut self, sprite_id: &str, pos: Option<(f32, f32)>) {
        let state = self
            .obj_camera_states
            .entry(sprite_id.to_string())
            .or_default();
        state.last_mouse_pos = pos;
        self.cached_obj_camera_states = None; // Invalidate cache
    }

    pub fn obj_last_mouse_pos(&self, sprite_id: &str) -> Option<(f32, f32)> {
        self.obj_camera_states
            .get(sprite_id)
            .and_then(|state| state.last_mouse_pos)
    }

    pub fn free_look_camera_engaged(&self) -> bool {
        self.free_look_camera
            .as_ref()
            .is_some_and(|state| state.active || state.pending_activate)
    }

    pub fn apply_free_look_key_events(
        &mut self,
        key_presses: &[KeyEvent],
        key_releases: &[KeyEvent],
    ) -> bool {
        if self.free_look_camera.is_none() {
            return false;
        }

        let mut toggled = false;
        for key in key_presses {
            if is_free_look_toggle(key) {
                self.toggle_free_look_camera();
                self.ui_state.keys_down.remove("f"); // mask toggle key from Rhai
                toggled = true;
                continue;
            }
            if let Some(name) = free_look_captured_key_name(key) {
                if let Some(state) = self.free_look_camera.as_mut() {
                    if state.active || state.pending_activate {
                        state.held_keys.insert(name.to_string());
                    }
                }
            }
        }

        for key in key_releases {
            if let Some(name) = free_look_captured_key_name(key) {
                if let Some(state) = self.free_look_camera.as_mut() {
                    state.held_keys.remove(name);
                }
            }
        }

        if self.free_look_camera_engaged() {
            self.mask_free_look_keys_from_scene_input();
        }

        toggled
    }

    pub fn apply_free_look_mouse_moves(&mut self, mouse_moves: &[(f32, f32)]) {
        let Some(state) = self.free_look_camera.as_mut() else {
            return;
        };
        if !(state.active || state.pending_activate) {
            return;
        }

        let Some((mut prev_x, mut prev_y)) = state.last_mouse_pos else {
            state.last_mouse_pos = mouse_moves.last().copied();
            return;
        };

        let mut total_dyaw = 0.0f32;
        let mut total_dpitch = 0.0f32;
        for &(x, y) in mouse_moves {
            let dc = x - prev_x;
            let dr = y - prev_y;
            total_dyaw += dc * FREE_LOOK_HORIZONTAL_LOOK_SCALE * state.mouse_sensitivity;
            total_dpitch -= dr * FREE_LOOK_VERTICAL_LOOK_SCALE * state.mouse_sensitivity;
            prev_x = x;
            prev_y = y;
        }

        state.last_mouse_pos = Some((prev_x, prev_y));
        if state.active {
            state.yaw_deg += total_dyaw;
            state.pitch_deg = (state.pitch_deg + total_dpitch)
                .clamp(-FREE_LOOK_PITCH_LIMIT_DEG, FREE_LOOK_PITCH_LIMIT_DEG);
        }
    }

    pub fn step_free_look_camera(&mut self, dt_ms: u64) -> bool {
        let current_camera = self.scene_camera_3d;
        let Some(state) = self.free_look_camera.as_mut() else {
            return false;
        };
        if !(state.active || state.pending_activate) {
            return false;
        }

        if state.pending_activate {
            let forward = current_camera.forward();
            state.position = current_camera.eye;
            state.yaw_deg = forward[0].atan2(-forward[2]).to_degrees();
            state.pitch_deg = forward[1].clamp(-1.0, 1.0).asin().to_degrees();
            state.pending_activate = false;
            state.active = true;
        }

        let dt_s = dt_ms as f32 / 1000.0;
        let forward = free_look_forward(state.yaw_deg, state.pitch_deg);
        let right = normalize3(cross3(forward, [0.0, 1.0, 0.0]));
        let up = normalize3(cross3(right, forward));

        let mut move_dir = [0.0f32, 0.0, 0.0];
        if state.held_keys.contains("w") {
            move_dir = add3(move_dir, forward);
        }
        if state.held_keys.contains("s") {
            move_dir = add3(move_dir, scale3(forward, -1.0));
        }
        if state.held_keys.contains("d") {
            move_dir = add3(move_dir, right);
        }
        if state.held_keys.contains("a") {
            move_dir = add3(move_dir, scale3(right, -1.0));
        }
        if state.held_keys.contains("e") {
            move_dir = add3(move_dir, up);
        }
        if state.held_keys.contains("q") {
            move_dir = add3(move_dir, scale3(up, -1.0));
        }

        let move_len = length3(move_dir);
        if move_len > 0.0 {
            let step = scale3(normalize3(move_dir), state.move_speed * dt_s);
            state.position = add3(state.position, step);
        }

        let mut camera = current_camera;
        camera.eye = state.position;
        camera.look_at = add3(state.position, forward);
        camera.up = up;
        self.set_scene_camera_3d_internal(camera);
        true
    }

    fn toggle_free_look_camera(&mut self) {
        let Some(state) = self.free_look_camera.as_mut() else {
            return;
        };
        if state.active || state.pending_activate {
            for key in &state.held_keys {
                self.ui_state.keys_down.insert(key.clone());
            }
            state.active = false;
            state.pending_activate = false;
            state.last_mouse_pos = None;
            return;
        }

        state.held_keys.clear();
        for key in FREE_LOOK_CAPTURED_KEYS {
            if self.ui_state.keys_down.contains(*key) {
                state.held_keys.insert((*key).to_string());
            }
        }
        state.last_mouse_pos = None;
        state.pending_activate = true;
        self.mask_free_look_keys_from_scene_input();
    }

    fn mask_free_look_keys_from_scene_input(&mut self) {
        for key in FREE_LOOK_CAPTURED_KEYS {
            self.ui_state.keys_down.remove(*key);
        }
    }
}

/// Visit every [`Sprite::Obj`] mutably in a tree, recursing into grids.
fn for_each_obj_mut(sprites: &mut [Sprite], f: &mut impl FnMut(&mut Sprite)) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Obj { .. } => f(sprite),
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => for_each_obj_mut(children, f),
            _ => {}
        }
    }
}

fn obj_orbit_active_in_sprites(sprites: &[Sprite], sprite_id: &str) -> Option<bool> {
    for sprite in sprites {
        match sprite {
            Sprite::Obj {
                id,
                rotate_y_deg_per_sec,
                ..
            } => {
                if id.as_deref() == Some(sprite_id) {
                    return Some(rotate_y_deg_per_sec.unwrap_or(0.0).abs() > f32::EPSILON);
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if let Some(result) = obj_orbit_active_in_sprites(children, sprite_id) {
                    return Some(result);
                }
            }
            Sprite::Text { .. }
            | Sprite::Image { .. }
            | Sprite::Planet { .. }
            | Sprite::Scene3D { .. }
            | Sprite::Vector { .. } => {}
        }
    }
    None
}

fn is_free_look_toggle(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('f') | KeyCode::Char('F'))
        && key.modifiers.contains(KeyModifiers::CONTROL)
}

fn free_look_captured_key_name(key: &KeyEvent) -> Option<&'static str> {
    match key.code {
        KeyCode::Char('w') | KeyCode::Char('W') => Some("w"),
        KeyCode::Char('a') | KeyCode::Char('A') => Some("a"),
        KeyCode::Char('s') | KeyCode::Char('S') => Some("s"),
        KeyCode::Char('d') | KeyCode::Char('D') => Some("d"),
        KeyCode::Char('q') | KeyCode::Char('Q') => Some("q"),
        KeyCode::Char('e') | KeyCode::Char('E') => Some("e"),
        KeyCode::Char(' ') => Some(" "),
        KeyCode::Up => Some("Up"),
        KeyCode::Down => Some("Down"),
        KeyCode::Left => Some("Left"),
        KeyCode::Right => Some("Right"),
        _ => None,
    }
}

fn free_look_forward(yaw_deg: f32, pitch_deg: f32) -> [f32; 3] {
    let yaw = yaw_deg.to_radians();
    let pitch = pitch_deg.to_radians();
    [
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    ]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = length3(v).max(1e-6);
    [v[0] / len, v[1] / len, v[2] / len]
}

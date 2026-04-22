use super::*;

const FREE_LOOK_CAPTURED_KEYS: &[&str] = &[
    "w", "a", "s", "d", "q", "e", " ", "Up", "Down", "Left", "Right",
];
const FREE_LOOK_TRANSIENT_DRAG_MARKER: &str = "__free_look_transient_drag__";
const FREE_LOOK_VERTICAL_LOOK_SCALE: f32 = 1.8;
const FREE_LOOK_HORIZONTAL_LOOK_SCALE: f32 = 1.8;
const FREE_LOOK_WORLD_PITCH_LIMIT_DEG: f32 = 89.0;
const FREE_LOOK_SURFACE_PITCH_LIMIT_DEG: f32 = 85.0;
const FREE_LOOK_ROTATION_LAG_SEC: f32 = 0.08;
const OBJ_ORBIT_ROTATION_LAG_SEC: f32 = 0.10;
const OBJ_ORBIT_DISTANCE_LAG_SEC: f32 = 0.12;
const OBJ_ORBIT_DISTANCE_HARD_MIN: f32 = 0.3;
const OBJ_ORBIT_DISTANCE_HARD_MAX: f32 = 10.0;
const OBJ_SCALE_MIN: f32 = 0.0000001;
const OBJ_SCALE_MAX: f32 = 8.0;

impl SceneRuntime {
    pub fn apply_catalog_camera_preset(
        &mut self,
        controller_kind: &str,
        controller_id: &str,
    ) -> bool {
        let Some(kind) = camera::CameraControllerKind::from_controller_kind_name(controller_kind)
        else {
            return false;
        };

        match kind {
            camera::CameraControllerKind::FreeLook | camera::CameraControllerKind::Orbit => {
                self.set_active_camera_controller(kind, None)
            }
            camera::CameraControllerKind::Fps
            | camera::CameraControllerKind::Chase
            | camera::CameraControllerKind::Cockpit => {
                let slot_ready = self.ensure_camera_controller_slot(kind, controller_id);
                if !slot_ready {
                    return false;
                }
                let fallback_pose = self.scene_camera_3d();
                let controller = self.camera_director.ensure_reserved_controller(kind, controller_id);
                if controller.target_pose.is_none() {
                    controller.target_pose = Some(fallback_pose);
                }
                self.set_active_camera_controller(kind, Some(controller_id))
            }
        }
    }

    fn free_look_state(&self) -> Option<&camera::FreeLookCameraState> {
        self.camera_director.free_look.as_ref()
    }

    fn free_look_state_mut(&mut self) -> Option<&mut camera::FreeLookCameraState> {
        self.camera_director.free_look.as_mut()
    }

    fn orbit_state(&self) -> Option<&camera::ObjOrbitCameraState> {
        self.camera_director.orbit.as_ref()
    }

    fn orbit_state_mut(&mut self) -> Option<&mut camera::ObjOrbitCameraState> {
        self.camera_director.orbit.as_mut()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn active_camera_controller_kind(&self) -> Option<camera::CameraControllerKind> {
        self.camera_director.active_controller
    }

    pub fn begin_camera_director_frame(&mut self) {
        self.camera_director.begin_frame();
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn ensure_camera_controller_slot(
        &mut self,
        kind: camera::CameraControllerKind,
        id: &str,
    ) -> bool {
        match kind {
            camera::CameraControllerKind::FreeLook | camera::CameraControllerKind::Orbit => false,
            _ => {
                self.camera_director.ensure_reserved_controller(kind, id);
                true
            }
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn set_reserved_camera_controller_pose(
        &mut self,
        kind: camera::CameraControllerKind,
        id: &str,
        pose: SceneCamera3D,
    ) -> bool {
        match kind {
            camera::CameraControllerKind::FreeLook | camera::CameraControllerKind::Orbit => false,
            _ => {
                let controller = self.camera_director.ensure_reserved_controller(kind, id);
                controller.target_pose = Some(pose);
                true
            }
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn set_active_camera_controller(
        &mut self,
        kind: camera::CameraControllerKind,
        id: Option<&str>,
    ) -> bool {
        match kind {
            camera::CameraControllerKind::FreeLook | camera::CameraControllerKind::Orbit => {
                let before = self.camera_director.active_controller;
                self.camera_director.set_active_if_available(kind);
                self.camera_director.active_controller != before
            }
            _ => {
                let Some(id) = id else {
                    return false;
                };
                if self.camera_director.reserved_controller(kind, id).is_none() {
                    return false;
                }
                self.camera_director.active_controller = Some(kind);
                self.camera_director.active_reserved_controller_id = Some(id.to_string());
                true
            }
        }
    }

    pub fn step_camera_director(&mut self, dt_ms: u64) -> bool {
        match self.resolve_camera_controller_for_step() {
            Some(camera::CameraControllerKind::FreeLook) => self.step_free_look_camera(dt_ms),
            Some(camera::CameraControllerKind::Orbit) => self.step_orbit_camera(dt_ms),
            Some(
                kind @ (camera::CameraControllerKind::Fps
                | camera::CameraControllerKind::Chase
                | camera::CameraControllerKind::Cockpit),
            ) => self.step_reserved_camera_controller(kind, dt_ms),
            None => false,
        }
    }

    pub fn step_camera_director_adapters(&mut self, dt_ms: u64) -> bool {
        if self.camera_director.adapter_step_consumed {
            return false;
        }
        self.camera_director.adapter_step_consumed = true;
        self.step_camera_director(dt_ms)
    }

    pub fn apply_camera_input_frame(&mut self, frame: &CameraInputFrame) -> bool {
        let mut handled = false;
        if self.legacy_camera_input_enabled() {
            if !frame.key_presses.is_empty() || !frame.key_releases.is_empty() {
                handled |= self.apply_free_look_key_events(&frame.key_presses, &frame.key_releases);
                handled |=
                    self.apply_orbit_camera_key_events(&frame.key_presses, &frame.key_releases);
            }
            for &(x, y) in &frame.alt_left_mouse_downs {
                handled |= self.begin_free_look_drag(x, y);
            }
            if frame.left_mouse_ups > 0 || frame.focus_lost {
                handled |= self.end_free_look_drag();
            }
            if !frame.mouse_moves.is_empty() {
                self.apply_free_look_mouse_moves(&frame.mouse_moves);
                self.apply_orbit_camera_mouse_moves(&frame.mouse_moves);
                handled = true;
            }
            if !frame.ctrl_scroll_deltas.is_empty() {
                self.apply_free_look_scroll(&frame.ctrl_scroll_deltas);
                handled = true;
            }
            if !frame.scroll_deltas.is_empty() {
                self.apply_orbit_camera_scroll(&frame.scroll_deltas);
                handled = true;
            }
        }
        handled
    }

    pub fn free_look_surface_mode_enabled(&self) -> bool {
        self.free_look_state()
            .is_some_and(|state| state.surface_mode)
    }

    fn legacy_camera_input_enabled(&self) -> bool {
        self.camera_director.active_controller.is_none_or(|kind| {
            matches!(
                kind,
                camera::CameraControllerKind::FreeLook | camera::CameraControllerKind::Orbit
            )
        })
    }

    fn resolve_camera_controller_for_step(&self) -> Option<camera::CameraControllerKind> {
        match self.camera_director.active_controller {
            Some(camera::CameraControllerKind::FreeLook) => self
                .free_look_state()
                .is_some()
                .then_some(camera::CameraControllerKind::FreeLook),
            Some(camera::CameraControllerKind::Orbit) => self
                .orbit_state()
                .is_some()
                .then_some(camera::CameraControllerKind::Orbit),
            Some(
                kind @ (camera::CameraControllerKind::Fps
                | camera::CameraControllerKind::Chase
                | camera::CameraControllerKind::Cockpit),
            ) => Some(kind),
            None => {
                if self.free_look_camera_engaged() {
                    Some(camera::CameraControllerKind::FreeLook)
                } else if self.orbit_state().is_some() {
                    Some(camera::CameraControllerKind::Orbit)
                } else {
                    None
                }
            }
        }
    }

    fn step_reserved_camera_controller(
        &mut self,
        kind: camera::CameraControllerKind,
        dt_ms: u64,
    ) -> bool {
        let active_id = self
            .camera_director
            .active_reserved_controller_id
            .as_deref();
        let Some(controller_index) = self
            .camera_director
            .reserved
            .iter()
            .position(|entry| entry.kind == kind && active_id.is_none_or(|id| entry.id == id))
            .or_else(|| {
                self.camera_director
                    .reserved
                    .iter()
                    .position(|entry| entry.kind == kind)
            })
        else {
            return false;
        };
        let dt_s = (dt_ms as f32 / 1000.0).max(0.0);
        let current_camera = self.scene_camera_3d;
        let controller = &mut self.camera_director.reserved[controller_index];
        let Some(target_pose) = controller.target_pose else {
            return false;
        };

        let next_pose = match kind {
            camera::CameraControllerKind::Fps | camera::CameraControllerKind::Cockpit => {
                let desired_pose = SceneCamera3D {
                    up: normalize_or_fallback(target_pose.up, current_camera.up),
                    ..target_pose
                };
                smooth_scene_camera_pose(
                    controller.applied_pose.unwrap_or(desired_pose),
                    desired_pose,
                    dt_s,
                    controller.position_lag_sec,
                    controller.aim_lag_sec,
                )
            }
            camera::CameraControllerKind::Chase => {
                let subject_forward = normalize_or_fallback(
                    sub3(target_pose.look_at, target_pose.eye),
                    current_camera.forward(),
                );
                let subject_up = normalize_or_fallback(target_pose.up, current_camera.up);
                let subject_right = normalize_or_fallback(
                    cross3(subject_forward, subject_up),
                    cross3(current_camera.forward(), current_camera.up),
                );
                let stable_up =
                    normalize_or_fallback(cross3(subject_right, subject_forward), subject_up);
                let desired_eye = add3(
                    target_pose.eye,
                    add3(
                        scale3(subject_forward, -controller.chase_distance.max(0.5)),
                        scale3(stable_up, controller.chase_height),
                    ),
                );
                let desired_look_at = add3(
                    target_pose.eye,
                    add3(
                        scale3(subject_forward, controller.chase_look_ahead.max(1.0)),
                        scale3(stable_up, controller.chase_height * 0.35),
                    ),
                );
                let desired_pose = SceneCamera3D {
                    eye: desired_eye,
                    look_at: desired_look_at,
                    up: stable_up,
                    fov_degrees: target_pose.fov_degrees,
                    near_clip: target_pose.near_clip,
                };
                smooth_scene_camera_pose(
                    controller.applied_pose.unwrap_or(desired_pose),
                    desired_pose,
                    dt_s,
                    controller.position_lag_sec,
                    controller.aim_lag_sec,
                )
            }
            camera::CameraControllerKind::FreeLook | camera::CameraControllerKind::Orbit => {
                return false;
            }
        };

        controller.applied_pose = Some(next_pose);
        self.set_scene_camera_3d_internal(next_pose);
        true
    }

    pub fn sync_free_look_surface_anchor(&mut self, anchor: Option<CameraSurfaceAnchor>) -> bool {
        let normalized = anchor.map(|anchor| CameraSurfaceAnchor {
            center: [
                if anchor.center[0].is_finite() {
                    anchor.center[0]
                } else {
                    0.0
                },
                if anchor.center[1].is_finite() {
                    anchor.center[1]
                } else {
                    0.0
                },
                if anchor.center[2].is_finite() {
                    anchor.center[2]
                } else {
                    0.0
                },
            ],
            radius: if anchor.radius.is_finite() {
                anchor.radius.max(0.001)
            } else {
                0.001
            },
        });
        if self.camera_director.surface_anchor == normalized {
            return false;
        }
        self.camera_director.surface_anchor = normalized;
        true
    }

    pub fn sync_free_look_surface_shell_2d(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
    ) -> bool {
        if !self.free_look_surface_mode_enabled() {
            return false;
        }
        self.sync_free_look_surface_anchor(Some(CameraSurfaceAnchor::new(
            [center_x, center_y, 0.0],
            radius,
        )))
    }

    pub(crate) fn clamp_orbit_camera_bootstrap(&mut self) {
        let Some(state) = self.orbit_state() else {
            return;
        };
        let target = state.target.clone();
        let yaw = state.yaw;
        let pitch = state.pitch;
        let authored_min = state.distance_min;
        let authored_max = state.distance_max;
        let authored_distance = state.distance;
        let (effective_min, effective_max) =
            self.obj_orbit_effective_distance_limits(&target, authored_min, authored_max);
        let clamped_distance = authored_distance.clamp(effective_min, effective_max);
        if let Some(state) = self.orbit_state_mut() {
            state.distance_min = effective_min;
            state.distance_max = effective_max;
            state.distance = clamped_distance;
            state.applied_distance = state.applied_distance.clamp(effective_min, effective_max);
        }
        let _ = self.set_obj_orbit_camera_fields(
            &target,
            Some(yaw),
            Some(pitch),
            Some(clamped_distance),
        );
        if self.camera_director.active_controller.is_none() {
            self.camera_director
                .set_active_if_available(camera::CameraControllerKind::Orbit);
        }
    }

    /// Returns `true` if the orbit camera is currently active.
    pub fn orbit_camera_active(&self) -> bool {
        self.orbit_state().is_some_and(|s| s.active)
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
        if self.orbit_state().is_none() || self.free_look_camera_engaged() {
            return false;
        }

        let _ = key_releases;

        let active = self.orbit_state().is_some_and(|s| s.active);
        if active {
            let (mut dist, dist_min, dist_max, step, target) = {
                let s = self.orbit_state().unwrap();
                (
                    s.distance,
                    s.distance_min,
                    s.distance_max,
                    s.distance_step,
                    s.target.clone(),
                )
            };
            let (effective_min, effective_max) =
                self.obj_orbit_effective_distance_limits(&target, dist_min, dist_max);
            let mut changed = false;
            for key in key_presses {
                match key.code {
                    KeyCode::Char('=') | KeyCode::Char('+') => {
                        dist = (dist - step).max(effective_min);
                        changed = true;
                    }
                    KeyCode::Char('-') => {
                        dist = (dist + step).min(effective_max);
                        changed = true;
                    }
                    _ => {}
                }
            }
            if changed {
                self.orbit_state_mut().unwrap().distance = dist;
            }
        }

        false
    }

    /// Apply mouse-wheel scroll to orbit camera zoom.
    /// Positive `delta_y` = scroll up = zoom in; negative = scroll down = zoom out.
    pub fn apply_orbit_camera_scroll(&mut self, scroll_deltas: &[f32]) {
        if self.orbit_state().is_none() || self.free_look_camera_engaged() {
            return;
        }
        let active = self.orbit_state().is_some_and(|s| s.active);
        if !active {
            return;
        }
        let (mut dist, dist_min, dist_max, step, target) = {
            let s = self.orbit_state().unwrap();
            (
                s.distance,
                s.distance_min,
                s.distance_max,
                s.distance_step,
                s.target.clone(),
            )
        };
        let (effective_min, effective_max) =
            self.obj_orbit_effective_distance_limits(&target, dist_min, dist_max);
        let mut changed = false;
        for &dy in scroll_deltas {
            if dy > 0.0 {
                dist = (dist - step).max(effective_min);
                changed = true;
            } else if dy < 0.0 {
                dist = (dist + step).min(effective_max);
                changed = true;
            }
        }
        if changed {
            self.orbit_state_mut().unwrap().distance = dist;
        }
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

        let Some(state) = self.orbit_state_mut() else {
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
        if total_dyaw.abs() < 0.0005 {
            total_dyaw = 0.0;
        }
        if total_dpitch.abs() < 0.0005 {
            total_dpitch = 0.0;
        }
        // Guard against occasional bursty mouse-event batches on slow frames.
        total_dyaw = total_dyaw.clamp(-18.0, 18.0);
        total_dpitch = total_dpitch.clamp(-18.0, 18.0);
        state.last_mouse_pos = Some((prev_x, prev_y));
        state.yaw += total_dyaw;
        let pitch_min = state.pitch_min;
        let pitch_max = state.pitch_max;
        state.pitch = (state.pitch + total_dpitch).clamp(pitch_min, pitch_max);
    }

    /// Apply orbit camera state to its target sprite each frame.
    ///
    /// Updates target OBJ orbit-camera fields (yaw, pitch, distance) to position the
    /// camera around the target sprite. Does not override auto-rotation — Rhai controls that.
    pub fn step_orbit_camera(&mut self, dt_ms: u64) -> bool {
        let Some(state) = self.orbit_state() else {
            return false;
        };
        if !state.active {
            return false;
        }
        let dt_s = (dt_ms as f32 / 1000.0).max(0.0);
        let rotation_alpha = smoothing_alpha(dt_s, OBJ_ORBIT_ROTATION_LAG_SEC);
        let distance_alpha = smoothing_alpha(dt_s, OBJ_ORBIT_DISTANCE_LAG_SEC);
        let (
            target,
            yaw,
            pitch,
            dist,
            applied_yaw,
            applied_pitch,
            applied_distance,
            dist_min,
            dist_max,
        ) = (
            state.target.clone(),
            state.yaw,
            state.pitch,
            state.distance,
            state.applied_yaw,
            state.applied_pitch,
            state.applied_distance,
            state.distance_min,
            state.distance_max,
        );
        let (effective_min, effective_max) =
            self.obj_orbit_effective_distance_limits(&target, dist_min, dist_max);
        let clamped_dist = dist.clamp(effective_min, effective_max);
        let next_applied_yaw = smooth_angle_deg(applied_yaw, yaw, rotation_alpha);
        let next_applied_pitch = lerp(applied_pitch, pitch, rotation_alpha);
        let next_applied_dist = if applied_distance < effective_min
            || applied_distance > effective_max
        {
            clamped_dist
        } else {
            lerp(applied_distance, clamped_dist, distance_alpha).clamp(effective_min, effective_max)
        };

        if let Some(state) = self.orbit_state_mut() {
            if state.target == target {
                state.distance = clamped_dist;
                state.applied_yaw = next_applied_yaw;
                state.applied_pitch = next_applied_pitch;
                state.applied_distance = next_applied_dist;
            }
        }

        let _ = self.set_obj_orbit_camera_fields(
            &target,
            Some(next_applied_yaw),
            Some(next_applied_pitch),
            Some(next_applied_dist),
        );
        self.camera_director
            .set_active_if_available(camera::CameraControllerKind::Orbit);
        true
    }

    fn set_obj_orbit_camera_fields(
        &mut self,
        sprite_id: &str,
        yaw_deg: Option<f32>,
        pitch_deg: Option<f32>,
        camera_distance: Option<f32>,
    ) -> bool {
        let safe_min_distance = self.obj_orbit_safe_distance_min(sprite_id);
        if let Some(&layer_idx) = self.sprite_id_to_layer.get(sprite_id) {
            if let Some(layer) = self.scene.layers.get_mut(layer_idx) {
                if set_obj_orbit_camera_fields_recursive(
                    &mut layer.sprites,
                    sprite_id,
                    yaw_deg,
                    pitch_deg,
                    camera_distance,
                    safe_min_distance,
                ) {
                    return true;
                }
            }
        }
        for layer in &mut self.scene.layers {
            if set_obj_orbit_camera_fields_recursive(
                &mut layer.sprites,
                sprite_id,
                yaw_deg,
                pitch_deg,
                camera_distance,
                safe_min_distance,
            ) {
                return true;
            }
        }
        false
    }

    fn obj_orbit_effective_distance_limits(
        &self,
        sprite_id: &str,
        authored_min: f32,
        authored_max: f32,
    ) -> (f32, f32) {
        let safe_min = self.obj_orbit_safe_distance_min(sprite_id);
        let effective_min = authored_min
            .max(safe_min)
            .clamp(OBJ_ORBIT_DISTANCE_HARD_MIN, OBJ_ORBIT_DISTANCE_HARD_MAX);
        let effective_max = authored_max
            .max(effective_min)
            .clamp(effective_min, OBJ_ORBIT_DISTANCE_HARD_MAX);
        (effective_min, effective_max)
    }

    fn obj_orbit_safe_distance_min(&self, sprite_id: &str) -> f32 {
        self.scene
            .layers
            .iter()
            .find_map(|layer| find_obj_orbit_safe_distance_in_sprites(&layer.sprites, sprite_id))
            .unwrap_or(OBJ_ORBIT_DISTANCE_HARD_MIN)
            .clamp(OBJ_ORBIT_DISTANCE_HARD_MIN, OBJ_ORBIT_DISTANCE_HARD_MAX)
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
                        *scale = Some(
                            (scale.unwrap_or(1.0) + delta).clamp(OBJ_SCALE_MIN, OBJ_SCALE_MAX),
                        );
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
        self.free_look_state()
            .is_some_and(|state| state.active || state.pending_activate)
    }

    pub fn apply_free_look_key_events(
        &mut self,
        key_presses: &[KeyEvent],
        key_releases: &[KeyEvent],
    ) -> bool {
        if self.free_look_state().is_none() {
            return false;
        }

        let mut toggled = false;
        for key in key_presses {
            if self
                .free_look_state()
                .is_some_and(|state| state.matches_toggle_key(key))
            {
                let toggle_key = self
                    .free_look_state()
                    .map(|state| state.toggle_key.clone())
                    .unwrap_or_default();
                self.toggle_free_look_camera();
                if !toggle_key.is_empty() {
                    self.ui_state.keys_down.remove(&toggle_key);
                }
                toggled = true;
                continue;
            }
            if let Some(name) = free_look_captured_key_name(key) {
                if let Some(state) = self.free_look_state_mut() {
                    if state.active || state.pending_activate {
                        state.held_keys.insert(name.to_string());
                    }
                }
            }
        }

        for key in key_releases {
            if let Some(name) = free_look_captured_key_name(key) {
                if let Some(state) = self.free_look_state_mut() {
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
        let Some(state) = self.free_look_state_mut() else {
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
            let pitch_limit = free_look_pitch_limit_deg(state.surface_mode);
            state.target_yaw_deg += total_dyaw;
            state.target_pitch_deg =
                (state.target_pitch_deg + total_dpitch).clamp(-pitch_limit, pitch_limit);
        }
    }

    /// Apply Ctrl+mouse-wheel zoom to free-look camera.
    /// Positive `delta_y` = zoom in (move closer), negative = zoom out.
    pub fn apply_free_look_scroll(&mut self, scroll_deltas: &[f32]) {
        let Some(state) = self.free_look_state_mut() else {
            return;
        };
        if !(state.active || state.pending_activate) || scroll_deltas.is_empty() {
            return;
        }

        if state.surface_mode {
            let min_alt = state.surface_min_altitude.max(0.0);
            let max_alt = state.surface_max_altitude.max(min_alt);
            let step = (state.surface_vertical_speed * 0.12)
                .max(state.surface_radius * 0.04)
                .max(0.01);
            for &dy in scroll_deltas {
                if dy > 0.0 {
                    state.surface_altitude = (state.surface_altitude - step).max(min_alt);
                } else if dy < 0.0 {
                    state.surface_altitude = (state.surface_altitude + step).min(max_alt);
                }
            }
            return;
        }

        let forward = free_look_forward(state.yaw_deg, state.pitch_deg);
        let step = (state.move_speed * 0.16).max(0.05);
        for &dy in scroll_deltas {
            if dy > 0.0 {
                state.position = add3(state.position, scale3(forward, step));
            } else if dy < 0.0 {
                state.position = add3(state.position, scale3(forward, -step));
            }
        }
    }

    pub fn begin_free_look_drag(&mut self, mouse_x: f32, mouse_y: f32) -> bool {
        if self.free_look_drag_hits_gui(mouse_x, mouse_y) {
            return false;
        }
        let Some(state) = self.free_look_state_mut() else {
            return false;
        };
        let was_engaged = state.active || state.pending_activate;
        state.drag_hold = true;
        state.last_mouse_pos = Some((mouse_x, mouse_y));
        if was_engaged {
            state.held_keys.remove(FREE_LOOK_TRANSIENT_DRAG_MARKER);
        } else {
            state.held_keys.clear();
            state
                .held_keys
                .insert(FREE_LOOK_TRANSIENT_DRAG_MARKER.to_string());
            state.pending_activate = true;
        }
        self.mask_free_look_keys_from_scene_input();
        true
    }

    pub fn end_free_look_drag(&mut self) -> bool {
        let reinject_keys = {
            let Some(state) = self.free_look_state_mut() else {
                return false;
            };
            if !state.drag_hold {
                return false;
            }
            state.drag_hold = false;
            state.last_mouse_pos = None;
            let had_marker = state.held_keys.remove(FREE_LOOK_TRANSIENT_DRAG_MARKER);
            let transient_drag = state.pending_activate && (state.active || had_marker);
            if !transient_drag {
                return true;
            }
            let reinject_keys = state
                .held_keys
                .iter()
                .filter(|key| is_free_look_captured_key_name(key))
                .cloned()
                .collect::<Vec<_>>();
            if state.active || state.pending_activate {
                state.active = false;
                state.pending_activate = false;
            }
            reinject_keys
        };
        for key in reinject_keys {
            self.ui_state.keys_down.insert(key);
        }
        true
    }

    pub fn step_free_look_camera(&mut self, dt_ms: u64) -> bool {
        let current_camera = self.scene_camera_3d;
        let mut activate_free_look = false;
        let surface_anchor = self.camera_director.surface_anchor;
        let Some(state) = self.free_look_state_mut() else {
            return false;
        };
        if state.surface_mode {
            if let Some(anchor) = surface_anchor {
                state.surface_center = anchor.center;
                state.surface_radius = anchor.radius;
            }
        }
        if !(state.active || state.pending_activate) {
            return false;
        }

        if state.pending_activate && !state.active {
            let forward = current_camera.forward();
            if state.surface_mode {
                let min_alt = state.surface_min_altitude.max(0.0);
                let max_alt = state.surface_max_altitude.max(min_alt);
                let radial_offset = sub3(current_camera.eye, state.surface_center);
                let radial = normalize3(radial_offset);
                let current_altitude = (length3(radial_offset) - state.surface_radius).max(0.0);
                state.surface_altitude = current_altitude.clamp(min_alt, max_alt);
                state.position = add3(
                    state.surface_center,
                    scale3(radial, state.surface_radius + state.surface_altitude),
                );
                let (yaw_deg, pitch_deg) = surface_free_look_angles_from_forward(forward, radial);
                state.yaw_deg = yaw_deg;
                state.pitch_deg = pitch_deg;
            } else {
                state.position = current_camera.eye;
                state.yaw_deg = forward[0].atan2(-forward[2]).to_degrees();
                state.pitch_deg = forward[1].clamp(-1.0, 1.0).asin().to_degrees();
            }
            state.target_yaw_deg = state.yaw_deg;
            state.target_pitch_deg = state.pitch_deg;
            state.active = true;
            state.pending_activate =
                state.drag_hold && state.held_keys.contains(FREE_LOOK_TRANSIENT_DRAG_MARKER);
            activate_free_look = true;
        }

        let dt_s = dt_ms as f32 / 1000.0;
        let rotation_alpha = smoothing_alpha(dt_s, FREE_LOOK_ROTATION_LAG_SEC);
        let pitch_limit = free_look_pitch_limit_deg(state.surface_mode);
        state.yaw_deg = smooth_angle_deg(state.yaw_deg, state.target_yaw_deg, rotation_alpha);
        state.pitch_deg = lerp(state.pitch_deg, state.target_pitch_deg, rotation_alpha)
            .clamp(-pitch_limit, pitch_limit);
        let mut camera = current_camera;

        if state.surface_mode {
            let min_alt = state.surface_min_altitude.max(0.0);
            let max_alt = state.surface_max_altitude.max(min_alt);
            state.surface_altitude = state.surface_altitude.clamp(min_alt, max_alt);

            let mut up = normalize3(sub3(state.position, state.surface_center));
            let (mut tangent_forward, _) = surface_free_look_basis(up, state.yaw_deg, 0.0);
            let mut right = normalize3(cross3(tangent_forward, up));
            if length3(right) <= 1e-5 {
                right = normalize3(cross3([1.0, 0.0, 0.0], up));
            }

            let mut tangent_move = [0.0f32, 0.0, 0.0];
            if state.held_keys.contains("w") {
                tangent_move = add3(tangent_move, tangent_forward);
            }
            if state.held_keys.contains("s") {
                tangent_move = add3(tangent_move, scale3(tangent_forward, -1.0));
            }
            if state.held_keys.contains("d") {
                tangent_move = add3(tangent_move, right);
            }
            if state.held_keys.contains("a") {
                tangent_move = add3(tangent_move, scale3(right, -1.0));
            }

            let tangent_len = length3(tangent_move);
            if tangent_len > 0.0 {
                let step = scale3(normalize3(tangent_move), state.move_speed * dt_s);
                state.position = add3(state.position, step);
            }

            if state.held_keys.contains("e") {
                state.surface_altitude =
                    (state.surface_altitude + state.surface_vertical_speed * dt_s).min(max_alt);
            }
            if state.held_keys.contains("q") {
                state.surface_altitude =
                    (state.surface_altitude - state.surface_vertical_speed * dt_s).max(min_alt);
            }

            up = normalize3(sub3(state.position, state.surface_center));
            state.position = add3(
                state.surface_center,
                scale3(up, state.surface_radius + state.surface_altitude),
            );
            let (next_tangent_forward, forward) =
                surface_free_look_basis(up, state.yaw_deg, state.pitch_deg);
            tangent_forward = next_tangent_forward;
            right = normalize3(cross3(tangent_forward, up));
            if length3(right) <= 1e-5 {
                right = surface_reference_right(up);
            }
            let camera_up = normalize3(cross3(right, forward));
            camera.eye = state.position;
            camera.look_at = add3(state.position, forward);
            camera.up = camera_up;
        } else {
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
            camera.eye = state.position;
            camera.look_at = add3(state.position, forward);
            camera.up = up;
        }

        self.set_scene_camera_3d_internal(camera);
        if activate_free_look {
            self.camera_director
                .set_active_if_available(camera::CameraControllerKind::FreeLook);
        }
        true
    }

    fn toggle_free_look_camera(&mut self) {
        let has_orbit = self.orbit_state().is_some();
        let mut activate_free_look = false;
        let mut activate_orbit = false;
        let mut mask_inputs = false;
        let mut reinject_keys = Vec::new();
        let captured_scene_keys = FREE_LOOK_CAPTURED_KEYS
            .iter()
            .filter(|key| self.ui_state.keys_down.contains(**key))
            .map(|key| (*key).to_string())
            .collect::<Vec<_>>();
        let Some(state) = self.free_look_state_mut() else {
            return;
        };
        if state.active || state.pending_activate {
            reinject_keys = state
                .held_keys
                .iter()
                .filter(|key| is_free_look_captured_key_name(key))
                .cloned()
                .collect::<Vec<_>>();
            state.held_keys.remove(FREE_LOOK_TRANSIENT_DRAG_MARKER);
            state.active = false;
            state.pending_activate = false;
            state.drag_hold = false;
            state.last_mouse_pos = None;
            activate_orbit = has_orbit;
        } else {
            state.drag_hold = false;
            state.held_keys.clear();
            for key in &captured_scene_keys {
                state.held_keys.insert(key.clone());
            }
            state.last_mouse_pos = None;
            state.pending_activate = true;
            activate_free_look = true;
            mask_inputs = true;
        }
        for key in reinject_keys {
            self.ui_state.keys_down.insert(key);
        }
        if activate_orbit {
            self.camera_director
                .set_active_if_available(camera::CameraControllerKind::Orbit);
            return;
        }
        if mask_inputs {
            self.mask_free_look_keys_from_scene_input();
        }
        if activate_free_look {
            self.camera_director
                .set_active_if_available(camera::CameraControllerKind::FreeLook);
        }
    }

    fn mask_free_look_keys_from_scene_input(&mut self) {
        for key in FREE_LOOK_CAPTURED_KEYS {
            self.ui_state.keys_down.remove(*key);
        }
    }

    fn free_look_drag_hits_gui(&self, mouse_x: f32, mouse_y: f32) -> bool {
        self.gui_widgets.iter().any(|widget| {
            let initial_state;
            let widget_state = if let Some(state) = self.gui_state.widgets.get(widget.id()) {
                state
            } else {
                initial_state = widget.initial_state();
                &initial_state
            };
            widget
                .bounds(widget_state)
                .is_some_and(|bounds| bounds.hit_test(mouse_x, mouse_y))
        })
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

fn set_obj_orbit_camera_fields_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    yaw_deg: Option<f32>,
    pitch_deg: Option<f32>,
    camera_distance: Option<f32>,
    safe_min_distance: f32,
) -> bool {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Obj {
                id: Some(id),
                yaw_deg: current_yaw,
                pitch_deg: current_pitch,
                camera_distance: current_distance,
                ..
            } if id == sprite_id => {
                let mut updated = false;
                if let Some(next_yaw) = yaw_deg {
                    if current_yaw.map_or(true, |current| (current - next_yaw).abs() > f32::EPSILON)
                    {
                        *current_yaw = Some(next_yaw);
                        updated = true;
                    }
                }
                if let Some(next_pitch) = pitch_deg {
                    if current_pitch
                        .map_or(true, |current| (current - next_pitch).abs() > f32::EPSILON)
                    {
                        *current_pitch = Some(next_pitch);
                        updated = true;
                    }
                }
                if let Some(next_distance) = camera_distance {
                    let clamped =
                        next_distance.clamp(safe_min_distance, OBJ_ORBIT_DISTANCE_HARD_MAX);
                    if current_distance
                        .map_or(true, |current| (current - clamped).abs() > f32::EPSILON)
                    {
                        *current_distance = Some(clamped);
                        updated = true;
                    }
                }
                return updated;
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if set_obj_orbit_camera_fields_recursive(
                    children,
                    sprite_id,
                    yaw_deg,
                    pitch_deg,
                    camera_distance,
                    safe_min_distance,
                ) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
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

fn is_free_look_captured_key_name(key: &str) -> bool {
    FREE_LOOK_CAPTURED_KEYS.contains(&key)
}

fn free_look_pitch_limit_deg(surface_mode: bool) -> f32 {
    if surface_mode {
        FREE_LOOK_SURFACE_PITCH_LIMIT_DEG
    } else {
        FREE_LOOK_WORLD_PITCH_LIMIT_DEG
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

fn smoothing_alpha(dt_s: f32, lag_s: f32) -> f32 {
    if lag_s <= 0.0001 {
        return 1.0;
    }
    (dt_s / (lag_s + dt_s)).clamp(0.0, 1.0)
}

fn shortest_angle_delta_deg(current: f32, target: f32) -> f32 {
    let mut delta = (target - current) % 360.0;
    if delta > 180.0 {
        delta -= 360.0;
    } else if delta < -180.0 {
        delta += 360.0;
    }
    delta
}

fn smooth_angle_deg(current: f32, target: f32, alpha: f32) -> f32 {
    current + shortest_angle_delta_deg(current, target) * alpha.clamp(0.0, 1.0)
}

fn lerp(current: f32, target: f32, alpha: f32) -> f32 {
    current + (target - current) * alpha.clamp(0.0, 1.0)
}

fn normalize_or_fallback(v: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    if length3(v) > 1e-5 {
        normalize3(v)
    } else {
        normalize3(fallback)
    }
}

fn smooth_scene_camera_pose(
    current: SceneCamera3D,
    target: SceneCamera3D,
    dt_s: f32,
    position_lag_sec: f32,
    aim_lag_sec: f32,
) -> SceneCamera3D {
    let pos_alpha = smoothing_alpha(dt_s, position_lag_sec);
    let aim_alpha = smoothing_alpha(dt_s, aim_lag_sec);
    SceneCamera3D {
        eye: [
            lerp(current.eye[0], target.eye[0], pos_alpha),
            lerp(current.eye[1], target.eye[1], pos_alpha),
            lerp(current.eye[2], target.eye[2], pos_alpha),
        ],
        look_at: [
            lerp(current.look_at[0], target.look_at[0], aim_alpha),
            lerp(current.look_at[1], target.look_at[1], aim_alpha),
            lerp(current.look_at[2], target.look_at[2], aim_alpha),
        ],
        up: normalize_or_fallback(
            [
                lerp(current.up[0], target.up[0], aim_alpha),
                lerp(current.up[1], target.up[1], aim_alpha),
                lerp(current.up[2], target.up[2], aim_alpha),
            ],
            target.up,
        ),
        fov_degrees: lerp(current.fov_degrees, target.fov_degrees, aim_alpha),
        near_clip: lerp(current.near_clip, target.near_clip, aim_alpha),
    }
}

fn surface_reference_forward(up: [f32; 3]) -> [f32; 3] {
    for candidate in [[0.0, 1.0, 0.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0]] {
        let tangent = project_on_plane(candidate, up);
        if length3(tangent) > 1e-5 {
            return normalize3(tangent);
        }
    }
    [0.0, 0.0, -1.0]
}

fn surface_reference_right(up: [f32; 3]) -> [f32; 3] {
    let forward = surface_reference_forward(up);
    let right = normalize3(cross3(forward, up));
    if length3(right) > 1e-5 {
        right
    } else {
        [1.0, 0.0, 0.0]
    }
}

fn surface_free_look_angles_from_forward(forward: [f32; 3], up: [f32; 3]) -> (f32, f32) {
    let base_forward = surface_reference_forward(up);
    let base_right = surface_reference_right(up);
    let pitch_deg = dot3(forward, up)
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees()
        .clamp(
            -FREE_LOOK_SURFACE_PITCH_LIMIT_DEG,
            FREE_LOOK_SURFACE_PITCH_LIMIT_DEG,
        );
    let tangent_forward = project_on_plane(forward, up);
    if length3(tangent_forward) <= 1e-5 {
        return (0.0, pitch_deg);
    }
    let tangent_forward = normalize3(tangent_forward);
    let yaw_deg = dot3(tangent_forward, base_right)
        .atan2(dot3(tangent_forward, base_forward))
        .to_degrees();
    (yaw_deg, pitch_deg)
}

fn surface_free_look_basis(up: [f32; 3], yaw_deg: f32, pitch_deg: f32) -> ([f32; 3], [f32; 3]) {
    let base_forward = surface_reference_forward(up);
    let base_right = surface_reference_right(up);
    let yaw = yaw_deg.to_radians();
    let pitch = pitch_deg.to_radians();
    let tangent_forward = normalize3(add3(
        scale3(base_forward, yaw.cos()),
        scale3(base_right, yaw.sin()),
    ));
    let forward = normalize3(add3(
        scale3(tangent_forward, pitch.cos()),
        scale3(up, pitch.sin()),
    ));
    (tangent_forward, forward)
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn project_on_plane(v: [f32; 3], plane_normal: [f32; 3]) -> [f32; 3] {
    sub3(v, scale3(plane_normal, dot3(v, plane_normal)))
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = length3(v).max(1e-6);
    [v[0] / len, v[1] / len, v[2] / len]
}

fn find_obj_orbit_safe_distance_in_sprites(sprites: &[Sprite], sprite_id: &str) -> Option<f32> {
    for sprite in sprites {
        match sprite {
            Sprite::Obj {
                id: Some(id),
                size,
                width,
                height,
                scale,
                fov_degrees,
                near_clip,
                atmo_height,
                atmo_density,
                atmo_strength,
                atmo_rayleigh_amount,
                atmo_haze_amount,
                atmo_limb_boost,
                atmo_halo_strength,
                atmo_halo_width,
                world_gen_displacement_scale,
                ..
            } if id == sprite_id => {
                return Some(estimate_obj_orbit_safe_distance(
                    *size,
                    *width,
                    *height,
                    *scale,
                    *fov_degrees,
                    *near_clip,
                    *atmo_height,
                    *atmo_density,
                    *atmo_strength,
                    *atmo_rayleigh_amount,
                    *atmo_haze_amount,
                    *atmo_limb_boost,
                    *atmo_halo_strength,
                    *atmo_halo_width,
                    *world_gen_displacement_scale,
                ));
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if let Some(value) = find_obj_orbit_safe_distance_in_sprites(children, sprite_id) {
                    return Some(value);
                }
            }
            _ => {}
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn estimate_obj_orbit_safe_distance(
    size: Option<engine_core::scene::SpriteSizePreset>,
    width: Option<u16>,
    height: Option<u16>,
    scale: Option<f32>,
    fov_degrees: Option<f32>,
    near_clip: Option<f32>,
    atmo_height: Option<f32>,
    atmo_density: Option<f32>,
    atmo_strength: Option<f32>,
    atmo_rayleigh_amount: Option<f32>,
    atmo_haze_amount: Option<f32>,
    atmo_limb_boost: Option<f32>,
    atmo_halo_strength: Option<f32>,
    atmo_halo_width: Option<f32>,
    world_gen_displacement_scale: Option<f32>,
) -> f32 {
    fn estimated_aspect_ratio(
        size: Option<engine_core::scene::SpriteSizePreset>,
        width: Option<u16>,
        height: Option<u16>,
    ) -> f32 {
        let (w, h) = match (width, height) {
            (Some(w), Some(h)) => (w.max(1), h.max(1)),
            (Some(w), None) => (w.max(1), 24),
            (None, Some(h)) => (64, h.max(1)),
            (None, None) => size
                .unwrap_or(engine_core::scene::SpriteSizePreset::Medium)
                .obj_dimensions(),
        };
        (w as f32 / h as f32).clamp(0.2, 8.0)
    }

    let base_radius = scale.unwrap_or(1.0).clamp(0.05, 8.0);
    let atmo_density = atmo_density.unwrap_or(0.0).clamp(0.0, 1.0);
    let atmo_strength = atmo_strength.unwrap_or(0.0).clamp(0.0, 1.0);
    let atmo_rayleigh_amount = atmo_rayleigh_amount.unwrap_or(0.0).clamp(0.0, 1.0);
    let atmo_haze_amount = atmo_haze_amount.unwrap_or(0.0).clamp(0.0, 1.0);
    let atmo_enabled = atmo_density > 0.001
        || atmo_strength > 0.001
        || atmo_rayleigh_amount > 0.001
        || atmo_haze_amount > 0.001;
    let atmo_height = atmo_height.unwrap_or(0.0).clamp(0.0, 1.0);
    let atmo_shell = if atmo_enabled { atmo_height } else { 0.0 };
    let authored_halo_shell = if atmo_halo_strength.unwrap_or(0.0) > 0.01 {
        atmo_halo_width.unwrap_or(0.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let limb_boost = atmo_limb_boost.unwrap_or(1.0).clamp(0.0, 4.0);
    // Keep camera-safety envelope aligned with render-time halo construction in engine-render-3d.
    let derived_halo_shell = if atmo_enabled {
        let base_halo = (0.02 + atmo_height * (0.58 + 1.05 * atmo_haze_amount)).clamp(0.02, 0.75);
        let density_weight = (0.35
            + 0.40 * atmo_density
            + 0.25 * atmo_rayleigh_amount.max(atmo_haze_amount)
            + 0.08 * (limb_boost / 4.0))
            .clamp(0.35, 1.0);
        (base_halo * density_weight).clamp(0.0, 0.75)
    } else {
        0.0
    };
    let halo_shell = authored_halo_shell.max(derived_halo_shell);
    // Worldgen displacement scales to ~[-disp, +disp] on sphere radius, so keep
    // a conservative radial envelope for safe zoom limits.
    let displacement_shell = world_gen_displacement_scale
        .unwrap_or(0.0)
        .abs()
        .clamp(0.0, 1.0)
        * 0.9;
    let effective_radius = base_radius * (1.0 + atmo_shell + halo_shell + displacement_shell);

    let fov_rad = fov_degrees.unwrap_or(60.0).clamp(10.0, 170.0).to_radians();
    let half_fov_v = (fov_rad * 0.5).clamp(5.0_f32.to_radians(), 85.0_f32.to_radians());
    let aspect = estimated_aspect_ratio(size, width, height);
    let half_fov_h = (half_fov_v.tan() * aspect).atan();
    let limiting_half_fov = half_fov_v.min(half_fov_h).max(5.0_f32.to_radians());
    // For a sphere, apparent angular radius alpha satisfies sin(alpha)=r/d.
    // Keeping alpha within the limiting half-FOV avoids edge clipping.
    let fit_distance = effective_radius / limiting_half_fov.sin().max(0.05);
    let near_distance = effective_radius + near_clip.unwrap_or(0.001).max(0.0001) + 0.08;
    let safe_distance = fit_distance.max(near_distance) * 1.08 + 0.06;
    safe_distance.clamp(OBJ_ORBIT_DISTANCE_HARD_MIN, OBJ_ORBIT_DISTANCE_HARD_MAX)
}

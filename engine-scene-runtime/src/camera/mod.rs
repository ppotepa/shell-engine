use engine_core::scene::{FreeLookCameraControls, ObjOrbitCameraControls};
use engine_core::scene_runtime_types::SceneCamera3D;
use engine_events::{KeyEvent, KeyModifiers};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraSurfaceAnchor {
    pub center: [f32; 3],
    pub radius: f32,
}

impl CameraSurfaceAnchor {
    pub fn new(center: [f32; 3], radius: f32) -> Self {
        Self {
            center,
            radius: radius.max(0.001),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CameraInputFrame {
    pub key_presses: Vec<KeyEvent>,
    pub key_releases: Vec<KeyEvent>,
    pub mouse_moves: Vec<(f32, f32)>,
    pub scroll_deltas: Vec<f32>,
    pub ctrl_scroll_deltas: Vec<f32>,
    pub alt_left_mouse_downs: Vec<(f32, f32)>,
    pub left_mouse_ups: usize,
    pub focus_lost: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum CameraControllerKind {
    FreeLook,
    Orbit,
    Fps,
    Chase,
    Cockpit,
}

impl CameraControllerKind {
    pub fn from_camera_preset(preset: &str) -> Option<Self> {
        match normalize_camera_preset_name(preset).as_str() {
            "free-look-camera" | "surface-free-look" => Some(Self::FreeLook),
            "orbit-camera" => Some(Self::Orbit),
            _ => None,
        }
    }

    pub fn from_controller_kind_name(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "free_look" | "free-look" | "freelook" | "free-look-camera" | "surface-free-look" => {
                Some(Self::FreeLook)
            }
            "orbit" | "orbit-camera" => Some(Self::Orbit),
            "fps" | "first-person" | "first_person" => Some(Self::Fps),
            "chase" | "third-person" | "third_person" => Some(Self::Chase),
            "cockpit" => Some(Self::Cockpit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub(crate) struct ReservedCameraControllerState {
    pub kind: CameraControllerKind,
    pub id: String,
    pub target_pose: Option<SceneCamera3D>,
    pub applied_pose: Option<SceneCamera3D>,
    pub position_lag_sec: f32,
    pub aim_lag_sec: f32,
    pub chase_distance: f32,
    pub chase_height: f32,
    pub chase_look_ahead: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct FreeLookCameraState {
    pub active: bool,
    pub pending_activate: bool,
    pub drag_hold: bool,
    pub toggle_key: String,
    pub toggle_with_ctrl: bool,
    pub position: [f32; 3],
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub target_yaw_deg: f32,
    pub target_pitch_deg: f32,
    pub move_speed: f32,
    pub mouse_sensitivity: f32,
    pub surface_mode: bool,
    pub surface_center: [f32; 3],
    pub surface_radius: f32,
    pub surface_altitude: f32,
    pub surface_min_altitude: f32,
    pub surface_max_altitude: f32,
    pub surface_vertical_speed: f32,
    pub last_mouse_pos: Option<(f32, f32)>,
    pub held_keys: HashSet<String>,
}

impl FreeLookCameraState {
    pub fn from_controls(controls: &FreeLookCameraControls) -> Self {
        Self {
            active: false,
            pending_activate: controls.start_active,
            drag_hold: false,
            toggle_key: normalize_toggle_key_name(&controls.toggle_key),
            toggle_with_ctrl: controls.toggle_with_ctrl,
            position: [0.0, 0.0, 0.0],
            yaw_deg: 0.0,
            pitch_deg: 0.0,
            target_yaw_deg: 0.0,
            target_pitch_deg: 0.0,
            move_speed: controls.move_speed,
            mouse_sensitivity: controls.mouse_sensitivity,
            surface_mode: controls.surface_mode,
            surface_center: [
                controls.surface_center_x,
                controls.surface_center_y,
                controls.surface_center_z,
            ],
            surface_radius: controls.surface_radius.max(0.001),
            surface_altitude: controls.surface_altitude.max(0.0),
            surface_min_altitude: controls.surface_min_altitude.max(0.0),
            surface_max_altitude: controls.surface_max_altitude.max(0.0),
            surface_vertical_speed: controls.surface_vertical_speed.max(0.001),
            last_mouse_pos: None,
            held_keys: HashSet::new(),
        }
    }

    pub fn matches_toggle_key(&self, key: &KeyEvent) -> bool {
        let code_name = normalize_toggle_key_name(&key.code.to_string());
        if code_name.is_empty() || code_name != self.toggle_key {
            return false;
        }
        key.modifiers.contains(KeyModifiers::CONTROL) == self.toggle_with_ctrl
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ObjOrbitCameraState {
    pub target: String,
    pub active: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub applied_yaw: f32,
    pub applied_pitch: f32,
    pub applied_distance: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
    pub distance_min: f32,
    pub distance_max: f32,
    pub distance_step: f32,
    pub drag_sensitivity: f32,
    pub last_mouse_pos: Option<(f32, f32)>,
}

impl ObjOrbitCameraState {
    pub fn from_controls(controls: &ObjOrbitCameraControls) -> Self {
        Self {
            target: controls.target.clone(),
            active: true,
            yaw: controls.yaw,
            pitch: controls.pitch,
            distance: controls.distance,
            applied_yaw: controls.yaw,
            applied_pitch: controls.pitch,
            applied_distance: controls.distance,
            pitch_min: controls.pitch_min,
            pitch_max: controls.pitch_max,
            distance_min: controls.distance_min,
            distance_max: controls.distance_max,
            distance_step: controls.distance_step,
            drag_sensitivity: controls.drag_sensitivity,
            last_mouse_pos: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CameraDirectorRuntime {
    pub surface_anchor: Option<CameraSurfaceAnchor>,
    pub active_controller: Option<CameraControllerKind>,
    pub active_reserved_controller_id: Option<String>,
    pub adapter_step_consumed: bool,
    pub free_look: Option<FreeLookCameraState>,
    pub orbit: Option<ObjOrbitCameraState>,
    pub reserved: Vec<ReservedCameraControllerState>,
}

impl CameraDirectorRuntime {
    pub fn set_active_if_available(&mut self, kind: CameraControllerKind) {
        let available = match kind {
            CameraControllerKind::FreeLook => self.free_look.is_some(),
            CameraControllerKind::Orbit => self.orbit.is_some(),
            CameraControllerKind::Fps
            | CameraControllerKind::Chase
            | CameraControllerKind::Cockpit => self.reserved.iter().any(|entry| entry.kind == kind),
        };
        if available {
            self.active_controller = Some(kind);
            self.active_reserved_controller_id = None;
        }
    }

    pub fn select_active_controller_from_camera_preset(&mut self, camera_preset: Option<&str>) {
        if let Some(kind) = camera_preset.and_then(CameraControllerKind::from_camera_preset) {
            self.active_controller = Some(kind);
            self.active_reserved_controller_id = None;
            return;
        }
        self.select_legacy_default_controller();
    }

    fn select_legacy_default_controller(&mut self) {
        if self.orbit.is_some() {
            self.set_active_if_available(CameraControllerKind::Orbit);
        } else if self.free_look.is_some() {
            self.set_active_if_available(CameraControllerKind::FreeLook);
        }
    }

    pub fn begin_frame(&mut self) {
        self.adapter_step_consumed = false;
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn ensure_reserved_controller(
        &mut self,
        kind: CameraControllerKind,
        id: impl Into<String>,
    ) -> &mut ReservedCameraControllerState {
        let id = id.into();
        if let Some(index) = self
            .reserved
            .iter()
            .position(|entry| entry.kind == kind && entry.id == id)
        {
            return &mut self.reserved[index];
        }
        self.reserved.push(ReservedCameraControllerState {
            kind,
            id,
            target_pose: None,
            applied_pose: None,
            position_lag_sec: reserved_position_lag_sec(kind),
            aim_lag_sec: reserved_aim_lag_sec(kind),
            chase_distance: reserved_chase_distance(kind),
            chase_height: reserved_chase_height(kind),
            chase_look_ahead: reserved_chase_look_ahead(kind),
        });
        self.reserved
            .last_mut()
            .expect("reserved controller just pushed")
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn reserved_controller(
        &self,
        kind: CameraControllerKind,
        id: &str,
    ) -> Option<&ReservedCameraControllerState> {
        self.reserved
            .iter()
            .find(|entry| entry.kind == kind && entry.id == id)
    }
}

fn normalize_toggle_key_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("space") {
        return " ".to_string();
    }
    if trimmed.eq_ignore_ascii_case("up") || trimmed.eq_ignore_ascii_case("arrowup") {
        return "up".to_string();
    }
    if trimmed.eq_ignore_ascii_case("down") || trimmed.eq_ignore_ascii_case("arrowdown") {
        return "down".to_string();
    }
    if trimmed.eq_ignore_ascii_case("left") || trimmed.eq_ignore_ascii_case("arrowleft") {
        return "left".to_string();
    }
    if trimmed.eq_ignore_ascii_case("right") || trimmed.eq_ignore_ascii_case("arrowright") {
        return "right".to_string();
    }
    trimmed.to_ascii_lowercase()
}

fn normalize_camera_preset_name(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('_', "-")
}

fn reserved_position_lag_sec(kind: CameraControllerKind) -> f32 {
    match kind {
        CameraControllerKind::Fps => 0.03,
        CameraControllerKind::Chase => 0.14,
        CameraControllerKind::Cockpit => 0.07,
        CameraControllerKind::FreeLook | CameraControllerKind::Orbit => 0.0,
    }
}

fn reserved_aim_lag_sec(kind: CameraControllerKind) -> f32 {
    match kind {
        CameraControllerKind::Fps => 0.04,
        CameraControllerKind::Chase => 0.16,
        CameraControllerKind::Cockpit => 0.08,
        CameraControllerKind::FreeLook | CameraControllerKind::Orbit => 0.0,
    }
}

fn reserved_chase_distance(kind: CameraControllerKind) -> f32 {
    match kind {
        CameraControllerKind::Chase => 6.5,
        _ => 0.0,
    }
}

fn reserved_chase_height(kind: CameraControllerKind) -> f32 {
    match kind {
        CameraControllerKind::Chase => 2.2,
        _ => 0.0,
    }
}

fn reserved_chase_look_ahead(kind: CameraControllerKind) -> f32 {
    match kind {
        CameraControllerKind::Fps => 1.6,
        CameraControllerKind::Chase => 9.0,
        CameraControllerKind::Cockpit => 1.9,
        CameraControllerKind::FreeLook | CameraControllerKind::Orbit => 0.0,
    }
}

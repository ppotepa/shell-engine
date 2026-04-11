//! Pure data types shared between scene_runtime and behavior system.
//!
//! These types have no coupling to behavior or engine internals — they live
//! in engine-core so that both `engine-behavior-registry` and `engine` can
//! reference them without circular dependencies.

use crate::effects::Region;
use std::collections::{BTreeMap, HashMap};

/// Resolves authored target aliases to runtime object ids after scene materialization.
#[derive(Debug, Clone, Default)]
pub struct TargetResolver {
    scene_object_id: String,
    aliases: HashMap<String, String>,
    layer_ids: BTreeMap<usize, String>,
    sprite_ids: HashMap<String, String>,
}

impl TargetResolver {
    pub fn new(scene_object_id: String) -> Self {
        Self {
            scene_object_id,
            aliases: HashMap::new(),
            layer_ids: BTreeMap::new(),
            sprite_ids: HashMap::new(),
        }
    }

    pub fn from_parts(
        scene_object_id: String,
        aliases: HashMap<String, String>,
        layer_ids: BTreeMap<usize, String>,
        sprite_ids: HashMap<String, String>,
    ) -> Self {
        Self {
            scene_object_id,
            aliases,
            layer_ids,
            sprite_ids,
        }
    }

    /// Returns the runtime id of the scene root object.
    pub fn scene_object_id(&self) -> &str {
        &self.scene_object_id
    }

    /// Resolves an authored target alias or object id to its runtime object id.
    pub fn resolve_alias(&self, target: &str) -> Option<&str> {
        self.aliases.get(target).map(String::as_str)
    }

    pub fn register_alias(&mut self, alias: String, object_id: String) {
        self.aliases.insert(alias, object_id);
    }

    /// Returns a snapshot of all alias -> runtime object id bindings.
    pub fn aliases_snapshot(&self) -> HashMap<String, String> {
        self.aliases.clone()
    }

    /// Resolves a compositor layer index to its runtime layer object id.
    pub fn layer_object_id(&self, layer_idx: usize) -> Option<&str> {
        self.layer_ids.get(&layer_idx).map(String::as_str)
    }

    /// Resolves a sprite path within a layer to the corresponding runtime sprite object id.
    pub fn sprite_object_id(&self, layer_idx: usize, sprite_path: &[usize]) -> Option<&str> {
        self.sprite_ids
            .get(&path_key(layer_idx, sprite_path))
            .map(String::as_str)
    }

    /// Resolves the authored target region for an effect, falling back to the
    /// caller-provided default region when no target is bound.
    pub fn effect_region(
        &self,
        target: Option<&str>,
        default_region: Region,
        object_regions: &HashMap<String, Region>,
    ) -> Region {
        let Some(target) = target.filter(|v| !v.trim().is_empty()) else {
            return default_region;
        };
        self.resolve_alias(target)
            .and_then(|object_id| object_regions.get(object_id).copied())
            .unwrap_or(default_region)
    }

    pub fn register_layer(&mut self, layer_idx: usize, object_id: String) {
        self.layer_ids.insert(layer_idx, object_id);
    }

    pub fn register_sprite(&mut self, layer_idx: usize, sprite_path: &[usize], object_id: String) {
        self.sprite_ids
            .insert(path_key(layer_idx, sprite_path), object_id);
    }
}

fn path_key(layer_idx: usize, sprite_path: &[usize]) -> String {
    let mut key = layer_idx.to_string();
    for idx in sprite_path {
        key.push('/');
        key.push_str(&idx.to_string());
    }
    key
}

/// Runtime state accumulated by behaviors on top of the authored scene data.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectRuntimeState {
    pub visible: bool,
    pub offset_x: i32,
    pub offset_y: i32,
    /// Heading in radians, synced from `Transform2D` by `visual_sync_system`.
    /// Used to rotate vector sprites around their local origin at render time.
    pub heading: f32,
}

impl Default for ObjectRuntimeState {
    fn default() -> Self {
        Self {
            visible: true,
            offset_x: 0,
            offset_y: 0,
            heading: 0.0,
        }
    }
}

/// Domain-agnostic key event exposed to Rhai scripts.
#[derive(Debug, Clone, Default)]
pub struct RawKeyEvent {
    /// Key code as string: "a".."z", "0".."9", "Enter", "Backspace", "Tab",
    /// "Up", "Down", "Left", "Right", "Esc", "F1".."F12", etc.
    pub code: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// True for key-down, false for key-up.
    pub pressed: bool,
}

/// Sidecar IO frame snapshot: output lines, clear events, fullscreen mode, custom events.
#[derive(Debug, Clone, Default)]
pub struct SidecarIoFrameState {
    pub output_lines: Vec<String>,
    pub clear_count: u64,
    pub screen_full_lines: Option<Vec<String>>,
    pub custom_events: Vec<String>,
}

/// 3D OBJ camera state: pan, look (yaw/pitch), mouse tracking.
#[derive(Debug, Clone, Default)]
pub struct ObjCameraState {
    pub pan_x: f32,
    pub pan_y: f32,
    pub look_yaw: f32,
    pub look_pitch: f32,
    pub last_mouse_pos: Option<(u16, u16)>,
}

/// Shared scene-level 3D camera state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneCamera3D {
    pub eye: [f32; 3],
    pub look_at: [f32; 3],
    pub up: [f32; 3],
    pub fov_degrees: f32,
    pub near_clip: f32,
}

impl Default for SceneCamera3D {
    fn default() -> Self {
        Self {
            eye: [0.0, 0.0, 3.0],
            look_at: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov_degrees: 60.0,
            near_clip: 0.001,
        }
    }
}

impl SceneCamera3D {
    pub fn forward(&self) -> [f32; 3] {
        let dx = self.look_at[0] - self.eye[0];
        let dy = self.look_at[1] - self.eye[1];
        let dz = self.look_at[2] - self.eye[2];
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-6);
        [dx / len, dy / len, dz / len]
    }

    pub fn right(&self) -> [f32; 3] {
        let fwd = self.forward();
        let up = self.up;
        let dx = fwd[1] * up[2] - fwd[2] * up[1];
        let dy = fwd[2] * up[0] - fwd[0] * up[2];
        let dz = fwd[0] * up[1] - fwd[1] * up[0];
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-6);
        [dx / len, dy / len, dz / len]
    }
}

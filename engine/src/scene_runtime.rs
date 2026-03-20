//! Runtime scene materialization and object graph helpers derived from the
//! authored scene model.

use crate::behavior::{
    built_in_behavior, Behavior, BehaviorCommand, BehaviorContext, SceneAudioBehavior,
};
use crate::effects::Region;
use crate::game_object::{GameObject, GameObjectKind};
use crate::rasterizer::generic::GenericMode;
use crate::scene::{
    resolve_ui_theme_or_default, BehaviorSpec, Scene, SceneRenderedMode, Sprite, TermColour,
    TerminalShellControls, UiThemeStyle,
};
use crate::systems::animator::SceneStage;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::BTreeMap;
use tui_input::{Input, InputRequest};

/// Materialized runtime view of a [`Scene`] with stable object ids, behavior
/// bindings, and per-frame mutable state.
pub struct SceneRuntime {
    scene: Scene,
    root_id: String,
    objects: BTreeMap<String, GameObject>,
    object_states: BTreeMap<String, ObjectRuntimeState>,
    layer_ids: BTreeMap<usize, String>,
    sprite_ids: BTreeMap<String, String>,
    behaviors: Vec<ObjectBehaviorRuntime>,
    resolver_cache: TargetResolver,
    object_regions: BTreeMap<String, Region>,
    obj_orbit_default_speed: BTreeMap<String, f32>,
    obj_camera_states: BTreeMap<String, ObjCameraState>,
    terminal_shell_state: Option<TerminalShellState>,
    terminal_shell_scene_elapsed_ms: u64,
    ui_state: UiRuntimeState,
}

#[derive(Debug, Clone, Copy, Default)]
/// Mutable free-camera state tracked for interactive OBJ sprites.
pub struct ObjCameraState {
    pub pan_x: f32,
    pub pan_y: f32,
    pub look_yaw: f32,
    pub look_pitch: f32,
    pub last_mouse_pos: Option<(u16, u16)>,
}

#[derive(Debug, Clone, Default)]
/// Resolves authored target aliases to runtime object ids after scene
/// materialization.
pub struct TargetResolver {
    scene_object_id: String,
    aliases: BTreeMap<String, String>,
    layer_ids: BTreeMap<usize, String>,
    sprite_ids: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime state accumulated by behaviors on top of the authored scene data.
pub struct ObjectRuntimeState {
    pub visible: bool,
    pub offset_x: i32,
    pub offset_y: i32,
}

#[derive(Debug, Clone)]
struct TerminalShellState {
    controls: TerminalShellControls,
    input: Input,
    output_lines: Vec<String>,
    history: Vec<String>,
    history_cursor: Option<usize>,
    prompt_panel_height: Option<f32>,
    last_layout_sync_ms: u64,
}

#[derive(Debug, Clone, Copy)]
struct PanelLayoutSpec {
    width: u16,
    border_width: u16,
    padding: u16,
    height: u16,
}

#[derive(Debug, Clone)]
struct TextLayoutSpec {
    y: i32,
    font: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct ObjSpritePropertySnapshot {
    scale: Option<f32>,
    yaw: Option<f32>,
    pitch: Option<f32>,
    roll: Option<f32>,
    orbit_speed: Option<f32>,
    surface_mode: Option<String>,
}

#[derive(Debug, Clone)]
struct UiTextEvent {
    target_id: String,
    text: String,
}

#[derive(Debug, Clone, Default)]
struct UiRuntimeState {
    focus_order: Vec<String>,
    focused_index: usize,
    theme_id: Option<String>,
    theme_style: Option<UiThemeStyle>,
    last_submit: Option<UiTextEvent>,
    last_change: Option<UiTextEvent>,
    /// Last raw key press this frame — exposed to Rhai as `key { code, ctrl, alt, shift }`.
    pub last_raw_key: Option<RawKeyEvent>,
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
}

impl Default for ObjectRuntimeState {
    fn default() -> Self {
        Self {
            visible: true,
            offset_x: 0,
            offset_y: 0,
        }
    }
}

impl TerminalShellState {
    fn new(controls: TerminalShellControls) -> Self {
        let mut state = Self {
            output_lines: controls.banner.clone(),
            controls,
            input: Input::default(),
            history: Vec::new(),
            history_cursor: None,
            prompt_panel_height: None,
            last_layout_sync_ms: 0,
        };
        state.trim_output();
        state
    }

    fn prompt_line(&self, scene_elapsed_ms: u64) -> String {
        // Default shell prompt (`>`) uses a blinking marker.
        if self.controls.prompt_prefix.trim() == ">" {
            let blink_on = ((scene_elapsed_ms / 450) % 2) == 0;
            let prefix = if blink_on { ">" } else { " " };
            return format!("{prefix}{}", self.input.value());
        }
        format!("{}{}", self.controls.prompt_prefix, self.input.value())
    }

    fn trim_output(&mut self) {
        let max_lines = self.controls.max_lines.max(1);
        if self.output_lines.len() > max_lines {
            let drop_count = self.output_lines.len() - max_lines;
            self.output_lines.drain(0..drop_count);
        }
    }

    fn push_output_line(&mut self, line: String) {
        self.output_lines.push(line);
        self.trim_output();
    }

    fn clear_output(&mut self) {
        self.output_lines.clear();
    }

    fn output_text(&self) -> String {
        self.output_lines.join("\n")
    }

    fn execute_command(&mut self, raw_command: &str) {
        use crate::scene::TerminalShellMode;

        let command_line = raw_command.trim();
        if command_line.is_empty() {
            return;
        }

        self.push_output_line(format!("{}{}", self.controls.prompt_prefix, command_line));
        self.history.push(command_line.to_string());
        self.history_cursor = None;

        // In scripted mode, skip all built-in command execution.
        // Scripts should handle commands via ui.has_submit and ui.submit_text.
        if self.controls.mode == TerminalShellMode::Scripted {
            return;
        }

        let mut parts = command_line.split_whitespace();
        let command = parts.next().unwrap_or_default();
        let args = parts.collect::<Vec<_>>();

        if command.eq_ignore_ascii_case("clear") {
            self.clear_output();
            return;
        }

        if command.eq_ignore_ascii_case("help") {
            self.push_output_line("Built-ins: help, clear, ls, pwd, echo, whoami".to_string());
            if !self.controls.commands.is_empty() {
                let custom_lines: Vec<String> = self
                    .controls
                    .commands
                    .iter()
                    .map(|command| {
                        let description =
                            command.description.as_deref().unwrap_or("no description");
                        format!("  {} — {}", command.name, description)
                    })
                    .collect();
                self.push_output_line("Custom commands:".to_string());
                for line in custom_lines {
                    self.push_output_line(line);
                }
            }
            return;
        }

        if command.eq_ignore_ascii_case("pwd") {
            self.push_output_line("/world/terminal".to_string());
            return;
        }

        if command.eq_ignore_ascii_case("whoami") {
            self.push_output_line("operator".to_string());
            return;
        }

        if command.eq_ignore_ascii_case("echo") {
            self.push_output_line(args.join(" "));
            return;
        }

        if command.eq_ignore_ascii_case("ls") {
            if let Some(custom_lines) = self.custom_command_lines("ls") {
                for line in custom_lines {
                    self.push_output_line(line);
                }
            } else {
                self.push_output_line("logs  vault  airlock  notes".to_string());
            }
            return;
        }

        if let Some(custom_lines) = self.custom_command_lines(command) {
            for line in custom_lines {
                self.push_output_line(line);
            }
            return;
        }

        if let Some(message) = &self.controls.unknown_message {
            self.push_output_line(message.clone());
        } else {
            self.push_output_line(format!("unknown command: {command}"));
        }
    }

    fn custom_command_lines(&self, name: &str) -> Option<Vec<String>> {
        self.controls
            .commands
            .iter()
            .find(|command| command.name.eq_ignore_ascii_case(name))
            .and_then(|command| command.output.as_ref().map(|output| output.lines()))
    }
}

impl SceneRuntime {
    /// Materializes a runtime scene graph from the authored [`Scene`] model.
    ///
    /// # Invariants
    ///
    /// The runtime assigns stable ids to the scene root, layers, and sprites,
    /// attaches declared behaviors, and keeps resolver indices aligned with the
    /// z-sorted layer and sprite order used by the compositor.
    pub fn new(mut scene: Scene) -> Self {
        scene.layers.sort_by_key(|l| l.z_index);
        for layer in &mut scene.layers {
            layer.sprites.sort_by_key(|s| s.z_index());
        }
        let root_id = format!("scene:{}", sanitize_fragment(&scene.id));
        let mut objects = BTreeMap::new();
        let mut object_states = BTreeMap::new();
        let mut layer_ids = BTreeMap::new();
        let mut sprite_ids = BTreeMap::new();
        let mut behavior_bindings = Vec::new();
        insert_object(
            &mut objects,
            &mut object_states,
            GameObject {
                id: root_id.clone(),
                name: scene.id.clone(),
                kind: GameObjectKind::Scene,
                aliases: vec![scene.id.clone()],
                parent_id: None,
                children: Vec::new(),
            },
        );
        if !scene.behaviors.is_empty() {
            behavior_bindings.push(BehaviorBinding {
                object_id: root_id.clone(),
                specs: scene.behaviors.clone(),
            });
        }

        for (layer_idx, layer) in scene.layers.iter().enumerate() {
            let layer_name = if layer.name.trim().is_empty() {
                format!("layer-{layer_idx}")
            } else {
                layer.name.clone()
            };
            let layer_id = format!(
                "{root_id}/layer:{}:{}",
                layer_idx,
                sanitize_fragment(&layer_name)
            );
            insert_object(
                &mut objects,
                &mut object_states,
                GameObject {
                    id: layer_id.clone(),
                    name: layer_name,
                    kind: GameObjectKind::Layer,
                    aliases: layer_aliases(scene.layers[layer_idx].name.as_str()),
                    parent_id: Some(root_id.clone()),
                    children: Vec::new(),
                },
            );
            layer_ids.insert(layer_idx, layer_id.clone());

            if let Some(root) = objects.get_mut(&root_id) {
                root.children.push(layer_id.clone());
            }
            if !layer.behaviors.is_empty() {
                behavior_bindings.push(BehaviorBinding {
                    object_id: layer_id.clone(),
                    specs: layer.behaviors.clone(),
                });
            }

            for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
                build_sprite_objects(
                    &mut objects,
                    &mut object_states,
                    &mut sprite_ids,
                    &mut behavior_bindings,
                    layer_idx,
                    &[sprite_idx],
                    &layer_id,
                    sprite,
                    sprite_idx,
                );
            }
        }

        let mut runtime = Self {
            scene,
            root_id,
            objects,
            object_states,
            layer_ids,
            sprite_ids,
            behaviors: Vec::new(),
            resolver_cache: TargetResolver::default(),
            object_regions: BTreeMap::new(),
            obj_orbit_default_speed: BTreeMap::new(),
            obj_camera_states: BTreeMap::new(),
            terminal_shell_state: None,
            terminal_shell_scene_elapsed_ms: 0,
            ui_state: UiRuntimeState::default(),
        };
        runtime.obj_orbit_default_speed = collect_obj_orbit_defaults(&runtime.scene);
        runtime.terminal_shell_state = runtime
            .scene
            .input
            .terminal_shell
            .clone()
            .map(TerminalShellState::new);
        runtime.initialize_ui_state();
        runtime.sync_terminal_shell_sprites();
        runtime.attach_default_behaviors();
        runtime.attach_declared_behaviors(behavior_bindings);
        runtime.resolver_cache = runtime.build_target_resolver();
        runtime
    }

    /// Returns the runtime scene model after load-time normalization and sorting.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn set_scene_rendered_mode(&mut self, mode: SceneRenderedMode) {
        self.scene.rendered_mode = mode;
    }

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
    }

    /// Accumulate free-camera look rotation (degrees) for a sprite.
    pub fn apply_obj_camera_look(&mut self, sprite_id: &str, dyaw: f32, dpitch: f32) {
        let state = self
            .obj_camera_states
            .entry(sprite_id.to_string())
            .or_default();
        state.look_yaw += dyaw;
        state.look_pitch = (state.look_pitch + dpitch).clamp(-85.0, 85.0);
    }

    pub fn obj_camera_state(&self, sprite_id: &str) -> ObjCameraState {
        self.obj_camera_states
            .get(sprite_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn set_obj_last_mouse_pos(&mut self, sprite_id: &str, pos: Option<(u16, u16)>) {
        let state = self
            .obj_camera_states
            .entry(sprite_id.to_string())
            .or_default();
        state.last_mouse_pos = pos;
    }

    pub fn obj_last_mouse_pos(&self, sprite_id: &str) -> Option<(u16, u16)> {
        self.obj_camera_states
            .get(sprite_id)
            .and_then(|state| state.last_mouse_pos)
    }

    pub fn has_terminal_shell(&self) -> bool {
        self.terminal_shell_state.is_some()
    }

    /// Pushes a line to the terminal shell output transcript.
    /// Does nothing if no terminal shell is active.
    pub fn terminal_push_output(&mut self, line: String) {
        if let Some(state) = self.terminal_shell_state.as_mut() {
            state.push_output_line(line);
            self.sync_terminal_shell_sprites();
        }
    }

    /// Clears the terminal shell output transcript.
    /// Does nothing if no terminal shell is active.
    pub fn terminal_clear_output(&mut self) {
        if let Some(state) = self.terminal_shell_state.as_mut() {
            state.clear_output();
            self.sync_terminal_shell_sprites();
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
            if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                continue;
            }
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

    pub fn terminal_shell_back_requested(&self, key_presses: &[KeyEvent]) -> bool {
        let Some(state) = self.terminal_shell_state.as_ref() else {
            return false;
        };
        if !self.is_ui_target_focused(&state.controls.prompt_sprite_id) {
            return false;
        }
        if !state.input.value().is_empty() {
            return false;
        }
        key_presses.iter().any(|key| {
            matches!(key.code, KeyCode::Esc)
                || (matches!(key.code, KeyCode::Char('q' | 'Q'))
                    && key.modifiers.contains(KeyModifiers::CONTROL))
        })
    }

    pub fn handle_terminal_shell_keys(&mut self, key_presses: &[KeyEvent]) -> bool {
        let Some(prompt_id) = self
            .terminal_shell_state
            .as_ref()
            .map(|state| state.controls.prompt_sprite_id.clone())
        else {
            return false;
        };
        if !self.is_ui_target_focused(&prompt_id) {
            return false;
        }
        if key_presses.is_empty() {
            return false;
        }

        let (changed, submit_event, change_event) = {
            let Some(state) = self.terminal_shell_state.as_mut() else {
                return false;
            };

            let mut changed = false;
            let mut submit_event = None;
            let mut change_event = None;
            for key in key_presses {
                if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                    continue;
                }
                match key.code {
                    KeyCode::Esc => {
                        if !state.input.value().is_empty() {
                            state.input = Input::default();
                            state.history_cursor = None;
                            change_event = Some(UiTextEvent {
                                target_id: prompt_id.clone(),
                                text: String::new(),
                            });
                            changed = true;
                        }
                    }
                    KeyCode::Up => {
                        if !state.history.is_empty() {
                            let next_cursor = state
                                .history_cursor
                                .unwrap_or(state.history.len())
                                .saturating_sub(1)
                                .min(state.history.len() - 1);
                            state.history_cursor = Some(next_cursor);
                            state.input = Input::new(state.history[next_cursor].clone());
                            change_event = Some(UiTextEvent {
                                target_id: prompt_id.clone(),
                                text: state.input.value().to_string(),
                            });
                            changed = true;
                        }
                    }
                    KeyCode::Down => {
                        if let Some(cursor) = state.history_cursor {
                            let next = cursor + 1;
                            if next < state.history.len() {
                                state.history_cursor = Some(next);
                                state.input = Input::new(state.history[next].clone());
                            } else {
                                state.history_cursor = None;
                                state.input = Input::default();
                            }
                            change_event = Some(UiTextEvent {
                                target_id: prompt_id.clone(),
                                text: state.input.value().to_string(),
                            });
                            changed = true;
                        }
                    }
                    KeyCode::Enter => {
                        let command_line = state.input.value().to_string();
                        if !command_line.trim().is_empty() {
                            submit_event = Some(UiTextEvent {
                                target_id: prompt_id.clone(),
                                text: command_line.clone(),
                            });
                        }
                        state.execute_command(&command_line);
                        state.input = Input::default();
                        change_event = Some(UiTextEvent {
                            target_id: prompt_id.clone(),
                            text: String::new(),
                        });
                        changed = true;
                    }
                    _ => {
                        let before = state.input.value().to_string();
                        if let Some(request) = terminal_input_request(key) {
                            state.input.handle(request);
                        }
                        if state.input.value() != before {
                            state.history_cursor = None;
                            change_event = Some(UiTextEvent {
                                target_id: prompt_id.clone(),
                                text: state.input.value().to_string(),
                            });
                            changed = true;
                        }
                    }
                }
            }
            (changed, submit_event, change_event)
        };

        if let Some(event) = submit_event {
            self.ui_state.last_submit = Some(event);
        }
        if let Some(event) = change_event {
            self.ui_state.last_change = Some(event);
        }

        if changed {
            self.sync_terminal_shell_sprites();
        }
        changed
    }

    pub fn text_sprite_content(&self, sprite_id: &str) -> Option<&str> {
        for layer in &self.scene.layers {
            if let Some(content) = find_text_content(&layer.sprites, sprite_id) {
                return Some(content);
            }
        }
        None
    }

    /// Returns the runtime object id assigned to the scene root node.
    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    /// Looks up a materialized runtime object by its stable runtime id.
    pub fn object(&self, id: &str) -> Option<&GameObject> {
        self.objects.get(id)
    }

    /// Iterates over all materialized runtime objects in the scene graph.
    pub fn objects(&self) -> impl Iterator<Item = &GameObject> {
        self.objects.values()
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Returns the direct mutable runtime state stored on a specific object.
    pub fn object_state(&self, id: &str) -> Option<&ObjectRuntimeState> {
        self.object_states.get(id)
    }

    pub fn object_states_snapshot(&self) -> BTreeMap<String, ObjectRuntimeState> {
        self.object_states.clone()
    }

    pub fn object_kind_snapshot(&self) -> BTreeMap<String, String> {
        self.objects
            .iter()
            .map(|(id, object)| (id.clone(), object_kind_name(&object.kind).to_string()))
            .collect()
    }

    pub fn object_text_snapshot(&self) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for (object_id, object) in &self.objects {
            let Some(sprite_id) = object.aliases.first() else {
                continue;
            };
            let Some(content) = self.text_sprite_content(sprite_id) else {
                continue;
            };
            out.insert(object_id.clone(), content.to_string());
        }
        out
    }

    pub fn object_props_snapshot(&self) -> BTreeMap<String, JsonValue> {
        let mut out = BTreeMap::new();
        for (object_id, object) in &self.objects {
            let Some(sprite_id) = object.aliases.first() else {
                continue;
            };
            let mut props = JsonMap::new();
            if let Some((font, fg, bg)) = self.text_sprite_style(sprite_id) {
                let mut text = JsonMap::new();
                if let Some(value) = font {
                    text.insert("font".to_string(), JsonValue::String(value));
                }
                if let Some(value) = fg {
                    text.insert("fg".to_string(), term_colour_to_json(&value));
                }
                if let Some(value) = bg {
                    text.insert("bg".to_string(), term_colour_to_json(&value));
                }
                if !text.is_empty() {
                    props.insert("text".to_string(), JsonValue::Object(text.clone()));
                    props.insert("style".to_string(), JsonValue::Object(text));
                }
            }
            if let Some(obj) = self.obj_sprite_properties(sprite_id) {
                let mut obj_props = JsonMap::new();
                if let Some(value) = obj.scale {
                    obj_props.insert("scale".to_string(), JsonValue::from(value));
                }
                if let Some(value) = obj.yaw {
                    obj_props.insert("yaw".to_string(), JsonValue::from(value));
                }
                if let Some(value) = obj.pitch {
                    obj_props.insert("pitch".to_string(), JsonValue::from(value));
                }
                if let Some(value) = obj.roll {
                    obj_props.insert("roll".to_string(), JsonValue::from(value));
                }
                if let Some(value) = obj.orbit_speed {
                    obj_props.insert("orbit_speed".to_string(), JsonValue::from(value));
                }
                if let Some(value) = obj.surface_mode {
                    obj_props.insert("surface_mode".to_string(), JsonValue::String(value));
                }
                if !obj_props.is_empty() {
                    props.insert("obj".to_string(), JsonValue::Object(obj_props));
                }
            }
            if !props.is_empty() {
                out.insert(object_id.clone(), JsonValue::Object(props));
            }
        }
        out
    }

    fn text_sprite_style(
        &self,
        sprite_id: &str,
    ) -> Option<(Option<String>, Option<TermColour>, Option<TermColour>)> {
        self.scene
            .layers
            .iter()
            .find_map(|layer| find_text_style_recursive(&layer.sprites, sprite_id))
    }

    fn obj_sprite_properties(&self, sprite_id: &str) -> Option<ObjSpritePropertySnapshot> {
        self.scene
            .layers
            .iter()
            .find_map(|layer| find_obj_properties_recursive(&layer.sprites, sprite_id))
    }

    pub fn obj_camera_states_snapshot(&self) -> BTreeMap<String, ObjCameraState> {
        self.obj_camera_states.clone()
    }

    /// Returns the effective object state after inheriting visibility and
    /// offsets from all runtime parents.
    pub fn effective_object_state(&self, id: &str) -> Option<ObjectRuntimeState> {
        let mut state = self.object_states.get(id)?.clone();
        let mut parent_id = self
            .objects
            .get(id)
            .and_then(|object| object.parent_id.as_deref());

        while let Some(current_parent_id) = parent_id {
            let parent_state = self
                .object_states
                .get(current_parent_id)
                .cloned()
                .unwrap_or_default();
            state.visible &= parent_state.visible;
            state.offset_x = state.offset_x.saturating_add(parent_state.offset_x);
            state.offset_y = state.offset_y.saturating_add(parent_state.offset_y);
            parent_id = self
                .objects
                .get(current_parent_id)
                .and_then(|object| object.parent_id.as_deref());
        }

        Some(state)
    }

    /// Snapshots effective state for every runtime object for behavior and
    /// rendering consumers.
    pub fn effective_object_states_snapshot(&self) -> BTreeMap<String, ObjectRuntimeState> {
        self.objects
            .keys()
            .filter_map(|object_id| {
                self.effective_object_state(object_id)
                    .map(|state| (object_id.clone(), state))
            })
            .collect()
    }

    /// Returns a resolver for authored target names, layer indices, and sprite
    /// paths against the current materialized runtime scene.
    pub fn target_resolver(&self) -> TargetResolver {
        self.resolver_cache.clone()
    }

    fn build_target_resolver(&self) -> TargetResolver {
        let mut aliases = BTreeMap::new();

        for (object_id, object) in &self.objects {
            aliases.insert(object_id.clone(), object_id.clone());
            aliases.insert(object.name.clone(), object_id.clone());
            for alias in &object.aliases {
                aliases.insert(alias.clone(), object_id.clone());
            }
        }

        TargetResolver {
            scene_object_id: self.root_id.clone(),
            aliases,
            layer_ids: self.layer_ids.clone(),
            sprite_ids: self.sprite_ids.clone(),
        }
    }

    /// Updates attached runtime behaviors for the active scene stage and
    /// applies the generated commands immediately.
    pub fn update_behaviors(
        &mut self,
        stage: SceneStage,
        scene_elapsed_ms: u64,
        stage_elapsed_ms: u64,
        menu_selected_index: usize,
        game_state: Option<crate::game_state::GameState>,
    ) -> Vec<BehaviorCommand> {
        self.terminal_shell_scene_elapsed_ms = scene_elapsed_ms;
        self.sync_terminal_shell_sprites();
        // Clone once per frame — shared across all behavior ticks this frame.
        let resolver = self.resolver_cache.clone();
        let object_regions = self.object_regions.clone();
        let object_kinds = self.object_kind_snapshot();
        let object_props = self.object_props_snapshot();
        let object_text = self.object_text_snapshot();
        let ui_focused_target_id = self.focused_ui_target_id().map(str::to_string);
        let ui_last_submit = self.ui_state.last_submit.clone();
        let ui_last_change = self.ui_state.last_change.clone();
        let ui_theme_id = self.ui_state.theme_id.clone();
        let mut commands = Vec::new();
        let mut current_states = self.effective_object_states_snapshot();
        for idx in 0..self.behaviors.len() {
            let object_id = self.behaviors[idx].object_id.clone();
            let Some(object) = self.objects.get(&object_id).cloned() else {
                continue;
            };
            let ctx = BehaviorContext {
                stage,
                scene_elapsed_ms,
                stage_elapsed_ms,
                menu_selected_index,
                target_resolver: resolver.clone(),
                object_states: current_states.clone(),
                object_kinds: object_kinds.clone(),
                object_props: object_props.clone(),
                object_regions: object_regions.clone(),
                object_text: object_text.clone(),
                ui_focused_target_id: ui_focused_target_id.clone(),
                ui_theme_id: ui_theme_id.clone(),
                ui_last_submit_target_id: ui_last_submit
                    .as_ref()
                    .map(|event| event.target_id.clone()),
                ui_last_submit_text: ui_last_submit.as_ref().map(|event| event.text.clone()),
                ui_last_change_target_id: ui_last_change
                    .as_ref()
                    .map(|event| event.target_id.clone()),
                ui_last_change_text: ui_last_change.as_ref().map(|event| event.text.clone()),
                game_state: game_state.clone(),
                last_raw_key: self.ui_state.last_raw_key.clone(),
            };
            let mut local_commands = Vec::new();
            self.behaviors[idx]
                .behavior
                .update(&object, &self.scene, &ctx, &mut local_commands);
            self.apply_behavior_commands(&resolver, &local_commands);
            if idx + 1 < self.behaviors.len() {
                current_states = self.effective_object_states_snapshot();
            }
            commands.extend(local_commands.iter().cloned());
        }
        self.ui_state.last_submit = None;
        self.ui_state.last_change = None;
        commands
    }

    pub fn reset_frame_state(&mut self) {
        for state in self.object_states.values_mut() {
            *state = ObjectRuntimeState::default();
        }
        self.ui_state.last_raw_key = None;
    }

    fn sync_terminal_shell_sprites(&mut self) {
        let Some(mut state) = self.terminal_shell_state.clone() else {
            return;
        };
        let prompt_id = state.controls.prompt_sprite_id.clone();
        let output_id = state.controls.output_sprite_id.clone();
        let prompt_line = state.prompt_line(self.terminal_shell_scene_elapsed_ms);
        let controls = state.controls.clone();
        let prompt_rendered = self.render_prompt_for_panel(&prompt_line, &controls, &mut state);
        let output_text = state.output_text();
        let _ = self.set_text_sprite_content(&prompt_id, prompt_rendered);
        let _ = self.set_text_sprite_content(&output_id, output_text);
        self.terminal_shell_state = Some(state);
    }

    fn render_prompt_for_panel(
        &mut self,
        prompt_line: &str,
        controls: &TerminalShellControls,
        state: &mut TerminalShellState,
    ) -> String {
        let Some(panel_id) = controls.prompt_panel_id.as_deref() else {
            return prompt_line.to_string();
        };
        let Some(layout) = self.resolve_panel_layout(panel_id) else {
            return prompt_line.to_string();
        };
        let inset = u16::saturating_add(layout.border_width, layout.padding);
        let inner_width = layout.width.saturating_sub(inset.saturating_mul(2)).max(1) as usize;
        let mut lines = if controls.prompt_wrap {
            wrap_text_to_width(prompt_line, inner_width)
        } else {
            vec![prompt_line.to_string()]
        };
        if lines.is_empty() {
            lines.push(String::new());
        }
        let min_lines = controls.prompt_min_lines.max(1) as usize;
        let max_lines = controls
            .prompt_max_lines
            .max(1)
            .max(controls.prompt_min_lines) as usize;
        let target_lines = if controls.prompt_auto_grow {
            lines.len().clamp(min_lines, max_lines)
        } else {
            min_lines
        };
        if lines.len() > target_lines {
            let start = lines.len().saturating_sub(target_lines);
            lines = lines[start..].to_vec();
        }
        while lines.len() < target_lines {
            lines.push(String::new());
        }
        if controls.prompt_auto_grow {
            let prompt_layout = self.resolve_text_layout(&controls.prompt_sprite_id);
            let slot_offset = prompt_layout
                .as_ref()
                .map(|layout| layout.y.max(0) as u16)
                .unwrap_or(0);
            let line_height = prompt_layout
                .as_ref()
                .map(|layout| text_line_height_for_font(layout.font.as_deref()))
                .unwrap_or(1);
            let prompt_inner_height = slot_offset
                .saturating_add((target_lines as u16).saturating_mul(line_height.max(1)));
            let target_height = prompt_inner_height
                .saturating_add(inset.saturating_mul(2))
                .max(layout.height.max(3));
            self.animate_prompt_panel_height(panel_id, target_height, controls, state);
        }
        lines.join("\n")
    }

    fn animate_prompt_panel_height(
        &mut self,
        panel_id: &str,
        target_height: u16,
        controls: &TerminalShellControls,
        state: &mut TerminalShellState,
    ) {
        let previous = state.prompt_panel_height.unwrap_or(target_height as f32);
        let dt = self
            .terminal_shell_scene_elapsed_ms
            .saturating_sub(state.last_layout_sync_ms);
        let animated = if controls.prompt_growth_ms == 0 {
            target_height as f32
        } else {
            let alpha = (dt as f32 / controls.prompt_growth_ms as f32).clamp(0.0, 1.0);
            previous + (target_height as f32 - previous) * alpha
        };
        state.prompt_panel_height = Some(animated);
        state.last_layout_sync_ms = self.terminal_shell_scene_elapsed_ms;
        let next_height = animated.round().max(3.0) as u16;
        let _ = self.set_panel_sprite_height(panel_id, next_height);
        if let Some(shadow_panel_id) = controls.prompt_shadow_panel_id.as_deref() {
            let _ = self.set_panel_sprite_height(shadow_panel_id, next_height);
        }
    }

    fn resolve_panel_layout(&self, panel_id: &str) -> Option<PanelLayoutSpec> {
        let scene_width = self
            .object_regions
            .get(self.resolver_cache.scene_object_id())
            .map(|region| region.width)
            .unwrap_or(120);
        self.scene
            .layers
            .iter()
            .find_map(|layer| find_panel_layout_recursive(&layer.sprites, panel_id, scene_width))
    }

    fn set_panel_sprite_height(&mut self, panel_id: &str, next_height: u16) -> bool {
        let mut updated = false;
        for layer in &mut self.scene.layers {
            set_panel_height_recursive(&mut layer.sprites, panel_id, next_height, &mut updated);
        }
        updated
    }

    fn resolve_text_layout(&self, sprite_id: &str) -> Option<TextLayoutSpec> {
        self.scene
            .layers
            .iter()
            .find_map(|layer| find_text_layout_recursive(&layer.sprites, sprite_id))
    }

    fn initialize_ui_state(&mut self) {
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

    fn is_ui_target_focused(&self, target_id: &str) -> bool {
        self.focused_ui_target_id()
            .map(|focused| focused == target_id)
            .unwrap_or(true)
    }

    fn set_text_sprite_content(&mut self, sprite_id: &str, next_content: String) -> bool {
        let mut updated = false;
        for layer in &mut self.scene.layers {
            set_text_content_recursive(&mut layer.sprites, sprite_id, &next_content, &mut updated);
        }
        updated
    }

    fn set_text_sprite_font(&mut self, sprite_id: &str, next_font: String) -> bool {
        let mut updated = false;
        for layer in &mut self.scene.layers {
            set_text_font_recursive(&mut layer.sprites, sprite_id, &next_font, &mut updated);
        }
        updated
    }

    fn set_text_sprite_fg_colour(&mut self, sprite_id: &str, next_colour: TermColour) -> bool {
        let mut updated = false;
        for layer in &mut self.scene.layers {
            set_text_fg_recursive(&mut layer.sprites, sprite_id, &next_colour, &mut updated);
        }
        updated
    }

    fn set_text_sprite_bg_colour(&mut self, sprite_id: &str, next_colour: TermColour) -> bool {
        let mut updated = false;
        for layer in &mut self.scene.layers {
            set_text_bg_recursive(&mut layer.sprites, sprite_id, &next_colour, &mut updated);
        }
        updated
    }

    fn set_obj_sprite_property(&mut self, sprite_id: &str, path: &str, value: &JsonValue) -> bool {
        let mut updated = false;
        for layer in &mut self.scene.layers {
            set_obj_property_recursive(&mut layer.sprites, sprite_id, path, value, &mut updated);
        }
        updated
    }

    fn object_alias_candidates(&self, object_id: &str, target: &str) -> Vec<String> {
        let mut out = vec![target.to_string()];
        if let Some(object) = self.objects.get(object_id) {
            for alias in &object.aliases {
                if alias.trim().is_empty() || out.iter().any(|current| current == alias) {
                    continue;
                }
                out.push(alias.clone());
            }
        }
        out
    }

    fn apply_text_property_for_target(
        &mut self,
        object_id: &str,
        target: &str,
        mut apply: impl FnMut(&mut Self, &str) -> bool,
    ) -> bool {
        for alias in self.object_alias_candidates(object_id, target) {
            if apply(self, &alias) {
                return true;
            }
        }
        false
    }

    pub fn set_object_regions(&mut self, object_regions: BTreeMap<String, Region>) {
        self.object_regions = object_regions;
    }

    /// Applies behavior commands to runtime object state using the supplied
    /// target resolver.
    pub fn apply_behavior_commands(
        &mut self,
        resolver: &TargetResolver,
        commands: &[BehaviorCommand],
    ) {
        for command in commands {
            match command {
                BehaviorCommand::PlayAudioCue { .. } => {}
                BehaviorCommand::SetVisibility { target, visible } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        continue;
                    };
                    if let Some(state) = self.object_states.get_mut(object_id) {
                        state.visible = *visible;
                    }
                }
                BehaviorCommand::SetOffset { target, dx, dy } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        continue;
                    };
                    if let Some(state) = self.object_states.get_mut(object_id) {
                        state.offset_x = state.offset_x.saturating_add(*dx);
                        state.offset_y = state.offset_y.saturating_add(*dy);
                    }
                }
                BehaviorCommand::SetText { target, text } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        continue;
                    };
                    let _ =
                        self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                            runtime.set_text_sprite_content(alias, text.clone())
                        });
                }
                BehaviorCommand::SetProps {
                    target,
                    visible,
                    dx,
                    dy,
                    text,
                } => {
                    let resolved_object_id = resolver.resolve_alias(target).map(str::to_string);
                    if let Some(object_id) = resolved_object_id.as_deref() {
                        if let Some(state) = self.object_states.get_mut(object_id) {
                            if let Some(next_visible) = visible {
                                state.visible = *next_visible;
                            }
                            if let Some(delta_x) = dx {
                                state.offset_x = state.offset_x.saturating_add(*delta_x);
                            }
                            if let Some(delta_y) = dy {
                                state.offset_y = state.offset_y.saturating_add(*delta_y);
                            }
                        }
                    }
                    if let Some(next_text) = text {
                        let Some(object_id) = resolved_object_id.as_deref() else {
                            continue;
                        };
                        let _ = self.apply_text_property_for_target(
                            object_id,
                            target,
                            |runtime, alias| {
                                runtime.set_text_sprite_content(alias, next_text.clone())
                            },
                        );
                    }
                }
                BehaviorCommand::SetProperty {
                    target,
                    path,
                    value,
                } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        continue;
                    };
                    match path.as_str() {
                        "visible" => {
                            let Some(next_visible) = value.as_bool() else {
                                continue;
                            };
                            if let Some(state) = self.object_states.get_mut(object_id) {
                                state.visible = next_visible;
                            }
                        }
                        "offset.x" | "position.x" => {
                            let Some(next_x) = value.as_i64() else {
                                continue;
                            };
                            if let Some(state) = self.object_states.get_mut(object_id) {
                                state.offset_x = next_x as i32;
                            }
                        }
                        "offset.y" | "position.y" => {
                            let Some(next_y) = value.as_i64() else {
                                continue;
                            };
                            if let Some(state) = self.object_states.get_mut(object_id) {
                                state.offset_y = next_y as i32;
                            }
                        }
                        "text.content" => {
                            let Some(next_text) = value.as_str() else {
                                continue;
                            };
                            let _ = self.apply_text_property_for_target(
                                object_id,
                                target,
                                |runtime, alias| {
                                    runtime.set_text_sprite_content(alias, next_text.to_string())
                                },
                            );
                        }
                        "text.font" => {
                            let Some(next_font) = value.as_str() else {
                                continue;
                            };
                            let _ = self.apply_text_property_for_target(
                                object_id,
                                target,
                                |runtime, alias| {
                                    runtime.set_text_sprite_font(alias, next_font.to_string())
                                },
                            );
                        }
                        "style.fg" | "text.fg" => {
                            let Some(next_colour) = parse_term_colour(value) else {
                                continue;
                            };
                            let _ = self.apply_text_property_for_target(
                                object_id,
                                target,
                                |runtime, alias| {
                                    runtime.set_text_sprite_fg_colour(alias, next_colour.clone())
                                },
                            );
                        }
                        "style.bg" | "text.bg" => {
                            let Some(next_colour) = parse_term_colour(value) else {
                                continue;
                            };
                            let _ = self.apply_text_property_for_target(
                                object_id,
                                target,
                                |runtime, alias| {
                                    runtime.set_text_sprite_bg_colour(alias, next_colour.clone())
                                },
                            );
                        }
                        "obj.scale" | "obj.yaw" | "obj.pitch" | "obj.roll" | "obj.orbit_speed"
                        | "obj.surface_mode" => {
                            let mut applied = self.set_obj_sprite_property(target, path, value);
                            if !applied {
                                for alias in self.object_alias_candidates(object_id, target) {
                                    if self.set_obj_sprite_property(&alias, path, value) {
                                        applied = true;
                                        break;
                                    }
                                }
                            }
                            if !applied {
                                continue;
                            }
                        }
                        _ => {}
                    }
                }
                BehaviorCommand::TerminalPushOutput { line } => {
                    self.terminal_push_output(line.clone());
                }
                BehaviorCommand::TerminalClearOutput => {
                    self.terminal_clear_output();
                }
                // ScriptError is consumed at the behavior system level (world access needed).
                BehaviorCommand::ScriptError { .. } => {}
            }
        }
    }

    fn attach_default_behaviors(&mut self) {
        if has_scene_audio(&self.scene) {
            self.behaviors.push(ObjectBehaviorRuntime {
                object_id: self.root_id.clone(),
                behavior: Box::new(SceneAudioBehavior::default()),
            });
        }
    }

    fn attach_declared_behaviors(&mut self, bindings: Vec<BehaviorBinding>) {
        for binding in bindings {
            for spec in binding.specs {
                if let Some(behavior) = built_in_behavior(&spec) {
                    self.behaviors.push(ObjectBehaviorRuntime {
                        object_id: binding.object_id.clone(),
                        behavior,
                    });
                }
            }
        }
    }
}

fn terminal_input_request(key: &KeyEvent) -> Option<InputRequest> {
    use InputRequest::*;
    match (key.code, key.modifiers) {
        (_, _) if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => None,
        (KeyCode::Backspace, KeyModifiers::NONE) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
            Some(DeletePrevChar)
        }
        (KeyCode::Delete, KeyModifiers::NONE) => Some(DeleteNextChar),
        (KeyCode::Left, KeyModifiers::NONE) | (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
            Some(GoToPrevChar)
        }
        (KeyCode::Left, KeyModifiers::CONTROL) | (KeyCode::Char('b'), KeyModifiers::ALT) => {
            Some(GoToPrevWord)
        }
        (KeyCode::Right, KeyModifiers::NONE) | (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
            Some(GoToNextChar)
        }
        (KeyCode::Right, KeyModifiers::CONTROL) | (KeyCode::Char('f'), KeyModifiers::ALT) => {
            Some(GoToNextWord)
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => Some(DeleteLine),
        (KeyCode::Char('w'), KeyModifiers::CONTROL)
        | (KeyCode::Char('d'), KeyModifiers::ALT)
        | (KeyCode::Backspace, KeyModifiers::ALT) => Some(DeletePrevWord),
        (KeyCode::Delete, KeyModifiers::CONTROL) => Some(DeleteNextWord),
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => Some(DeleteTillEnd),
        (KeyCode::Char('a'), KeyModifiers::CONTROL) | (KeyCode::Home, KeyModifiers::NONE) => {
            Some(GoToStart)
        }
        (KeyCode::Char('e'), KeyModifiers::CONTROL) | (KeyCode::End, KeyModifiers::NONE) => {
            Some(GoToEnd)
        }
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => Some(InsertChar(c)),
        _ => None,
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

struct ObjectBehaviorRuntime {
    object_id: String,
    behavior: Box<dyn Behavior + Send + Sync>,
}

struct BehaviorBinding {
    object_id: String,
    specs: Vec<BehaviorSpec>,
}

fn has_scene_audio(scene: &Scene) -> bool {
    !scene.audio.on_enter.is_empty()
        || !scene.audio.on_idle.is_empty()
        || !scene.audio.on_leave.is_empty()
}

impl TargetResolver {
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
    pub fn aliases_snapshot(&self) -> BTreeMap<String, String> {
        self.aliases.clone()
    }

    /// Resolves a compositor layer index to its runtime layer object id.
    pub fn layer_object_id(&self, layer_idx: usize) -> Option<&str> {
        self.layer_ids.get(&layer_idx).map(String::as_str)
    }

    /// Resolves a sprite path within a layer to the corresponding runtime
    /// sprite object id.
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
        object_regions: &BTreeMap<String, Region>,
    ) -> Region {
        let Some(target) = target.filter(|value| !value.trim().is_empty()) else {
            return default_region;
        };
        self.resolve_alias(target)
            .and_then(|object_id| object_regions.get(object_id).copied())
            .unwrap_or(default_region)
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

fn build_sprite_objects(
    objects: &mut BTreeMap<String, GameObject>,
    object_states: &mut BTreeMap<String, ObjectRuntimeState>,
    sprite_ids: &mut BTreeMap<String, String>,
    behavior_bindings: &mut Vec<BehaviorBinding>,
    layer_idx: usize,
    sprite_path: &[usize],
    parent_id: &str,
    sprite: &Sprite,
    sprite_idx: usize,
) {
    let (kind, name, aliases) = sprite_descriptor(sprite, sprite_idx);
    let sprite_id = format!("{parent_id}/{name}");
    insert_object(
        objects,
        object_states,
        GameObject {
            id: sprite_id.clone(),
            name: name.clone(),
            kind,
            aliases,
            parent_id: Some(parent_id.to_string()),
            children: Vec::new(),
        },
    );
    sprite_ids.insert(path_key(layer_idx, sprite_path), sprite_id.clone());
    if !sprite.behaviors().is_empty() {
        behavior_bindings.push(BehaviorBinding {
            object_id: sprite_id.clone(),
            specs: sprite.behaviors().to_vec(),
        });
    }
    if let Some(parent) = objects.get_mut(parent_id) {
        parent.children.push(sprite_id.clone());
    }

    if let Sprite::Grid { children, .. }
    | Sprite::Flex { children, .. }
    | Sprite::Panel { children, .. } = sprite
    {
        for (child_idx, child) in children.iter().enumerate() {
            let mut child_path = sprite_path.to_vec();
            child_path.push(child_idx);
            build_sprite_objects(
                objects,
                object_states,
                sprite_ids,
                behavior_bindings,
                layer_idx,
                &child_path,
                &sprite_id,
                child,
                child_idx,
            );
        }
    }
}

fn insert_object(
    objects: &mut BTreeMap<String, GameObject>,
    object_states: &mut BTreeMap<String, ObjectRuntimeState>,
    object: GameObject,
) {
    object_states.insert(object.id.clone(), ObjectRuntimeState::default());
    objects.insert(object.id.clone(), object);
}

fn sprite_descriptor(sprite: &Sprite, sprite_idx: usize) -> (GameObjectKind, String, Vec<String>) {
    match sprite {
        Sprite::Text { id, .. } => (
            GameObjectKind::TextSprite,
            sprite_name("text", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Image { id, .. } => (
            GameObjectKind::ImageSprite,
            sprite_name("image", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Obj { id, .. } => (
            GameObjectKind::ObjSprite,
            sprite_name("obj", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Panel { id, .. } => (
            GameObjectKind::PanelSprite,
            sprite_name("panel", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Grid { id, .. } => (
            GameObjectKind::GridSprite,
            sprite_name("grid", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Flex { id, .. } => (
            GameObjectKind::FlexSprite,
            sprite_name("flex", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
    }
}

fn sprite_name(prefix: &str, explicit_id: Option<&str>, sprite_idx: usize) -> String {
    if let Some(id) = explicit_id.filter(|value| !value.trim().is_empty()) {
        format!("{prefix}:{}", sanitize_fragment(id))
    } else {
        format!("{prefix}:{sprite_idx}")
    }
}

fn sprite_aliases(explicit_id: Option<&str>) -> Vec<String> {
    explicit_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| vec![value.to_string()])
        .unwrap_or_default()
}

fn layer_aliases(name: &str) -> Vec<String> {
    if name.trim().is_empty() {
        Vec::new()
    } else {
        vec![name.to_string()]
    }
}

fn sanitize_fragment(input: &str) -> String {
    let sanitized = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':') {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let collapsed = sanitized
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if collapsed.is_empty() {
        "unnamed".to_string()
    } else {
        collapsed
    }
}

fn object_kind_name(kind: &GameObjectKind) -> &'static str {
    match kind {
        GameObjectKind::Scene => "scene",
        GameObjectKind::Layer => "layer",
        GameObjectKind::TextSprite => "text",
        GameObjectKind::ImageSprite => "image",
        GameObjectKind::ObjSprite => "obj",
        GameObjectKind::PanelSprite => "panel",
        GameObjectKind::GridSprite => "grid",
        GameObjectKind::FlexSprite => "flex",
    }
}

fn collect_obj_orbit_defaults(scene: &Scene) -> BTreeMap<String, f32> {
    let mut out = BTreeMap::new();
    for layer in &scene.layers {
        for_each_obj(&layer.sprites, &mut |sprite| {
            if let Sprite::Obj {
                id: Some(id),
                rotate_y_deg_per_sec,
                ..
            } = sprite
            {
                out.entry(id.to_string())
                    .or_insert(rotate_y_deg_per_sec.unwrap_or(20.0));
            }
        });
    }
    out
}

/// Visit every [`Sprite::Obj`] in a tree, recursing into grids.
fn for_each_obj(sprites: &[Sprite], f: &mut impl FnMut(&Sprite)) {
    for sprite in sprites {
        match sprite {
            Sprite::Obj { .. } => f(sprite),
            Sprite::Grid { children, .. } => for_each_obj(children, f),
            _ => {}
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

fn find_text_content<'a>(sprites: &'a [Sprite], sprite_id: &str) -> Option<&'a str> {
    for sprite in sprites {
        match sprite {
            Sprite::Text {
                id: Some(id),
                content,
                ..
            } if id == sprite_id => return Some(content.as_str()),
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if let Some(content) = find_text_content(children, sprite_id) {
                    return Some(content);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_text_layout_recursive(sprites: &[Sprite], sprite_id: &str) -> Option<TextLayoutSpec> {
    for sprite in sprites {
        match sprite {
            Sprite::Text {
                id: Some(id),
                y,
                font,
                ..
            } if id == sprite_id => {
                return Some(TextLayoutSpec {
                    y: *y,
                    font: font.clone(),
                });
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if let Some(layout) = find_text_layout_recursive(children, sprite_id) {
                    return Some(layout);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_text_style_recursive(
    sprites: &[Sprite],
    sprite_id: &str,
) -> Option<(Option<String>, Option<TermColour>, Option<TermColour>)> {
    for sprite in sprites {
        match sprite {
            Sprite::Text {
                id: Some(id),
                font,
                fg_colour,
                bg_colour,
                ..
            } if id == sprite_id => {
                return Some((font.clone(), fg_colour.clone(), bg_colour.clone()));
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if let Some(style) = find_text_style_recursive(children, sprite_id) {
                    return Some(style);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_obj_properties_recursive(
    sprites: &[Sprite],
    sprite_id: &str,
) -> Option<ObjSpritePropertySnapshot> {
    for sprite in sprites {
        match sprite {
            Sprite::Obj {
                id: Some(id),
                scale,
                yaw_deg,
                pitch_deg,
                roll_deg,
                rotate_y_deg_per_sec,
                surface_mode,
                ..
            } if id == sprite_id => {
                return Some(ObjSpritePropertySnapshot {
                    scale: *scale,
                    yaw: *yaw_deg,
                    pitch: *pitch_deg,
                    roll: *roll_deg,
                    orbit_speed: *rotate_y_deg_per_sec,
                    surface_mode: surface_mode.clone(),
                });
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if let Some(props) = find_obj_properties_recursive(children, sprite_id) {
                    return Some(props);
                }
            }
            _ => {}
        }
    }
    None
}

fn text_line_height_for_font(font: Option<&str>) -> u16 {
    let Some(font_name) = font else {
        return 1;
    };
    if !font_name.starts_with("generic") {
        return 1;
    }
    match GenericMode::from_font_name(font_name) {
        GenericMode::Tiny => 5,
        GenericMode::Standard => 7,
        GenericMode::Large => 14,
        GenericMode::Half => 4,
        GenericMode::Quad => 4,
        GenericMode::Braille => 2,
    }
}

fn set_text_content_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_content: &str,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Text {
                id: Some(id),
                content,
                ..
            } if id == sprite_id => {
                if content.as_str() != next_content {
                    *content = next_content.to_string();
                    *updated = true;
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_text_content_recursive(children, sprite_id, next_content, updated)
            }
            _ => {}
        }
    }
}

fn set_text_font_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_font: &str,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Text {
                id: Some(id), font, ..
            } if id == sprite_id => {
                if font.as_deref() != Some(next_font) {
                    *font = Some(next_font.to_string());
                    *updated = true;
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_text_font_recursive(children, sprite_id, next_font, updated)
            }
            _ => {}
        }
    }
}

fn set_text_fg_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_colour: &TermColour,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Text {
                id: Some(id),
                fg_colour,
                ..
            } if id == sprite_id => {
                if fg_colour.as_ref() != Some(next_colour) {
                    *fg_colour = Some(next_colour.clone());
                    *updated = true;
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_text_fg_recursive(children, sprite_id, next_colour, updated)
            }
            _ => {}
        }
    }
}

fn set_text_bg_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_colour: &TermColour,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Text {
                id: Some(id),
                bg_colour,
                ..
            } if id == sprite_id => {
                if bg_colour.as_ref() != Some(next_colour) {
                    *bg_colour = Some(next_colour.clone());
                    *updated = true;
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_text_bg_recursive(children, sprite_id, next_colour, updated)
            }
            _ => {}
        }
    }
}

fn set_obj_property_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    path: &str,
    value: &JsonValue,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Obj {
                id: Some(id),
                scale,
                yaw_deg,
                pitch_deg,
                roll_deg,
                rotate_y_deg_per_sec,
                surface_mode,
                ..
            } if id == sprite_id => match path {
                "obj.scale" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    let next = next.clamp(0.1, 8.0);
                    if (scale.unwrap_or(1.0) - next).abs() > f32::EPSILON {
                        *scale = Some(next);
                        *updated = true;
                    }
                }
                "obj.yaw" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    if (yaw_deg.unwrap_or(0.0) - next).abs() > f32::EPSILON {
                        *yaw_deg = Some(next);
                        *updated = true;
                    }
                }
                "obj.pitch" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    if (pitch_deg.unwrap_or(0.0) - next).abs() > f32::EPSILON {
                        *pitch_deg = Some(next);
                        *updated = true;
                    }
                }
                "obj.roll" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    if (roll_deg.unwrap_or(0.0) - next).abs() > f32::EPSILON {
                        *roll_deg = Some(next);
                        *updated = true;
                    }
                }
                "obj.orbit_speed" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    if (rotate_y_deg_per_sec.unwrap_or(0.0) - next).abs() > f32::EPSILON {
                        *rotate_y_deg_per_sec = Some(next);
                        *updated = true;
                    }
                }
                "obj.surface_mode" => {
                    let Some(next) = value.as_str() else {
                        continue;
                    };
                    if surface_mode.as_deref() != Some(next) {
                        *surface_mode = Some(next.to_string());
                        *updated = true;
                    }
                }
                _ => {}
            },
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_obj_property_recursive(children, sprite_id, path, value, updated);
            }
            _ => {}
        }
    }
}

fn parse_term_colour(value: &JsonValue) -> Option<TermColour> {
    let raw = value.as_str()?;
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "black" => Some(TermColour::Black),
        "white" => Some(TermColour::White),
        "gray" | "grey" => Some(TermColour::Gray),
        "silver" => Some(TermColour::Silver),
        "red" => Some(TermColour::Red),
        "green" => Some(TermColour::Green),
        "blue" => Some(TermColour::Blue),
        "yellow" => Some(TermColour::Yellow),
        "cyan" => Some(TermColour::Cyan),
        "magenta" => Some(TermColour::Magenta),
        _ => {
            let hex = normalized.strip_prefix('#')?;
            if hex.len() != 6 {
                return None;
            }
            let Ok(r) = u8::from_str_radix(&hex[0..2], 16) else {
                return None;
            };
            let Ok(g) = u8::from_str_radix(&hex[2..4], 16) else {
                return None;
            };
            let Ok(b) = u8::from_str_radix(&hex[4..6], 16) else {
                return None;
            };
            Some(TermColour::Rgb(r, g, b))
        }
    }
}

fn term_colour_to_json(colour: &TermColour) -> JsonValue {
    match colour {
        TermColour::Black => JsonValue::String("black".to_string()),
        TermColour::White => JsonValue::String("white".to_string()),
        TermColour::Gray => JsonValue::String("gray".to_string()),
        TermColour::Silver => JsonValue::String("silver".to_string()),
        TermColour::Red => JsonValue::String("red".to_string()),
        TermColour::Green => JsonValue::String("green".to_string()),
        TermColour::Blue => JsonValue::String("blue".to_string()),
        TermColour::Yellow => JsonValue::String("yellow".to_string()),
        TermColour::Cyan => JsonValue::String("cyan".to_string()),
        TermColour::Magenta => JsonValue::String("magenta".to_string()),
        TermColour::Rgb(r, g, b) => JsonValue::String(format!("#{r:02x}{g:02x}{b:02x}")),
    }
}

fn json_value_to_f32(value: &JsonValue) -> Option<f32> {
    value
        .as_f64()
        .map(|number| number as f32)
        .or_else(|| value.as_i64().map(|number| number as f32))
}

fn wrap_text_to_width(input: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out = Vec::new();
    for raw_line in input.split('\n') {
        if raw_line.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut current = String::new();
        let mut count = 0usize;
        for ch in raw_line.chars() {
            if count >= width {
                out.push(current);
                current = String::new();
                count = 0;
            }
            current.push(ch);
            count += 1;
        }
        out.push(current);
    }
    out
}

fn find_panel_layout_recursive(
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
            Sprite::Text { .. } | Sprite::Image { .. } | Sprite::Obj { .. } => {}
        }
    }
    None
}

fn set_panel_height_recursive(
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
            Sprite::Text { .. } | Sprite::Image { .. } | Sprite::Obj { .. } => {}
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
            Sprite::Text { .. } | Sprite::Image { .. } => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::SceneRuntime;
    use crate::behavior::BehaviorCommand;
    use crate::game_object::GameObjectKind;
    use crate::scene::{Scene, SceneRenderedMode, Sprite, TermColour};

    fn intro_scene() -> Scene {
        serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: UI
    sprites:
      - type: grid
        id: root-grid
        width: 10
        height: 5
        columns: ["1fr"]
        rows: ["1fr"]
        children:
          - type: text
            id: title
            content: HELLO
"#,
        )
        .expect("scene should parse")
    }

    fn obj_scene(extra_fields: &str) -> Scene {
        serde_yaml::from_str(&format!(
            r#"
id: playground-3d-scene
title: 3D
bg_colour: black
layers:
  - name: obj
    sprites:
      - type: obj
        id: helsinki-uni-wireframe
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
{extra_fields}"#
        ))
        .expect("scene should parse")
    }

    #[test]
    fn builds_object_hierarchy_for_layers_and_nested_sprites() {
        let runtime = SceneRuntime::new(intro_scene());

        assert_eq!(runtime.object_count(), 4);
        let root = runtime
            .object(runtime.root_id())
            .expect("scene root should exist");
        assert_eq!(root.kind, GameObjectKind::Scene);
        assert_eq!(root.children.len(), 1);

        let grid = runtime
            .objects()
            .find(|object| object.kind == GameObjectKind::GridSprite)
            .expect("grid object");
        assert_eq!(grid.children.len(), 1);

        let text = runtime
            .objects()
            .find(|object| object.kind == GameObjectKind::TextSprite)
            .expect("text object");
        assert_eq!(text.parent_id.as_deref(), Some(grid.id.as_str()));
    }

    #[test]
    fn target_resolver_supports_alias_lookup_and_sprite_paths() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: HUD
    sprites:
      - type: grid
        id: root-grid
        columns: ["1fr"]
        rows: ["1fr"]
        children:
          - type: text
            id: title
            content: HELLO
"#,
        )
        .expect("scene should parse");
        let runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();

        let title_id = resolver.resolve_alias("title").expect("title alias");
        assert_eq!(resolver.resolve_alias("HUD"), resolver.layer_object_id(0));
        assert_eq!(resolver.sprite_object_id(0, &[0, 0]), Some(title_id));
    }

    #[test]
    fn resolves_ui_theme_in_runtime_state() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: ui-theme-runtime
title: UI Theme Runtime
ui:
  theme: windows_98
layers: []
"#,
        )
        .expect("scene should parse");
        let runtime = SceneRuntime::new(scene);
        assert_eq!(runtime.ui_theme_id(), Some("win98"));
        let style = runtime.ui_theme_style().expect("theme style");
        assert_eq!(style.id, "win98");
    }

    #[test]
    fn falls_back_to_engine_default_theme_when_ui_theme_missing() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: ui-theme-runtime-default
title: UI Theme Runtime Default
layers: []
"#,
        )
        .expect("scene should parse");
        let runtime = SceneRuntime::new(scene);
        assert_eq!(runtime.ui_theme_id(), Some("engine-default"));
        let style = runtime.ui_theme_style().expect("theme style");
        assert_eq!(style.id, "engine-default");
    }

    #[test]
    fn effective_object_state_accumulates_parent_visibility_and_offsets() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SetOffset {
                    target: "intro".to_string(),
                    dx: 1,
                    dy: 0,
                },
                BehaviorCommand::SetVisibility {
                    target: "UI".to_string(),
                    visible: false,
                },
                BehaviorCommand::SetOffset {
                    target: "UI".to_string(),
                    dx: 2,
                    dy: 0,
                },
                BehaviorCommand::SetOffset {
                    target: "root-grid".to_string(),
                    dx: 3,
                    dy: 0,
                },
                BehaviorCommand::SetOffset {
                    target: "title".to_string(),
                    dx: 4,
                    dy: 0,
                },
            ],
        );

        let title_id = resolver.resolve_alias("title").expect("title id");
        let state = runtime
            .effective_object_state(title_id)
            .expect("effective state");

        assert!(!state.visible);
        assert_eq!(state.offset_x, 10);
        assert_eq!(state.offset_y, 0);
    }

    #[test]
    fn apply_behavior_commands_updates_text_content_from_set_text() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::SetText {
                target: "title".to_string(),
                text: "WORLD".to_string(),
            }],
        );
        assert_eq!(runtime.text_sprite_content("title"), Some("WORLD"));
    }

    #[test]
    fn apply_behavior_commands_updates_text_content_from_runtime_target_alias() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let title_runtime_id = resolver
            .resolve_alias("title")
            .expect("title runtime object id")
            .to_string();
        runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::SetText {
                target: title_runtime_id,
                text: "UPDATED".to_string(),
            }],
        );
        assert_eq!(runtime.text_sprite_content("title"), Some("UPDATED"));
    }

    #[test]
    fn apply_behavior_commands_set_props_updates_state_and_text() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[BehaviorCommand::SetProps {
                target: "title".to_string(),
                visible: Some(false),
                dx: Some(3),
                dy: Some(-1),
                text: Some("PROPS".to_string()),
            }],
        );
        assert_eq!(runtime.text_sprite_content("title"), Some("PROPS"));
        let title_id = resolver.resolve_alias("title").expect("title id");
        let state = runtime
            .object_state(title_id)
            .expect("object runtime state");
        assert!(!state.visible);
        assert_eq!(state.offset_x, 3);
        assert_eq!(state.offset_y, -1);
    }

    #[test]
    fn apply_behavior_commands_set_property_updates_runtime_object_paths() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "visible".to_string(),
                    value: serde_json::json!(false),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "position.x".to_string(),
                    value: serde_json::json!(9),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "position.y".to_string(),
                    value: serde_json::json!(-2),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "text.content".to_string(),
                    value: serde_json::json!("PATH-SET"),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "text.font".to_string(),
                    value: serde_json::json!("generic:half"),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "style.fg".to_string(),
                    value: serde_json::json!("yellow"),
                },
                BehaviorCommand::SetProperty {
                    target: "title".to_string(),
                    path: "style.bg".to_string(),
                    value: serde_json::json!("#112233"),
                },
            ],
        );
        assert_eq!(runtime.text_sprite_content("title"), Some("PATH-SET"));
        let title_id = resolver.resolve_alias("title").expect("title id");
        let state = runtime
            .object_state(title_id)
            .expect("object runtime state");
        assert!(!state.visible);
        assert_eq!(state.offset_x, 9);
        assert_eq!(state.offset_y, -2);
        let text_style = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Grid { children, .. } => children.iter().find_map(|child| match child {
                    Sprite::Text {
                        id,
                        font,
                        fg_colour,
                        bg_colour,
                        ..
                    } if id.as_deref() == Some("title") => {
                        Some((font.clone(), fg_colour.clone(), bg_colour.clone()))
                    }
                    _ => None,
                }),
                _ => None,
            })
            .expect("text style");
        assert_eq!(text_style.0.as_deref(), Some("generic:half"));
        assert_eq!(text_style.1, Some(TermColour::Yellow));
        assert_eq!(text_style.2, Some(TermColour::Rgb(0x11, 0x22, 0x33)));
    }

    #[test]
    fn apply_behavior_commands_set_property_updates_obj_paths() {
        let mut runtime = SceneRuntime::new(obj_scene(""));
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.scale".to_string(),
                    value: serde_json::json!(1.5),
                },
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.yaw".to_string(),
                    value: serde_json::json!(15),
                },
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.pitch".to_string(),
                    value: serde_json::json!(-10),
                },
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.roll".to_string(),
                    value: serde_json::json!(2),
                },
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.orbit_speed".to_string(),
                    value: serde_json::json!(22),
                },
                BehaviorCommand::SetProperty {
                    target: "helsinki-uni-wireframe".to_string(),
                    path: "obj.surface_mode".to_string(),
                    value: serde_json::json!("wireframe"),
                },
            ],
        );
        let obj_props = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    scale,
                    yaw_deg,
                    pitch_deg,
                    roll_deg,
                    rotate_y_deg_per_sec,
                    surface_mode,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => Some((
                    *scale,
                    *yaw_deg,
                    *pitch_deg,
                    *roll_deg,
                    *rotate_y_deg_per_sec,
                    surface_mode.clone(),
                )),
                _ => None,
            })
            .expect("obj properties");
        assert_eq!(obj_props.0, Some(1.5));
        assert_eq!(obj_props.1, Some(15.0));
        assert_eq!(obj_props.2, Some(-10.0));
        assert_eq!(obj_props.3, Some(2.0));
        assert_eq!(obj_props.4, Some(22.0));
        assert_eq!(obj_props.5.as_deref(), Some("wireframe"));
    }

    #[test]
    fn adjusts_obj_scale_for_target_sprite_id() {
        let mut runtime = SceneRuntime::new(obj_scene("        scale: 1.0"));
        runtime.set_scene_rendered_mode(SceneRenderedMode::Braille);
        assert!(runtime.adjust_obj_scale("helsinki-uni-wireframe", 0.2));

        let obj_scale = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj { id, scale, .. }
                    if id.as_deref() == Some("helsinki-uni-wireframe") =>
                {
                    *scale
                }
                _ => None,
            })
            .expect("obj scale");
        assert!((obj_scale - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn toggles_obj_surface_mode() {
        let mut runtime = SceneRuntime::new(obj_scene(""));
        assert!(runtime.toggle_obj_surface_mode("helsinki-uni-wireframe"));
        let mode = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id, surface_mode, ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => surface_mode.clone(),
                _ => None,
            })
            .expect("surface mode");
        assert_eq!(mode, "wireframe");
    }

    #[test]
    fn toggles_obj_orbit_speed_on_and_off() {
        let mut runtime = SceneRuntime::new(obj_scene("        rotate-y-deg-per-sec: 14"));
        assert!(runtime.toggle_obj_orbit("helsinki-uni-wireframe"));
        let speed_off = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    rotate_y_deg_per_sec,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => *rotate_y_deg_per_sec,
                _ => None,
            })
            .expect("orbit speed");
        assert!((speed_off - 0.0).abs() < f32::EPSILON);

        assert!(runtime.toggle_obj_orbit("helsinki-uni-wireframe"));
        let speed_on = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    rotate_y_deg_per_sec,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => *rotate_y_deg_per_sec,
                _ => None,
            })
            .expect("orbit speed");
        assert!((speed_on - 14.0).abs() < f32::EPSILON);
    }
}

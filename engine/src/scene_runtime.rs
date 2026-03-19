//! Runtime scene materialization and object graph helpers derived from the
//! authored scene model.

use crate::behavior::{
    built_in_behavior, Behavior, BehaviorCommand, BehaviorContext, SceneAudioBehavior,
};
use crate::effects::Region;
use crate::game_object::{GameObject, GameObjectKind};
use crate::scene::{
    resolve_ui_theme_or_default, BehaviorSpec, Scene, SceneRenderedMode, Sprite,
    TerminalShellControls, UiThemeStyle,
};
use crate::systems::animator::SceneStage;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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
        let command_line = raw_command.trim();
        if command_line.is_empty() {
            return;
        }

        self.push_output_line(format!("{}{}", self.controls.prompt_prefix, command_line));
        self.history.push(command_line.to_string());
        self.history_cursor = None;

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
    ) -> Vec<BehaviorCommand> {
        self.terminal_shell_scene_elapsed_ms = scene_elapsed_ms;
        self.sync_terminal_shell_sprites();
        // Clone once per frame — shared across all behavior ticks this frame.
        let resolver = self.resolver_cache.clone();
        let object_regions = self.object_regions.clone();
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
                object_regions: object_regions.clone(),
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
            let target_height = (target_lines as u16)
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
                *content = next_content.to_string();
                *updated = true;
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
    use crate::scene::{Scene, SceneRenderedMode, Sprite};

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

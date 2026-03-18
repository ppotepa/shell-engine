//! Runtime scene materialization and object graph helpers derived from the
//! authored scene model.

use crate::behavior::{
    built_in_behavior, Behavior, BehaviorCommand, BehaviorContext, SceneAudioBehavior,
};
use crate::effects::Region;
use crate::game_object::{GameObject, GameObjectKind};
use crate::scene::{BehaviorSpec, Scene, SceneRenderedMode, Sprite, TerminalShellControls};
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
        };
        state.trim_output();
        state
    }

    fn prompt_line(&self) -> String {
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
        };
        runtime.obj_orbit_default_speed = collect_obj_orbit_defaults(&runtime.scene);
        runtime.terminal_shell_state = runtime
            .scene
            .input
            .terminal_shell
            .clone()
            .map(TerminalShellState::new);
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

    pub fn handle_terminal_shell_keys(&mut self, key_presses: &[KeyEvent]) -> bool {
        let Some(state) = self.terminal_shell_state.as_mut() else {
            return false;
        };
        if key_presses.is_empty() {
            return false;
        }

        let mut changed = false;
        for key in key_presses {
            match key.code {
                KeyCode::Esc => {
                    if !state.input.value().is_empty() {
                        state.input = Input::default();
                        state.history_cursor = None;
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
                        changed = true;
                    }
                }
                KeyCode::Enter => {
                    let command_line = state.input.value().to_string();
                    state.execute_command(&command_line);
                    state.input = Input::default();
                    changed = true;
                }
                _ => {
                    let before = state.input.value().to_string();
                    if let Some(request) = terminal_input_request(key) {
                        state.input.handle(request);
                    }
                    if state.input.value() != before {
                        state.history_cursor = None;
                        changed = true;
                    }
                }
            }
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
        // Clone once per frame — shared across all behavior ticks this frame.
        let resolver = self.resolver_cache.clone();
        let object_regions = self.object_regions.clone();
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
        commands
    }

    pub fn reset_frame_state(&mut self) {
        for state in self.object_states.values_mut() {
            *state = ObjectRuntimeState::default();
        }
    }

    fn sync_terminal_shell_sprites(&mut self) {
        let Some(state) = self.terminal_shell_state.as_ref() else {
            return;
        };
        let prompt_id = state.controls.prompt_sprite_id.clone();
        let output_id = state.controls.output_sprite_id.clone();
        let prompt_line = state.prompt_line();
        let output_text = state.output_text();
        let _ = self.set_text_sprite_content(&prompt_id, prompt_line);
        let _ = self.set_text_sprite_content(&output_id, output_text);
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

    if let Sprite::Grid { children, .. } = sprite {
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
            Sprite::Grid { children, .. } | Sprite::Flex { children, .. } => {
                for_each_obj_mut(children, f)
            }
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
            Sprite::Grid { children, .. } | Sprite::Flex { children, .. } => {
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
            Sprite::Grid { children, .. } | Sprite::Flex { children, .. } => {
                set_text_content_recursive(children, sprite_id, next_content, updated)
            }
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
            Sprite::Grid { children, .. } | Sprite::Flex { children, .. } => {
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

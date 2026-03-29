//! Terminal shell state and command input handling.
//!
//! Manages the interactive terminal shell including:
//! - Shell output buffer and history
//! - Command input and masking
//! - Layout and rendering state for terminal UI elements
//!
//! This module encapsulates the state for the REPL-like shell interface.

use super::*;

impl TerminalShellState {
    pub(crate) fn new(controls: TerminalShellControls) -> Self {
        let mut state = Self {
            output_lines: controls.banner.clone(),
            controls,
            input: Input::default(),
            input_masked: false,
            sidecar_fullscreen_mode: false,
            history: Vec::new(),
            history_cursor: None,
            prompt_panel_height: None,
            last_layout_sync_ms: 0,
        };
        state.trim_output();
        state
    }

    fn prompt_line(&self, scene_elapsed_ms: u64) -> String {
        let raw_input = if self.input_masked {
            "*".repeat(self.input.value().chars().count())
        } else {
            self.input.value().to_string()
        };

        // Dim user input by 10% relative to the sprite base fg to give it
        // a subtle CRT-styled feel.
        let input_value = if raw_input.is_empty() {
            raw_input
        } else {
            format!("[#adadad]{}[/]", raw_input)
        };

        // Default shell prompt (`>`) uses a blinking marker.
        if self.controls.prompt_prefix.trim() == ">" {
            let blink_on = ((scene_elapsed_ms / 450) % 2) == 0;
            let prefix = if blink_on { ">" } else { " " };
            return format!("{prefix}{input_value}");
        }
        format!("{}{}", self.controls.prompt_prefix, input_value)
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
        use engine_core::scene::TerminalShellMode;

        let command_line = raw_command.trim();
        if command_line.is_empty() {
            return;
        }

        // Track history for Up/Down even when command execution is external.
        self.history.push(command_line.to_string());
        self.history_cursor = None;

        match self.controls.mode {
            TerminalShellMode::Sidecar => {
                // External process owns transcript + semantics.
                return;
            }
            TerminalShellMode::Scripted => {
                // Scripts own semantics but we still echo the submitted line into the transcript.
                self.push_output_line(format!("{}{}", self.controls.prompt_prefix, command_line));
                return;
            }
            TerminalShellMode::Builtin => {
                self.push_output_line(format!("{}{}", self.controls.prompt_prefix, command_line));
            }
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
    pub fn has_terminal_shell(&self) -> bool {
        self.terminal_shell_state.is_some()
    }

    pub fn terminal_shell_controls_snapshot(&self) -> Option<TerminalShellControls> {
        self.terminal_shell_state
            .as_ref()
            .map(|state| state.controls.clone())
    }

    /// Pushes a line to the terminal shell output transcript.
    /// Does nothing if no terminal shell is active.
    pub fn terminal_push_output(&mut self, line: String) {
        self.cached_sidecar_io = None;
        self.ui_state.sidecar_io.screen_full_lines = None;
        self.ui_state.sidecar_io.output_lines.push(line.clone());
        if let Some(state) = self.terminal_shell_state.as_mut() {
            state.sidecar_fullscreen_mode = false;
            state.push_output_line(line);
            self.sync_terminal_shell_sprites();
        }
    }

    /// Clears the terminal shell output transcript.
    /// Does nothing if no terminal shell is active.
    pub fn terminal_clear_output(&mut self) {
        self.cached_sidecar_io = None;
        if let Some(state) = self.terminal_shell_state.as_mut() {
            state.sidecar_fullscreen_mode = false;
            state.clear_output();
            self.sync_terminal_shell_sprites();
        }
        self.ui_state.sidecar_io.screen_full_lines = None;
        self.ui_state.sidecar_io.clear_count =
            self.ui_state.sidecar_io.clear_count.saturating_add(1);
    }

    pub fn terminal_set_prompt_prefix(&mut self, prefix: String) {
        if let Some(state) = self.terminal_shell_state.as_mut() {
            state.controls.prompt_prefix = prefix;
            self.sync_terminal_shell_sprites();
        }
    }

    pub fn terminal_set_prompt_masked(&mut self, masked: bool) {
        if let Some(state) = self.terminal_shell_state.as_mut() {
            state.input_masked = masked;
            self.sync_terminal_shell_sprites();
        }
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
            self.ui_state.submit_seq = self.ui_state.submit_seq.saturating_add(1);
            self.ui_state.last_submit = Some(event);
        }
        if let Some(event) = change_event {
            self.ui_state.change_seq = self.ui_state.change_seq.saturating_add(1);
            self.ui_state.last_change = Some(event);
        }

        if changed {
            self.sync_terminal_shell_sprites();
        }
        changed
    }

    pub fn sidecar_mark_screen_full(&mut self, lines: Vec<String>) {
        self.cached_sidecar_io = None;
        self.ui_state.sidecar_io.screen_full_lines = Some(lines);
        if let Some(state) = self.terminal_shell_state.as_mut() {
            state.sidecar_fullscreen_mode = true;
            state.output_lines = self
                .ui_state
                .sidecar_io
                .screen_full_lines
                .clone()
                .unwrap_or_default();
            self.sync_terminal_shell_sprites();
        }
    }

    pub fn sidecar_push_custom_event(&mut self, payload: String) {
        self.cached_sidecar_io = None;
        self.ui_state.sidecar_io.custom_events.push(payload);
    }

    pub(crate) fn sync_terminal_shell_sprites(&mut self) {
        let Some(mut state) = self.terminal_shell_state.clone() else {
            return;
        };
        let prompt_id = state.controls.prompt_sprite_id.clone();
        let output_id = state.controls.output_sprite_id.clone();
        let prompt_line = state.prompt_line(self.terminal_shell_scene_elapsed_ms);
        let controls = state.controls.clone();
        if matches!(
            state.controls.mode,
            engine_core::scene::TerminalShellMode::Sidecar
        ) {
            if state.sidecar_fullscreen_mode {
                let output_text = self.viewport_clipped_output(&state);
                let _ = self.set_text_sprite_content(&output_id, output_text);
                let _ = self.set_text_sprite_content(&prompt_id, String::new());
                self.terminal_shell_state = Some(state);
                return;
            }
            let (output_text, prompt_rendered) =
                self.render_terminal_stacked_output_and_prompt(&state, &prompt_line);
            let _ = self.set_text_sprite_content(&output_id, output_text);
            let _ = self.set_text_sprite_content(&prompt_id, prompt_rendered);
        } else {
            let prompt_rendered = self.render_prompt_for_panel(&prompt_line, &controls, &mut state);
            let output_text = state.output_text();
            let _ = self.set_text_sprite_content(&prompt_id, prompt_rendered);
            let _ = self.set_text_sprite_content(&output_id, output_text);
        }
        self.terminal_shell_state = Some(state);
    }

    fn render_terminal_stacked_output_and_prompt(
        &self,
        state: &TerminalShellState,
        prompt_line: &str,
    ) -> (String, String) {
        let Some(output_layout) = self.resolve_text_layout(&state.controls.output_sprite_id) else {
            return (state.output_text(), prompt_line.to_string());
        };
        let Some(prompt_layout) = self.resolve_text_layout(&state.controls.prompt_sprite_id) else {
            return (state.output_text(), prompt_line.to_string());
        };

        // Compute available character width for word-wrapping.
        let scene_width = self
            .object_regions
            .get(self.resolver_cache.scene_object_id())
            .map(|r| r.width)
            .unwrap_or(120);
        let cell_w = text_cell_width_for_font(output_layout.font.as_deref()).max(1) as usize;
        let start_x = output_layout.x.max(0) as u16;
        let usable = scene_width.saturating_sub(start_x).max(1) as usize;
        let wrap_width = (usable / cell_w).max(1);

        let line_height = 1u16;
        let vertical_space = prompt_layout.y.saturating_sub(output_layout.y).max(1) as u16;
        let viewport_lines = (vertical_space / line_height).max(1) as usize;
        let target_rows = viewport_lines.min(state.controls.max_lines.max(1) as usize);
        if target_rows <= 1 {
            return (state.output_text(), String::new());
        }

        // Reserve last row for prompt, render transcript top-to-bottom above it.
        let transcript_rows = target_rows - 1;
        let wrapped: Vec<String> = state
            .output_lines
            .iter()
            .flat_map(|line| wrap_text_to_width(line, wrap_width))
            .collect();
        let lines: Vec<String> = if wrapped.len() <= transcript_rows {
            wrapped
        } else {
            wrapped[wrapped.len() - transcript_rows..].to_vec()
        };
        (lines.join("\n"), prompt_line.to_string())
    }

    /// Clip output lines to the available viewport for fullscreen sidecar mode.
    /// Uses the vertical distance between the output and prompt sprites (the
    /// same area the non-fullscreen path calculates), falling back to
    /// `max_lines` when layout info is unavailable.
    ///
    /// Long lines are word-wrapped to the available character width so they
    /// never overflow the panel boundary.
    fn viewport_clipped_output(&self, state: &TerminalShellState) -> String {
        let output_layout = self.resolve_text_layout(&state.controls.output_sprite_id);
        let viewport_rows = output_layout
            .as_ref()
            .and_then(|out_layout| {
                self.resolve_text_layout(&state.controls.prompt_sprite_id)
                    .map(|prm_layout| prm_layout.y.saturating_sub(out_layout.y).max(1) as usize)
            })
            .unwrap_or(state.controls.max_lines.max(1));

        // Determine available character width for word-wrapping.
        let scene_width = self
            .object_regions
            .get(self.resolver_cache.scene_object_id())
            .map(|r| r.width)
            .unwrap_or(120);
        let wrap_width = output_layout
            .as_ref()
            .map(|layout| {
                let cell_w = text_cell_width_for_font(layout.font.as_deref()).max(1) as usize;
                let start_x = layout.x.max(0) as u16;
                let usable = scene_width.saturating_sub(start_x).max(1) as usize;
                (usable / cell_w).max(1)
            })
            .unwrap_or(scene_width as usize);

        // Word-wrap each line, then take the last N rows.
        let wrapped: Vec<String> = state
            .output_lines
            .iter()
            .flat_map(|line| wrap_text_to_width(line, wrap_width))
            .collect();

        let rows = viewport_rows.min(state.controls.max_lines.max(1));
        if wrapped.len() <= rows {
            wrapped.join("\n")
        } else {
            wrapped[wrapped.len() - rows..].join("\n")
        }
    }

    fn render_prompt_for_panel(
        &mut self,
        prompt_line: &str,
        controls: &TerminalShellControls,
        state: &mut TerminalShellState,
    ) -> String {
        let Some(panel_id) = controls.prompt_panel_id.as_deref() else {
            return self.render_prompt_tail_in_viewport(prompt_line, &controls.prompt_sprite_id);
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

    fn render_prompt_tail_in_viewport(&self, prompt_line: &str, prompt_sprite_id: &str) -> String {
        let Some(layout) = self.resolve_text_layout(prompt_sprite_id) else {
            return prompt_line.to_string();
        };
        let scene_width = self
            .object_regions
            .get(self.resolver_cache.scene_object_id())
            .map(|region| region.width)
            .unwrap_or(120);
        let cell_width = text_cell_width_for_font(layout.font.as_deref()) as usize;
        let start_x = layout.x.max(0) as u16;
        let usable_cells = scene_width.saturating_sub(start_x).max(1) as usize;
        let max_chars = (usable_cells / cell_width.max(1)).max(1);
        let total_chars = prompt_line.chars().count();
        if total_chars <= max_chars {
            return prompt_line.to_string();
        }
        prompt_line
            .chars()
            .skip(total_chars - max_chars)
            .collect::<String>()
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
}

fn terminal_input_request(key: &KeyEvent) -> Option<InputRequest> {
    use InputRequest::*;
    match (key.code, key.modifiers) {
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

fn text_cell_width_for_font(font: Option<&str>) -> u16 {
    let Some(font_name) = font else {
        return 1;
    };
    if !font_name.starts_with("generic") {
        return 1;
    }
    match GenericMode::from_font_name(font_name) {
        GenericMode::Tiny => 4,
        GenericMode::Standard => 6,
        GenericMode::Large => 12,
        GenericMode::Half => 6,
        GenericMode::Quad => 3,
        GenericMode::Braille => 3,
    }
}

pub(crate) fn wrap_text_to_width(input: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out = Vec::new();
    for raw_line in input.split('\n') {
        if raw_line.is_empty() {
            out.push(String::new());
            continue;
        }
        let visible_len = engine_core::markup::strip_markup(raw_line).chars().count();
        if visible_len <= width {
            out.push(raw_line.to_string());
            continue;
        }
        let tokens = tokenize_markup_words(raw_line);
        let mut line_buf = String::new();
        let mut line_vis = 0usize;
        let mut open_tag: Option<String> = None;

        for token in &tokens {
            match token {
                WrapToken::Tag { raw, is_close } => {
                    if *is_close {
                        open_tag = None;
                    } else {
                        open_tag = Some(raw.clone());
                    }
                    line_buf.push('[');
                    line_buf.push_str(raw);
                    line_buf.push(']');
                }
                WrapToken::Word(word) => {
                    let wlen = word.chars().count();
                    if wlen == 0 {
                        continue;
                    }
                    if line_vis + wlen <= width {
                        line_buf.push_str(word);
                        line_vis += wlen;
                        continue;
                    }
                    if wlen <= width && line_vis > 0 {
                        emit_wrapped_line(&mut out, &mut line_buf, &open_tag);
                        reopen_tag(&mut line_buf, &open_tag);
                        line_vis = 0;
                        line_buf.push_str(word);
                        line_vis += wlen;
                        continue;
                    }
                    // Word too long — hard-break character by character.
                    for ch in word.chars() {
                        if line_vis >= width {
                            emit_wrapped_line(&mut out, &mut line_buf, &open_tag);
                            reopen_tag(&mut line_buf, &open_tag);
                            line_vis = 0;
                        }
                        line_buf.push(ch);
                        line_vis += 1;
                    }
                }
                WrapToken::Space(sp) => {
                    let slen = sp.chars().count();
                    if line_vis + slen > width {
                        continue;
                    }
                    line_buf.push_str(sp);
                    line_vis += slen;
                }
            }
        }
        if !line_buf.is_empty() {
            out.push(line_buf);
        }
    }
    out
}

fn emit_wrapped_line(out: &mut Vec<String>, line_buf: &mut String, open_tag: &Option<String>) {
    if open_tag.is_some() {
        line_buf.push_str("[/]");
    }
    let line = std::mem::take(line_buf);
    out.push(line.trim_end().to_string());
}

fn reopen_tag(line_buf: &mut String, open_tag: &Option<String>) {
    if let Some(ref t) = open_tag {
        line_buf.push('[');
        line_buf.push_str(t);
        line_buf.push(']');
    }
}

#[derive(Debug)]
enum WrapToken {
    Tag { raw: String, is_close: bool },
    Word(String),
    Space(String),
}

fn tokenize_markup_words(input: &str) -> Vec<WrapToken> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    let mut in_space = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            let mut tag = String::new();
            let mut closed = false;
            for tc in chars.by_ref() {
                if tc == ']' {
                    closed = true;
                    break;
                }
                tag.push(tc);
            }
            if closed {
                if !buf.is_empty() {
                    tokens.push(if in_space {
                        WrapToken::Space(std::mem::take(&mut buf))
                    } else {
                        WrapToken::Word(std::mem::take(&mut buf))
                    });
                }
                in_space = false;
                tokens.push(WrapToken::Tag {
                    is_close: tag.starts_with('/'),
                    raw: tag,
                });
            } else {
                buf.push('[');
                buf.push_str(&tag);
            }
            continue;
        }
        let is_ws = ch == ' ' || ch == '\t';
        if is_ws != in_space && !buf.is_empty() {
            tokens.push(if in_space {
                WrapToken::Space(std::mem::take(&mut buf))
            } else {
                WrapToken::Word(std::mem::take(&mut buf))
            });
        }
        in_space = is_ws;
        buf.push(ch);
    }
    if !buf.is_empty() {
        tokens.push(if in_space {
            WrapToken::Space(buf)
        } else {
            WrapToken::Word(buf)
        });
    }
    tokens
}

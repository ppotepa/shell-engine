//! Behavior runtime and command application.
//!
//! Handles:
//! - Attaching behaviors to objects (built-in and mod-defined)
//! - Updating behavior state each frame
//! - Applying behavior-generated commands to scene state
//!
//! Note: This module contains the mutable state update logic for behaviors.
//! It is tightly coupled to behavior types and command processing.

use super::*;
use crate::mutations::SetSpritePropertyMutation;

impl SceneRuntime {
    /// Updates attached runtime behaviors for the active scene stage and
    /// applies the generated commands immediately.
    #[allow(clippy::too_many_arguments)]
    pub fn update_behaviors(
        &mut self,
        stage: SceneStage,
        scene_elapsed_ms: u64,
        stage_elapsed_ms: u64,
        menu_selected_index: usize,
        game_state: Option<engine_core::game_state::GameState>,
        level_state: Option<engine_core::level_state::LevelState>,
        persistence: Option<engine_persistence::PersistenceStore>,
        gameplay_world: Option<engine_game::GameplayWorld>,
        emitter_state: Option<engine_behavior::EmitterState>,
        collisions: std::sync::Arc<Vec<engine_game::CollisionHit>>,
        catalogs: std::sync::Arc<engine_behavior::catalog::ModCatalogs>,
        palettes: std::sync::Arc<engine_behavior::palette::PaletteStore>,
        default_palette: Option<String>,
        debug_enabled: bool,
    ) -> Vec<BehaviorCommand> {
        // reset_frame_state() reinitializes object runtime state each frame, so
        // state-derived snapshots must be rebuilt even on mutation-free frames.
        self.effective_states_dirty = true;
        self.cached_object_states = None;
        self.cached_effective_states = None;
        // sidecar_io: build Arc once if not already cached from a prior
        // mutation-free frame; invalidated at each sidecar write site.
        let sidecar_io = match &self.cached_sidecar_io {
            Some(cached) => std::sync::Arc::clone(cached),
            None => {
                let arc = std::sync::Arc::new(self.ui_state.sidecar_io.clone());
                self.cached_sidecar_io = Some(std::sync::Arc::clone(&arc));
                arc
            }
        };
        // Wrap read-only per-frame data in Arc once — each behavior gets a
        // cheap O(1) refcount clone instead of a deep BTreeMap copy.
        let resolver = std::sync::Arc::clone(&self.resolver_cache);
        let object_regions = std::sync::Arc::clone(&self.object_regions);
        let layout_regions_stale = self.layout_regions_stale();
        let object_kinds = self.object_kind_snapshot();
        let object_props = self.object_props_snapshot();
        let object_text = self.object_text_snapshot();
        // UI strings: build Arc<str> once, clone is a single atomic increment per behavior.
        let ui_focused_target_id: Option<std::sync::Arc<str>> =
            self.focused_ui_target_id().map(std::sync::Arc::from);
        let ui_theme_id: Option<std::sync::Arc<str>> =
            self.ui_state.theme_id.as_deref().map(std::sync::Arc::from);
        let ui_last_submit_target_id: Option<std::sync::Arc<str>> = self
            .ui_state
            .last_submit
            .as_ref()
            .map(|ev| std::sync::Arc::from(ev.target_id.as_str()));
        let ui_last_submit_text: Option<std::sync::Arc<str>> = self
            .ui_state
            .last_submit
            .as_ref()
            .map(|ev| std::sync::Arc::from(ev.text.as_str()));
        let ui_last_change_target_id: Option<std::sync::Arc<str>> = self
            .ui_state
            .last_change
            .as_ref()
            .map(|ev| std::sync::Arc::from(ev.target_id.as_str()));
        let ui_last_change_text: Option<std::sync::Arc<str>> = self
            .ui_state
            .last_change
            .as_ref()
            .map(|ev| std::sync::Arc::from(ev.text.as_str()));
        let last_raw_key = self
            .ui_state
            .last_raw_key
            .as_ref()
            .map(|k| std::sync::Arc::new(k.clone()));
        let keys_down = std::sync::Arc::new(self.keys_down_snapshot());
        let keys_just_pressed = std::sync::Arc::new(
            keys_down
                .difference(&self.prev_keys_down)
                .cloned()
                .collect::<std::collections::HashSet<_>>(),
        );
        // Save current snapshot as previous before we move keys_down into context.
        let keys_down_for_prev = std::sync::Arc::clone(&keys_down);

        // Phase 7C: Build Rhai maps once per frame and wrap in Arc.
        // Behaviors will clone these Arc refs (O(1) refcount) instead of cloning maps (O(n_map)).
        use rhai::Map as RhaiMap;
        #[allow(clippy::arc_with_non_send_sync)]
        let rhai_menu_map = {
            let mut menu_map = RhaiMap::new();
            menu_map.insert(
                "selected_index".into(),
                (menu_selected_index as rhai::INT).into(),
            );
            menu_map.insert(
                "count".into(),
                (self.scene.menu_options.len() as rhai::INT).into(),
            );
            std::sync::Arc::new(menu_map)
        };

        #[allow(clippy::arc_with_non_send_sync)]
        let rhai_time_map = {
            let mut time_map = RhaiMap::new();
            time_map.insert(
                "scene_elapsed_ms".into(),
                (scene_elapsed_ms as rhai::INT).into(),
            );
            time_map.insert(
                "stage_elapsed_ms".into(),
                (stage_elapsed_ms as rhai::INT).into(),
            );
            let stage_str: &str = match stage {
                engine_animation::SceneStage::OnEnter => "on_enter",
                engine_animation::SceneStage::OnIdle => "on_idle",
                engine_animation::SceneStage::OnLeave => "on_leave",
                engine_animation::SceneStage::Done => "done",
            };
            time_map.insert("stage".into(), stage_str.into());
            std::sync::Arc::new(time_map)
        };

        #[allow(clippy::arc_with_non_send_sync)]
        let rhai_key_map = {
            let mut key_map = rhai::Map::new();
            build_base_key_fields(&mut key_map, self.ui_state.last_raw_key.as_ref());
            std::sync::Arc::new(key_map)
        };

        // Engine-level key metadata for Rhai scope (separate `engine` namespace)
        #[allow(clippy::arc_with_non_send_sync)]
        let engine_key_map = {
            let mut engine_key = rhai::Map::new();
            build_base_key_fields(&mut engine_key, self.ui_state.last_raw_key.as_ref());
            if let Some(k) = &self.ui_state.last_raw_key {
                let is_quit =
                    k.ctrl && (k.code == "q" || k.code == "Q" || k.code == "c" || k.code == "C");
                engine_key.insert("is_quit".into(), is_quit.into());
            } else {
                engine_key.insert("is_quit".into(), false.into());
            }
            engine_key.insert("any_down".into(), (!keys_down.is_empty()).into());
            engine_key.insert("down_count".into(), (keys_down.len() as rhai::INT).into());
            std::sync::Arc::new(engine_key)
        };

        let mut commands = Vec::new();

        // Compute collision enter/stay/exit from current vs previous frame pairs.
        let current_pairs: std::collections::HashSet<(u64, u64)> = collisions
            .iter()
            .map(|h| (h.a.min(h.b), h.a.max(h.b)))
            .collect();
        let collision_enters: std::sync::Arc<Vec<engine_game::CollisionHit>> = std::sync::Arc::new(
            current_pairs
                .iter()
                .filter(|p| !self.prev_collision_pairs.contains(p))
                .map(|&(a, b)| engine_game::CollisionHit {
                    a,
                    b,
                    normal_x: 0.0,
                    normal_y: 0.0,
                })
                .collect(),
        );
        let collision_stays: std::sync::Arc<Vec<engine_game::CollisionHit>> = std::sync::Arc::new(
            current_pairs
                .iter()
                .filter(|p| self.prev_collision_pairs.contains(p))
                .map(|&(a, b)| engine_game::CollisionHit {
                    a,
                    b,
                    normal_x: 0.0,
                    normal_y: 0.0,
                })
                .collect(),
        );
        let collision_exits: std::sync::Arc<Vec<engine_game::CollisionHit>> = std::sync::Arc::new(
            self.prev_collision_pairs
                .iter()
                .filter(|p| !current_pairs.contains(p))
                .map(|&(a, b)| engine_game::CollisionHit {
                    a,
                    b,
                    normal_x: 0.0,
                    normal_y: 0.0,
                })
                .collect(),
        );
        self.prev_collision_pairs = current_pairs;

        // Construct context once; only `object_states` mutates between iterations.
        let mut ctx = BehaviorContext {
            stage,
            scene_elapsed_ms,
            stage_elapsed_ms,
            menu_selected_index,
            target_resolver: resolver.clone(),
            object_states: self.effective_object_states_snapshot(),
            object_kinds,
            object_props,
            object_regions,
            layout_regions_stale,
            object_text,
            ui_focused_target_id,
            ui_theme_id,
            ui_last_submit_target_id,
            ui_last_submit_text,
            ui_last_change_target_id,
            ui_last_change_text,
            game_state,
            level_state,
            persistence,
            catalogs,
            palettes,
            default_palette,
            gameplay_world,
            emitter_state,
            collisions,
            collision_enters,
            collision_stays,
            collision_exits,
            last_raw_key,
            keys_down,
            keys_just_pressed,
            sidecar_io,
            rhai_time_map,
            rhai_menu_map,
            rhai_key_map,
            engine_key_map,
            debug_enabled,
            frame_ms: scene_elapsed_ms
                .saturating_sub(self.prev_scene_elapsed_ms)
                .max(1),
            action_bindings: match &self.cached_action_bindings {
                Some(cached) => std::sync::Arc::clone(cached),
                None => {
                    let arc = std::sync::Arc::new(self.action_bindings.clone());
                    self.cached_action_bindings = Some(std::sync::Arc::clone(&arc));
                    arc
                }
            },
            mouse_x: self.gui_state.mouse_x,
            mouse_y: self.gui_state.mouse_y,
            gui_state: Some(self.gui_state_arc()),
            spatial_meters_per_world_unit: Some(self.spatial_context.scale.meters_per_world_unit),
            orbit_active: self.orbit_camera_active(),
        };
        let mut local_commands = Vec::new();
        for idx in 0..self.behaviors.len() {
            let object_id = &self.behaviors[idx].object_id;
            let Some(object) = self.objects.get(object_id) else {
                continue;
            };
            local_commands.clear();
            self.behaviors[idx]
                .behavior
                .update(object, &self.scene, &ctx, &mut local_commands);
            let diagnostic_commands = self.apply_behavior_commands(&resolver, &local_commands);
            commands.extend(local_commands.iter().cloned());
            commands.extend(diagnostic_commands);
            // Update snapshots after each behavior emits commands, so subsequent
            // behaviors see same-frame text/property/state mutations.
            // effective_object_states_snapshot() uses gen-counter gating to skip rebuilds
            // on mutation-free frames (the common case).
            if !local_commands.is_empty() && idx + 1 < self.behaviors.len() {
                ctx.object_states = self.effective_object_states_snapshot();
                ctx.object_props = self.object_props_snapshot();
                ctx.object_text = self.object_text_snapshot();
                ctx.object_regions = std::sync::Arc::clone(&self.object_regions);
                ctx.layout_regions_stale = self.layout_regions_stale();
            }
        }
        // Update effective_states once after all behaviors run, not per-behavior.
        // This was previously updated in the loop above (line 1221) for each
        // command emission, causing redundant O(n) rebuilds. Now deferred to once
        // after the loop with gen-counter gating in effective_object_states_snapshot().
        self.cached_effective_states = None;
        self.ui_state.last_submit = None;
        self.ui_state.last_change = None;
        // Carry forward current keys as previous for next-frame just_pressed computation.
        self.prev_keys_down = (*keys_down_for_prev).clone();
        self.prev_scene_elapsed_ms = scene_elapsed_ms;
        commands
    }

    /// Applies palette color bindings if the active palette has changed since the last
    /// application. Called once per frame before behavior script execution so that sprites
    /// with `@palette.<key>` YAML bindings always reflect the current palette.
    pub fn apply_palette_bindings_if_changed(
        &mut self,
        palettes: &engine_behavior::palette::PaletteStore,
    ) {
        if self.scene.palette_bindings.is_empty() {
            return;
        }
        let current_version = palettes.version.load(std::sync::atomic::Ordering::Relaxed);
        if current_version == self.palette_applied_version {
            return;
        }
        let Some(palette) = palettes.resolve(None, None) else {
            return;
        };
        let commands: Vec<engine_behavior::BehaviorCommand> = self
            .scene
            .palette_bindings
            .iter()
            .filter_map(|binding| {
                let color = palette.colors.get(&binding.key)?;
                let value = serde_json::Value::String(color.clone());
                let request = engine_api::commands::scene_mutation_request_from_set_path(
                    &binding.target,
                    &binding.prop,
                    &value,
                    None,
                )?;
                Some(engine_behavior::BehaviorCommand::ApplySceneMutation { request })
            })
            .collect();
        let resolver = std::sync::Arc::clone(&self.resolver_cache);
        self.apply_behavior_commands(&resolver, &commands);
        self.palette_applied_version = current_version;
    }

    /// Applies game_state text bindings every frame. For each sprite with a
    /// `@game_state.<path>` binding, reads the current value and updates sprite text
    /// content if it changed. Only marks caches dirty if at least one sprite changed.
    pub fn apply_game_state_bindings_if_changed(
        &mut self,
        game_state: &engine_core::game_state::GameState,
    ) {
        if self.scene.game_state_bindings.is_empty() {
            return;
        }
        let current_version = game_state
            .version
            .load(std::sync::atomic::Ordering::Relaxed);
        if current_version == self.game_state_applied_version {
            return;
        }
        let mut any_changed = false;
        let bindings = self.scene.game_state_bindings.clone();
        for binding in &bindings {
            let text = game_state
                .get(&binding.path)
                .map(|v| match &v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            if self.set_text_sprite_content(&binding.target, text) {
                any_changed = true;
            }
        }
        if any_changed {
            self.apply_runtime_mutation_impact(RuntimeMutationImpact::text().with_layout());
        }
        self.game_state_applied_version = current_version;
    }

    /// Applies behavior commands to runtime object state using the supplied
    /// target resolver.
    pub fn apply_behavior_commands(
        &mut self,
        resolver: &TargetResolver,
        commands: &[BehaviorCommand],
    ) -> Vec<BehaviorCommand> {
        if commands.is_empty() {
            return Vec::new();
        }

        // Collect despawn targets for batched removal (single graph rebuild).
        let mut pending_despawns: Vec<String> = Vec::new();
        // Enable batch spawn mode: defer refresh_runtime_caches() per spawn.
        self.spawn_batch_depth += 1;
        let mut had_graph_spawns = false;
        let mut diagnostics = Vec::new();

        for command in commands {
            match self.scene_mutation_from_behavior_command(resolver, command) {
                Ok(Some(mutation)) => {
                    let result = self.apply_scene_mutation(resolver, &mutation);
                    if let Some(error) = result.error.as_ref() {
                        diagnostics.push(self.scene_mutation_debug_log(error));
                    }
                    continue;
                }
                Ok(None) => {}
                Err(error) => {
                    diagnostics.push(self.scene_mutation_debug_log(&error));
                    continue;
                }
            }

            match command {
                BehaviorCommand::PlayAudioCue { .. } => {}
                BehaviorCommand::PlayAudioEvent { .. } => {}
                BehaviorCommand::PlaySong { .. } => {}
                BehaviorCommand::StopSong => {}
                BehaviorCommand::ApplySceneMutation { request } => match request {
                    engine_api::scene::SceneMutationRequest::SpawnObject { template, target } => {
                        match self.apply_spawn_request(resolver, template, target) {
                            Ok(impact) => {
                                if impact.graph {
                                    had_graph_spawns = true;
                                } else {
                                    self.apply_runtime_mutation_impact(impact);
                                }
                            }
                            Err(error) => diagnostics.push(self.scene_mutation_debug_log(&error)),
                        }
                    }
                    engine_api::scene::SceneMutationRequest::DespawnObject { target } => {
                        match self.validate_despawn_request(resolver, target) {
                            Ok(()) => pending_despawns.push(target.clone()),
                            Err(error) => diagnostics.push(self.scene_mutation_debug_log(&error)),
                        }
                    }
                    _ => {}
                },
                BehaviorCommand::SceneTransition { .. } => {}
                BehaviorCommand::DebugLog { .. } => {}
                BehaviorCommand::BindInputAction { action, keys } => {
                    self.action_bindings.insert(action.clone(), keys.clone());
                    self.cached_action_bindings = None; // invalidate cache
                }
                // ScriptError is consumed at the behavior system level (world access needed).
                BehaviorCommand::ScriptError { .. } => {}
                // TriggerEffect is consumed by the compositor system (world resource access needed).
                BehaviorCommand::TriggerEffect { .. } => {}
                BehaviorCommand::SetSceneBg { color } => {
                    self.scene.bg_colour = engine_core::scene::color::parse_colour_str(color);
                }
                BehaviorCommand::SetCamera { .. } => {}
                BehaviorCommand::SetCameraZoom { .. } => {}
                BehaviorCommand::SetCamera3DLookAt { .. } => {}
                BehaviorCommand::SetCamera3DUp { .. } => {}
                BehaviorCommand::SetGuiValue { widget_id, value } => {
                    if let Some(ws) = self.gui_state.widgets.get_mut(widget_id) {
                        ws.value = *value;
                        ws.selected_index = Some(value.round().max(0.0) as usize);
                        ws.changed = true;
                        self.gui_state.last_changed = Some(widget_id.clone());
                        self.cached_gui_state = None;
                    }
                    self.sync_widget_visuals();
                }
                _ => {}
            }
        }

        // End batch spawn mode and do a single cache refresh if any spawns happened.
        self.spawn_batch_depth -= 1;
        if had_graph_spawns && self.spawn_batch_depth == 0 {
            self.refresh_runtime_caches();
            self.apply_runtime_mutation_impact(RuntimeMutationImpact::graph());
        }

        // Batch-apply all collected despawns with a single graph rebuild.
        if !pending_despawns.is_empty() && self.batch_despawn_targets(resolver, &pending_despawns) {
            self.apply_runtime_mutation_impact(RuntimeMutationImpact::graph());
        }
        diagnostics
    }

    fn scene_mutation_from_behavior_command(
        &self,
        _resolver: &TargetResolver,
        command: &BehaviorCommand,
    ) -> Result<Option<SceneMutation>, engine_api::scene::SceneMutationError> {
        match command {
            BehaviorCommand::SetVisibility { target, visible } => {
                Ok(Some(SceneMutation::Set2DProps(Set2DPropsMutation {
                    target: target.clone(),
                    visible: Some(*visible),
                    dx: None,
                    dy: None,
                    text: None,
                })))
            }
            BehaviorCommand::SetOffset { target, dx, dy } => {
                Ok(Some(SceneMutation::Set2DProps(Set2DPropsMutation {
                    target: target.clone(),
                    visible: None,
                    dx: Some(*dx),
                    dy: Some(*dy),
                    text: None,
                })))
            }
            BehaviorCommand::SetText { target, text } => {
                Ok(Some(SceneMutation::Set2DProps(Set2DPropsMutation {
                    target: target.clone(),
                    visible: None,
                    dx: None,
                    dy: None,
                    text: Some(text.clone()),
                })))
            }
            BehaviorCommand::SetProps {
                target,
                visible,
                dx,
                dy,
                text,
            } => Ok(Some(SceneMutation::Set2DProps(Set2DPropsMutation {
                target: target.clone(),
                visible: *visible,
                dx: *dx,
                dy: *dy,
                text: text.clone(),
            }))),
            BehaviorCommand::SetCamera { x, y } => {
                Ok(Some(SceneMutation::SetCamera2D(SetCamera2DMutation {
                    x: x.round() as i32,
                    y: y.round() as i32,
                    zoom: None,
                })))
            }
            BehaviorCommand::SetCameraZoom { zoom } => {
                Ok(Some(SceneMutation::SetCamera2D(SetCamera2DMutation {
                    x: self.camera_x,
                    y: self.camera_y,
                    zoom: Some(*zoom),
                })))
            }
            BehaviorCommand::SetCamera3DLookAt { eye, look_at } => {
                crate::request_adapter::scene_mutation_from_request_result(
                    &engine_api::scene::SceneMutationRequest::SetCamera3d(
                        engine_api::scene::Camera3dMutationRequest::LookAt {
                            eye: *eye,
                            look_at: *look_at,
                        },
                    ),
                    self.scene_camera_3d,
                )
                .map(Some)
            }
            BehaviorCommand::SetCamera3DUp { up } => {
                crate::request_adapter::scene_mutation_from_request_result(
                    &engine_api::scene::SceneMutationRequest::SetCamera3d(
                        engine_api::scene::Camera3dMutationRequest::Up { up: *up },
                    ),
                    self.scene_camera_3d,
                )
                .map(Some)
            }
            BehaviorCommand::ApplySceneMutation { request } => {
                if matches!(
                    request,
                    engine_api::scene::SceneMutationRequest::SpawnObject { .. }
                        | engine_api::scene::SceneMutationRequest::DespawnObject { .. }
                ) {
                    return Ok(None);
                }
                crate::request_adapter::scene_mutation_from_request_result(
                    request,
                    self.scene_camera_3d,
                )
                .map(Some)
            }
            _ => Ok(None),
        }
    }

    fn apply_scene_mutation(
        &mut self,
        resolver: &TargetResolver,
        mutation: &SceneMutation,
    ) -> engine_api::scene::SceneMutationResult {
        let mut mutation_applied = false;
        let mut impact = RuntimeMutationImpact::NONE;
        match mutation {
            SceneMutation::Set2DProps(props) => {
                let Some(object_id) = resolver.resolve_alias(&props.target) else {
                    return engine_api::scene::SceneMutationResult::rejected(
                        engine_api::scene::SceneMutationError::target_not_found(
                            props.target.clone(),
                        ),
                    );
                };
                if props.visible.is_none()
                    && props.dx.is_none()
                    && props.dy.is_none()
                    && props.text.is_none()
                {
                    return engine_api::scene::SceneMutationResult::rejected(
                        engine_api::scene::SceneMutationError::invalid_request(
                            "set_2d_props",
                            "set_2d_props requires at least one field",
                        ),
                    );
                }
                if props.text.is_some()
                    && !self.sprite_kind_matches(object_id, RuntimeSpriteKind::Text)
                {
                    return self.unsupported_target_mutation(
                        "set_2d_props",
                        &props.target,
                        "text.content",
                    );
                }
                if let Some(state) = self.object_states.get_mut(object_id) {
                    if let Some(next_visible) = props.visible {
                        if state.visible != next_visible {
                            state.visible = next_visible;
                            mutation_applied = true;
                            impact.merge(RuntimeMutationImpact::state().with_layout());
                        }
                    }
                    if let Some(delta_x) = props.dx {
                        let next_offset_x = state.offset_x.saturating_add(delta_x);
                        if state.offset_x != next_offset_x {
                            state.offset_x = next_offset_x;
                            mutation_applied = true;
                            impact.merge(RuntimeMutationImpact::state().with_layout());
                        }
                    }
                    if let Some(delta_y) = props.dy {
                        let next_offset_y = state.offset_y.saturating_add(delta_y);
                        if state.offset_y != next_offset_y {
                            state.offset_y = next_offset_y;
                            mutation_applied = true;
                            impact.merge(RuntimeMutationImpact::state().with_layout());
                        }
                    }
                }
                if let Some(next_text) = &props.text {
                    if self.apply_text_property_for_target(
                        object_id,
                        &props.target,
                        |runtime, alias| runtime.set_text_sprite_content(alias, next_text.clone()),
                    ) {
                        mutation_applied = true;
                        impact.merge(RuntimeMutationImpact::text().with_layout());
                    }
                }
            }
            SceneMutation::SetSpriteProperty { target, mutation } => {
                let Some(object_id) = resolver.resolve_alias(&target) else {
                    return engine_api::scene::SceneMutationResult::rejected(
                        engine_api::scene::SceneMutationError::target_not_found(target.clone()),
                    );
                };
                match mutation {
                    SetSpritePropertyMutation::Heading { heading } => {
                        let mut changed = false;
                        if let Some(state) = self.object_states.get_mut(object_id) {
                            if (state.heading - *heading).abs() > f32::EPSILON {
                                state.heading = *heading;
                                changed = true;
                            }
                        }
                        if let Some(obj) = self.objects.get(object_id) {
                            if matches!(obj.kind, GameObjectKind::Layer) {
                                let n = obj.children.len();
                                for i in 0..n {
                                    let cid = self.objects[object_id].children[i].clone();
                                    if let Some(state) = self.object_states.get_mut(&cid) {
                                        if (state.heading - *heading).abs() > f32::EPSILON {
                                            state.heading = *heading;
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                        if changed {
                            mutation_applied = true;
                            impact.merge(RuntimeMutationImpact::state().with_layout());
                        }
                    }
                    SetSpritePropertyMutation::TextFont { font } => {
                        if !self.sprite_kind_matches(object_id, RuntimeSpriteKind::Text) {
                            return self.unsupported_target_mutation(
                                "set_sprite_property",
                                target,
                                "text.font",
                            );
                        }
                        if self.apply_text_property_for_target(
                            object_id,
                            &target,
                            |runtime, alias| runtime.set_text_sprite_font(alias, font.clone()),
                        ) {
                            mutation_applied = true;
                            impact.merge(RuntimeMutationImpact::props().with_layout());
                        }
                    }
                    SetSpritePropertyMutation::TextColour { fg, value } => {
                        if !self.sprite_kind_matches_any(
                            object_id,
                            &[RuntimeSpriteKind::Text, RuntimeSpriteKind::Vector],
                        ) {
                            return self.unsupported_target_mutation(
                                "set_sprite_property",
                                target,
                                if *fg { "style.fg" } else { "style.bg" },
                            );
                        }
                        let Some(next_colour) = parse_term_colour(value) else {
                            return engine_api::scene::SceneMutationResult::rejected(
                                engine_api::scene::SceneMutationError::invalid_request(
                                    "set_sprite_property",
                                    format!(
                                        "target `{target}` received an unsupported colour value"
                                    ),
                                ),
                            );
                        };
                        let text_applied = if *fg {
                            self.apply_text_property_for_target(
                                object_id,
                                &target,
                                |runtime, alias| {
                                    runtime.set_text_sprite_fg_colour(alias, next_colour.clone())
                                },
                            )
                        } else {
                            self.apply_text_property_for_target(
                                object_id,
                                &target,
                                |runtime, alias| {
                                    runtime.set_text_sprite_bg_colour(alias, next_colour.clone())
                                },
                            )
                        };
                        let mut applied = text_applied;
                        if !applied {
                            let path = if *fg { "style.fg" } else { "style.bg" };
                            for alias in self.object_alias_candidates(object_id, &target) {
                                if self.set_vector_sprite_property(&alias, path, value) {
                                    applied = true;
                                    break;
                                }
                            }
                        }
                        if applied {
                            mutation_applied = true;
                            if text_applied {
                                impact.merge(RuntimeMutationImpact::props());
                            }
                        }
                    }
                    SetSpritePropertyMutation::VectorProperty { path, value } => {
                        if !self.sprite_kind_matches(object_id, RuntimeSpriteKind::Vector) {
                            return self.unsupported_target_mutation(
                                "set_sprite_property",
                                target,
                                path,
                            );
                        }
                        let mut applied = self.set_vector_sprite_property(&target, path, value);
                        if !applied {
                            for alias in self.object_alias_candidates(object_id, &target) {
                                if self.set_vector_sprite_property(&alias, path, value) {
                                    applied = true;
                                    break;
                                }
                            }
                        }
                        if applied {
                            mutation_applied = true;
                            impact.merge(runtime_impact_for_vector_property(path));
                        }
                    }
                    SetSpritePropertyMutation::ImageFrame { frame_index } => {
                        if !self.sprite_kind_matches(object_id, RuntimeSpriteKind::Image) {
                            return self.unsupported_target_mutation(
                                "set_sprite_property",
                                target,
                                "image.frame_index",
                            );
                        }
                        let mut applied = self.set_image_sprite_frame_index(&target, *frame_index);
                        if !applied {
                            for alias in self.object_alias_candidates(object_id, &target) {
                                if self.set_image_sprite_frame_index(&alias, *frame_index) {
                                    applied = true;
                                    break;
                                }
                            }
                        }
                        if applied {
                            mutation_applied = true;
                            impact.merge(RuntimeMutationImpact::layout());
                        }
                    }
                }
            }
            SceneMutation::SetCamera2D(camera) => {
                if self.camera_x != camera.x || self.camera_y != camera.y {
                    self.set_camera_internal(camera.x, camera.y);
                    mutation_applied = true;
                    impact.merge(RuntimeMutationImpact::layout());
                }
                if let Some(zoom) = camera.zoom {
                    if (self.camera_zoom - zoom).abs() > f32::EPSILON {
                        self.set_camera_zoom_internal(zoom);
                        mutation_applied = true;
                        impact.merge(RuntimeMutationImpact::layout());
                    }
                }
            }
            SceneMutation::SetCamera3D(camera) => {
                self.set_scene_camera_3d_internal(
                    engine_core::scene_runtime_types::SceneCamera3D {
                        eye: camera.eye,
                        look_at: camera.look_at,
                        up: camera.up,
                        fov_degrees: camera.fov_deg,
                        near_clip: self.scene_camera_3d.near_clip,
                    },
                );
                mutation_applied = true;
            }
            SceneMutation::SetRender3D(render3d) => match render3d {
                Render3DMutation::SetGroupedParams { target, params } => {
                    if params.is_empty() {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::invalid_request(
                                "set_render3d",
                                "grouped render params must not be empty",
                            ),
                        );
                    }
                    let Some(target_name) = target.as_deref() else {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::unsupported_request(
                                "set_render3d",
                                "grouped render params require a target",
                            ),
                        );
                    };
                    let Some(object_id) = resolver.resolve_alias(target_name) else {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::target_not_found(target_name),
                        );
                    };
                    for (param, _) in params {
                        if !self.grouped_render_param_supported(object_id, param) {
                            return self.unsupported_target_mutation(
                                "set_render3d",
                                target_name,
                                grouped_render_param_label(param),
                            );
                        }
                    }
                    for (param, value) in params {
                        if self.apply_grouped_render3d_param(
                            resolver,
                            target.as_deref(),
                            param,
                            value,
                        ) {
                            mutation_applied = true;
                            impact.merge(runtime_impact_for_grouped_render3d_param(param));
                        }
                    }
                }
                Render3DMutation::SetProfile { slot, profile } => {
                    if self.apply_profile_selection(*slot, profile) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetViewProfile { profile } => {
                    if self.apply_profile_selection(
                        crate::mutations::Render3DProfileSlot::View,
                        profile,
                    ) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetLightingProfile { profile } => {
                    if self.apply_profile_selection(
                        crate::mutations::Render3DProfileSlot::Lighting,
                        profile,
                    ) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetSpaceEnvironmentProfile { profile } => {
                    if self.apply_profile_selection(
                        crate::mutations::Render3DProfileSlot::SpaceEnvironment,
                        profile,
                    ) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetProfileParam { param, value } => {
                    if self.apply_profile_param(param, value) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetLightingParam { param, value } => {
                    if self.apply_lighting_profile_param(param, value) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetSpaceEnvironmentParam { param, value } => {
                    if self.apply_space_environment_param(param, value) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetNodeVisibility { target, visible } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::target_not_found(target.clone()),
                        );
                    };
                    if let Some(state) = self.object_states.get_mut(object_id) {
                        if state.visible != *visible {
                            state.visible = *visible;
                            mutation_applied = true;
                            impact.merge(RuntimeMutationImpact::state().with_layout());
                        }
                    }
                }
                Render3DMutation::SetNodeTransform { target, transform } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::target_not_found(target.clone()),
                        );
                    };
                    if let Some(state) = self.object_states.get_mut(object_id) {
                        let next_offset_x = transform.translation[0].round() as i32;
                        let next_offset_y = transform.translation[1].round() as i32;
                        if state.offset_x != next_offset_x || state.offset_y != next_offset_y {
                            state.offset_x = next_offset_x;
                            state.offset_y = next_offset_y;
                            mutation_applied = true;
                            impact.merge(RuntimeMutationImpact::state().with_layout());
                        }
                    }
                }
                Render3DMutation::SetSceneCamera { camera } => {
                    self.set_scene_camera_3d_internal(
                        engine_core::scene_runtime_types::SceneCamera3D {
                            eye: camera.eye,
                            look_at: camera.look_at,
                            up: camera.up,
                            fov_degrees: camera.fov_deg,
                            near_clip: self.scene_camera_3d.near_clip,
                        },
                    );
                    mutation_applied = true;
                }
                Render3DMutation::SetScene3DFrame { target, frame } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::target_not_found(target.clone()),
                        );
                    };
                    if !self.sprite_kind_matches(object_id, RuntimeSpriteKind::Scene3D) {
                        return self.unsupported_target_mutation(
                            "set_render3d",
                            target,
                            "scene3d.frame",
                        );
                    }
                    if !self.apply_scene3d_frame_for_target(object_id, target, frame) {
                        self.apply_runtime_mutation_impact(impact);
                        return engine_api::scene::SceneMutationResult::applied();
                    }
                    mutation_applied = true;
                    impact.merge(RuntimeMutationImpact::layout());
                }
                Render3DMutation::SetObjMaterialParam {
                    target,
                    param,
                    value,
                } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::target_not_found(target.clone()),
                        );
                    };
                    if !self.sprite_kind_matches(object_id, RuntimeSpriteKind::Obj) {
                        return self.unsupported_target_mutation(
                            "set_render3d",
                            target,
                            obj_material_param_label(param),
                        );
                    };
                    if self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                        runtime.set_obj_material_typed_wrapper(alias, param, value)
                    }) {
                        mutation_applied = true;
                        impact.merge(runtime_impact_for_obj_material_param(param));
                    }
                }
                Render3DMutation::SetAtmosphereParamTyped {
                    target,
                    param,
                    value,
                } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::target_not_found(target.clone()),
                        );
                    };
                    if !self.sprite_kind_matches(object_id, RuntimeSpriteKind::Obj) {
                        return self.unsupported_target_mutation(
                            "set_render3d",
                            target,
                            atmosphere_param_label(param),
                        );
                    };
                    if self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                        runtime.set_obj_atmosphere_typed_wrapper(alias, param, value)
                    }) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetTerrainParamTyped {
                    target,
                    param,
                    value,
                } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::target_not_found(target.clone()),
                        );
                    };
                    if !self.sprite_kind_matches(object_id, RuntimeSpriteKind::Obj) {
                        return self.unsupported_target_mutation(
                            "set_render3d",
                            target,
                            terrain_param_label(param),
                        );
                    };
                    if self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                        runtime.set_obj_terrain_typed_wrapper(alias, param, value)
                    }) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetWorldgenParamTyped {
                    target,
                    param,
                    value,
                } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::target_not_found(target.clone()),
                        );
                    };
                    if !self.sprite_kind_matches(object_id, RuntimeSpriteKind::Obj) {
                        return self.unsupported_target_mutation(
                            "set_render3d",
                            target,
                            worldgen_param_label(param),
                        );
                    };
                    if self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                        runtime.set_obj_worldgen_typed_wrapper(alias, param, value)
                    }) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetPlanetParamTyped {
                    target,
                    param,
                    value,
                } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return engine_api::scene::SceneMutationResult::rejected(
                            engine_api::scene::SceneMutationError::target_not_found(target.clone()),
                        );
                    };
                    if !self.sprite_kind_matches(object_id, RuntimeSpriteKind::Planet) {
                        return self.unsupported_target_mutation(
                            "set_render3d",
                            target,
                            planet_param_label(param),
                        );
                    };
                    if self.apply_text_property_for_target(object_id, target, |runtime, alias| {
                        runtime.set_planet_typed_wrapper(alias, param, value)
                    }) {
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetLight { .. }
                | Render3DMutation::RebuildMesh { .. }
                | Render3DMutation::RebuildWorldgen { .. } => {
                    mutation_applied = true;
                }
            },
            SceneMutation::SpawnObject { .. } => {
                return engine_api::scene::SceneMutationResult::rejected(
                    engine_api::scene::SceneMutationError::unsupported_request(
                        "spawn_object",
                        "spawn_object must be applied through behavior command dispatch",
                    ),
                );
            }
            SceneMutation::DespawnObject { .. } => {
                return engine_api::scene::SceneMutationResult::rejected(
                    engine_api::scene::SceneMutationError::unsupported_request(
                        "despawn_object",
                        "despawn_object must be applied through behavior command dispatch",
                    ),
                );
            }
        }
        if mutation_applied {
            let dirty = dirty_for_scene_mutation(mutation);
            self.render3d_dirty_mask.insert(dirty);
            self.track_render3d_rebuild_cause(dirty);
        }
        self.apply_runtime_mutation_impact(impact);
        engine_api::scene::SceneMutationResult::applied()
    }

    fn apply_grouped_render3d_param(
        &mut self,
        resolver: &TargetResolver,
        target: Option<&str>,
        param: &crate::mutations::Render3DGroupedParam,
        value: &engine_core::render_types::MaterialValue,
    ) -> bool {
        match param {
            crate::mutations::Render3DGroupedParam::View(param) => {
                let material_param = match param {
                    crate::mutations::ViewParam::Distance => {
                        crate::mutations::ObjMaterialParam::CameraDistance
                    }
                    crate::mutations::ViewParam::Yaw => crate::mutations::ObjMaterialParam::Yaw,
                    crate::mutations::ViewParam::Pitch => crate::mutations::ObjMaterialParam::Pitch,
                    crate::mutations::ViewParam::Roll => crate::mutations::ObjMaterialParam::Roll,
                };
                self.apply_targeted_grouped_render3d_param(resolver, target, |runtime, alias| {
                    runtime.set_obj_material_typed_wrapper(alias, &material_param, value)
                })
            }
            crate::mutations::Render3DGroupedParam::Material(param) => self
                .apply_targeted_grouped_render3d_param(resolver, target, |runtime, alias| {
                    runtime.set_obj_material_typed_wrapper(alias, param, value)
                }),
            crate::mutations::Render3DGroupedParam::Atmosphere(param) => self
                .apply_targeted_grouped_render3d_param(resolver, target, |runtime, alias| {
                    runtime.set_obj_atmosphere_typed_wrapper(alias, param, value)
                }),
            crate::mutations::Render3DGroupedParam::Surface(param) => self
                .apply_targeted_grouped_render3d_param(resolver, target, |runtime, alias| {
                    runtime.set_obj_terrain_typed_wrapper(alias, param, value)
                }),
            crate::mutations::Render3DGroupedParam::Generator(param) => self
                .apply_targeted_grouped_render3d_param(resolver, target, |runtime, alias| {
                    runtime.set_obj_worldgen_typed_wrapper(alias, param, value)
                }),
            crate::mutations::Render3DGroupedParam::Body(param) => self
                .apply_targeted_grouped_render3d_param(resolver, target, |runtime, alias| {
                    runtime.set_planet_typed_wrapper(alias, param, value)
                }),
        }
    }

    fn apply_targeted_grouped_render3d_param(
        &mut self,
        resolver: &TargetResolver,
        target: Option<&str>,
        mut apply: impl FnMut(&mut Self, &str) -> bool,
    ) -> bool {
        let target = match target {
            Some(target) => target,
            None => return false,
        };
        let Some(object_id) = resolver.resolve_alias(target) else {
            return false;
        };
        self.apply_text_property_for_target(object_id, target, |runtime, alias| {
            apply(runtime, alias)
        })
    }

    fn scene_mutation_debug_log(
        &self,
        error: &engine_api::scene::SceneMutationError,
    ) -> BehaviorCommand {
        let severity = match error {
            engine_api::scene::SceneMutationError::InvalidRequest { .. } => {
                engine_api::commands::DebugLogSeverity::Error
            }
            engine_api::scene::SceneMutationError::UnsupportedRequest { .. }
            | engine_api::scene::SceneMutationError::TargetNotFound { .. } => {
                engine_api::commands::DebugLogSeverity::Warn
            }
        };
        BehaviorCommand::DebugLog {
            scene_id: self.scene.id.clone(),
            source: Some("scene-mutation".to_string()),
            severity,
            message: format!(
                "scene mutation rejected: {}",
                format_scene_mutation_error(error)
            ),
        }
    }

    fn unsupported_target_mutation(
        &self,
        request: &str,
        target: &str,
        property: impl AsRef<str>,
    ) -> engine_api::scene::SceneMutationResult {
        engine_api::scene::SceneMutationResult::rejected(
            engine_api::scene::SceneMutationError::unsupported_request(
                request,
                format!("target `{target}` does not support `{}`", property.as_ref()),
            ),
        )
    }

    fn grouped_render_param_supported(
        &self,
        object_id: &str,
        param: &crate::mutations::Render3DGroupedParam,
    ) -> bool {
        match param {
            crate::mutations::Render3DGroupedParam::View(_)
            | crate::mutations::Render3DGroupedParam::Material(_)
            | crate::mutations::Render3DGroupedParam::Atmosphere(_)
            | crate::mutations::Render3DGroupedParam::Surface(_)
            | crate::mutations::Render3DGroupedParam::Generator(_) => {
                self.sprite_kind_matches(object_id, RuntimeSpriteKind::Obj)
            }
            crate::mutations::Render3DGroupedParam::Body(_) => {
                self.sprite_kind_matches(object_id, RuntimeSpriteKind::Planet)
            }
        }
    }

    fn sprite_kind_matches(&self, object_id: &str, expected: RuntimeSpriteKind) -> bool {
        self.runtime_sprite_kind(object_id) == Some(expected)
    }

    fn sprite_kind_matches_any(&self, object_id: &str, expected: &[RuntimeSpriteKind]) -> bool {
        self.runtime_sprite_kind(object_id)
            .is_some_and(|kind| expected.contains(&kind))
    }

    fn runtime_sprite_kind(&self, object_id: &str) -> Option<RuntimeSpriteKind> {
        let (layer_idx, sprite_path) = self.find_sprite_path_for_object(object_id)?;
        let sprite = sprite_at_path(
            self.scene.layers.get(layer_idx)?.sprites.as_slice(),
            &sprite_path,
        )?;
        Some(match sprite {
            Sprite::Text { .. } => RuntimeSpriteKind::Text,
            Sprite::Image { .. } => RuntimeSpriteKind::Image,
            Sprite::Obj { .. } => RuntimeSpriteKind::Obj,
            Sprite::Planet { .. } => RuntimeSpriteKind::Planet,
            Sprite::Scene3D { .. } => RuntimeSpriteKind::Scene3D,
            Sprite::Vector { .. } => RuntimeSpriteKind::Vector,
            Sprite::Panel { .. } | Sprite::Grid { .. } | Sprite::Flex { .. } => {
                RuntimeSpriteKind::Container
            }
        })
    }

    fn spawn_runtime_clone(
        &mut self,
        resolver: &TargetResolver,
        template: &str,
        target: &str,
    ) -> RuntimeMutationImpact {
        if template.trim().is_empty() || target.trim().is_empty() {
            return RuntimeMutationImpact::NONE;
        }
        let current_resolver = self.build_target_resolver();
        let existing = resolver
            .resolve_alias(target)
            .or_else(|| current_resolver.resolve_alias(target))
            .map(str::to_string);
        if let Some(object_id) = existing {
            return if self.set_target_visibility_recursive(&object_id, true) {
                RuntimeMutationImpact::state().with_layout()
            } else {
                RuntimeMutationImpact::NONE
            };
        }

        let template_id = if let Some(id) = resolver.resolve_alias(template) {
            id.to_string()
        } else if let Some(id) = current_resolver.resolve_alias(template) {
            id.to_string()
        } else {
            return RuntimeMutationImpact::NONE;
        };
        let Some(template_object) = self.objects.get(&template_id).cloned() else {
            return RuntimeMutationImpact::NONE;
        };
        let spawned = if matches!(template_object.kind, GameObjectKind::Layer) {
            self.spawn_layer_clone_from_object(template_object, target)
        } else {
            self.spawn_sprite_clone(&template_object, target)
        };

        if spawned {
            RuntimeMutationImpact::graph()
        } else {
            RuntimeMutationImpact::NONE
        }
    }

    fn apply_spawn_request(
        &mut self,
        resolver: &TargetResolver,
        template: &str,
        target: &str,
    ) -> Result<RuntimeMutationImpact, engine_api::scene::SceneMutationError> {
        if template.trim().is_empty() || target.trim().is_empty() {
            return Err(engine_api::scene::SceneMutationError::invalid_request(
                "spawn_object",
                "spawn_object requires non-empty template and target",
            ));
        }

        let current_resolver = self.build_target_resolver();
        let existing = resolver
            .resolve_alias(target)
            .or_else(|| current_resolver.resolve_alias(target));
        if existing.is_none()
            && resolver
                .resolve_alias(template)
                .or_else(|| current_resolver.resolve_alias(template))
                .is_none()
        {
            return Err(engine_api::scene::SceneMutationError::invalid_request(
                "spawn_object",
                format!("template `{template}` was not found"),
            ));
        }

        let impact = self.spawn_runtime_clone(resolver, template, target);
        if impact.is_empty() && existing.is_none() {
            return Err(engine_api::scene::SceneMutationError::invalid_request(
                "spawn_object",
                format!("template `{template}` could not be materialized"),
            ));
        }
        Ok(impact)
    }

    fn validate_despawn_request(
        &self,
        resolver: &TargetResolver,
        target: &str,
    ) -> Result<(), engine_api::scene::SceneMutationError> {
        if target.trim().is_empty() {
            return Err(engine_api::scene::SceneMutationError::invalid_request(
                "despawn_object",
                "despawn_object requires a non-empty target",
            ));
        }
        let current_resolver = self.build_target_resolver();
        if resolver
            .resolve_alias(target)
            .or_else(|| current_resolver.resolve_alias(target))
            .is_none()
        {
            return Err(engine_api::scene::SceneMutationError::target_not_found(
                target.to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn attach_default_behaviors(&mut self) {
        if has_scene_audio(&self.scene) {
            self.behaviors.push(ObjectBehaviorRuntime {
                object_id: self.root_id.clone(),
                behavior: Box::new(SceneAudioBehavior::default()),
            });
        }
    }

    pub(crate) fn attach_declared_behaviors(
        &mut self,
        bindings: Vec<BehaviorBinding>,
        mod_registry: Option<&ModBehaviorRegistry>,
    ) {
        let mut unresolved: Vec<BehaviorBinding> = Vec::new();
        for binding in bindings {
            let mut pending_specs = Vec::new();
            for spec in binding.specs {
                if let Some(behavior) = built_in_behavior(&spec) {
                    self.behaviors.push(ObjectBehaviorRuntime {
                        object_id: binding.object_id.clone(),
                        behavior,
                    });
                } else {
                    let resolved = if let Some(registry) = mod_registry {
                        // Mod-defined behavior: look up script in registry, create a
                        // RhaiScriptBehavior with the spec params and the mod script injected.
                        if let Some(mod_behavior) = registry.get(spec.name.trim()) {
                            let mut params = spec.params.clone();
                            params.script = Some(mod_behavior.script.clone());
                            params.src = mod_behavior
                                .src
                                .clone()
                                .or_else(|| Some(format!("mod:{}", mod_behavior.name)));
                            let behavior = Box::new(RhaiScriptBehavior::from_params(&params));
                            self.behaviors.push(ObjectBehaviorRuntime {
                                object_id: binding.object_id.clone(),
                                behavior,
                            });
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !resolved {
                        pending_specs.push(spec);
                    }
                }
            }
            if !pending_specs.is_empty() {
                unresolved.push(BehaviorBinding {
                    object_id: binding.object_id.clone(),
                    specs: pending_specs,
                });
            }
        }
        self.pending_bindings = unresolved;
    }

    /// Returns `true` if there are unresolved behavior bindings waiting for the mod registry.
    pub fn has_pending_bindings(&self) -> bool {
        !self.pending_bindings.is_empty()
    }

    /// Resolves any behaviors that were not matched by built-in behaviors against
    /// `registry`. Call this once after scene construction, passing the world's
    /// [`ModBehaviorRegistry`].
    ///
    /// After this call, `pending_bindings` is always cleared — any names not found in the
    /// registry are silently dropped (they are genuinely unknown) and will not be retried.
    pub fn apply_mod_behavior_registry(&mut self, registry: &ModBehaviorRegistry) {
        if self.pending_bindings.is_empty() {
            return;
        }
        let bindings = std::mem::take(&mut self.pending_bindings);
        // Pass the registry; remaining unknowns are stored back in pending_bindings by
        // attach_declared_behaviors, but we clear it afterwards to prevent per-frame retries.
        self.attach_declared_behaviors(bindings, Some(registry));
        // Clear any leftover unknowns — they are unresolvable.
        self.pending_bindings.clear();
    }
}

fn runtime_impact_for_vector_property(path: &str) -> RuntimeMutationImpact {
    match path {
        "vector.points" | "vector.closed" | "vector.draw_char" | "style.border"
        | "style.shadow" => RuntimeMutationImpact::layout(),
        _ => RuntimeMutationImpact::NONE,
    }
}

fn runtime_impact_for_obj_material_param(
    param: &crate::mutations::ObjMaterialParam,
) -> RuntimeMutationImpact {
    use crate::mutations::ObjMaterialParam;

    match param {
        ObjMaterialParam::Scale
        | ObjMaterialParam::Yaw
        | ObjMaterialParam::Pitch
        | ObjMaterialParam::Roll
        | ObjMaterialParam::OrbitSpeed
        | ObjMaterialParam::SurfaceMode => RuntimeMutationImpact::props(),
        ObjMaterialParam::ClipYMin | ObjMaterialParam::ClipYMax => RuntimeMutationImpact::layout(),
        _ => RuntimeMutationImpact::NONE,
    }
}

fn runtime_impact_for_grouped_render3d_param(
    param: &crate::mutations::Render3DGroupedParam,
) -> RuntimeMutationImpact {
    match param {
        crate::mutations::Render3DGroupedParam::Material(param) => {
            runtime_impact_for_obj_material_param(param)
        }
        crate::mutations::Render3DGroupedParam::View(param) => match param {
            crate::mutations::ViewParam::Yaw
            | crate::mutations::ViewParam::Pitch
            | crate::mutations::ViewParam::Roll => RuntimeMutationImpact::props(),
            crate::mutations::ViewParam::Distance => RuntimeMutationImpact::NONE,
        },
        _ => RuntimeMutationImpact::NONE,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeSpriteKind {
    Text,
    Image,
    Obj,
    Planet,
    Scene3D,
    Vector,
    Container,
}

fn grouped_render_param_label(param: &crate::mutations::Render3DGroupedParam) -> String {
    format!("{param:?}")
}

fn obj_material_param_label(param: &crate::mutations::ObjMaterialParam) -> String {
    format!("{param:?}")
}

fn atmosphere_param_label(param: &crate::mutations::AtmosphereParam) -> String {
    format!("{param:?}")
}

fn terrain_param_label(param: &crate::mutations::TerrainParam) -> String {
    format!("{param:?}")
}

fn worldgen_param_label(param: &crate::mutations::WorldgenParam) -> String {
    format!("{param:?}")
}

fn planet_param_label(param: &crate::mutations::PlanetParam) -> String {
    format!("{param:?}")
}

fn format_scene_mutation_error(error: &engine_api::scene::SceneMutationError) -> String {
    match error {
        engine_api::scene::SceneMutationError::InvalidRequest { request, detail }
        | engine_api::scene::SceneMutationError::UnsupportedRequest { request, detail } => {
            format!("request={request} detail={detail}")
        }
        engine_api::scene::SceneMutationError::TargetNotFound { target } => {
            format!("target `{target}` was not found")
        }
    }
}

fn has_scene_audio(scene: &Scene) -> bool {
    !scene.audio.on_enter.is_empty()
        || !scene.audio.on_idle.is_empty()
        || !scene.audio.on_leave.is_empty()
}

impl SceneRuntime {
    fn spawn_layer_clone_from_object(&mut self, template_object: GameObject, target: &str) -> bool {
        let layer_object_id = if matches!(template_object.kind, GameObjectKind::Layer) {
            template_object.id
        } else {
            let Some(parent_id) = template_object.parent_id else {
                return false;
            };
            let Some(parent_object) = self.objects.get(&parent_id) else {
                return false;
            };
            if !matches!(parent_object.kind, GameObjectKind::Layer) {
                return false;
            }
            parent_id
        };

        let Some((template_layer_idx, _)) = self
            .layer_ids
            .iter()
            .find(|(_, object_id)| *object_id == &layer_object_id)
            .map(|(idx, object_id)| (*idx, object_id.clone()))
        else {
            return false;
        };
        if template_layer_idx >= self.scene.layers.len() {
            return false;
        }

        let mut cloned_layer = self.scene.layers[template_layer_idx].clone();
        cloned_layer.name = target.to_string();
        cloned_layer.visible = true; // clones are always visible (template may be hidden)
        let mut id_counter: usize = 0;
        for sprite in &mut cloned_layer.sprites {
            retag_sprite_ids(sprite, target, &mut id_counter);
        }

        let new_layer_idx = self.scene.layers.len();
        let new_layer_object_id = format!(
            "{}/layer:{}:{}",
            self.root_id,
            new_layer_idx,
            sanitize_fragment_runtime(target)
        );
        self.objects.insert(
            new_layer_object_id.clone(),
            GameObject {
                id: new_layer_object_id.clone(),
                name: target.to_string(),
                kind: GameObjectKind::Layer,
                aliases: vec![target.to_string()],
                parent_id: Some(self.root_id.clone()),
                children: Vec::new(),
            },
        );
        self.object_states
            .insert(new_layer_object_id.clone(), ObjectRuntimeState::default());
        self.layer_ids
            .insert(new_layer_idx, new_layer_object_id.clone());
        if let Some(root) = self.objects.get_mut(&self.root_id) {
            root.children.push(new_layer_object_id.clone());
        }

        // Register sprites before pushing the layer (avoids redundant clone).
        for (sprite_idx, sprite) in cloned_layer.sprites.iter().enumerate() {
            register_runtime_sprite(
                &mut self.objects,
                &mut self.object_states,
                &mut self.sprite_ids,
                new_layer_idx,
                &[sprite_idx],
                &new_layer_object_id,
                sprite,
                sprite_idx,
            );
        }

        // Push the layer without cloning again (was previously cloned redundantly).
        self.scene.layers.push(cloned_layer);

        // Runtime clones intentionally reserve the target alias for the layer object.
        // Child sprites are still addressable by their authored `id` values via the
        // scene tree, but they must not compete for the same resolver alias.
        self.clear_conflicting_child_aliases(&new_layer_object_id);

        // Defer cache refresh if we are inside a spawn batch (e.g. apply_behavior_commands).
        if self.spawn_batch_depth == 0 {
            self.refresh_runtime_caches();
        }
        true
    }

    fn spawn_sprite_clone(&mut self, template_object: &GameObject, target: &str) -> bool {
        let Some((layer_idx, sprite_path)) = self.find_sprite_path_for_object(&template_object.id)
        else {
            return false;
        };
        let Some(parent_id) = template_object.parent_id.as_deref() else {
            return false;
        };
        let Some(mut cloned_sprite) = self.sprite_clone_at_path(layer_idx, &sprite_path) else {
            return false;
        };

        let mut id_counter = 0usize;
        retag_sprite_ids(&mut cloned_sprite, target, &mut id_counter);

        let parent_path = &sprite_path[..sprite_path.len().saturating_sub(1)];
        let Some(siblings) =
            sprite_children_mut_at_path(&mut self.scene.layers[layer_idx].sprites, parent_path)
        else {
            return false;
        };
        let new_index = siblings.len();
        siblings.push(cloned_sprite.clone());

        let mut new_path = parent_path.to_vec();
        new_path.push(new_index);
        register_runtime_sprite(
            &mut self.objects,
            &mut self.object_states,
            &mut self.sprite_ids,
            layer_idx,
            &new_path,
            parent_id,
            &cloned_sprite,
            new_index,
        );
        self.refresh_runtime_caches();
        true
    }

    fn find_sprite_path_for_object(&self, object_id: &str) -> Option<(usize, Vec<usize>)> {
        self.sprite_ids.iter().find_map(|(path_key, runtime_id)| {
            if runtime_id != object_id {
                return None;
            }
            parse_path_key_runtime(path_key)
        })
    }

    fn sprite_clone_at_path(&self, layer_idx: usize, sprite_path: &[usize]) -> Option<Sprite> {
        let sprites = self.scene.layers.get(layer_idx)?.sprites.as_slice();
        sprite_at_path(sprites, sprite_path).cloned()
    }

    fn refresh_runtime_caches(&mut self) {
        self.cached_object_kinds = std::sync::Arc::new(
            self.objects
                .iter()
                .map(|(id, object)| (id.clone(), runtime_kind_name(&object.kind).to_string()))
                .collect(),
        );
        self.resolver_cache = std::sync::Arc::new(self.build_target_resolver());
        self.rebuild_sprite_id_to_layer();
    }

    /// Builds an O(1) lookup from sprite `id` attribute to the layer index that
    /// contains it. Used by `set_vector_sprite_property` and friends to skip
    /// the O(n_layers × n_sprites) linear scan.
    pub(crate) fn rebuild_sprite_id_to_layer(&mut self) {
        self.sprite_id_to_layer.clear();
        for (layer_idx, layer) in self.scene.layers.iter().enumerate() {
            collect_sprite_ids_recursive(&layer.sprites, layer_idx, &mut self.sprite_id_to_layer);
        }
    }

    /// Batch-remove multiple scene targets with a single graph rebuild at the end.
    fn batch_despawn_targets(&mut self, resolver: &TargetResolver, targets: &[String]) -> bool {
        if targets.is_empty() {
            return false;
        }

        // For a single target, use the existing path (handles edge cases like
        // remove_target_recursive for non-layer sprites).
        if targets.len() == 1 {
            return self.soft_despawn_target(resolver, &targets[0]);
        }

        // Build a fresh resolver once for the entire batch.
        let current_resolver = self.build_target_resolver();
        let mut layers_to_remove: Vec<usize> = Vec::new();
        let mut non_layer_targets: Vec<String> = Vec::new();
        let mut any_removed = false;

        for target in targets {
            let object_id = if let Some(id) = current_resolver.resolve_alias(target) {
                id.to_string()
            } else if let Some(id) = resolver.resolve_alias(target) {
                id.to_string()
            } else {
                continue;
            };

            // Check if it's a sprite inside a layer
            if let Some((layer_idx, sprite_path)) = self.find_sprite_path_for_object(&object_id) {
                if let Some(layer) = self.scene.layers.get_mut(layer_idx) {
                    if remove_sprite_at_path(&mut layer.sprites, &sprite_path).is_some() {
                        any_removed = true;
                    }
                    if layer.sprites.is_empty() {
                        layers_to_remove.push(layer_idx);
                    }
                }
                continue;
            }
            // Check if it's a layer itself
            if let Some(layer_idx) = self
                .layer_ids
                .iter()
                .find(|(_, id)| *id == &object_id)
                .map(|(idx, _)| *idx)
            {
                if layer_idx < self.scene.layers.len() {
                    layers_to_remove.push(layer_idx);
                }
                continue;
            }
            // Non-layer target (rare)
            non_layer_targets.push(object_id);
        }

        // Remove layers in reverse index order to avoid invalidating earlier indices.
        layers_to_remove.sort_unstable();
        layers_to_remove.dedup();
        for &layer_idx in layers_to_remove.iter().rev() {
            if layer_idx < self.scene.layers.len() {
                self.scene.layers.remove(layer_idx);
                any_removed = true;
            }
        }

        // Single graph rebuild for all batch removals.
        if !layers_to_remove.is_empty() {
            self.rebuild_runtime_graph_preserving_state();
        }

        // Handle non-layer targets individually (rare path, typically 0).
        for object_id in non_layer_targets {
            any_removed |= self.remove_target_recursive(&object_id);
        }

        any_removed
    }

    fn soft_despawn_target(&mut self, resolver: &TargetResolver, target: &str) -> bool {
        // Prefer the fresh resolver: after a previous despawn in the same batch,
        // rebuild_runtime_graph_preserving_state renumbers layer indices, making
        // the passed resolver's object_ids stale. Trying the stale resolver first
        // would silently fail and leak orphan visuals.
        let current_resolver = self.build_target_resolver();
        let object_id = if let Some(id) = current_resolver.resolve_alias(target) {
            id.to_string()
        } else if let Some(id) = resolver.resolve_alias(target) {
            id.to_string()
        } else {
            return false;
        };
        if let Some((layer_idx, sprite_path)) = self.find_sprite_path_for_object(&object_id) {
            let Some(layer) = self.scene.layers.get_mut(layer_idx) else {
                return false;
            };
            if remove_sprite_at_path(&mut layer.sprites, &sprite_path).is_none() {
                return false;
            }
            // When a runtime-cloned layer's last child sprite is removed, the
            // layer becomes an empty shell.  Remove it to prevent orphaned empty
            // layers from accumulating (e.g. particle FX layers).
            if self
                .scene
                .layers
                .get(layer_idx)
                .is_some_and(|l| l.sprites.is_empty())
            {
                self.scene.layers.remove(layer_idx);
            }
            self.rebuild_runtime_graph_preserving_state();
            return true;
        }
        if let Some(layer_idx) = self
            .layer_ids
            .iter()
            .find(|(_, id)| *id == &object_id)
            .map(|(idx, _)| *idx)
        {
            if layer_idx >= self.scene.layers.len() {
                return false;
            }
            self.scene.layers.remove(layer_idx);
            self.rebuild_runtime_graph_preserving_state();
            return true;
        }
        self.remove_target_recursive(&object_id)
    }

    fn rebuild_runtime_graph_preserving_state(&mut self) {
        let preserved_states = self.object_states.clone();
        let preserved_camera_states = self.obj_camera_states.clone();
        let mut objects = HashMap::new();
        let mut object_states = HashMap::new();
        let mut layer_ids = BTreeMap::new();
        let mut sprite_ids = HashMap::new();
        let mut obj_camera_states = HashMap::new();

        objects.insert(
            self.root_id.clone(),
            GameObject {
                id: self.root_id.clone(),
                name: self.scene.id.clone(),
                kind: GameObjectKind::Scene,
                aliases: vec![self.scene.id.clone()],
                parent_id: None,
                children: Vec::new(),
            },
        );
        object_states.insert(
            self.root_id.clone(),
            preserved_states
                .get(&self.root_id)
                .cloned()
                .unwrap_or_default(),
        );

        for (layer_idx, layer) in self.scene.layers.iter().enumerate() {
            let layer_name = if layer.name.trim().is_empty() {
                format!("layer-{layer_idx}")
            } else {
                layer.name.clone()
            };
            let layer_id = format!(
                "{}/layer:{}:{}",
                self.root_id,
                layer_idx,
                sanitize_fragment_runtime(&layer_name)
            );
            objects.insert(
                layer_id.clone(),
                GameObject {
                    id: layer_id.clone(),
                    name: layer_name,
                    kind: GameObjectKind::Layer,
                    aliases: if layer.name.trim().is_empty() {
                        vec![]
                    } else {
                        vec![layer.name.clone()]
                    },
                    parent_id: Some(self.root_id.clone()),
                    children: Vec::new(),
                },
            );
            object_states.insert(
                layer_id.clone(),
                preserved_states.get(&layer_id).cloned().unwrap_or_default(),
            );
            layer_ids.insert(layer_idx, layer_id.clone());
            if let Some(root) = objects.get_mut(&self.root_id) {
                root.children.push(layer_id.clone());
            }

            for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
                register_runtime_sprite_preserving_state(
                    &mut objects,
                    &mut object_states,
                    &mut sprite_ids,
                    &mut obj_camera_states,
                    &preserved_states,
                    &preserved_camera_states,
                    layer_idx,
                    &[sprite_idx],
                    &layer_id,
                    sprite,
                    sprite_idx,
                );
            }
        }

        self.objects = objects;
        self.object_states = object_states;
        self.layer_ids = layer_ids;
        self.sprite_ids = sprite_ids;
        self.obj_camera_states = obj_camera_states;
        self.cached_obj_camera_states = None;
        self.clear_conflicting_child_aliases_for_all_layers();
        self.refresh_runtime_caches();
    }

    fn remove_target_recursive(&mut self, root_id: &str) -> bool {
        let mut queue = vec![root_id.to_string()];
        let mut removed_count = 0;

        while let Some(id) = queue.pop() {
            // Collect children before removing
            if let Some(object) = self.objects.get(&id) {
                for child_id in &object.children {
                    queue.push(child_id.clone());
                }
            }

            // Remove the object and its state
            if self.objects.remove(&id).is_some() {
                removed_count += 1;
            }
            if self.object_states.remove(&id).is_some() {
                removed_count += 1;
            }
        }

        removed_count > 0
    }

    fn set_target_visibility_recursive(&mut self, root_id: &str, visible: bool) -> bool {
        let mut queue = vec![root_id.to_string()];
        let mut any = false;
        while let Some(id) = queue.pop() {
            if let Some(state) = self.object_states.get_mut(&id) {
                state.visible = visible;
                any = true;
            }
            if let Some(object) = self.objects.get(&id) {
                for child_id in &object.children {
                    queue.push(child_id.clone());
                }
            }
        }
        any
    }

    fn clear_conflicting_child_aliases_for_all_layers(&mut self) {
        let layer_ids: Vec<String> = self.layer_ids.values().cloned().collect();
        for layer_id in layer_ids {
            self.clear_conflicting_child_aliases(&layer_id);
        }
    }

    fn clear_conflicting_child_aliases(&mut self, layer_object_id: &str) {
        let Some(layer_obj) = self.objects.get(layer_object_id) else {
            return;
        };
        if layer_obj.aliases.is_empty() {
            return;
        }
        let layer_aliases = layer_obj.aliases.clone();
        let child_ids = layer_obj.children.clone();
        for child_id in child_ids {
            let Some(child_obj) = self.objects.get_mut(&child_id) else {
                continue;
            };
            if child_obj
                .aliases
                .iter()
                .any(|alias| layer_aliases.iter().any(|layer_alias| layer_alias == alias))
            {
                child_obj.aliases.clear();
            }
        }
    }
}

/// Build the shared key fields (code/ctrl/alt/shift/pressed/released) for Rhai key maps.
fn build_base_key_fields(map: &mut rhai::Map, key: Option<&RawKeyEvent>) {
    if let Some(k) = key {
        map.insert("code".into(), k.code.clone().into());
        map.insert("ctrl".into(), k.ctrl.into());
        map.insert("alt".into(), k.alt.into());
        map.insert("shift".into(), k.shift.into());
        map.insert("pressed".into(), k.pressed.into());
        map.insert("released".into(), (!k.pressed).into());
    } else {
        map.insert("code".into(), "".into());
        map.insert("ctrl".into(), false.into());
        map.insert("alt".into(), false.into());
        map.insert("shift".into(), false.into());
        map.insert("pressed".into(), false.into());
        map.insert("released".into(), false.into());
    }
}

fn sanitize_fragment_runtime(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "_".to_string()
    } else {
        out
    }
}

fn path_key_runtime(layer_idx: usize, sprite_path: &[usize]) -> String {
    let mut key = layer_idx.to_string();
    for idx in sprite_path {
        key.push('/');
        key.push_str(&idx.to_string());
    }
    key
}

fn parse_path_key_runtime(path_key: &str) -> Option<(usize, Vec<usize>)> {
    let mut parts = path_key.split('/');
    let layer_idx = parts.next()?.parse::<usize>().ok()?;
    let mut sprite_path = Vec::new();
    for part in parts {
        sprite_path.push(part.parse::<usize>().ok()?);
    }
    Some((layer_idx, sprite_path))
}

fn sprite_at_path<'a>(sprites: &'a [Sprite], sprite_path: &[usize]) -> Option<&'a Sprite> {
    let (first, rest) = sprite_path.split_first()?;
    let sprite = sprites.get(*first)?;
    if rest.is_empty() {
        return Some(sprite);
    }
    match sprite {
        Sprite::Grid { children, .. }
        | Sprite::Flex { children, .. }
        | Sprite::Panel { children, .. } => sprite_at_path(children, rest),
        _ => None,
    }
}

fn sprite_children_mut_at_path<'a>(
    sprites: &'a mut Vec<Sprite>,
    sprite_path: &[usize],
) -> Option<&'a mut Vec<Sprite>> {
    let (first, rest) = match sprite_path.split_first() {
        Some(parts) => parts,
        None => return Some(sprites),
    };
    let sprite = sprites.get_mut(*first)?;
    match sprite {
        Sprite::Grid { children, .. }
        | Sprite::Flex { children, .. }
        | Sprite::Panel { children, .. } => sprite_children_mut_at_path(children, rest),
        _ => None,
    }
}

fn remove_sprite_at_path(sprites: &mut Vec<Sprite>, sprite_path: &[usize]) -> Option<Sprite> {
    let (first, rest) = sprite_path.split_first()?;
    if rest.is_empty() {
        if *first < sprites.len() {
            return Some(sprites.remove(*first));
        }
        return None;
    }
    let sprite = sprites.get_mut(*first)?;
    match sprite {
        Sprite::Grid { children, .. }
        | Sprite::Flex { children, .. }
        | Sprite::Panel { children, .. } => remove_sprite_at_path(children, rest),
        _ => None,
    }
}

fn runtime_kind_name(kind: &GameObjectKind) -> &'static str {
    match kind {
        GameObjectKind::Scene => "scene",
        GameObjectKind::Layer => "layer",
        GameObjectKind::TextSprite => "text",
        GameObjectKind::ImageSprite => "image",
        GameObjectKind::ObjSprite => "obj",
        GameObjectKind::PanelSprite => "panel",
        GameObjectKind::GridSprite => "grid",
        GameObjectKind::FlexSprite => "flex",
        GameObjectKind::VectorSprite => "vector",
    }
}

fn sprite_descriptor_runtime(
    sprite: &Sprite,
    sprite_idx: usize,
) -> (GameObjectKind, String, Vec<String>) {
    let id = sprite.id().unwrap_or_default().to_string();
    let name = if id.trim().is_empty() {
        format!("sprite-{sprite_idx}")
    } else {
        id.clone()
    };
    let aliases = if id.trim().is_empty() {
        vec![]
    } else {
        vec![id]
    };
    let kind = match sprite {
        Sprite::Text { .. } => GameObjectKind::TextSprite,
        Sprite::Image { .. } => GameObjectKind::ImageSprite,
        Sprite::Obj { .. } | Sprite::Planet { .. } | Sprite::Scene3D { .. } => {
            GameObjectKind::ObjSprite
        }
        Sprite::Panel { .. } => GameObjectKind::PanelSprite,
        Sprite::Grid { .. } => GameObjectKind::GridSprite,
        Sprite::Flex { .. } => GameObjectKind::FlexSprite,
        Sprite::Vector { .. } => GameObjectKind::VectorSprite,
    };
    (kind, name, aliases)
}

#[allow(clippy::too_many_arguments)]
fn register_runtime_sprite(
    objects: &mut HashMap<String, GameObject>,
    object_states: &mut HashMap<String, ObjectRuntimeState>,
    sprite_ids: &mut HashMap<String, String>,
    layer_idx: usize,
    sprite_path: &[usize],
    parent_id: &str,
    sprite: &Sprite,
    sprite_idx: usize,
) {
    let (kind, name, aliases) = sprite_descriptor_runtime(sprite, sprite_idx);
    let sprite_id = format!("{parent_id}/{name}");
    objects.insert(
        sprite_id.clone(),
        GameObject {
            id: sprite_id.clone(),
            name,
            kind,
            aliases,
            parent_id: Some(parent_id.to_string()),
            children: Vec::new(),
        },
    );
    object_states.insert(sprite_id.clone(), ObjectRuntimeState::default());
    sprite_ids.insert(path_key_runtime(layer_idx, sprite_path), sprite_id.clone());
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
            register_runtime_sprite(
                objects,
                object_states,
                sprite_ids,
                layer_idx,
                &child_path,
                &sprite_id,
                child,
                child_idx,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn register_runtime_sprite_preserving_state(
    objects: &mut HashMap<String, GameObject>,
    object_states: &mut HashMap<String, ObjectRuntimeState>,
    sprite_ids: &mut HashMap<String, String>,
    obj_camera_states: &mut HashMap<String, ObjCameraState>,
    preserved_states: &HashMap<String, ObjectRuntimeState>,
    preserved_camera_states: &HashMap<String, ObjCameraState>,
    layer_idx: usize,
    sprite_path: &[usize],
    parent_id: &str,
    sprite: &Sprite,
    sprite_idx: usize,
) {
    let (kind, name, aliases) = sprite_descriptor_runtime(sprite, sprite_idx);
    let sprite_id = format!("{parent_id}/{name}");
    objects.insert(
        sprite_id.clone(),
        GameObject {
            id: sprite_id.clone(),
            name,
            kind,
            aliases,
            parent_id: Some(parent_id.to_string()),
            children: Vec::new(),
        },
    );
    object_states.insert(
        sprite_id.clone(),
        preserved_states
            .get(&sprite_id)
            .cloned()
            .unwrap_or_default(),
    );
    if let Some(camera_state) = preserved_camera_states.get(&sprite_id) {
        obj_camera_states.insert(sprite_id.clone(), camera_state.clone());
    }
    sprite_ids.insert(path_key_runtime(layer_idx, sprite_path), sprite_id.clone());
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
            register_runtime_sprite_preserving_state(
                objects,
                object_states,
                sprite_ids,
                obj_camera_states,
                preserved_states,
                preserved_camera_states,
                layer_idx,
                &child_path,
                &sprite_id,
                child,
                child_idx,
            );
        }
    }
}

fn retag_sprite_ids(sprite: &mut Sprite, base: &str, counter: &mut usize) {
    let next_id = if *counter == 0 {
        base.to_string()
    } else {
        format!("{base}-{}", *counter)
    };
    *counter += 1;
    match sprite {
        Sprite::Text { id, .. }
        | Sprite::Image { id, .. }
        | Sprite::Obj { id, .. }
        | Sprite::Planet { id, .. }
        | Sprite::Panel { id, .. }
        | Sprite::Grid { id, .. }
        | Sprite::Flex { id, .. }
        | Sprite::Scene3D { id, .. }
        | Sprite::Vector { id, .. } => {
            *id = Some(next_id);
        }
    }
    match sprite {
        Sprite::Grid { children, .. }
        | Sprite::Flex { children, .. }
        | Sprite::Panel { children, .. } => {
            for child in children {
                retag_sprite_ids(child, base, counter);
            }
        }
        _ => {}
    }
}

/// Collects sprite `id` → layer_idx mappings for O(1) property mutation lookup.
fn collect_sprite_ids_recursive(
    sprites: &[Sprite],
    layer_idx: usize,
    out: &mut HashMap<String, usize>,
) {
    for sprite in sprites {
        if let Some(id) = sprite.id() {
            if !id.is_empty() {
                out.insert(id.to_string(), layer_idx);
            }
        }
        match sprite {
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                collect_sprite_ids_recursive(children, layer_idx, out);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::{
        commands::{scene_mutation_request_from_set_path, DebugLogSeverity},
        scene::{SceneMutationError, SceneMutationRequest, SceneMutationResult},
    };
    use engine_behavior::BehaviorCommand;
    use engine_core::scene::Scene;

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

    fn assert_single_debug_log(
        diagnostics: &[BehaviorCommand],
        severity: DebugLogSeverity,
        expected_fragment: &str,
    ) {
        assert!(matches!(
            diagnostics,
            [BehaviorCommand::DebugLog {
                scene_id,
                source,
                severity: actual_severity,
                message,
            }] if scene_id == "intro"
                && source.as_deref() == Some("scene-mutation")
                && *actual_severity == severity
                && message.contains(expected_fragment)
        ));
    }

    #[test]
    fn missing_target_returns_rejected_result_and_debug_log() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let command = BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::Set2dProps {
                target: "missing".to_string(),
                visible: Some(false),
                dx: None,
                dy: None,
                text: None,
            },
        };

        let mutation = runtime
            .scene_mutation_from_behavior_command(&resolver, &command)
            .expect("translation should succeed")
            .expect("typed mutation");
        let result = runtime.apply_scene_mutation(&resolver, &mutation);

        assert_eq!(
            result,
            SceneMutationResult::rejected(SceneMutationError::TargetNotFound {
                target: "missing".to_string(),
            })
        );

        let diagnostics = runtime.apply_behavior_commands(&resolver, &[command]);
        assert_single_debug_log(
            &diagnostics,
            DebugLogSeverity::Warn,
            "target `missing` was not found",
        );
    }

    #[test]
    fn invalid_payload_returns_explicit_error_and_debug_log() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let command = BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::SetSpriteProperty {
                target: "title".to_string(),
                path: "image.frame_index".to_string(),
                value: serde_json::json!("next"),
            },
        };

        let error = runtime
            .scene_mutation_from_behavior_command(&resolver, &command)
            .expect_err("invalid payload should be rejected");

        assert_eq!(
            error,
            SceneMutationError::InvalidRequest {
                request: "set_sprite_property".to_string(),
                detail: "path `image.frame_index` expects a u16 frame index".to_string(),
            }
        );

        let diagnostics = runtime.apply_behavior_commands(&resolver, &[command]);
        assert_single_debug_log(
            &diagnostics,
            DebugLogSeverity::Error,
            "path `image.frame_index` expects a u16 frame index",
        );
    }

    #[test]
    fn empty_set_2d_props_returns_explicit_error_and_debug_log() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let command = BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::Set2dProps {
                target: "title".to_string(),
                visible: None,
                dx: None,
                dy: None,
                text: None,
            },
        };

        let error = runtime
            .scene_mutation_from_behavior_command(&resolver, &command)
            .expect_err("empty payload should be rejected");

        assert_eq!(
            error,
            SceneMutationError::InvalidRequest {
                request: "set_2d_props".to_string(),
                detail: "set_2d_props requires at least one field".to_string(),
            }
        );

        let diagnostics = runtime.apply_behavior_commands(&resolver, &[command]);
        assert_single_debug_log(
            &diagnostics,
            DebugLogSeverity::Error,
            "set_2d_props requires at least one field",
        );
    }

    #[test]
    fn unsupported_legacy_set_path_returns_explicit_error_and_debug_log() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let command = BehaviorCommand::ApplySceneMutation {
            request: scene_mutation_request_from_set_path(
                "title",
                "audio.pitch",
                &serde_json::json!(2.0),
                None,
            )
            .expect("legacy wrapper should preserve rejection"),
        };

        let error = runtime
            .scene_mutation_from_behavior_command(&resolver, &command)
            .expect_err("unsupported path should be rejected");

        assert_eq!(
            error,
            SceneMutationError::UnsupportedRequest {
                request: "set_path".to_string(),
                detail: "target `title` does not support `audio.pitch`".to_string(),
            }
        );

        let diagnostics = runtime.apply_behavior_commands(&resolver, &[command]);
        assert_single_debug_log(
            &diagnostics,
            DebugLogSeverity::Warn,
            "target `title` does not support `audio.pitch`",
        );
    }

    #[test]
    fn empty_grouped_render_params_return_explicit_error_and_debug_log() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let command = BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::SetRender3d(
                engine_api::scene::Render3dMutationRequest::SetMaterialParams {
                    target: "title".to_string(),
                    params: serde_json::json!({}),
                },
            ),
        };

        let error = runtime
            .scene_mutation_from_behavior_command(&resolver, &command)
            .expect_err("empty grouped params should be rejected");

        assert_eq!(
            error,
            SceneMutationError::InvalidRequest {
                request: "set_material_params".to_string(),
                detail: "grouped params must not be empty".to_string(),
            }
        );

        let diagnostics = runtime.apply_behavior_commands(&resolver, &[command]);
        assert_single_debug_log(
            &diagnostics,
            DebugLogSeverity::Error,
            "grouped params must not be empty",
        );
    }

    #[test]
    fn spawn_missing_template_returns_explicit_error_and_debug_log() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let command = BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::SpawnObject {
                template: "missing-template".to_string(),
                target: "spawned".to_string(),
            },
        };

        let diagnostics = runtime.apply_behavior_commands(&resolver, &[command]);
        assert_single_debug_log(
            &diagnostics,
            DebugLogSeverity::Error,
            "template `missing-template` was not found",
        );
    }

    #[test]
    fn spawn_empty_template_returns_explicit_error_and_debug_log() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let command = BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::SpawnObject {
                template: "".to_string(),
                target: "spawned".to_string(),
            },
        };

        let diagnostics = runtime.apply_behavior_commands(&resolver, &[command]);
        assert_single_debug_log(
            &diagnostics,
            DebugLogSeverity::Error,
            "spawn_object requires non-empty template and target",
        );
    }

    #[test]
    fn despawn_missing_target_returns_explicit_error_and_debug_log() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let command = BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::DespawnObject {
                target: "missing".to_string(),
            },
        };

        let diagnostics = runtime.apply_behavior_commands(&resolver, &[command]);
        assert_single_debug_log(
            &diagnostics,
            DebugLogSeverity::Warn,
            "target `missing` was not found",
        );
    }

    #[test]
    fn despawn_empty_target_returns_explicit_error_and_debug_log() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let command = BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::DespawnObject {
                target: "".to_string(),
            },
        };

        let diagnostics = runtime.apply_behavior_commands(&resolver, &[command]);
        assert_single_debug_log(
            &diagnostics,
            DebugLogSeverity::Error,
            "despawn_object requires a non-empty target",
        );
    }

    #[test]
    fn text_mutation_on_non_text_target_returns_explicit_error_and_debug_log() {
        let mut runtime = SceneRuntime::new(intro_scene());
        let resolver = runtime.target_resolver();
        let command = BehaviorCommand::ApplySceneMutation {
            request: SceneMutationRequest::Set2dProps {
                target: "root-grid".to_string(),
                visible: None,
                dx: None,
                dy: None,
                text: Some("HELLO".to_string()),
            },
        };

        let mutation = runtime
            .scene_mutation_from_behavior_command(&resolver, &command)
            .expect("translation should succeed")
            .expect("typed mutation");
        let result = runtime.apply_scene_mutation(&resolver, &mutation);

        assert_eq!(
            result,
            SceneMutationResult::rejected(SceneMutationError::UnsupportedRequest {
                request: "set_2d_props".to_string(),
                detail: "target `root-grid` does not support `text.content`".to_string(),
            })
        );

        let diagnostics = runtime.apply_behavior_commands(&resolver, &[command]);
        assert_single_debug_log(
            &diagnostics,
            DebugLogSeverity::Warn,
            "target `root-grid` does not support `text.content`",
        );
    }
}

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
        // Mark all per-frame derived caches dirty.
        self.effective_states_dirty = true;
        self.cached_object_states = None;
        self.cached_object_props = None;
        self.cached_object_text = None;
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
        let object_regions = std::sync::Arc::clone(&self.cached_object_regions);
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
            self.apply_behavior_commands(&resolver, &local_commands);
            commands.extend(local_commands.iter().cloned());
            // Update ctx object_states after each behavior emits commands, so subsequent
            // behaviors see the mutations. effective_object_states_snapshot() uses gen-counter
            // gating to skip rebuilds on mutation-free frames (the common case).
            if !local_commands.is_empty() && idx + 1 < self.behaviors.len() {
                ctx.object_states = self.effective_object_states_snapshot();
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
                Some(engine_behavior::BehaviorCommand::SetProperty {
                    target: binding.target.clone(),
                    path: binding.prop.clone(),
                    value: serde_json::Value::String(color.clone()),
                })
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
            self.effective_states_dirty = true;
            self.object_mutation_gen = self.object_mutation_gen.wrapping_add(1);
            self.cached_object_text = None;
        }
        self.game_state_applied_version = current_version;
    }

    /// Applies behavior commands to runtime object state using the supplied
    /// target resolver.
    pub fn apply_behavior_commands(
        &mut self,
        resolver: &TargetResolver,
        commands: &[BehaviorCommand],
    ) {
        if commands.is_empty() {
            return;
        }
        self.effective_states_dirty = true;
        self.object_mutation_gen = self.object_mutation_gen.wrapping_add(1);
        self.cached_object_states = None;
        self.cached_object_props = None;
        self.cached_object_text = None;

        // Collect despawn targets for batched removal (single graph rebuild).
        let mut pending_despawns: Vec<String> = Vec::new();
        // Enable batch spawn mode: defer refresh_runtime_caches() per spawn.
        self.spawn_batch_depth += 1;
        let mut had_spawns = false;

        for command in commands {
            if let Some(mutation) = self.scene_mutation_from_behavior_command(command) {
                self.apply_scene_mutation(resolver, &mutation);
                continue;
            }

            match command {
                BehaviorCommand::PlayAudioCue { .. } => {}
                BehaviorCommand::PlayAudioEvent { .. } => {}
                BehaviorCommand::PlaySong { .. } => {}
                BehaviorCommand::StopSong => {}
                BehaviorCommand::SetVisibility { .. } => {}
                BehaviorCommand::SetOffset { .. } => {}
                BehaviorCommand::SetText { .. } => {}
                BehaviorCommand::SetProps { .. } => {}
                BehaviorCommand::ApplySceneMutation { request } => match request {
                    engine_api::scene::SceneMutationRequest::SpawnObject { template, target } => {
                        if self.spawn_runtime_clone(resolver, template, target) {
                            had_spawns = true;
                        }
                    }
                    engine_api::scene::SceneMutationRequest::DespawnObject { target } => {
                        pending_despawns.push(target.clone());
                    }
                    _ => {}
                },
                BehaviorCommand::SetProperty {
                    target,
                    path,
                    value,
                } => {
                    if crate::render3d_state::is_render3d_compat_param_path(path) {
                        continue;
                    }
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
                            let Some(next_x) = json_value_to_rounded_i32(value) else {
                                continue;
                            };
                            if let Some(state) = self.object_states.get_mut(object_id) {
                                state.offset_x = next_x;
                            }
                        }
                        "offset.y" | "position.y" => {
                            let Some(next_y) = json_value_to_rounded_i32(value) else {
                                continue;
                            };
                            if let Some(state) = self.object_states.get_mut(object_id) {
                                state.offset_y = next_y;
                            }
                        }
                        "transform.heading" => {
                            let Some(next_heading) = value.as_f64() else {
                                continue;
                            };
                            let heading = next_heading as f32;
                            if let Some(state) = self.object_states.get_mut(object_id) {
                                state.heading = heading;
                            }
                            // Cascade heading to child sprites when target is a layer.
                            // Uses index-based iteration to avoid cloning the children Vec.
                            if let Some(obj) = self.objects.get(object_id) {
                                if matches!(obj.kind, GameObjectKind::Layer) {
                                    let n = obj.children.len();
                                    for i in 0..n {
                                        let cid = self.objects[object_id].children[i].clone();
                                        if let Some(state) = self.object_states.get_mut(&cid) {
                                            state.heading = heading;
                                        }
                                    }
                                }
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
                            let mut applied = self.apply_text_property_for_target(
                                object_id,
                                target,
                                |runtime, alias| {
                                    runtime.set_text_sprite_fg_colour(alias, next_colour.clone())
                                },
                            );
                            if !applied {
                                applied =
                                    self.set_vector_sprite_property(target, "style.fg", value);
                            }
                            if !applied {
                                for alias in self.object_alias_candidates(object_id, target) {
                                    if self.set_vector_sprite_property(&alias, "style.fg", value) {
                                        applied = true;
                                        break;
                                    }
                                }
                            }
                            if !applied {
                                continue;
                            }
                        }
                        "style.bg" | "text.bg" => {
                            let Some(next_colour) = parse_term_colour(value) else {
                                continue;
                            };
                            let mut applied = self.apply_text_property_for_target(
                                object_id,
                                target,
                                |runtime, alias| {
                                    runtime.set_text_sprite_bg_colour(alias, next_colour.clone())
                                },
                            );
                            if !applied {
                                applied =
                                    self.set_vector_sprite_property(target, "style.bg", value);
                            }
                            if !applied {
                                for alias in self.object_alias_candidates(object_id, target) {
                                    if self.set_vector_sprite_property(&alias, "style.bg", value) {
                                        applied = true;
                                        break;
                                    }
                                }
                            }
                            if !applied {
                                continue;
                            }
                        }
                        "vector.points" | "vector.closed" | "vector.draw_char" | "vector.fg"
                        | "vector.bg" | "style.border" | "style.shadow" => {
                            let mut applied = self.set_vector_sprite_property(target, path, value);
                            if !applied {
                                for alias in self.object_alias_candidates(object_id, target) {
                                    if self.set_vector_sprite_property(&alias, path, value) {
                                        applied = true;
                                        break;
                                    }
                                }
                            }
                            if !applied {
                                continue;
                            }
                        }
                        "image.frame_index" => {
                            let Some(next_frame) = value.as_u64() else {
                                continue;
                            };
                            let mut applied =
                                self.set_image_sprite_frame_index(target, next_frame as u16);
                            if !applied {
                                for alias in self.object_alias_candidates(object_id, target) {
                                    if self.set_image_sprite_frame_index(&alias, next_frame as u16)
                                    {
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
                BehaviorCommand::SceneSpawn { template, target } => {
                    if self.spawn_runtime_clone(resolver, template, target) {
                        had_spawns = true;
                    }
                }
                BehaviorCommand::SceneDespawn { target } => {
                    pending_despawns.push(target.clone());
                }
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
                        ws.changed = true;
                        self.gui_state.last_changed = Some(widget_id.clone());
                        self.cached_gui_state = None;
                    }
                    self.sync_widget_visuals();
                }
            }
        }

        // End batch spawn mode and do a single cache refresh if any spawns happened.
        self.spawn_batch_depth -= 1;
        if had_spawns && self.spawn_batch_depth == 0 {
            self.refresh_runtime_caches();
        }

        // Batch-apply all collected despawns with a single graph rebuild.
        if !pending_despawns.is_empty() {
            self.batch_despawn_targets(resolver, &pending_despawns);
        }
    }

    fn scene_mutation_from_behavior_command(
        &self,
        command: &BehaviorCommand,
    ) -> Option<SceneMutation> {
        match command {
            BehaviorCommand::SetVisibility { target, visible } => {
                Some(SceneMutation::Set2DProps(Set2DPropsMutation {
                    target: target.clone(),
                    visible: Some(*visible),
                    dx: None,
                    dy: None,
                    text: None,
                }))
            }
            BehaviorCommand::SetOffset { target, dx, dy } => {
                Some(SceneMutation::Set2DProps(Set2DPropsMutation {
                    target: target.clone(),
                    visible: None,
                    dx: Some(*dx),
                    dy: Some(*dy),
                    text: None,
                }))
            }
            BehaviorCommand::SetText { target, text } => {
                Some(SceneMutation::Set2DProps(Set2DPropsMutation {
                    target: target.clone(),
                    visible: None,
                    dx: None,
                    dy: None,
                    text: Some(text.clone()),
                }))
            }
            BehaviorCommand::SetProps {
                target,
                visible,
                dx,
                dy,
                text,
            } => Some(SceneMutation::Set2DProps(Set2DPropsMutation {
                target: target.clone(),
                visible: *visible,
                dx: *dx,
                dy: *dy,
                text: text.clone(),
            })),
            BehaviorCommand::SetCamera { x, y } => {
                Some(SceneMutation::SetCamera2D(SetCamera2DMutation {
                    x: x.round() as i32,
                    y: y.round() as i32,
                    zoom: None,
                }))
            }
            BehaviorCommand::SetCameraZoom { zoom } => {
                Some(SceneMutation::SetCamera2D(SetCamera2DMutation {
                    x: self.camera_x,
                    y: self.camera_y,
                    zoom: Some(*zoom),
                }))
            }
            BehaviorCommand::SetCamera3DLookAt { eye, look_at } => scene_mutation_from_request(
                &engine_api::scene::SceneMutationRequest::SetCamera3d(
                    engine_api::scene::Camera3dMutationRequest::LookAt {
                        eye: *eye,
                        look_at: *look_at,
                    },
                ),
                self.scene_camera_3d,
            ),
            BehaviorCommand::SetCamera3DUp { up } => scene_mutation_from_request(
                &engine_api::scene::SceneMutationRequest::SetCamera3d(
                    engine_api::scene::Camera3dMutationRequest::Up { up: *up },
                ),
                self.scene_camera_3d,
            ),
            BehaviorCommand::ApplySceneMutation { request } => {
                if matches!(
                    request,
                    engine_api::scene::SceneMutationRequest::SpawnObject { .. }
                        | engine_api::scene::SceneMutationRequest::DespawnObject { .. }
                ) {
                    return None;
                }
                scene_mutation_from_request(request, self.scene_camera_3d)
            }
            BehaviorCommand::SetProperty {
                target,
                path,
                value,
            } => scene_mutation_from_set_property_3d(target, path, value),
            _ => None,
        }
    }

    fn apply_scene_mutation(&mut self, resolver: &TargetResolver, mutation: &SceneMutation) {
        let mut mutation_applied = false;
        match mutation {
            SceneMutation::Set2DProps(props) => {
                let Some(object_id) = resolver.resolve_alias(&props.target) else {
                    return;
                };
                if let Some(state) = self.object_states.get_mut(object_id) {
                    if let Some(next_visible) = props.visible {
                        state.visible = next_visible;
                        mutation_applied = true;
                    }
                    if let Some(delta_x) = props.dx {
                        state.offset_x = state.offset_x.saturating_add(delta_x);
                        mutation_applied = true;
                    }
                    if let Some(delta_y) = props.dy {
                        state.offset_y = state.offset_y.saturating_add(delta_y);
                        mutation_applied = true;
                    }
                }
                if let Some(next_text) = &props.text {
                    let _ = self.apply_text_property_for_target(
                        object_id,
                        &props.target,
                        |runtime, alias| runtime.set_text_sprite_content(alias, next_text.clone()),
                    );
                    mutation_applied = true;
                }
            }
            SceneMutation::SetCamera2D(camera) => {
                self.set_camera_internal(camera.x, camera.y);
                mutation_applied = true;
                if let Some(zoom) = camera.zoom {
                    self.set_camera_zoom_internal(zoom);
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
                Render3DMutation::SetNodeVisibility { target, visible } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return;
                    };
                    if let Some(state) = self.object_states.get_mut(object_id) {
                        state.visible = *visible;
                        mutation_applied = true;
                    }
                }
                Render3DMutation::SetNodeTransform { target, transform } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return;
                    };
                    if let Some(state) = self.object_states.get_mut(object_id) {
                        state.offset_x = transform.translation[0].round() as i32;
                        state.offset_y = transform.translation[1].round() as i32;
                        mutation_applied = true;
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
                Render3DMutation::SetWorldgenParam {
                    target,
                    param,
                    value,
                } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return;
                    };
                    let Some(property) = crate::render3d_state::render3d_compat_property_from_param(
                        param,
                        value.clone(),
                    ) else {
                        return;
                    };
                    if !self.apply_render3d_compat_property_for_target(object_id, target, &property)
                    {
                        return;
                    }
                    mutation_applied = true;
                }
                Render3DMutation::SetCompatProperty { target, property } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        return;
                    };
                    if !self.apply_render3d_compat_property_for_target(object_id, target, property)
                    {
                        return;
                    }
                    mutation_applied = true;
                }
                Render3DMutation::SetMaterialParam { .. }
                | Render3DMutation::SetAtmosphereParam { .. }
                | Render3DMutation::SetLight { .. }
                | Render3DMutation::RebuildMesh { .. }
                | Render3DMutation::RebuildWorldgen { .. } => {
                    // Runtime does not directly mutate sprite fields for these, but
                    // downstream 3D render pipelines must still see invalidation.
                    mutation_applied = true;
                }
            },
            SceneMutation::SpawnObject { .. } => {}
            SceneMutation::DespawnObject { .. } => {}
        }
        if mutation_applied {
            let dirty = dirty_for_scene_mutation(mutation);
            self.render3d_dirty_mask.insert(dirty);
            self.track_render3d_rebuild_cause(dirty);
        }
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

fn has_scene_audio(scene: &Scene) -> bool {
    !scene.audio.on_enter.is_empty()
        || !scene.audio.on_idle.is_empty()
        || !scene.audio.on_leave.is_empty()
}

impl SceneRuntime {
    fn spawn_runtime_clone(
        &mut self,
        resolver: &TargetResolver,
        template: &str,
        target: &str,
    ) -> bool {
        if template.trim().is_empty() || target.trim().is_empty() {
            return false;
        }
        let current_resolver = self.build_target_resolver();
        let existing = resolver
            .resolve_alias(target)
            .or_else(|| current_resolver.resolve_alias(target))
            .map(str::to_string);
        if let Some(object_id) = existing {
            self.set_target_visibility_recursive(&object_id, true);
            return true;
        }

        let template_id = if let Some(id) = resolver.resolve_alias(template) {
            id.to_string()
        } else if let Some(id) = current_resolver.resolve_alias(template) {
            id.to_string()
        } else {
            return false;
        };
        let Some(template_object) = self.objects.get(&template_id).cloned() else {
            return false;
        };
        if matches!(template_object.kind, GameObjectKind::Layer) {
            return self.spawn_layer_clone_from_object(template_object, target);
        }

        self.spawn_sprite_clone(&template_object, target)
    }

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
    fn batch_despawn_targets(&mut self, resolver: &TargetResolver, targets: &[String]) {
        if targets.is_empty() {
            return;
        }

        // For a single target, use the existing path (handles edge cases like
        // remove_target_recursive for non-layer sprites).
        if targets.len() == 1 {
            let _ = self.soft_despawn_target(resolver, &targets[0]);
            return;
        }

        // Build a fresh resolver once for the entire batch.
        let current_resolver = self.build_target_resolver();
        let mut layers_to_remove: Vec<usize> = Vec::new();
        let mut non_layer_targets: Vec<String> = Vec::new();

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
                    let _ = remove_sprite_at_path(&mut layer.sprites, &sprite_path);
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
            }
        }

        // Single graph rebuild for all batch removals.
        if !layers_to_remove.is_empty() {
            self.rebuild_runtime_graph_preserving_state();
        }

        // Handle non-layer targets individually (rare path, typically 0).
        for object_id in non_layer_targets {
            self.remove_target_recursive(&object_id);
        }
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

fn json_value_to_rounded_i32(value: &JsonValue) -> Option<i32> {
    if let Some(number) = value.as_i64() {
        return i32::try_from(number).ok();
    }
    value
        .as_f64()
        .and_then(|number| i32::try_from(number.round() as i64).ok())
}

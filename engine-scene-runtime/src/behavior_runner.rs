use super::*;

impl SceneRuntime {
    /// Updates attached runtime behaviors for the active scene stage and
    /// applies the generated commands immediately.
    pub fn update_behaviors(
        &mut self,
        stage: SceneStage,
        scene_elapsed_ms: u64,
        stage_elapsed_ms: u64,
        menu_selected_index: usize,
        game_state: Option<engine_core::game_state::GameState>,
    ) -> Vec<BehaviorCommand> {
        self.terminal_shell_scene_elapsed_ms = scene_elapsed_ms;
        self.sync_terminal_shell_sprites();
        // Mark all per-frame derived caches dirty.
        self.effective_states_dirty = true;
        self.cached_object_states = None;
        self.cached_object_props = None;
        self.cached_object_text = None;
        // sidecar_io: build Arc once if not already cached from a prior
        // mutation-free frame; invalidated at each sidecar write site.
        let _sidecar_io = match &self.cached_sidecar_io {
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
        let sidecar_io = std::sync::Arc::new(self.ui_state.sidecar_io.clone());
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

        // Phase 7C: Build Rhai maps once per frame and wrap in Arc.
        // Behaviors will clone these Arc refs (O(1) refcount) instead of cloning maps (O(n_map)).
        use rhai::Map as RhaiMap;
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

        let rhai_key_map = {
            let mut key_map = RhaiMap::new();
            if let Some(k) = &self.ui_state.last_raw_key {
                key_map.insert("code".into(), k.code.clone().into());
                key_map.insert("ctrl".into(), k.ctrl.into());
                key_map.insert("alt".into(), k.alt.into());
                key_map.insert("shift".into(), k.shift.into());
                key_map.insert("pressed".into(), true.into());
            } else {
                key_map.insert("code".into(), "".into());
                key_map.insert("ctrl".into(), false.into());
                key_map.insert("alt".into(), false.into());
                key_map.insert("shift".into(), false.into());
                key_map.insert("pressed".into(), false.into());
            }
            std::sync::Arc::new(key_map)
        };

        // Engine-level key metadata for Rhai scope (separate `engine` namespace)
        let engine_key_map = {
            let mut engine_key = RhaiMap::new();
            if let Some(k) = &self.ui_state.last_raw_key {
                engine_key.insert("code".into(), k.code.clone().into());
                engine_key.insert("ctrl".into(), k.ctrl.into());
                engine_key.insert("alt".into(), k.alt.into());
                engine_key.insert("shift".into(), k.shift.into());
                engine_key.insert("pressed".into(), true.into());
                // Mark quit keys so behaviors can check without handling them
                let is_quit =
                    k.ctrl && (k.code == "q" || k.code == "Q" || k.code == "c" || k.code == "C");
                engine_key.insert("is_quit".into(), is_quit.into());
            } else {
                engine_key.insert("code".into(), "".into());
                engine_key.insert("ctrl".into(), false.into());
                engine_key.insert("alt".into(), false.into());
                engine_key.insert("shift".into(), false.into());
                engine_key.insert("pressed".into(), false.into());
                engine_key.insert("is_quit".into(), false.into());
            }
            std::sync::Arc::new(engine_key)
        };

        let mut commands = Vec::new();
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
            last_raw_key,
            sidecar_io,
            rhai_time_map,
            rhai_menu_map,
            rhai_key_map,
            engine_key_map,
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
        commands
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
                        | "obj.surface_mode" | "obj.clip_y_min" | "obj.clip_y_max" => {
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
                        "scene3d.frame" => {
                            let Some(next_frame) = value.as_str() else {
                                continue;
                            };
                            let mut applied = self.set_scene3d_sprite_frame(target, next_frame);
                            if !applied {
                                for alias in self.object_alias_candidates(object_id, target) {
                                    if self.set_scene3d_sprite_frame(&alias, next_frame) {
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
                            params.src = Some(format!("mod:{}", mod_behavior.name));
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

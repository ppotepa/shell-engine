//! Behavior system — evaluates all scene-object behaviors and dispatches resulting commands each frame.

use crate::audio::AudioCommand;
use crate::audio_sequencer::AudioSequencerState;
use crate::behavior::{BehaviorCommand, DebugLogSeverity as BehaviorDebugLogSeverity};
use crate::debug_log::{DebugLogBuffer, DebugLogEntry, DebugSeverity};
use crate::events::EngineEvent;
use crate::services::EngineWorldAccess;
use crate::systems::gameplay_events::GameplayEventBuffer;
use crate::world::World;
use engine_api::commands::{planet_apply_behavior_commands, PlanetApplySpec, PlanetBodyPatchSpec};
use engine_core::logging;

/// Runs all registered behaviors against the current scene runtime state and dispatches their commands.
pub fn behavior_system(world: &mut World) {
    let Some(animator) = world.animator() else {
        return;
    };
    let stage = animator.stage;
    let scene_elapsed_ms = animator.scene_elapsed_ms;
    let stage_elapsed_ms = animator.stage_elapsed_ms;
    let menu_selected_index = animator.menu_selected_index;

    let game_state = world.get::<crate::game_state::GameState>().cloned();
    let level_state = world.get::<crate::level_state::LevelState>().cloned();
    let persistence = world.get::<engine_persistence::PersistenceStore>().cloned();
    let gameplay_world = world.get::<crate::game::GameplayWorld>().cloned();
    let emitter_state = world.get::<engine_behavior::EmitterState>().cloned();
    let collisions = world
        .get::<GameplayEventBuffer>()
        .map(|buf| std::sync::Arc::new(buf.collisions.clone()))
        .unwrap_or_else(|| std::sync::Arc::new(Vec::new()));
    let catalogs = world
        .get::<engine_behavior::catalog::ModCatalogs>()
        .map(|cat| std::sync::Arc::new(cat.clone()))
        .unwrap_or_else(|| std::sync::Arc::new(engine_behavior::catalog::ModCatalogs::default()));
    let palettes = world
        .get::<engine_behavior::palette::PaletteStore>()
        .map(|p| std::sync::Arc::new(p.clone()))
        .unwrap_or_else(|| std::sync::Arc::new(engine_behavior::palette::PaletteStore::default()));
    let default_palette = world
        .get::<crate::mod_manifest::ModManifestData>()
        .and_then(|m| m.default_palette.clone());
    let debug_enabled = world
        .get::<crate::debug_features::DebugFeatures>()
        .map(|d| d.enabled)
        .unwrap_or(false);

    // Resolve any pending mod-behavior bindings on the first frame this scene is active.
    // The check for pending bindings avoids cloning the registry on every subsequent frame.
    let has_pending = world
        .scene_runtime()
        .map(|rt| rt.has_pending_bindings())
        .unwrap_or(false);
    if has_pending {
        let mod_registry = world
            .get::<crate::mod_behaviors::ModBehaviorRegistry>()
            .cloned();
        if let Some(registry) = mod_registry {
            if let Some(runtime) = world.scene_runtime_mut() {
                runtime.apply_mod_behavior_registry(&registry);
            }
        }
    }

    let (commands, render3d_rebuild_diagnostics, scene_id) = {
        let Some(runtime) = world.scene_runtime_mut() else {
            return;
        };
        runtime.reset_frame_state();
        runtime.apply_palette_bindings_if_changed(&palettes);
        if let Some(gs) = &game_state {
            runtime.apply_game_state_bindings_if_changed(gs);
        }
        let cmds = runtime.update_behaviors(
            stage,
            scene_elapsed_ms,
            stage_elapsed_ms,
            menu_selected_index,
            game_state,
            level_state,
            persistence,
            gameplay_world,
            emitter_state,
            collisions,
            catalogs,
            palettes,
            default_palette,
            debug_enabled,
        );
        // Re-sync widget visual positions after reset + all behavior commands applied.
        runtime.sync_widget_visuals();
        let diagnostics = runtime.take_render3d_rebuild_diagnostics();
        let scene_id = runtime.scene().id.clone();
        (cmds, diagnostics, scene_id)
    };

    if debug_enabled && !render3d_rebuild_diagnostics.is_empty() {
        let message = format!(
            "render3d rebuild causes: worldgen_dirty_events={} mesh_dirty_events={}",
            render3d_rebuild_diagnostics.worldgen_dirty_events,
            render3d_rebuild_diagnostics.mesh_dirty_events
        );
        logging::info("engine.render3d", format!("scene={} {}", scene_id, message));
        if let Some(log) = world.get_mut::<DebugLogBuffer>() {
            log.push(DebugLogEntry {
                severity: DebugSeverity::Info,
                subsystem: "render3d",
                scene_id: Some(scene_id.clone()),
                source: None,
                message,
            });
        }
    }

    for command in &commands {
        match command {
            BehaviorCommand::PlayAudioCue { cue, volume } => {
                if let Some(audio_runtime) = world.audio_runtime_mut() {
                    audio_runtime.queue(AudioCommand {
                        cue: cue.clone(),
                        volume: *volume,
                    });
                }
            }
            BehaviorCommand::PlayAudioEvent { event, gain } => {
                let resolved = world
                    .get_mut::<AudioSequencerState>()
                    .and_then(|sequencer| sequencer.trigger_event(event, scene_elapsed_ms, *gain));
                if let (Some((cue, volume)), Some(audio_runtime)) =
                    (resolved, world.audio_runtime_mut())
                {
                    audio_runtime.queue(AudioCommand {
                        cue,
                        volume: Some(volume),
                    });
                }
            }
            BehaviorCommand::PlaySong { song_id } => {
                if let Some(sequencer) = world.get_mut::<AudioSequencerState>() {
                    let _ = sequencer.play_song(song_id);
                }
            }
            BehaviorCommand::StopSong => {
                if let Some(sequencer) = world.get_mut::<AudioSequencerState>() {
                    sequencer.stop_song();
                }
            }
            BehaviorCommand::ScriptError {
                scene_id,
                source,
                message,
            } => {
                record_script_error(world, scene_id, source.clone(), message.clone());
            }
            BehaviorCommand::ApplyPlanetSpec {
                target,
                body_id,
                spec,
            } => {
                if let Err(error) = apply_planet_spec(world, target, body_id, spec) {
                    record_script_error(world, &scene_id, None, error);
                }
            }
            BehaviorCommand::SceneTransition { to_scene_id } => {
                if let Some(events) = world.events_mut() {
                    events.push(EngineEvent::SceneTransition {
                        to_scene_id: to_scene_id.clone(),
                    });
                }
            }
            BehaviorCommand::CopyToClipboard { text } => {
                if let Some(renderer) = world.renderer_mut() {
                    let _ = renderer.copy_to_clipboard(text);
                }
            }
            BehaviorCommand::DebugLog {
                scene_id,
                source,
                severity,
                message,
            } => {
                record_debug_log(world, scene_id, source.clone(), *severity, message.clone());
            }
            _ => {}
        }
    }

    // Apply runtime effects: push TriggerEffect commands into the scoped resource,
    // and expire stale entries. The compositor_system reads this resource each frame.
    let scene_elapsed_ms = world.animator().map(|a| a.scene_elapsed_ms).unwrap_or(0);

    // Ensure the scoped resource exists (initialised lazily on first TriggerEffect).
    let has_trigger_effect = commands
        .iter()
        .any(|c| matches!(c, BehaviorCommand::TriggerEffect { .. }));
    if has_trigger_effect {
        if world
            .get::<crate::runtime_effects::RuntimeEffectsResource>()
            .is_none()
        {
            world.register_scoped(crate::runtime_effects::RuntimeEffectsResource::new());
        }
        if let Some(res) = world.get_mut::<crate::runtime_effects::RuntimeEffectsResource>() {
            for command in &commands {
                if let BehaviorCommand::TriggerEffect {
                    name,
                    duration_ms,
                    looping,
                    params,
                } = command
                {
                    let effect_params = crate::runtime_effects::params_from_json(params);
                    res.push(
                        name.clone(),
                        *duration_ms,
                        *looping,
                        effect_params,
                        scene_elapsed_ms,
                    );
                }
            }
        }
    }
    // Expire stale non-looping effects each frame.
    if let Some(res) = world.get_mut::<crate::runtime_effects::RuntimeEffectsResource>() {
        res.retain_live(scene_elapsed_ms);
    }
}

fn record_script_error(world: &mut World, scene_id: &str, source: Option<String>, message: String) {
    let source_label = source.as_deref().unwrap_or("<inline>");
    logging::error(
        "engine.debug.overlay",
        format!(
            "script error: scene={} src={} message={}",
            scene_id, source_label, message
        ),
    );
    if let Some(log) = world.get_mut::<DebugLogBuffer>() {
        log.push(DebugLogEntry {
            severity: DebugSeverity::Error,
            subsystem: "rhai",
            scene_id: Some(scene_id.to_string()),
            source,
            message,
        });
    }
}

fn record_debug_log(
    world: &mut World,
    scene_id: &str,
    source: Option<String>,
    severity: BehaviorDebugLogSeverity,
    message: String,
) {
    let target = "engine.gameplay";
    match severity {
        BehaviorDebugLogSeverity::Info => logging::info(
            target,
            format!(
                "scene={} src={} {}",
                scene_id,
                source.as_deref().unwrap_or("-"),
                message
            ),
        ),
        BehaviorDebugLogSeverity::Warn => logging::warn(
            target,
            format!(
                "scene={} src={} {}",
                scene_id,
                source.as_deref().unwrap_or("-"),
                message
            ),
        ),
        BehaviorDebugLogSeverity::Error => logging::error(
            target,
            format!(
                "scene={} src={} {}",
                scene_id,
                source.as_deref().unwrap_or("-"),
                message
            ),
        ),
    }
    if let Some(log) = world.get_mut::<DebugLogBuffer>() {
        let severity = match severity {
            BehaviorDebugLogSeverity::Info => DebugSeverity::Info,
            BehaviorDebugLogSeverity::Warn => DebugSeverity::Warn,
            BehaviorDebugLogSeverity::Error => DebugSeverity::Error,
        };
        log.push(DebugLogEntry {
            severity,
            subsystem: "gameplay",
            scene_id: Some(scene_id.to_string()),
            source,
            message,
        });
    }
}

fn apply_planet_body_patch(
    body: &mut engine_behavior::catalog::BodyDef,
    patch: &PlanetBodyPatchSpec,
) {
    if let Some(value) = &patch.planet_type {
        body.planet_type = value.clone();
    }
    if let Some(value) = patch.center_x {
        body.center_x = value;
    }
    if let Some(value) = patch.center_y {
        body.center_y = value;
    }
    if let Some(value) = &patch.parent {
        body.parent = value.clone();
    }
    if let Some(value) = patch.orbit_radius {
        body.orbit_radius = value;
    }
    if let Some(value) = patch.orbit_period_sec {
        body.orbit_period_sec = value;
    }
    if let Some(value) = patch.orbit_phase_deg {
        body.orbit_phase_deg = value;
    }
    if let Some(value) = patch.radius_px {
        body.radius_px = value;
    }
    if let Some(value) = &patch.radius_km {
        body.radius_km = *value;
    }
    if let Some(value) = &patch.km_per_px {
        body.km_per_px = *value;
    }
    if let Some(value) = patch.gravity_mu {
        body.gravity_mu = value;
    }
    if let Some(value) = &patch.gravity_mu_km3_s2 {
        body.gravity_mu_km3_s2 = *value;
    }
    if let Some(value) = patch.surface_radius {
        body.surface_radius = value;
    }
    if let Some(value) = &patch.atmosphere_top {
        body.atmosphere_top = *value;
    }
    if let Some(value) = &patch.atmosphere_dense_start {
        body.atmosphere_dense_start = *value;
    }
    if let Some(value) = &patch.atmosphere_drag_max {
        body.atmosphere_drag_max = *value;
    }
    if let Some(value) = &patch.atmosphere_top_km {
        body.atmosphere_top_km = *value;
    }
    if let Some(value) = &patch.atmosphere_dense_start_km {
        body.atmosphere_dense_start_km = *value;
    }
    if let Some(value) = &patch.cloud_bottom_km {
        body.cloud_bottom_km = *value;
    }
    if let Some(value) = &patch.cloud_top_km {
        body.cloud_top_km = *value;
    }
}

fn apply_planet_spec(
    world: &mut World,
    target: &str,
    body_id: &str,
    spec: &PlanetApplySpec,
) -> Result<(), String> {
    let target = target.trim();
    let body_id = body_id.trim();
    if target.is_empty() {
        return Err("planet apply requires a non-empty target".to_string());
    }
    if body_id.is_empty() {
        return Err("planet apply requires a non-empty body_id".to_string());
    }
    if spec.is_empty() {
        return Err("planet apply requires at least one body or render field".to_string());
    }

    {
        let Some(catalogs) = world.get_mut::<engine_behavior::catalog::ModCatalogs>() else {
            return Err("missing celestial catalogs".to_string());
        };
        let body = catalogs
            .celestial
            .bodies
            .entry(body_id.to_string())
            .or_default();
        apply_planet_body_patch(body, &spec.body);
    }

    let render_commands = planet_apply_behavior_commands(target, body_id, spec);
    if render_commands.is_empty() {
        return Ok(());
    }

    let Some(runtime) = world.scene_runtime_mut() else {
        return Err("missing scene runtime".to_string());
    };
    let resolver = runtime.target_resolver();
    let diagnostics = runtime.apply_behavior_commands(&resolver, &render_commands);
    for command in diagnostics {
        match command {
            BehaviorCommand::DebugLog {
                scene_id,
                source,
                severity,
                message,
            } => record_debug_log(world, &scene_id, source, severity, message),
            BehaviorCommand::ScriptError {
                scene_id,
                source,
                message,
            } => record_script_error(world, &scene_id, source, message),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::behavior_system;
    use crate::audio::AudioRuntime;
    use crate::buffer::Buffer;
    use crate::events::{EngineEvent, EventQueue};
    use crate::runtime_settings::RuntimeSettings;
    use crate::scene::Scene;
    use crate::scene_loader::SceneLoader;
    use crate::scene_runtime::{RawKeyEvent, SceneRuntime};
    use crate::services::EngineWorldAccess;
    use crate::systems::compositor::compositor_system;
    use crate::systems::scene_lifecycle::SceneLifecycleManager;
    use crate::world::World;
    use engine_animation::{Animator, SceneStage};
    use engine_behavior::{catalog::ModCatalogs, init_behavior_system};
    use engine_core::scene::Sprite;
    use engine_events::{KeyCode, KeyEvent, KeyModifiers};
    use engine_persistence::PersistenceStore;
    use engine_render::{OverlayData, RenderError, RendererBackend, VectorOverlay};
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("engine crate should live under repo root")
            .to_path_buf()
    }

    fn make_idle_animator_with_frame(frame_ms: u64) -> Animator {
        Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: frame_ms,
            stage_elapsed_ms: frame_ms,
            scene_elapsed_ms: frame_ms,
            next_scene_override: None,
            menu_selected_index: 0,
        }
    }

    fn key_pressed(code: KeyCode) -> EngineEvent {
        EngineEvent::KeyDown {
            key: KeyEvent::new(code, KeyModifiers::NONE),
            repeat: false,
        }
    }

    struct ClipboardTestRenderer {
        copied: Arc<Mutex<Vec<String>>>,
    }

    impl ClipboardTestRenderer {
        fn new(copied: Arc<Mutex<Vec<String>>>) -> Self {
            Self { copied }
        }
    }

    impl RendererBackend for ClipboardTestRenderer {
        fn present_frame(&mut self, _buffer: &Buffer) {}

        fn present_overlay(&mut self, _overlay: &OverlayData) {}

        fn present_vectors(&mut self, _vectors: &VectorOverlay) {}

        fn output_size(&self) -> (u16, u16) {
            (1, 1)
        }

        fn copy_to_clipboard(&mut self, text: &str) -> bool {
            if let Ok(mut copied) = self.copied.lock() {
                copied.push(text.to_string());
            }
            true
        }

        fn clear(&mut self) -> Result<(), RenderError> {
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), RenderError> {
            Ok(())
        }
    }

    #[test]
    fn behavior_system_queues_audio_commands_from_scene_runtime() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
audio:
  on_enter:
    - at_ms: 100
      cue: thunder
      volume: 0.6
layers: []
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnEnter,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 100,
            scene_elapsed_ms: 100,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);

        let audio = world.get::<AudioRuntime>().expect("audio runtime");
        assert_eq!(audio.pending_len(), 1);
    }

    #[test]
    fn behavior_system_preserves_raw_key_for_rhai_scope() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: key-scope
title: Key Scope
bg_colour: black
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: idle
        behaviors:
          - name: rhai-script
            params:
              script: |
                if key.pressed {
                  scene.set("title", "text.content", key.code);
                }
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(10, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        {
            let runtime = world.scene_runtime_mut().expect("scene runtime");
            runtime.set_last_raw_key(RawKeyEvent {
                code: "x".to_string(),
                ctrl: false,
                alt: false,
                shift: false,
                pressed: true,
            });
        }

        behavior_system(&mut world);

        let runtime = world.scene_runtime().expect("scene runtime");
        assert_eq!(runtime.text_sprite_content("title"), Some("x"));
    }

    #[test]
    fn behavior_system_updates_runtime_state_used_by_compositor() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: A
        behaviors:
          - name: blink
            params:
              visible_ms: 100
              hidden_ms: 100
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(4, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 150,
            scene_elapsed_ms: 150,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);
        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(0, 0).expect("cell").symbol, ' ');
    }

    #[test]
    fn follow_behavior_uses_same_frame_target_state() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: ui
    sprites:
      - type: text
        id: leader
        content: A
        behaviors:
          - name: bob
            params:
              amplitude_x: 1
              amplitude_y: 0
              period_ms: 1000
              phase_ms: 250
      - type: text
        id: follower
        x: 1
        content: B
        behaviors:
          - name: follow
            params:
              target: leader
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(5, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);
        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(1, 0).expect("leader cell").symbol, 'A');
        assert_eq!(buffer.get(2, 0).expect("follower cell").symbol, 'B');
    }

    #[test]
    fn follow_behavior_observes_target_parent_offset() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: source
    behaviors:
      - name: bob
        params:
          amplitude_x: 1
          amplitude_y: 0
          period_ms: 1000
          phase_ms: 250
    sprites:
      - type: text
        id: leader
        content: A
  - name: target
    sprites:
      - type: text
        id: follower
        x: 1
        content: B
        behaviors:
          - name: follow
            params:
              target: leader
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(5, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);
        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(1, 0).expect("leader cell").symbol, 'A');
        assert_eq!(buffer.get(2, 0).expect("followed offset cell").symbol, 'B');
    }

    #[test]
    fn layer_blink_behavior_hides_whole_layer() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: ui
    behaviors:
      - name: blink
        params:
          visible_ms: 100
          hidden_ms: 100
    sprites:
      - type: text
        id: title
        content: A
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(4, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 150,
            scene_elapsed_ms: 150,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);
        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(0, 0).expect("cell").symbol, ' ');
    }

    #[test]
    fn layer_bob_behavior_offsets_whole_layer() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: ui
    behaviors:
      - name: bob
        params:
          amplitude_x: 1
          amplitude_y: 0
          period_ms: 1000
          phase_ms: 250
    sprites:
      - type: text
        id: title
        content: A
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(4, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);
        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(1, 0).expect("offset cell").symbol, 'A');
    }

    #[test]
    fn stage_visibility_behavior_hides_sprite_outside_selected_stage() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: A
        behaviors:
          - name: stage-visibility
            params:
              stages: [on-idle]
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(4, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnEnter,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);
        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(0, 0).expect("cell").symbol, ' ');
    }

    #[test]
    fn timed_visibility_behavior_hides_sprite_before_start_time() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: A
        behaviors:
          - name: timed-visibility
            params:
              start_ms: 100
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(4, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 50,
            scene_elapsed_ms: 50,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);
        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(0, 0).expect("cell").symbol, ' ');
    }

    #[test]
    fn timed_visibility_behavior_can_use_stage_time_scope() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: A
        behaviors:
          - name: timed-visibility
            params:
              time_scope: stage
              start_ms: 100
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(4, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 50,
            scene_elapsed_ms: 500,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);
        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(0, 0).expect("cell").symbol, ' ');
    }

    #[test]
    fn scene_blink_behavior_hides_whole_scene() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
behaviors:
  - name: blink
    params:
      visible_ms: 100
      hidden_ms: 100
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: A
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(4, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 150,
            scene_elapsed_ms: 150,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);
        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(0, 0).expect("cell").symbol, ' ');
    }

    #[test]
    fn scene_bob_behavior_offsets_whole_scene() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
behaviors:
  - name: bob
    params:
      amplitude_x: 1
      amplitude_y: 0
      period_ms: 1000
      phase_ms: 250
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: A
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(4, 2));
        world.register(RuntimeSettings::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);
        compositor_system(&mut world);

        let buffer = world.get::<Buffer>().expect("buffer");
        assert_eq!(buffer.get(1, 0).expect("offset cell").symbol, 'A');
    }

    #[test]
    fn behavior_system_dispatches_copy_to_clipboard_command_to_renderer() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: copy-test
title: Copy Test
bg_colour: black
layers:
  - name: ui
    sprites:
      - type: text
        id: title
        content: idle
        behaviors:
          - name: rhai-script
            params:
              script: |
                ui.copy_to_clipboard("SEED:42 BARREN");
                []
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        let copied = Arc::new(Mutex::new(Vec::<String>::new()));
        world.register(AudioRuntime::null());
        world.register(Buffer::new(10, 2));
        world.register(RuntimeSettings::default());
        world
            .register(Box::new(ClipboardTestRenderer::new(Arc::clone(&copied)))
                as Box<dyn RendererBackend>);
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);

        assert_eq!(
            *copied.lock().expect("copied lock"),
            vec!["SEED:42 BARREN".to_string()]
        );
    }

    #[test]
    fn behavior_system_applies_unified_planet_update_to_body_and_render_state() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: planet-apply
title: Planet Apply
bg_colour: black
layers:
  - name: planet
    sprites:
      - type: obj
        id: planet-mesh
        source: /assets/3d/sphere.obj
        camera-distance: 3.0
        behaviors:
          - name: rhai-script
            params:
              script: |
                let ok_apply = world.apply_planet_spec("planet-mesh", "generated-planet", #{
                  body: #{
                    planet_type: "earth_like",
                    radius_px: 210.0,
                    surface_radius: 205.0,
                    gravity_mu_km3_s2: 4321.5
                  },
                  view_params: #{
                    distance: 9.5
                  }
                });
                game.set("/test/ok_apply", ok_apply);
"#,
        )
        .expect("scene should parse");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(10, 2));
        world.register(RuntimeSettings::default());
        world.register(ModCatalogs::default());
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        behavior_system(&mut world);

        let catalogs = world.get::<ModCatalogs>().expect("catalogs");
        let body = catalogs
            .celestial
            .bodies
            .get("generated-planet")
            .expect("generated planet body");
        assert_eq!(body.planet_type.as_deref(), Some("earth_like"));
        assert_eq!(body.radius_px, 210.0);
        assert_eq!(body.surface_radius, 205.0);
        assert_eq!(body.gravity_mu_km3_s2, Some(4321.5));

        let runtime = world.scene_runtime().expect("scene runtime");
        let camera_distance = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    camera_distance,
                    ..
                } if id.as_deref() == Some("planet-mesh") => *camera_distance,
                _ => None,
            })
            .expect("planet mesh camera distance");
        assert!((camera_distance - 9.5).abs() < f32::EPSILON);
    }

    #[test]
    fn planet_generator_flight_scene_seeds_ship_above_surface_and_drives_cockpit_camera() {
        let mod_root = repo_root().join("mods/planet-generator");
        init_behavior_system(
            mod_root
                .to_str()
                .expect("planet-generator mod path should be valid UTF-8"),
        );
        let loader = SceneLoader::new(mod_root.clone()).expect("scene loader");
        let scene = loader
            .load_by_path("/scenes/flight/scene.yml")
            .expect("flight scene");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(640, 360));
        world.register(RuntimeSettings::default());
        world.register(
            ModCatalogs::load_from_directory(&mod_root.join("catalogs"))
                .expect("planet-generator catalogs"),
        );
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator_with_frame(16));

        behavior_system(&mut world);

        let runtime = world.scene_runtime_mut().expect("scene runtime");
        let (ship_x, ship_y, ship_z, ship_scale, ship_visible): (f32, f32, f32, f32, bool) =
            runtime
                .scene()
                .layers
                .iter()
                .flat_map(|layer| layer.sprites.iter())
                .find_map(|sprite| match sprite {
                    Sprite::Obj {
                        id,
                        scale,
                        world_x,
                        world_y,
                        world_z,
                        visible,
                        ..
                    } if id.as_deref() == Some("flight-player") => Some((
                        world_x.unwrap_or(0.0),
                        world_y.unwrap_or(0.0),
                        world_z.unwrap_or(0.0),
                        scale.unwrap_or(0.0),
                        *visible,
                    )),
                    _ => None,
                })
                .expect("flight-player world position");
        let (
            cockpit_scale,
            cockpit_visible,
            cockpit_x,
            cockpit_y,
            cockpit_z,
            cockpit_cam_x,
            cockpit_cam_y,
            cockpit_cam_z,
        ): (f32, bool, f32, f32, f32, f32, f32, f32) = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    scale,
                    visible,
                    world_x,
                    world_y,
                    world_z,
                    cam_world_x,
                    cam_world_y,
                    cam_world_z,
                    ..
                } if id.as_deref() == Some("flight-cockpit") => Some((
                    scale.unwrap_or(0.0),
                    *visible,
                    world_x.unwrap_or(0.0),
                    world_y.unwrap_or(0.0),
                    world_z.unwrap_or(0.0),
                    cam_world_x.unwrap_or(0.0),
                    cam_world_y.unwrap_or(0.0),
                    cam_world_z.unwrap_or(0.0),
                )),
                _ => None,
            })
            .expect("flight-cockpit render params");
        let ship_radius = (ship_x * ship_x + ship_y * ship_y + ship_z * ship_z).sqrt();
        let camera = runtime.scene_camera_3d();
        let eye = camera.eye;
        let look_at = camera.look_at;
        let eye_radius = (eye[0] * eye[0] + eye[1] * eye[1] + eye[2] * eye[2]).sqrt();
        let look_radius =
            (look_at[0] * look_at[0] + look_at[1] * look_at[1] + look_at[2] * look_at[2]).sqrt();
        let ship_to_eye = [eye[0] - ship_x, eye[1] - ship_y, eye[2] - ship_z];
        let ship_to_look = [
            look_at[0] - ship_x,
            look_at[1] - ship_y,
            look_at[2] - ship_z,
        ];
        let cockpit_to_eye = [eye[0] - cockpit_x, eye[1] - cockpit_y, eye[2] - cockpit_z];
        let cockpit_cam_to_eye = [
            eye[0] - cockpit_cam_x,
            eye[1] - cockpit_cam_y,
            eye[2] - cockpit_cam_z,
        ];
        let eye_distance = (ship_to_eye[0] * ship_to_eye[0]
            + ship_to_eye[1] * ship_to_eye[1]
            + ship_to_eye[2] * ship_to_eye[2])
            .sqrt();
        let look_distance = (ship_to_look[0] * ship_to_look[0]
            + ship_to_look[1] * ship_to_look[1]
            + ship_to_look[2] * ship_to_look[2])
            .sqrt();
        let cockpit_anchor_distance = (cockpit_to_eye[0] * cockpit_to_eye[0]
            + cockpit_to_eye[1] * cockpit_to_eye[1]
            + cockpit_to_eye[2] * cockpit_to_eye[2])
            .sqrt();
        let cockpit_cam_distance = (cockpit_cam_to_eye[0] * cockpit_cam_to_eye[0]
            + cockpit_cam_to_eye[1] * cockpit_cam_to_eye[1]
            + cockpit_cam_to_eye[2] * cockpit_cam_to_eye[2])
            .sqrt();

        assert!(
            ship_radius > 1.0,
            "ship should seed above the unit render shell, got ship_radius={ship_radius}"
        );
        assert!(
            ship_scale > 0.0 && ship_scale < 0.000001,
            "placeholder ship should stay physically tiny against the planet shell, got ship_scale={ship_scale}"
        );
        assert!(
            !ship_visible,
            "proxy ship should stay hidden in first-person mode"
        );
        assert!(
            cockpit_visible,
            "cockpit mesh should be visible in the default flight view"
        );
        assert!(
            cockpit_scale > 0.0 && cockpit_scale < ship_scale,
            "cockpit mesh should stay smaller than the proxy ship scale, got cockpit_scale={cockpit_scale} ship_scale={ship_scale}"
        );
        assert!(
            eye_radius > ship_radius,
            "cockpit camera eye should stay slightly above the proxy ship centre, got eye_radius={eye_radius} ship_radius={ship_radius}"
        );
        assert!(
            eye_distance > ship_scale * 0.2 && eye_distance < ship_scale * 3.0,
            "cockpit camera eye should stay close to the proxy ship, got eye_distance={eye_distance} ship_scale={ship_scale}"
        );
        assert!(
            look_distance > ship_scale * 40.0,
            "cockpit camera should look far ahead of the ship instead of at its centre, got look_distance={look_distance} ship_scale={ship_scale}"
        );
        assert!(
            look_radius > 1.0,
            "camera should keep looking outward from the planet shell, got look_radius={look_radius}"
        );
        assert!(
            cockpit_anchor_distance < ship_scale * 0.2,
            "cockpit world anchor should stay glued to the camera eye, got cockpit_anchor_distance={cockpit_anchor_distance}"
        );
        assert!(
            cockpit_cam_distance < ship_scale * 0.2,
            "cockpit local camera basis should reuse the scene camera eye, got cockpit_cam_distance={cockpit_cam_distance}"
        );
    }

    #[test]
    fn planet_generator_flight_scene_pushes_readable_planet_defaults() {
        let mod_root = repo_root().join("mods/planet-generator");
        init_behavior_system(
            mod_root
                .to_str()
                .expect("planet-generator mod path should be valid UTF-8"),
        );
        let loader = SceneLoader::new(mod_root.clone()).expect("scene loader");
        let scene = loader
            .load_by_path("/scenes/flight/scene.yml")
            .expect("flight scene");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(640, 360));
        world.register(RuntimeSettings::default());
        world.register(
            ModCatalogs::load_from_directory(&mod_root.join("catalogs"))
                .expect("planet-generator catalogs"),
        );
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator_with_frame(16));

        behavior_system(&mut world);

        let runtime = world.scene_runtime_mut().expect("scene runtime");
        let (seed, displacement, ambient, atmo_height, atmo_density, rayleigh, haze) = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    world_gen_seed,
                    world_gen_displacement_scale,
                    ambient,
                    atmo_height,
                    atmo_density,
                    atmo_rayleigh_amount,
                    atmo_haze_amount,
                    ..
                } if id.as_deref() == Some("planet-mesh") => Some((
                    world_gen_seed.unwrap_or(0),
                    world_gen_displacement_scale.unwrap_or(0.0),
                    ambient.unwrap_or(0.0),
                    atmo_height.unwrap_or(0.0),
                    atmo_density.unwrap_or(0.0),
                    atmo_rayleigh_amount.unwrap_or(0.0),
                    atmo_haze_amount.unwrap_or(0.0),
                )),
                _ => None,
            })
            .expect("planet-mesh render params");

        assert_ne!(
            seed, 0,
            "flight scene should not ship with the bland seed 0"
        );
        assert!(
            displacement >= 0.0015,
            "flight scene should keep visible terrain relief, got displacement={displacement}"
        );
        assert!(
            ambient <= 0.10,
            "flight scene should keep enough light contrast for readable landforms, got ambient={ambient}"
        );
        assert!(
            atmo_height >= 0.15 && atmo_density >= 0.55,
            "flight scene should push a visible atmosphere shell, got height={atmo_height} density={atmo_density}"
        );
        assert!(
            rayleigh >= 0.45 && haze >= 0.30,
            "flight scene should keep both rayleigh and haze components visible, got rayleigh={rayleigh} haze={haze}"
        );
    }

    #[test]
    fn planet_generator_main_scene_f10_hands_off_current_planet_to_flight() {
        let mod_root = repo_root().join("mods/planet-generator");
        init_behavior_system(
            mod_root
                .to_str()
                .expect("planet-generator mod path should be valid UTF-8"),
        );
        let loader = SceneLoader::new(mod_root.clone()).expect("scene loader");
        let scene = loader
            .load_by_path("/scenes/main/scene.yml")
            .expect("main scene");
        let temp = tempdir().expect("temp dir");

        let mut world = World::new();
        world.register(AudioRuntime::null());
        world.register(Buffer::new(1280, 720));
        world.register(RuntimeSettings::default());
        world.register(loader);
        world.register(EventQueue::new());
        world.register(PersistenceStore::from_root(
            temp.path(),
            "planet-generator-f10-test",
        ));
        world.register(
            ModCatalogs::load_from_directory(&mod_root.join("catalogs"))
                .expect("planet-generator catalogs"),
        );
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator_with_frame(16));

        SceneLifecycleManager::process_events(&mut world, vec![key_pressed(KeyCode::F(2))]);
        behavior_system(&mut world);

        SceneLifecycleManager::process_events(&mut world, vec![key_pressed(KeyCode::F(10))]);
        behavior_system(&mut world);

        let snapshot = world
            .get::<PersistenceStore>()
            .expect("persistence")
            .get("/planet_generator/flight_launch")
            .expect("flight handoff snapshot");
        assert_eq!(
            snapshot.pointer("/planet_type").and_then(|v| v.as_str()),
            Some("barren")
        );
        assert_eq!(
            snapshot.pointer("/seed").and_then(|v| v.as_f64()),
            Some(42.0)
        );
        assert_eq!(
            snapshot.pointer("/has_ocean").and_then(|v| v.as_bool()),
            Some(false)
        );

        let transition_events = world.events_mut().expect("event queue").drain();
        assert!(
            transition_events.iter().any(|event| matches!(
                event,
                EngineEvent::SceneTransition { to_scene_id }
                    if to_scene_id == "/scenes/flight/scene.yml"
            )),
            "expected F10 to queue a transition into the flight scene, got {transition_events:?}"
        );

        SceneLifecycleManager::process_events(&mut world, transition_events);
        behavior_system(&mut world);

        assert_eq!(
            world.scene_runtime().expect("scene runtime").scene().id,
            "planet-generator-flight"
        );
        assert!(
            world
                .get::<PersistenceStore>()
                .expect("persistence")
                .get("/planet_generator/flight_launch")
                .is_none(),
            "flight scene should consume the handoff snapshot after boot"
        );

        let runtime = world.scene_runtime().expect("scene runtime");
        let (seed, has_ocean, atmosphere_density) = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    world_gen_seed,
                    world_gen_has_ocean,
                    atmo_density,
                    ..
                } if id.as_deref() == Some("planet-mesh") => Some((
                    world_gen_seed.unwrap_or(0),
                    world_gen_has_ocean.unwrap_or(true),
                    atmo_density.unwrap_or(0.0),
                )),
                _ => None,
            })
            .expect("planet-mesh render params");

        assert_eq!(
            seed, 42,
            "flight scene should reuse the generator seed selected before F10"
        );
        assert!(
            !has_ocean,
            "flight scene should keep the generator ocean toggle after F10"
        );
        assert!(
            (atmosphere_density - 0.20).abs() < 0.0001,
            "flight scene should keep the generator atmosphere settings after F10, got atmo_density={atmosphere_density}"
        );
    }
}

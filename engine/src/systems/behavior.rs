//! Behavior system — evaluates all scene-object behaviors and dispatches resulting commands each frame.

use crate::audio::AudioCommand;
use crate::audio_sequencer::AudioSequencerState;
use crate::behavior::{BehaviorCommand, DebugLogSeverity as BehaviorDebugLogSeverity};
use crate::debug_log::{DebugLogBuffer, DebugLogEntry, DebugSeverity};
use crate::events::EngineEvent;
use crate::services::EngineWorldAccess;
use crate::systems::gameplay_events::GameplayEventBuffer;
use crate::world::World;
use engine_core::logging;

/// Runs all registered behaviors against the current scene runtime state and dispatches their commands.
pub fn behavior_system(world: &mut World) {
    let Some(animator) = world.animator() else {
        return;
    };
    let stage = animator.stage.clone();
    let scene_elapsed_ms = animator.scene_elapsed_ms;
    let stage_elapsed_ms = animator.stage_elapsed_ms;
    let menu_selected_index = animator.menu_selected_index;

    let game_state = world.get::<crate::game_state::GameState>().cloned();
    let level_state = world.get::<crate::level_state::LevelState>().cloned();
    let persistence = world.get::<engine_persistence::PersistenceStore>().cloned();
    let gameplay_world = world.get::<crate::game::GameplayWorld>().cloned();
    let collisions = world
        .get::<GameplayEventBuffer>()
        .map(|buf| std::sync::Arc::new(buf.collisions.clone()))
        .unwrap_or_else(|| std::sync::Arc::new(Vec::new()));

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

    let commands = {
        let Some(runtime) = world.scene_runtime_mut() else {
            return;
        };
        runtime.reset_frame_state();
        runtime.update_behaviors(
            stage,
            scene_elapsed_ms,
            stage_elapsed_ms,
            menu_selected_index,
            game_state,
            level_state,
            persistence,
            gameplay_world,
            collisions,
        )
    };

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
                        scene_id: Some(scene_id.clone()),
                        source: source.clone(),
                        message: message.clone(),
                    });
                }
            }
            BehaviorCommand::SceneTransition { to_scene_id } => {
                if let Some(events) = world.events_mut() {
                    events.push(EngineEvent::SceneTransition {
                        to_scene_id: to_scene_id.clone(),
                    });
                }
            }
            BehaviorCommand::DebugLog {
                scene_id,
                source,
                severity,
                message,
            } => {
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
                        scene_id: Some(scene_id.clone()),
                        source: source.clone(),
                        message: message.clone(),
                    });
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::behavior_system;
    use crate::audio::AudioRuntime;
    use crate::buffer::Buffer;
    use crate::runtime_settings::RuntimeSettings;
    use crate::scene::Scene;
    use crate::scene_runtime::{RawKeyEvent, SceneRuntime};
    use crate::services::EngineWorldAccess;
    use crate::systems::compositor::compositor_system;
    use crate::world::World;
    use engine_animation::{Animator, SceneStage};

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
}

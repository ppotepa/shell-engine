mod debug_controls;

use crate::audio::AudioCommand;
use crate::debug_log::DebugLogBuffer;
use crate::events::EngineEvent;
use crate::scene::{self};
use crate::scene_runtime::{RawKeyEvent, SceneRuntime};
use crate::services::EngineWorldAccess;
use crate::systems::menu::{evaluate_menu_action, MenuAction};
use crate::world::World;
use engine_animation::{Animator, SceneStage};
use engine_core::logging;
use engine_events::{InputEvent, KeyCode, KeyEvent, KeyModifiers};

pub struct SceneLifecycleManager;
const PLAYGROUND_MENU_ID: &str = "playground-menu";
const PLAYGROUND_EXIT_ID: &str = "playground-exit";

#[derive(Default)]
struct LifecycleEvents {
    quit: bool,
    key_presses: Vec<KeyEvent>,
    key_releases: Vec<KeyEvent>,
    input_focus_lost: bool,
    last_key_snapshot: Option<RawKeyEvent>,
    transitions: Vec<String>,
    resizes: Vec<(u16, u16)>,
    input_events: Vec<InputEvent>,
}

impl SceneLifecycleManager {
    /// Process frame events related to scene lifecycle and transitions.
    /// Returns `true` when a quit event was requested.
    pub fn process_events(world: &mut World, events: Vec<EngineEvent>) -> bool {
        let lifecycle = classify_events(events);
        for (width, height) in &lifecycle.resizes {
            let should_resize_buffer = world
                .runtime_settings()
                .map(|settings| settings.render_size_matches_output())
                .unwrap_or(true);
            if should_resize_buffer {
                if let Some(buf) = world.buffer_mut() {
                    buf.resize(*width, *height);
                }
            }
        }

        Self::update_frame_input_state(
            world,
            lifecycle.last_key_snapshot,
            &lifecycle.key_presses,
            &lifecycle.key_releases,
            lifecycle.input_focus_lost,
        );

        // Collect mouse moves for 3D camera consumers (extract from input_events).
        let mouse_moves: Vec<(f32, f32)> = lifecycle.input_events.iter().filter_map(|e| {
            if let InputEvent::MouseMoved { x, y } = e { Some((*x, *y)) } else { None }
        }).collect();

        // Collect scroll wheel deltas for orbit camera zoom.
        let scroll_deltas: Vec<f32> = lifecycle.input_events.iter().filter_map(|e| {
            if let InputEvent::MouseWheel { delta_y } = e { Some(*delta_y) } else { None }
        }).collect();

        handle_scene_free_look_input(
            world,
            &lifecycle.key_presses,
            &lifecycle.key_releases,
            &mouse_moves,
        );
        handle_scene_orbit_camera_input(
            world,
            &lifecycle.key_presses,
            &lifecycle.key_releases,
            &mouse_moves,
            &scroll_deltas,
        );
        if !lifecycle.key_presses.is_empty() {
            Self::advance_on_any_key(world, &lifecycle.key_presses);
        }
        if !mouse_moves.is_empty() {
            handle_playground_3d_mouse(world, &mouse_moves);
        }
        // Fan-out all input events (keys + mouse) to GUI.
        if !lifecycle.input_events.is_empty() {
            handle_gui_input_events(world, lifecycle.input_events);
        }
        let quit_from_transition = Self::apply_transitions(world, lifecycle.transitions);
        lifecycle.quit || quit_from_transition
    }

    fn update_frame_input_state(
        world: &mut World,
        snapshot: Option<RawKeyEvent>,
        key_presses: &[KeyEvent],
        key_releases: &[KeyEvent],
        input_focus_lost: bool,
    ) {
        let Some(runtime) = world.scene_runtime_mut() else {
            return;
        };
        if input_focus_lost {
            runtime.clear_keys_down();
            runtime.clear_last_raw_key();
            return;
        }
        for key in key_presses {
            let raw = key_event_to_raw(key, true);
            runtime.set_key_down(&raw);
        }
        for key in key_releases {
            let raw = key_event_to_raw(key, false);
            runtime.set_key_up(&raw);
        }
        if let Some(snapshot) = snapshot {
            runtime.set_last_raw_key(snapshot);
        }
    }

    fn advance_on_any_key(world: &mut World, key_presses: &[KeyEvent]) {
        if debug_controls::handle_debug_controls(world, key_presses) {
            return;
        }
        let ui_focus_handled = handle_ui_focus_controls(world, key_presses);
        let routed_keys = if ui_focus_handled {
            key_presses
                .iter()
                .filter(|key| !is_focus_navigation_key(key))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            key_presses.to_vec()
        };
        if routed_keys.is_empty() {
            return;
        }
        if handle_playground_escape_to_menu(world, &routed_keys) {
            return;
        }
        if handle_obj_viewer_controls(world, &routed_keys) {
            return;
        }

        let (menu_options, any_key_trigger) = world
            .scene_runtime()
            .map(|r| {
                let opts = r.scene().menu_options.clone();
                let any_key = matches!(
                    r.scene().stages.on_idle.trigger,
                    scene::StageTrigger::AnyKey
                );
                (opts, any_key)
            })
            .unwrap_or_default();
        let selected_index = world.animator().map(|a| a.menu_selected_index).unwrap_or(0);
        let menu_action = evaluate_menu_action(&menu_options, selected_index, &routed_keys);

        if !is_scene_idle(world) || !any_key_trigger {
            return;
        }

        let mut play_move = false;
        let mut play_select = false;
        if let Some(animator) = world.animator_mut() {
            match menu_action {
                MenuAction::Navigate(index) => {
                    if animator.menu_selected_index != index {
                        animator.menu_selected_index = index;
                        play_move = true;
                    }
                }
                MenuAction::Activate(next_scene) => {
                    animator.next_scene_override = Some(next_scene);
                    begin_leave(animator);
                    play_select = true;
                }
                MenuAction::None if menu_options.is_empty() => begin_leave(animator),
                MenuAction::None => {}
            }
        }
        if play_move {
            emit_ui_menu_audio_event(world, "ui.menu.move");
        }
        if play_select {
            emit_ui_menu_audio_event(world, "ui.menu.select");
        }
    }

    fn apply_transitions(world: &mut World, transitions: Vec<String>) -> bool {
        for to_scene_ref in transitions {
            logging::info(
                "engine.scene",
                format!("transition requested: to={to_scene_ref}"),
            );
            let Some(new_scene) = world
                .scene_loader()
                .and_then(|loader| loader.load_by_ref(&to_scene_ref).ok())
            else {
                let msg = format!("transition target could not be resolved: to={to_scene_ref}");
                logging::warn("engine.scene", &msg);
                if let Some(log) = world.get_mut::<DebugLogBuffer>() {
                    log.push_warn("scene", None, None, msg);
                }
                continue;
            };
            if new_scene.id == PLAYGROUND_EXIT_ID {
                logging::info("engine.scene", "received playground-exit transition");
                return true;
            }
            Self::apply_virtual_size_override(world, &new_scene);
            // Clone the Arc before the mutable clear so there is no borrow conflict.
            let pipeline = world
                .get::<std::sync::Arc<crate::scene_pipeline::ScenePipeline>>()
                .cloned();
            // 1. Discard every scoped resource from the outgoing scene (SceneRuntime,
            //    prerendered frames, any future scoped resources).
            world.clear_scoped();
            if let Some(sequencer) = world.get_mut::<crate::audio_sequencer::AudioSequencerState>()
            {
                sequencer.stop_song();
            }
            if let Some(gameplay_world) = world.get_mut::<crate::game::GameplayWorld>() {
                gameplay_world.clear();
            }
            // 1b. Invalidate buffer dirty region so the first frame of the new scene
            //     gets a full diff (required for dirty-region diff correctness).
            if let Some(buf) = world.buffer_mut() {
                buf.invalidate();
            }
            // 2. Run the scene preparation pipeline for the incoming scene.
            //    Steps register their outputs as scoped resources.
            if let Some(pipeline) = pipeline {
                pipeline.prepare(&new_scene, world);
            }
            // 3. Activate the scene.
            world.register_scoped(SceneRuntime::new(new_scene));
            world.register_scoped(Animator::new());
            if let Some(runtime) = world.scene_runtime() {
                let scene = runtime.scene();
                let audio_cue_count = scene.audio.on_enter.len()
                    + scene.audio.on_idle.len()
                    + scene.audio.on_leave.len();
                logging::info(
                    "engine.scene",
                    format!(
                        "transition applied: active_scene={} title={} audio_cues={} behaviors={}",
                        scene.id,
                        scene.title,
                        audio_cue_count,
                        runtime.behavior_count(),
                    ),
                );
            }
        }
        false
    }

    fn apply_virtual_size_override(world: &mut World, scene: &scene::Scene) {
        let output_dimensions = world.output_dimensions().unwrap_or((80, 24));
        let new_size = {
            let Some(settings) = world.runtime_settings() else {
                return;
            };
            crate::runtime_settings::scene_render_size_override(
                settings,
                scene,
                output_dimensions.0,
                output_dimensions.1,
            )
        };
        let Some((new_width, new_height)) = new_size else {
            return;
        };
        if let Some(buffer) = world.buffer_mut() {
            if buffer.width != new_width || buffer.height != new_height {
                buffer.resize(new_width, new_height);
            }
        }
    }
}

fn is_focus_navigation_key(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Tab | KeyCode::BackTab)
}

fn classify_events(events: Vec<EngineEvent>) -> LifecycleEvents {
    let mut lifecycle = LifecycleEvents::default();
    for event in events {
        // Always try to produce an InputEvent first (for GUI fan-out).
        if let Some(input_event) = event.as_input_event() {
            lifecycle.input_events.push(input_event);
        }
        match event {
            EngineEvent::Quit => lifecycle.quit = true,
            EngineEvent::KeyDown { key, .. } => {
                lifecycle.last_key_snapshot = Some(key_event_to_raw(&key, true));
                lifecycle.key_presses.push(key);
            }
            EngineEvent::KeyUp { key } => {
                lifecycle.last_key_snapshot = Some(key_event_to_raw(&key, false));
                lifecycle.key_releases.push(key);
            }
            EngineEvent::InputFocusLost => {
                lifecycle.input_focus_lost = true;
                lifecycle.last_key_snapshot = None;
            }
            EngineEvent::SceneTransition { to_scene_id } => lifecycle.transitions.push(to_scene_id),
            EngineEvent::OutputResized { width, height } => {
                lifecycle.resizes.push((width, height));
            }
            _ => {}
        }
    }
    lifecycle
}

pub(super) fn is_scene_idle(world: &World) -> bool {
    world
        .animator()
        .map(|a| a.stage == SceneStage::OnIdle)
        .unwrap_or(false)
}

pub(super) fn begin_leave(a: &mut engine_animation::Animator) {
    a.stage = SceneStage::OnLeave;
    a.step_idx = 0;
    a.elapsed_ms = 0;
    a.stage_elapsed_ms = 0;
}

#[allow(dead_code)]
fn reset_timeout_idle_clock(world: &mut World) {
    let should_reset = {
        let Some(animator) = world.animator() else {
            return;
        };
        if animator.stage != SceneStage::OnIdle {
            return;
        }
        world
            .scene_runtime()
            .map(|runtime| {
                matches!(
                    runtime.scene().stages.on_idle.trigger,
                    scene::StageTrigger::Timeout
                )
            })
            .unwrap_or(false)
    };
    if !should_reset {
        return;
    }
    if let Some(animator) = world.animator_mut() {
        animator.elapsed_ms = 0;
        animator.stage_elapsed_ms = 0;
    }
}

fn emit_ui_menu_audio_event(world: &mut World, event: &str) {
    let now_ms = world
        .animator()
        .map(|animator| animator.scene_elapsed_ms)
        .unwrap_or(0);
    let hit = world
        .get_mut::<crate::audio_sequencer::AudioSequencerState>()
        .and_then(|sequencer| sequencer.trigger_event(event, now_ms, Some(1.0)));
    if let (Some((cue, gain)), Some(audio_runtime)) = (hit, world.audio_runtime_mut()) {
        audio_runtime.queue(AudioCommand {
            cue,
            volume: Some(gain),
        });
    }
}

fn handle_playground_escape_to_menu(world: &mut World, key_presses: &[KeyEvent]) -> bool {
    if !is_scene_idle(world) {
        return false;
    }
    if !key_presses
        .iter()
        .any(|key| matches!(key.code, KeyCode::Esc))
    {
        return false;
    }

    let Some(scene_id) = world
        .scene_runtime()
        .map(|runtime| runtime.scene().id.clone())
    else {
        return false;
    };
    if !scene_id.starts_with("playground-")
        || scene_id == PLAYGROUND_MENU_ID
        || scene_id == PLAYGROUND_EXIT_ID
    {
        return false;
    }

    if let Some(animator) = world.animator_mut() {
        animator.next_scene_override = Some(PLAYGROUND_MENU_ID.to_string());
        begin_leave(animator);
    }
    true
}

fn handle_ui_focus_controls(world: &mut World, key_presses: &[KeyEvent]) -> bool {
    if !is_scene_idle(world) {
        return false;
    }
    let Some(runtime) = world.scene_runtime_mut() else {
        return false;
    };
    runtime.handle_ui_focus_keys(key_presses)
}

fn handle_obj_viewer_controls(world: &mut World, key_presses: &[KeyEvent]) -> bool {
    if !is_scene_idle(world) {
        return false;
    }
    world
        .scene_runtime_mut()
        .map(|runtime| runtime.apply_obj_viewer_key_presses(key_presses))
        .unwrap_or(false)
}

fn handle_playground_3d_mouse(world: &mut World, mouse_moves: &[(f32, f32)]) {
    if !is_scene_idle(world) {
        return;
    }
    if let Some(runtime) = world.scene_runtime_mut() {
        runtime.apply_obj_viewer_mouse_moves(mouse_moves);
    }
}

fn handle_scene_orbit_camera_input(
    world: &mut World,
    key_presses: &[KeyEvent],
    key_releases: &[KeyEvent],
    mouse_moves: &[(f32, f32)],
    scroll_deltas: &[f32],
) {
    if !is_scene_idle(world) {
        return;
    }
    if let Some(runtime) = world.scene_runtime_mut() {
        let _ = runtime.apply_orbit_camera_key_events(key_presses, key_releases);
        if !mouse_moves.is_empty() {
            runtime.apply_orbit_camera_mouse_moves(mouse_moves);
        }
        if !scroll_deltas.is_empty() {
            runtime.apply_orbit_camera_scroll(scroll_deltas);
        }
    }
}

fn handle_scene_free_look_input(
    world: &mut World,
    key_presses: &[KeyEvent],
    key_releases: &[KeyEvent],
    mouse_moves: &[(f32, f32)],
) {
    if !is_scene_idle(world) {
        return;
    }
    if let Some(runtime) = world.scene_runtime_mut() {
        let _ = runtime.apply_free_look_key_events(key_presses, key_releases);
        if !mouse_moves.is_empty() {
            runtime.apply_free_look_mouse_moves(mouse_moves);
        }
    }
}

fn handle_gui_input_events(world: &mut World, events: Vec<InputEvent>) {
    if let Some(runtime) = world.scene_runtime_mut() {
        runtime.update_gui(events);
    }
}

/// Convert an engine `KeyEvent` into a domain-agnostic `RawKeyEvent` for Rhai scripts.
fn key_event_to_raw(key: &KeyEvent, pressed: bool) -> RawKeyEvent {
    let code = match key.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::F(n) => format!("F{n}"),
        _ => "".to_string(),
    };
    RawKeyEvent {
        code,
        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
        alt: key.modifiers.contains(KeyModifiers::ALT),
        shift: key.modifiers.contains(KeyModifiers::SHIFT),
        pressed,
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_events, SceneLifecycleManager};
    use crate::buffer::Buffer;
    use crate::debug_features::{DebugFeatures, DebugOverlayMode};
    use crate::events::EngineEvent;
    use crate::runtime_settings::{RenderSize, RuntimeSettings};
    use crate::scene::{
        MenuOption, Scene, SceneAudio, SceneStages, Sprite, Stage, StageTrigger,
        TermColour,
    };
    use crate::scene_loader::SceneLoader;
    use crate::scene_runtime::SceneRuntime;
    use crate::services::EngineWorldAccess;
    use crate::world::World;
    use engine_animation::{Animator, SceneStage};
    use engine_events::{KeyCode, KeyEvent, KeyModifiers};
    use std::fs;
    use tempfile::tempdir;

    fn key_pressed(code: KeyCode) -> EngineEvent {
        EngineEvent::KeyDown { key: KeyEvent::new(code, KeyModifiers::NONE), repeat: false }
    }

    fn key_released(code: KeyCode) -> EngineEvent {
        EngineEvent::KeyUp { key: KeyEvent::new(code, KeyModifiers::NONE) }
    }

    fn key_pressed_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> EngineEvent {
        EngineEvent::KeyDown { key: KeyEvent::new(code, modifiers), repeat: false }
    }

    fn make_idle_animator() -> Animator {
        Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        }
    }

    fn make_menu_option(key: &str, label: &str, next: &str) -> MenuOption {
        MenuOption {
            key: key.into(),
            label: Some(label.into()),
            scene: None,
            next: next.into(),
        }
    }

    fn make_menu_scene(menu_options: Vec<MenuOption>) -> Scene {
        Scene {
            id: "menu".into(),
            title: "Menu".into(),
            cutscene: false,
            target_fps: None,
            space: Default::default(),
            celestial: Default::default(),
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages {
                on_enter: Stage::default(),
                on_idle: Stage {
                    trigger: StageTrigger::AnyKey,
                    steps: Vec::new(),
                    looping: true,
                },
                on_leave: Stage::default(),
            },
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            ui: Default::default(),
            layers: Vec::new(),
            menu_options,
            input: Default::default(),
            postfx: Vec::new(),
            next: Some("playground-3d-scene".into()),
            prerender: false,
            palette_bindings: Vec::new(),
            game_state_bindings: Vec::new(),
            gui: Default::default(),
        }
    }

    fn make_cutscene(id: &str, next: Option<&str>) -> Scene {
        Scene {
            id: id.into(),
            title: id.into(),
            cutscene: true,
            target_fps: None,
            space: Default::default(),
            celestial: Default::default(),
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            ui: Default::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            input: Default::default(),
            postfx: Vec::new(),
            next: next.map(Into::into),
            prerender: false,
            palette_bindings: Vec::new(),
            game_state_bindings: Vec::new(),
            gui: Default::default(),
        }
    }

    const OBJ_VIEWER_SCENE_YAML: &str = r#"
id: playground-3d-scene
title: 3D
bg_colour: black
input:
  obj-viewer:
    sprite_id: helsinki-uni-wireframe
stages:
  on_idle:
    trigger: any-key
    steps: []
layers:
  - name: obj
    sprites:
      - type: obj
        id: helsinki-uni-wireframe
        source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
        scale: 1.0
        rotate-y-deg-per-sec: 14
"#;

    #[test]
    fn fixed_render_size_ignores_output_resize_events() {
        let mut world = World::new();
        world.register(RuntimeSettings {
            render_size: RenderSize::Fixed {
                width: 180,
                height: 30,
            },
            ..RuntimeSettings::default()
        });
        world.register(Buffer::new(180, 30));

        let quit = SceneLifecycleManager::process_events(
            &mut world,
            vec![EngineEvent::OutputResized {
                width: 210,
                height: 109,
            }],
        );

        assert!(!quit);
        let buffer = world.get::<Buffer>().expect("buffer present");
        assert_eq!((buffer.width, buffer.height), (180, 30));
    }

    #[test]
    fn match_output_render_size_follows_output_resize_events() {
        let mut world = World::new();
        world.register(RuntimeSettings {
            render_size: RenderSize::MatchOutput,
            ..RuntimeSettings::default()
        });
        world.register(Buffer::new(80, 24));

        let quit = SceneLifecycleManager::process_events(
            &mut world,
            vec![EngineEvent::OutputResized {
                width: 100,
                height: 40,
            }],
        );

        assert!(!quit);
        let buffer = world.get::<Buffer>().expect("buffer present");
        assert_eq!((buffer.width, buffer.height), (100, 40));
    }

    #[test]
    fn any_key_moves_idle_scene_to_leave_when_trigger_is_any_key() {
        let mut scene = make_cutscene("intro", None);
        scene.stages.on_idle = Stage {
            trigger: StageTrigger::AnyKey,
            steps: Vec::new(),
            looping: true,
        };

        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 3,
            elapsed_ms: 42,
            stage_elapsed_ms: 42,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        let quit =
            SceneLifecycleManager::process_events(&mut world, vec![key_pressed(KeyCode::Enter)]);

        assert!(!quit);
        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnLeave);
        assert_eq!(animator.step_idx, 0);
        assert_eq!(animator.elapsed_ms, 0);
    }

    #[test]
    fn playground_3d_controls_consume_keys_and_update_runtime() {
        let scene: Scene = serde_yaml::from_str(OBJ_VIEWER_SCENE_YAML).expect("scene parse");

        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator());

        let _ = SceneLifecycleManager::process_events(
            &mut world,
            vec![
                key_pressed(KeyCode::Char('A')),
                key_pressed(KeyCode::Char('5')),
                key_pressed(KeyCode::Char('O')),
            ],
        );

        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnIdle);
        let runtime = world.get::<SceneRuntime>().expect("runtime present");
        let (scale, surface_mode, orbit_speed) = runtime
            .scene()
            .layers
            .iter()
            .flat_map(|layer| layer.sprites.iter())
            .find_map(|sprite| match sprite {
                Sprite::Obj {
                    id,
                    scale,
                    surface_mode,
                    rotate_y_deg_per_sec,
                    ..
                } if id.as_deref() == Some("helsinki-uni-wireframe") => Some((
                    scale.unwrap_or(1.0),
                    surface_mode.clone(),
                    rotate_y_deg_per_sec.unwrap_or(0.0),
                )),
                _ => None,
            })
            .expect("obj props");
        assert!((scale - 1.1).abs() < f32::EPSILON);
        assert_eq!(surface_mode.as_deref(), Some("wireframe"));
        assert!((orbit_speed - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn playground_3d_enter_still_leaves_scene() {
        let scene: Scene = serde_yaml::from_str(OBJ_VIEWER_SCENE_YAML).expect("scene parse");

        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator());

        let _ =
            SceneLifecycleManager::process_events(&mut world, vec![key_pressed(KeyCode::Enter)]);
        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnLeave);
    }

    #[test]
    fn playground_escape_routes_scene_back_to_playground_menu() {
        let scene = make_cutscene("playground-layout-lab", Some("other"));
        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator());

        let _ = SceneLifecycleManager::process_events(&mut world, vec![key_pressed(KeyCode::Esc)]);

        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnLeave);
        assert_eq!(
            animator.next_scene_override.as_deref(),
            Some("playground-menu")
        );
    }

    #[test]
    fn debug_f4_moves_to_next_scene_in_loader_order() {
        let temp = tempdir().expect("temp dir");
        let mod_root = temp.path();
        fs::create_dir_all(mod_root.join("scenes")).expect("create scenes dir");
        fs::write(
            mod_root.join("scenes/a.yml"),
            "id: scene-a\ntitle: A\nbg_colour: black\nlayers: []\nnext: null\n",
        )
        .expect("write a");
        fs::write(
            mod_root.join("scenes/b.yml"),
            "id: scene-b\ntitle: B\nbg_colour: black\nlayers: []\nnext: null\n",
        )
        .expect("write b");

        let mut world = World::new();
        world.register(SceneLoader::new(mod_root.to_path_buf()).expect("scene loader"));
        world.register(DebugFeatures {
            enabled: true,
            overlay_visible: true,
            overlay_mode: Default::default(),
        });
        world.register_scoped(SceneRuntime::new(make_cutscene("scene-a", Some("scene-b"))));
        world.register_scoped(make_idle_animator());

        let _ = SceneLifecycleManager::process_events(&mut world, vec![key_pressed(KeyCode::F(4))]);

        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnLeave);
        assert_eq!(animator.next_scene_override.as_deref(), Some("scene-b"));
    }

    #[test]
    fn debug_tilde_toggles_overlay_visibility() {
        let mut world = World::new();
        world.register(DebugFeatures {
            enabled: true,
            overlay_visible: true,
            overlay_mode: Default::default(),
        });
        world.register_scoped(SceneRuntime::new(make_cutscene("scene-a", None)));
        world.register_scoped(make_idle_animator());

        let _ = SceneLifecycleManager::process_events(
            &mut world,
            vec![key_pressed(KeyCode::Char('`'))],
        );

        let debug = world
            .get::<DebugFeatures>()
            .expect("debug settings present");
        assert!(debug.enabled);
        assert!(!debug.overlay_visible);
        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnIdle);
    }

    #[test]
    fn debug_tilde_enables_debug_features_when_disabled() {
        let mut world = World::new();
        world.register(DebugFeatures {
            enabled: false,
            overlay_visible: false,
            overlay_mode: Default::default(),
        });
        world.register_scoped(SceneRuntime::new(make_cutscene("scene-a", None)));
        world.register_scoped(make_idle_animator());

        let _ = SceneLifecycleManager::process_events(
            &mut world,
            vec![key_pressed(KeyCode::Char('`'))],
        );

        let debug = world
            .get::<DebugFeatures>()
            .expect("debug settings present");
        assert!(debug.enabled);
        assert!(debug.overlay_visible);
    }

    #[test]
    fn debug_tab_switches_overlay_mode() {
        let mut world = World::new();
        world.register(DebugFeatures {
            enabled: true,
            overlay_visible: true,
            overlay_mode: DebugOverlayMode::Stats,
        });
        world.register_scoped(SceneRuntime::new(make_cutscene("scene-a", None)));
        world.register_scoped(make_idle_animator());

        let _ = SceneLifecycleManager::process_events(&mut world, vec![key_pressed(KeyCode::Tab)]);

        let debug = world
            .get::<DebugFeatures>()
            .expect("debug settings present");
        assert_eq!(debug.overlay_mode, DebugOverlayMode::Logs);
    }

    #[test]
    fn menu_option_key_sets_next_scene_override() {
        let scene = make_menu_scene(vec![
            make_menu_option("1", "3D SCENE", "playground-3d-scene"),
            MenuOption {
                scene: Some("playground-stop-animation".into()),
                ..make_menu_option("2", "STOP ANIMATION", "playground-stop-animation")
            },
        ]);

        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator());

        let quit = SceneLifecycleManager::process_events(
            &mut world,
            vec![key_pressed(KeyCode::Char('2'))],
        );

        assert!(!quit);
        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(
            animator.next_scene_override.as_deref(),
            Some("playground-stop-animation")
        );
    }

    #[test]
    fn menu_navigation_keys_change_selection_without_leaving_scene() {
        let scene = make_menu_scene(vec![
            make_menu_option("1", "3D SCENE", "playground-3d-scene"),
            make_menu_option("2", "STOP ANIMATION", "playground-stop-animation"),
        ]);

        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator());

        let _ = SceneLifecycleManager::process_events(&mut world, vec![key_pressed(KeyCode::Down)]);

        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.menu_selected_index, 1);
        assert_eq!(animator.stage, SceneStage::OnIdle);
        assert_eq!(animator.next_scene_override, None);
    }

    #[test]
    fn enter_activates_current_menu_selection() {
        let scene = make_menu_scene(vec![
            make_menu_option("1", "3D SCENE", "playground-3d-scene"),
            make_menu_option("2", "STOP ANIMATION", "playground-stop-animation"),
        ]);

        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            menu_selected_index: 1,
            ..make_idle_animator()
        });

        let _ =
            SceneLifecycleManager::process_events(&mut world, vec![key_pressed(KeyCode::Enter)]);

        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(
            animator.next_scene_override.as_deref(),
            Some("playground-stop-animation")
        );
        assert_eq!(animator.stage, SceneStage::OnLeave);
    }

    #[test]
    fn scene_transition_event_swaps_scene_and_resets_animator() {
        let temp = tempdir().expect("temp dir");
        let mod_root = temp.path();
        fs::create_dir_all(mod_root.join("scenes")).expect("create scenes dir");
        fs::write(
            mod_root.join("scenes/intro.yml"),
            "id: intro\ntitle: Intro\nbg_colour: black\nlayers: []\nnext: mainmenu\n",
        )
        .expect("write intro");
        fs::write(
            mod_root.join("scenes/mainmenu.yml"),
            "id: mainmenu\ntitle: Main Menu\nbg_colour: black\nlayers: []\nnext: null\n",
        )
        .expect("write mainmenu");

        let intro = make_cutscene("intro", Some("mainmenu"));

        let mut world = World::new();
        world.register(SceneLoader::new(mod_root.to_path_buf()).expect("scene loader"));
        world.register_scoped(SceneRuntime::new(intro));
        world.register_scoped(Animator {
            stage: SceneStage::Done,
            step_idx: 9,
            elapsed_ms: 999,
            stage_elapsed_ms: 999,
            scene_elapsed_ms: 999,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        let quit = SceneLifecycleManager::process_events(
            &mut world,
            vec![EngineEvent::SceneTransition {
                to_scene_id: "mainmenu".to_string(),
            }],
        );

        assert!(!quit);
        let scene = world.get::<SceneRuntime>().expect("scene present");
        assert_eq!(scene.scene().id, "mainmenu");
        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnEnter);
        assert_eq!(animator.step_idx, 0);
        assert_eq!(animator.elapsed_ms, 0);
        assert_eq!(animator.scene_elapsed_ms, 0);
    }

    #[test]
    fn classifies_quit_resize_and_transitions() {
        let lifecycle = classify_events(vec![
            EngineEvent::Tick,
            EngineEvent::OutputResized {
                width: 120,
                height: 40,
            },
            EngineEvent::SceneTransition {
                to_scene_id: "mainmenu".to_string(),
            },
            EngineEvent::Quit,
        ]);

        assert!(lifecycle.quit);
        assert_eq!(lifecycle.resizes, vec![(120, 40)]);
        assert_eq!(lifecycle.transitions, vec!["mainmenu".to_string()]);
    }

    #[test]
    fn scene_transition_supports_explicit_scene_path_reference() {
        let temp = tempdir().expect("temp dir");
        let mod_root = temp.path();
        fs::create_dir_all(mod_root.join("scenes")).expect("create scenes dir");
        fs::write(
            mod_root.join("scenes/intro.yml"),
            "id: intro\ntitle: Intro\nbg_colour: black\nlayers: []\nnext: /scenes/mainmenu.yml\n",
        )
        .expect("write intro");
        fs::write(
            mod_root.join("scenes/mainmenu.yml"),
            "id: mainmenu\ntitle: Main Menu\nbg_colour: black\nlayers: []\nnext: null\n",
        )
        .expect("write mainmenu");

        let intro = make_cutscene("intro", Some("/scenes/mainmenu.yml"));

        let mut world = World::new();
        world.register(SceneLoader::new(mod_root.to_path_buf()).expect("scene loader"));
        world.register_scoped(SceneRuntime::new(intro));
        world.register_scoped(Animator::new());

        let quit = SceneLifecycleManager::process_events(
            &mut world,
            vec![EngineEvent::SceneTransition {
                to_scene_id: "/scenes/mainmenu.yml".to_string(),
            }],
        );

        assert!(!quit);
        let scene = world.get::<SceneRuntime>().expect("scene present");
        assert_eq!(scene.scene().id, "mainmenu");
    }

    #[test]
    fn transitioning_to_playground_exit_requests_quit() {
        let temp = tempdir().expect("temp dir");
        let mod_root = temp.path();
        fs::create_dir_all(mod_root.join("scenes")).expect("create scenes dir");
        fs::write(
            mod_root.join("scenes/intro.yml"),
            "id: intro\ntitle: Intro\nbg_colour: black\nlayers: []\nnext: null\n",
        )
        .expect("write intro");
        fs::write(
            mod_root.join("scenes/exit.yml"),
            "id: playground-exit\ntitle: Exit\nbg_colour: black\nlayers: []\nnext: null\n",
        )
        .expect("write exit");

        let mut world = World::new();
        world.register(SceneLoader::new(mod_root.to_path_buf()).expect("scene loader"));
        world.register_scoped(SceneRuntime::new(make_cutscene(
            "intro",
            Some("playground-exit"),
        )));
        world.register_scoped(Animator::new());

        let quit = SceneLifecycleManager::process_events(
            &mut world,
            vec![EngineEvent::SceneTransition {
                to_scene_id: "playground-exit".to_string(),
            }],
        );

        assert!(quit);
        let scene = world.get::<SceneRuntime>().expect("scene present");
        assert_eq!(scene.scene().id, "intro");
    }

    #[test]
    fn key_release_clears_runtime_held_key_state() {
        let scene = make_menu_scene(Vec::new());
        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator());

        SceneLifecycleManager::process_events(&mut world, vec![key_pressed(KeyCode::Left)]);
        let runtime = world.scene_runtime().expect("runtime");
        assert!(runtime.keys_down_snapshot().contains("Left"));

        SceneLifecycleManager::process_events(&mut world, vec![key_released(KeyCode::Left)]);
        let runtime = world.scene_runtime().expect("runtime");
        assert!(!runtime.keys_down_snapshot().contains("Left"));
    }

    #[test]
    fn input_focus_lost_clears_all_held_keys() {
        let scene = make_menu_scene(Vec::new());
        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator());

        SceneLifecycleManager::process_events(
            &mut world,
            vec![key_pressed(KeyCode::Char('a')), key_pressed(KeyCode::Right)],
        );
        let runtime = world.scene_runtime().expect("runtime");
        assert!(runtime.keys_down_snapshot().contains("a"));
        assert!(runtime.keys_down_snapshot().contains("Right"));

        SceneLifecycleManager::process_events(&mut world, vec![EngineEvent::InputFocusLost]);
        let runtime = world.scene_runtime().expect("runtime");
        assert!(runtime.keys_down_snapshot().is_empty());
        assert!(runtime.last_raw_key_snapshot().is_none());
    }

    #[test]
    fn free_look_camera_masks_wasd_and_moves_scene_camera() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: free-look-test
title: Free Look
bg_colour: black
input:
  free-look-camera: {}
layers: []
next: null
"#,
        )
        .expect("scene parse");
        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator());

        SceneLifecycleManager::process_events(
            &mut world,
            vec![
                key_pressed_with_modifiers(KeyCode::Char('f'), KeyModifiers::CONTROL),
                key_pressed(KeyCode::Char('w')),
            ],
        );

        let runtime = world.scene_runtime().expect("runtime");
        assert!(runtime.free_look_camera_engaged());
        assert!(!runtime.keys_down_snapshot().contains("w"));

        crate::systems::free_look_camera::free_look_camera_system(&mut world, 1000);

        let runtime = world.scene_runtime().expect("runtime");
        assert!(runtime.scene_camera_3d().eye[2] < 3.0);
    }
}

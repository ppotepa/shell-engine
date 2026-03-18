use crate::events::EngineEvent;
use crate::scene::{self, SceneRenderedMode};
use crate::scene_runtime::SceneRuntime;
use crate::services::EngineWorldAccess;
use crate::systems::animator::{Animator, SceneStage};
use crate::systems::menu::{evaluate_menu_action, MenuAction};
use crate::world::World;
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::terminal::SetSize;
use std::io::stdout;

pub struct SceneLifecycleManager;

#[derive(Default)]
struct LifecycleEvents {
    quit: bool,
    key_presses: Vec<KeyEvent>,
    transitions: Vec<String>,
    resizes: Vec<(u16, u16)>,
    mouse_moves: Vec<(u16, u16)>,
}

impl SceneLifecycleManager {
    /// Process frame events related to scene lifecycle and transitions.
    /// Returns `true` when a quit event was requested.
    pub fn process_events(world: &mut World, events: Vec<EngineEvent>) -> bool {
        let lifecycle = classify_events(events);
        for (width, height) in &lifecycle.resizes {
            // Resize output buffer
            if let Some(buf) = world.buffer_mut() {
                buf.resize(*width, *height);
            }
            // Also resize virtual buffer if using max-available
            Self::handle_virtual_buffer_resize(world, *width, *height);
        }

        if !lifecycle.key_presses.is_empty() {
            Self::advance_on_any_key(world, &lifecycle.key_presses);
        }
        if !lifecycle.mouse_moves.is_empty() {
            handle_playground_3d_mouse(world, &lifecycle.mouse_moves);
        }
        Self::apply_transitions(world, lifecycle.transitions);
        lifecycle.quit
    }

    fn handle_virtual_buffer_resize(world: &mut World, term_width: u16, term_height: u16) {
        let Some(settings) = world.runtime_settings() else {
            return;
        };

        if !settings.use_virtual_buffer || !settings.virtual_size_max_available {
            return;
        }

        // Resize virtual buffer to match terminal when using max-available
        if let Some(vbuf) = world.virtual_buffer_mut() {
            let new_w = term_width.max(1);
            let new_h = term_height.max(1);
            if vbuf.0.width != new_w || vbuf.0.height != new_h {
                vbuf.0.resize(new_w, new_h);
            }
        }
    }

    fn advance_on_any_key(world: &mut World, key_presses: &[KeyEvent]) {
        if handle_terminal_shell_controls(world, key_presses) {
            return;
        }
        if handle_obj_viewer_controls(world, key_presses) {
            return;
        }
        if handle_terminal_size_tester_controls(world, key_presses) {
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
        let menu_action = evaluate_menu_action(&menu_options, selected_index, key_presses);

        if !is_scene_idle(world) || !any_key_trigger {
            return;
        }

        if let Some(animator) = world.animator_mut() {
            match menu_action {
                MenuAction::Navigate(index) => animator.menu_selected_index = index,
                MenuAction::Activate(next_scene) => {
                    animator.next_scene_override = Some(next_scene);
                    begin_leave(animator);
                }
                MenuAction::None if menu_options.is_empty() => begin_leave(animator),
                MenuAction::None => {}
            }
        }
    }

    fn apply_transitions(world: &mut World, transitions: Vec<String>) {
        for to_scene_ref in transitions {
            let Some(new_scene) = world
                .scene_loader()
                .and_then(|loader| loader.load_by_ref(&to_scene_ref).ok())
            else {
                continue;
            };
            Self::apply_virtual_size_override(world, &new_scene);
            world.clear_scoped();
            world.register_scoped(SceneRuntime::new(new_scene));
            world.register_scoped(Animator::new());
        }
    }

    fn apply_virtual_size_override(world: &mut World, scene: &scene::Scene) {
        let Some(settings) = world.runtime_settings() else {
            return;
        };
        if !settings.use_virtual_buffer {
            return;
        }
        let Some(size_override) = scene.virtual_size_override.as_deref() else {
            return;
        };
        let Some((w, h, is_max)) = crate::runtime_settings::parse_virtual_size_str(size_override)
        else {
            return;
        };
        let (new_width, new_height) = if is_max {
            let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
            (term_w.max(1), term_h.max(1))
        } else {
            (w, h)
        };
        if let Some(vbuf) = world.virtual_buffer_mut() {
            if vbuf.0.width != new_width || vbuf.0.height != new_height {
                vbuf.0.resize(new_width, new_height);
            }
        }
    }
}

fn classify_events(events: Vec<EngineEvent>) -> LifecycleEvents {
    let mut lifecycle = LifecycleEvents::default();
    for event in events {
        match event {
            EngineEvent::Quit => lifecycle.quit = true,
            EngineEvent::KeyPressed(code) => lifecycle.key_presses.push(code),
            EngineEvent::MouseMoved { column, row } => lifecycle.mouse_moves.push((column, row)),
            EngineEvent::SceneTransition { to_scene_id } => lifecycle.transitions.push(to_scene_id),
            EngineEvent::TerminalResized { width, height } => {
                lifecycle.resizes.push((width, height));
            }
            _ => {}
        }
    }
    lifecycle
}

fn is_scene_idle(world: &World) -> bool {
    world
        .animator()
        .map(|a| a.stage == SceneStage::OnIdle)
        .unwrap_or(false)
}

fn begin_leave(a: &mut crate::systems::animator::Animator) {
    a.stage = SceneStage::OnLeave;
    a.step_idx = 0;
    a.elapsed_ms = 0;
}

fn active_obj_viewer_target(world: &World) -> Option<String> {
    world
        .scene_runtime()
        .and_then(|runtime| runtime.scene().input.obj_viewer.as_ref())
        .map(|cfg| cfg.sprite_id.clone())
}

fn active_terminal_size_presets(world: &World) -> Option<Vec<(u16, u16)>> {
    let cfg = world
        .scene_runtime()
        .and_then(|runtime| runtime.scene().input.terminal_size_tester.clone())?;
    let mut out = Vec::new();
    for preset in cfg.presets {
        if let Some((w, h, is_max)) = crate::runtime_settings::parse_virtual_size_str(&preset) {
            if !is_max {
                out.push((w, h));
            }
        }
    }
    if out.is_empty() {
        out.extend([(80, 24), (100, 30), (120, 36), (160, 48)]);
    }
    Some(out)
}

fn apply_terminal_size_change(world: &mut World, width: u16, height: u16) {
    let _ = crossterm::execute!(stdout(), SetSize(width, height));
    if let Some(buf) = world.buffer_mut() {
        buf.resize(width, height);
    }
    if let Some(vbuf) = world.virtual_buffer_mut() {
        vbuf.0.resize(width, height);
    }
}

fn handle_terminal_size_tester_controls(world: &mut World, key_presses: &[KeyEvent]) -> bool {
    let Some(presets) = active_terminal_size_presets(world) else {
        return false;
    };
    if !is_scene_idle(world) {
        return false;
    }
    if key_presses
        .iter()
        .any(|key| matches!(key.code, KeyCode::Enter))
    {
        return false;
    }

    for key in key_presses {
        if let KeyCode::Char(c @ '1'..='9') = key.code {
            let i = (c as usize) - ('1' as usize);
            if let Some(&(w, h)) = presets.get(i) {
                apply_terminal_size_change(world, w, h);
                return true;
            }
        }
    }
    false
}

fn handle_terminal_shell_controls(world: &mut World, key_presses: &[KeyEvent]) -> bool {
    if !is_scene_idle(world) {
        return false;
    }
    let Some(runtime) = world.scene_runtime_mut() else {
        return false;
    };
    if !runtime.has_terminal_shell() {
        return false;
    }
    runtime.handle_terminal_shell_keys(key_presses)
}

fn handle_obj_viewer_controls(world: &mut World, key_presses: &[KeyEvent]) -> bool {
    let Some(sprite_id) = active_obj_viewer_target(world) else {
        return false;
    };
    if !is_scene_idle(world) {
        return false;
    }

    if key_presses
        .iter()
        .any(|key| matches!(key.code, KeyCode::Enter))
    {
        return false;
    }

    let orbit_active = world
        .scene_runtime()
        .map(|r| r.is_obj_orbit_active(&sprite_id))
        .unwrap_or(true);

    let mut zoom_delta = 0.0f32;
    let mut mode_switch: Option<SceneRenderedMode> = None;
    let mut toggle_wireframe = false;
    let mut toggle_orbit = false;
    let mut pan_dx = 0.0f32;
    let mut pan_dy = 0.0f32;

    for key in key_presses {
        match key.code {
            KeyCode::Char('a') | KeyCode::Char('A') => zoom_delta += 0.1,
            KeyCode::Char('z') | KeyCode::Char('Z') => zoom_delta -= 0.1,
            KeyCode::Char('1') | KeyCode::Char('6') => mode_switch = Some(SceneRenderedMode::Cell),
            KeyCode::Char('2') | KeyCode::Char('7') => {
                mode_switch = Some(SceneRenderedMode::HalfBlock)
            }
            KeyCode::Char('3') | KeyCode::Char('8') => {
                mode_switch = Some(SceneRenderedMode::QuadBlock)
            }
            KeyCode::Char('4') => mode_switch = Some(SceneRenderedMode::Braille),
            KeyCode::Char('5') => toggle_wireframe = true,
            KeyCode::Char('o') | KeyCode::Char('O') => toggle_orbit = true,
            // Arrow keys: pan camera when orbit is off.
            KeyCode::Left if !orbit_active => pan_dx -= 0.04,
            KeyCode::Right if !orbit_active => pan_dx += 0.04,
            KeyCode::Up if !orbit_active => pan_dy += 0.04,
            KeyCode::Down if !orbit_active => pan_dy -= 0.04,
            _ => {}
        }
    }

    if let Some(runtime) = world.scene_runtime_mut() {
        if zoom_delta != 0.0 {
            let _ = runtime.adjust_obj_scale(&sprite_id, zoom_delta);
        }
        if let Some(mode) = mode_switch {
            runtime.set_scene_rendered_mode(mode);
        }
        if toggle_wireframe {
            let _ = runtime.toggle_obj_surface_mode(&sprite_id);
        }
        if toggle_orbit {
            let _ = runtime.toggle_obj_orbit(&sprite_id);
            // Reset mouse reference so first mouse move after toggle doesn't jump.
            runtime.set_obj_last_mouse_pos(&sprite_id, None);
        }
        if pan_dx != 0.0 || pan_dy != 0.0 {
            runtime.apply_obj_camera_pan(&sprite_id, pan_dx, pan_dy);
        }
    }

    true
}

fn handle_playground_3d_mouse(world: &mut World, mouse_moves: &[(u16, u16)]) {
    let Some(sprite_id) = active_obj_viewer_target(world) else {
        return;
    };
    if !is_scene_idle(world) {
        return;
    }

    let orbit_active = world
        .scene_runtime()
        .map(|r| r.is_obj_orbit_active(&sprite_id))
        .unwrap_or(true);
    if orbit_active {
        // Orbit is on — mouse look is disabled; just update position reference.
        if let Some(last) = mouse_moves.last() {
            if let Some(runtime) = world.scene_runtime_mut() {
                runtime.set_obj_last_mouse_pos(&sprite_id, Some(*last));
            }
        }
        return;
    }

    let last_pos = world
        .scene_runtime()
        .and_then(|r| r.obj_last_mouse_pos(&sprite_id));

    let Some((mut prev_col, mut prev_row)) = last_pos else {
        // First event after orbit was toggled off — seed position, don't rotate.
        if let Some(last) = mouse_moves.last() {
            if let Some(runtime) = world.scene_runtime_mut() {
                runtime.set_obj_last_mouse_pos(&sprite_id, Some(*last));
            }
        }
        return;
    };

    let mut total_dyaw = 0.0f32;
    let mut total_dpitch = 0.0f32;

    for &(col, row) in mouse_moves {
        let dc = col as f32 - prev_col as f32;
        let dr = row as f32 - prev_row as f32;
        // Scale: 1 terminal cell ≈ 1.8 degrees horizontal, 2.8 degrees vertical.
        total_dyaw += dc * 1.8;
        total_dpitch += dr * 2.8;
        prev_col = col;
        prev_row = row;
    }

    if let Some(runtime) = world.scene_runtime_mut() {
        runtime.set_obj_last_mouse_pos(&sprite_id, Some((prev_col, prev_row)));
        if total_dyaw != 0.0 || total_dpitch != 0.0 {
            runtime.apply_obj_camera_look(&sprite_id, total_dyaw, total_dpitch);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_events, SceneLifecycleManager};
    use crate::events::EngineEvent;
    use crate::scene::{
        MenuOption, Scene, SceneAudio, SceneRenderedMode, SceneStages, Sprite, Stage, StageTrigger,
        TermColour,
    };
    use crate::scene_loader::SceneLoader;
    use crate::scene_runtime::SceneRuntime;
    use crate::systems::animator::{Animator, SceneStage};
    use crate::world::World;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::fs;
    use tempfile::tempdir;

    fn key_pressed(code: KeyCode) -> EngineEvent {
        EngineEvent::KeyPressed(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn key_pressed_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> EngineEvent {
        EngineEvent::KeyPressed(KeyEvent::new(code, modifiers))
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
            selected_effect: None,
            action: None,
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
            rendered_mode: SceneRenderedMode::Cell,
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
            layers: Vec::new(),
            menu_options,
            input: Default::default(),
            next: Some("playground-3d-scene".into()),
        }
    }

    fn make_cutscene(id: &str, next: Option<&str>) -> Scene {
        Scene {
            id: id.into(),
            title: id.into(),
            cutscene: true,
            target_fps: None,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            input: Default::default(),
            next: next.map(Into::into),
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

    const TERMINAL_SHELL_SCENE_YAML: &str = r#"
id: playground-terminal-shell
title: Terminal Shell
bg_colour: black
input:
  terminal-shell:
    prompt_sprite_id: terminal-prompt
    output_sprite_id: terminal-output
    prompt_prefix: "λ "
    max_lines: 20
    banner:
      - "connected: shell-node"
    commands:
      - name: status
        description: Show system status
        output:
          - "hull: 92%"
          - "power: online"
stages:
  on_idle:
    trigger: any-key
    looping: true
    steps: []
menu-options:
  - key: "1"
    next: playground-3d-scene
layers:
  - name: ui
    sprites:
      - type: text
        id: terminal-output
        at: lt
        content: ""
      - type: text
        id: terminal-prompt
        at: lb
        content: ""
"#;

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
                key_pressed(KeyCode::Char('4')),
                key_pressed(KeyCode::Char('5')),
                key_pressed(KeyCode::Char('O')),
            ],
        );

        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnIdle);
        let runtime = world.get::<SceneRuntime>().expect("runtime present");
        assert_eq!(runtime.scene().rendered_mode, SceneRenderedMode::Braille);
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
    fn terminal_shell_consumes_keys_and_updates_text_sprites() {
        let scene: Scene = serde_yaml::from_str(TERMINAL_SHELL_SCENE_YAML).expect("scene parse");
        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator());

        let _ = SceneLifecycleManager::process_events(
            &mut world,
            vec![
                key_pressed(KeyCode::Char('l')),
                key_pressed(KeyCode::Char('s')),
                key_pressed(KeyCode::Enter),
                key_pressed(KeyCode::Char('1')),
            ],
        );

        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnIdle);
        assert_eq!(animator.next_scene_override, None);

        let runtime = world.get::<SceneRuntime>().expect("runtime present");
        let output = runtime
            .text_sprite_content("terminal-output")
            .expect("terminal output sprite");
        assert!(output.contains("connected: shell-node"));
        assert!(output.contains("λ ls"));
        assert!(output.contains("logs  vault  airlock  notes"));

        let prompt = runtime
            .text_sprite_content("terminal-prompt")
            .expect("terminal prompt sprite");
        assert_eq!(prompt, "λ 1");
    }

    #[test]
    fn terminal_shell_supports_line_edit_shortcuts() {
        let scene: Scene = serde_yaml::from_str(TERMINAL_SHELL_SCENE_YAML).expect("scene parse");
        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(make_idle_animator());

        let _ = SceneLifecycleManager::process_events(
            &mut world,
            vec![
                key_pressed(KeyCode::Char('a')),
                key_pressed(KeyCode::Char('b')),
                key_pressed(KeyCode::Char('c')),
                key_pressed_with_modifiers(KeyCode::Char('a'), KeyModifiers::CONTROL),
                key_pressed(KeyCode::Char('x')),
                key_pressed_with_modifiers(KeyCode::Char('k'), KeyModifiers::CONTROL),
            ],
        );

        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnIdle);
        assert_eq!(animator.next_scene_override, None);

        let runtime = world.get::<SceneRuntime>().expect("runtime present");
        let prompt = runtime
            .text_sprite_content("terminal-prompt")
            .expect("terminal prompt sprite");
        assert_eq!(prompt, "λ x");

        let output = runtime
            .text_sprite_content("terminal-output")
            .expect("terminal output sprite");
        assert!(output.contains("connected: shell-node"));
        assert!(!output.contains("λ abc"));
    }

    #[test]
    fn menu_option_key_sets_next_scene_override() {
        let scene = make_menu_scene(vec![
            make_menu_option("1", "3D SCENE", "playground-3d-scene"),
            MenuOption {
                action: Some("goto.scene".into()),
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
            EngineEvent::TerminalResized {
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
}

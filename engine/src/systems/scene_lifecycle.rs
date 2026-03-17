use crate::events::EngineEvent;
use crate::scene::{self};
use crate::scene_runtime::SceneRuntime;
use crate::services::EngineWorldAccess;
use crate::systems::animator::{Animator, SceneStage};
use crate::world::World;
use crossterm::event::KeyCode;

pub struct SceneLifecycleManager;

#[derive(Default)]
struct LifecycleEvents {
    quit: bool,
    key_presses: Vec<KeyCode>,
    transitions: Vec<String>,
    resizes: Vec<(u16, u16)>,
}

impl SceneLifecycleManager {
    /// Process frame events related to scene lifecycle and transitions.
    /// Returns `true` when a quit event was requested.
    pub fn process_events(world: &mut World, events: Vec<EngineEvent>) -> bool {
        let lifecycle = classify_events(events);
        for (width, height) in &lifecycle.resizes {
            if let Some(buf) = world.buffer_mut() {
                buf.resize(*width, *height);
            }
        }

        if !lifecycle.key_presses.is_empty() {
            Self::advance_on_any_key(world, &lifecycle.key_presses);
        }
        Self::apply_transitions(world, lifecycle.transitions);
        lifecycle.quit
    }

    fn advance_on_any_key(world: &mut World, key_presses: &[KeyCode]) {
        let menu_options = world
            .scene_runtime()
            .map(|runtime| runtime.scene().menu_options.clone())
            .unwrap_or_default();
        let selected_index = world
            .animator()
            .map(|animator| animator.menu_selected_index)
            .unwrap_or(0);
        let menu_action = evaluate_menu_action(&menu_options, selected_index, key_presses);
        let should_leave = world
            .scene_runtime()
            .map(|runtime| {
                matches!(
                    runtime.scene().stages.on_idle.trigger,
                    scene::StageTrigger::AnyKey
                )
            })
            .unwrap_or(false)
            && world
                .animator()
                .map(|a| a.stage == SceneStage::OnIdle)
            .unwrap_or(false);

        if !should_leave {
            return;
        }

        if let Some(animator) = world.animator_mut() {
            match menu_action {
                MenuAction::Navigate(index) => {
                    animator.menu_selected_index = index;
                }
                MenuAction::Activate(next_scene) => {
                    animator.next_scene_override = Some(next_scene);
                    animator.stage = SceneStage::OnLeave;
                    animator.step_idx = 0;
                    animator.elapsed_ms = 0;
                }
                MenuAction::None => {
                    if menu_options.is_empty() {
                        animator.stage = SceneStage::OnLeave;
                        animator.step_idx = 0;
                        animator.elapsed_ms = 0;
                    }
                }
            }
        }
    }

    fn apply_transitions(world: &mut World, transitions: Vec<String>) {
        for to_scene_ref in transitions {
            let new_scene = world
                .scene_loader()
                .and_then(|loader| loader.load_by_ref(&to_scene_ref).ok());

            if let Some(new_scene) = new_scene {
                world.clear_scoped();
                world.register_scoped(SceneRuntime::new(new_scene));
                world.register_scoped(Animator::new());
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
            EngineEvent::SceneTransition { to_scene_id } => lifecycle.transitions.push(to_scene_id),
            EngineEvent::TerminalResized { width, height } => {
                lifecycle.resizes.push((width, height));
            }
            _ => {}
        }
    }
    lifecycle
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MenuAction {
    None,
    Navigate(usize),
    Activate(String),
}

fn evaluate_menu_action(
    options: &[crate::scene::MenuOption],
    selected_index: usize,
    key_presses: &[KeyCode],
) -> MenuAction {
    if options.is_empty() {
        return MenuAction::None;
    }
    let mut index = selected_index.min(options.len().saturating_sub(1));

    for key_code in key_presses {
        for (option_idx, option) in options.iter().enumerate() {
            if key_matches_binding(key_code, &option.key) {
                if let Some(target) = resolve_menu_target(option) {
                    return MenuAction::Activate(target);
                }
            }
            if option_idx == index && matches_confirm_key(key_code) {
                if let Some(target) = resolve_menu_target(option) {
                    return MenuAction::Activate(target);
                }
            }
        }

        if is_prev_key(key_code) {
            index = if index == 0 {
                options.len().saturating_sub(1)
            } else {
                index - 1
            };
            continue;
        }

        if is_next_key(key_code) {
            index = (index + 1) % options.len();
            continue;
        }
    }

    if index != selected_index {
        return MenuAction::Navigate(index);
    }
    MenuAction::None
}

fn resolve_menu_target(option: &crate::scene::MenuOption) -> Option<String> {
    let action = option
        .action
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase);
    match action.as_deref() {
        Some("goto.scene") => option.scene.clone().or_else(|| Some(option.next.clone())),
        _ => Some(option.next.clone()),
    }
}

fn key_matches_binding(key_code: &KeyCode, binding: &str) -> bool {
    let b = binding.trim().to_ascii_lowercase();
    match key_code {
        KeyCode::Char(c) => {
            b == c.to_ascii_lowercase().to_string() || (*c == ' ' && b == "space")
        }
        KeyCode::Enter => b == "enter",
        KeyCode::Esc => b == "esc" || b == "escape",
        KeyCode::Tab => b == "tab",
        KeyCode::Backspace => b == "backspace",
        KeyCode::Left => b == "left",
        KeyCode::Right => b == "right",
        KeyCode::Up => b == "up",
        KeyCode::Down => b == "down",
        KeyCode::Home => b == "home",
        KeyCode::End => b == "end",
        KeyCode::PageUp => b == "pageup" || b == "page-up",
        KeyCode::PageDown => b == "pagedown" || b == "page-down",
        KeyCode::Delete => b == "delete" || b == "del",
        KeyCode::Insert => b == "insert" || b == "ins",
        KeyCode::F(n) => b == format!("f{n}"),
        KeyCode::Null => b == "null",
        _ => false,
    }
}

fn is_prev_key(key_code: &KeyCode) -> bool {
    matches!(key_code, KeyCode::Up | KeyCode::Left)
}

fn is_next_key(key_code: &KeyCode) -> bool {
    matches!(key_code, KeyCode::Down | KeyCode::Right)
}

fn matches_confirm_key(key_code: &KeyCode) -> bool {
    matches!(key_code, KeyCode::Enter | KeyCode::Char(' '))
}

#[cfg(test)]
mod tests {
    use super::{classify_events, SceneLifecycleManager};
    use crate::events::EngineEvent;
    use crate::scene::{
        MenuOption, Scene, SceneAudio, SceneRenderedMode, SceneStages, Stage, StageTrigger,
        TermColour,
    };
    use crate::scene_loader::SceneLoader;
    use crate::scene_runtime::SceneRuntime;
    use crate::systems::animator::{Animator, SceneStage};
    use crate::world::World;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn any_key_moves_idle_scene_to_leave_when_trigger_is_any_key() {
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
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
            menu_options: Vec::new(),
            next: None,
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

        let quit = SceneLifecycleManager::process_events(
            &mut world,
            vec![EngineEvent::KeyPressed(crossterm::event::KeyCode::Enter)],
        );

        assert!(!quit);
        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.stage, SceneStage::OnLeave);
        assert_eq!(animator.step_idx, 0);
        assert_eq!(animator.elapsed_ms, 0);
    }

    #[test]
    fn menu_option_key_sets_next_scene_override() {
        let scene = Scene {
            id: "menu".to_string(),
            title: "Menu".to_string(),
            cutscene: false,
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
            menu_options: vec![
                MenuOption {
                    key: "1".to_string(),
                    label: Some("3D SCENE".to_string()),
                    selected_effect: None,
                    action: None,
                    scene: None,
                    next: "playground-3d-scene".to_string(),
                },
                MenuOption {
                    key: "2".to_string(),
                    label: Some("STOP ANIMATION".to_string()),
                    selected_effect: None,
                    action: Some("goto.scene".to_string()),
                    scene: Some("playground-stop-animation".to_string()),
                    next: "playground-stop-animation".to_string(),
                },
            ],
            next: Some("playground-3d-scene".to_string()),
        };

        let mut world = World::new();
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

        let quit = SceneLifecycleManager::process_events(
            &mut world,
            vec![EngineEvent::KeyPressed(crossterm::event::KeyCode::Char('2'))],
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
        let scene = Scene {
            id: "menu".to_string(),
            title: "Menu".to_string(),
            cutscene: false,
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
            menu_options: vec![
                MenuOption {
                    key: "1".to_string(),
                    label: Some("3D SCENE".to_string()),
                    selected_effect: None,
                    action: None,
                    scene: None,
                    next: "playground-3d-scene".to_string(),
                },
                MenuOption {
                    key: "2".to_string(),
                    label: Some("STOP ANIMATION".to_string()),
                    selected_effect: None,
                    action: None,
                    scene: None,
                    next: "playground-stop-animation".to_string(),
                },
            ],
            next: Some("playground-3d-scene".to_string()),
        };

        let mut world = World::new();
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

        let _ = SceneLifecycleManager::process_events(
            &mut world,
            vec![EngineEvent::KeyPressed(crossterm::event::KeyCode::Down)],
        );

        let animator = world.get::<Animator>().expect("animator present");
        assert_eq!(animator.menu_selected_index, 1);
        assert_eq!(animator.stage, SceneStage::OnIdle);
        assert_eq!(animator.next_scene_override, None);
    }

    #[test]
    fn enter_activates_current_menu_selection() {
        let scene = Scene {
            id: "menu".to_string(),
            title: "Menu".to_string(),
            cutscene: false,
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
            menu_options: vec![
                MenuOption {
                    key: "1".to_string(),
                    label: Some("3D SCENE".to_string()),
                    selected_effect: None,
                    action: None,
                    scene: None,
                    next: "playground-3d-scene".to_string(),
                },
                MenuOption {
                    key: "2".to_string(),
                    label: Some("STOP ANIMATION".to_string()),
                    selected_effect: None,
                    action: None,
                    scene: None,
                    next: "playground-stop-animation".to_string(),
                },
            ],
            next: Some("playground-3d-scene".to_string()),
        };

        let mut world = World::new();
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 1,
        });

        let _ = SceneLifecycleManager::process_events(
            &mut world,
            vec![EngineEvent::KeyPressed(crossterm::event::KeyCode::Enter)],
        );

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

        let intro = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: Some("mainmenu".to_string()),
        };

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

        let intro = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: Some("/scenes/mainmenu.yml".to_string()),
        };

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

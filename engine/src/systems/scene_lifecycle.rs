use crate::events::EngineEvent;
use crate::scene::{self};
use crate::scene_runtime::SceneRuntime;
use crate::services::EngineWorldAccess;
use crate::systems::animator::{Animator, SceneStage};
use crate::world::World;

pub struct SceneLifecycleManager;

#[derive(Default)]
struct LifecycleEvents {
    quit: bool,
    key_pressed: bool,
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

        if lifecycle.key_pressed {
            Self::advance_on_any_key(world);
        }
        Self::apply_transitions(world, lifecycle.transitions);
        lifecycle.quit
    }

    fn advance_on_any_key(world: &mut World) {
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

        if should_leave {
            if let Some(animator) = world.animator_mut() {
                animator.stage = SceneStage::OnLeave;
                animator.step_idx = 0;
                animator.elapsed_ms = 0;
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
            EngineEvent::KeyPressed(_) => lifecycle.key_pressed = true,
            EngineEvent::SceneTransition { to_scene_id } => lifecycle.transitions.push(to_scene_id),
            EngineEvent::TerminalResized { width, height } => {
                lifecycle.resizes.push((width, height));
            }
            _ => {}
        }
    }
    lifecycle
}

#[cfg(test)]
mod tests {
    use super::{classify_events, SceneLifecycleManager};
    use crate::events::EngineEvent;
    use crate::scene::{
        Scene, SceneAudio, SceneRenderedMode, SceneStages, Stage, StageTrigger, TermColour,
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

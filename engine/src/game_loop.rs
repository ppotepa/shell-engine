use crate::buffer;
use crate::effects::utils::math::TICK_MS;
use crate::error::EngineError;
use crate::events::{EngineEvent, EventQueue};
use crate::scene;
use crate::scene_loader::SceneLoader;
use crate::systems;
use crate::world::World;

pub fn game_loop(world: &mut World) -> Result<(), EngineError> {
    use crossterm::event::{self, Event, KeyCode};
    use systems::animator::Animator;

    loop {
        // --- INPUT ---
        if event::poll(std::time::Duration::from_millis(TICK_MS))?  {
            match event::read()? {
                Event::Key(key) => {
                    let ev = match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => EngineEvent::Quit,
                        code => EngineEvent::KeyPressed(code),
                    };
                    world.get_mut::<EventQueue>().unwrap().push(ev);
                }
                Event::Resize(w, h) => {
                    world
                        .get_mut::<EventQueue>()
                        .unwrap()
                        .push(EngineEvent::TerminalResized { width: w, height: h });
                }
                _ => {}
            }
        }

        world.get_mut::<EventQueue>().unwrap().push(EngineEvent::Tick);

        // --- DRAIN EVENTS ---
        let events = world.get_mut::<EventQueue>().unwrap().drain();

        let mut quit = false;
        let mut key_pressed = false;
        let mut transitions: Vec<String> = Vec::new();

        for event in events {
            match event {
                EngineEvent::Quit => quit = true,
                EngineEvent::KeyPressed(_) => key_pressed = true,
                EngineEvent::SceneTransition { to_scene_id } => transitions.push(to_scene_id),
                EngineEvent::TerminalResized { width, height } => {
                    if let Some(buf) = world.get_mut::<buffer::Buffer>() {
                        buf.resize(width, height);
                    }
                }
                _ => {}
            }
        }

        if quit {
            break;
        }

        // Any-key trigger: if on_idle with any-key trigger, advance to on_leave
        if key_pressed {
            use scene::StageTrigger;
            let should_leave = world.get::<scene::Scene>()
                .map(|s| matches!(s.stages.on_idle.trigger, StageTrigger::AnyKey))
                .unwrap_or(false)
                && world.get::<systems::animator::Animator>()
                    .map(|a| a.stage == systems::animator::SceneStage::OnIdle)
                    .unwrap_or(false);

            if should_leave {
                if let Some(a) = world.get_mut::<systems::animator::Animator>() {
                    a.stage      = systems::animator::SceneStage::OnLeave;
                    a.step_idx   = 0;
                    a.elapsed_ms = 0;
                }
            }
        }

        // Handle scene transitions
        for to_scene_id in transitions {
            let new_scene = world
                .get::<SceneLoader>()
                .and_then(|l| l.load_by_id(&to_scene_id).ok());
            if let Some(new_scene) = new_scene {
                world.clear_scoped();
                world.register_scoped(new_scene);
                world.register_scoped(Animator::new());
            }
        }

        // --- SYSTEMS ---
        systems::animator::animator_system(world);
        systems::compositor::compositor_system(world);
        systems::renderer::renderer_system(world);
    }
    Ok(())
}

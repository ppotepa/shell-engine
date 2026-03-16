use crate::buffer;
use crate::error::EngineError;
use crate::events::{EngineEvent, EventQueue};
use crate::scene;
use crate::scene_loader::SceneLoader;
use crate::systems;
use crate::world::World;

pub fn game_loop(world: &mut World, target_fps: u16) -> Result<(), EngineError> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use systems::animator::Animator;
    use std::time::{Duration, Instant};

    const FAST_FORWARD_TICKS: u8 = 8;
    let mut debug_fast_forward = false;
    let fps = target_fps.max(1) as u64;
    let tick_ms = (1000 / fps).max(1);
    let frame_budget = Duration::from_millis(tick_ms);

    loop {
        let frame_start = Instant::now();

        // --- INPUT ---
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }

                    if is_debug_fast_forward_toggle(key.code, key.modifiers) {
                        debug_fast_forward = !debug_fast_forward;
                        continue;
                    }

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
        let ticks_this_frame = if debug_fast_forward { FAST_FORWARD_TICKS } else { 1 };
        for _ in 0..ticks_this_frame {
            systems::animator::animator_system(world, tick_ms);
        }
        systems::compositor::compositor_system(world);
        systems::renderer::renderer_system(world);

        let elapsed = frame_start.elapsed();
        if elapsed < frame_budget {
            std::thread::sleep(frame_budget - elapsed);
        }
    }
    Ok(())
}

#[inline]
fn is_debug_fast_forward_toggle(code: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> bool {
    cfg!(debug_assertions)
        && modifiers.contains(crossterm::event::KeyModifiers::ALT)
        && matches!(code, crossterm::event::KeyCode::Char('f') | crossterm::event::KeyCode::Char('F'))
}

#[cfg(test)]
mod tests {
    use super::is_debug_fast_forward_toggle;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn detects_alt_f_combo_for_debug_fast_forward() {
        let expected = cfg!(debug_assertions);
        assert_eq!(is_debug_fast_forward_toggle(KeyCode::Char('f'), KeyModifiers::ALT), expected);
        assert_eq!(is_debug_fast_forward_toggle(KeyCode::Char('F'), KeyModifiers::ALT), expected);
    }

    #[test]
    fn ignores_non_alt_or_non_f_keys() {
        assert!(!is_debug_fast_forward_toggle(KeyCode::Char('f'), KeyModifiers::NONE));
        assert!(!is_debug_fast_forward_toggle(KeyCode::Char('g'), KeyModifiers::ALT));
        assert!(!is_debug_fast_forward_toggle(KeyCode::Esc, KeyModifiers::ALT));
    }
}

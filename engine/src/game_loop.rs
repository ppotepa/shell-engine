use crate::error::EngineError;
use crate::events::EngineEvent;
use crate::services::EngineWorldAccess;
use crate::systems;
use crate::world::World;

pub fn game_loop(world: &mut World, target_fps: u16) -> Result<(), EngineError> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use std::time::{Duration, Instant};
    use systems::scene_lifecycle::SceneLifecycleManager;

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
                    world.events_mut().unwrap().push(ev);
                }
                Event::Resize(w, h) => {
                    world
                        .events_mut()
                        .unwrap()
                        .push(EngineEvent::TerminalResized {
                            width: w,
                            height: h,
                        });
                }
                _ => {}
            }
        }

        world.events_mut().unwrap().push(EngineEvent::Tick);

        // --- DRAIN EVENTS ---
        let events = world.events_mut().unwrap().drain();

        let quit = SceneLifecycleManager::process_events(world, events);
        if quit {
            break;
        }

        // --- SYSTEMS ---
        let ticks_this_frame = if debug_fast_forward {
            FAST_FORWARD_TICKS
        } else {
            1
        };
        for _ in 0..ticks_this_frame {
            systems::animator::animator_system(world, tick_ms);
        }
        systems::behavior::behavior_system(world);
        systems::audio::audio_system(world);
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
fn is_debug_fast_forward_toggle(
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> bool {
    cfg!(debug_assertions)
        && modifiers.contains(crossterm::event::KeyModifiers::ALT)
        && matches!(
            code,
            crossterm::event::KeyCode::Char('f') | crossterm::event::KeyCode::Char('F')
        )
}

#[cfg(test)]
mod tests {
    use super::is_debug_fast_forward_toggle;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn detects_alt_f_combo_for_debug_fast_forward() {
        let expected = cfg!(debug_assertions);
        assert_eq!(
            is_debug_fast_forward_toggle(KeyCode::Char('f'), KeyModifiers::ALT),
            expected
        );
        assert_eq!(
            is_debug_fast_forward_toggle(KeyCode::Char('F'), KeyModifiers::ALT),
            expected
        );
    }

    #[test]
    fn ignores_non_alt_or_non_f_keys() {
        assert!(!is_debug_fast_forward_toggle(
            KeyCode::Char('f'),
            KeyModifiers::NONE
        ));
        assert!(!is_debug_fast_forward_toggle(
            KeyCode::Char('g'),
            KeyModifiers::ALT
        ));
        assert!(!is_debug_fast_forward_toggle(
            KeyCode::Esc,
            KeyModifiers::ALT
        ));
    }
}

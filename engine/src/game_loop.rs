//! Fixed-timestep game loop: polls input, ticks systems, and paces frames to the target FPS.

use crate::error::EngineError;
use crate::events::EngineEvent;
use crate::services::EngineWorldAccess;
use crate::systems;
use crate::world::World;

/// Runs the engine game loop for `world` at `target_fps` until the player quits.
pub fn game_loop(world: &mut World, target_fps: u16) -> Result<(), EngineError> {
    use crossterm::event::{self, Event, KeyEventKind, MouseEventKind};
    use std::time::{Duration, Instant};
    use systems::scene_lifecycle::SceneLifecycleManager;

    const FAST_FORWARD_TICKS: u8 = 8;
    let mut debug_fast_forward = false;

    let _mouse_capture_guard = MouseCaptureGuard::new(should_capture_mouse(world))?;

    loop {
        let frame_start = Instant::now();
        let scene_target_fps = world
            .scene_runtime()
            .and_then(|runtime| runtime.scene().target_fps);
        let fps = resolve_target_fps(target_fps, scene_target_fps) as u64;
        let tick_ms = (1000 / fps).max(1);
        let frame_budget = Duration::from_millis(tick_ms);

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
                        code if is_quit_key(code, key.modifiers) => EngineEvent::Quit,
                        _ => EngineEvent::KeyPressed(key),
                    };
                    world.events_mut().unwrap().push(ev);
                }
                Event::Mouse(mouse) => {
                    if matches!(mouse.kind, MouseEventKind::Moved) {
                        world.events_mut().unwrap().push(EngineEvent::MouseMoved {
                            column: mouse.column,
                            row: mouse.row,
                        });
                    }
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
        // Bridge external sidecar IO before behaviors run (behaviors clear UI submit/change each frame).
        systems::engine_io::engine_io_system(world, tick_ms);
        // Process transitions emitted by animator in the same frame to avoid
        // rendering one extra "done" frame that can briefly re-show sprites.
        let post_animator_events = world.events_mut().unwrap().drain();
        let quit_after_animator =
            SceneLifecycleManager::process_events(world, post_animator_events);
        if quit_after_animator {
            break;
        }
        systems::behavior::behavior_system(world);
        systems::audio::audio_system(world);
        systems::compositor::compositor_system(world);
        systems::postfx::postfx_system(world);
        systems::renderer::renderer_system(world);

        let elapsed = frame_start.elapsed();
        if elapsed < frame_budget {
            std::thread::sleep(frame_budget - elapsed);
        }
    }
    Ok(())
}

struct MouseCaptureGuard {
    enabled: bool,
}

impl MouseCaptureGuard {
    fn new(enabled: bool) -> std::io::Result<Self> {
        if enabled {
            crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;
        }
        Ok(Self { enabled })
    }
}

impl Drop for MouseCaptureGuard {
    fn drop(&mut self) {
        if self.enabled {
            let _ = crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
        }
    }
}

#[inline]
fn should_capture_mouse(world: &World) -> bool {
    world
        .get::<crate::debug_features::DebugFeatures>()
        .map(|debug| !debug.enabled)
        .unwrap_or(true)
}

#[inline]
fn is_quit_key(code: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> bool {
    modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
        && matches!(
            code,
            crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C')
        )
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

#[inline]
fn resolve_target_fps(default_target_fps: u16, scene_target_fps: Option<u16>) -> u16 {
    scene_target_fps
        .unwrap_or(default_target_fps)
        .clamp(1, crate::terminal_caps::MAX_TARGET_FPS)
}

#[cfg(test)]
mod tests {
    use super::{
        is_debug_fast_forward_toggle, is_quit_key, resolve_target_fps, should_capture_mouse,
    };
    use crate::debug_features::DebugFeatures;
    use crate::world::World;
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

    #[test]
    fn quit_key_includes_ctrl_c() {
        assert!(is_quit_key(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(is_quit_key(
            KeyCode::Char('C'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        ));
        assert!(!is_quit_key(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!is_quit_key(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(!is_quit_key(KeyCode::Char('c'), KeyModifiers::NONE));
    }

    #[test]
    fn scene_fps_override_has_priority() {
        assert_eq!(resolve_target_fps(60, Some(30)), 30);
    }

    #[test]
    fn fps_is_clamped_to_supported_range() {
        assert_eq!(resolve_target_fps(0, None), 1);
        assert_eq!(
            resolve_target_fps(crate::terminal_caps::MAX_TARGET_FPS + 100, None),
            crate::terminal_caps::MAX_TARGET_FPS
        );
        assert_eq!(
            resolve_target_fps(60, Some(crate::terminal_caps::MAX_TARGET_FPS + 1)),
            crate::terminal_caps::MAX_TARGET_FPS
        );
    }

    #[test]
    fn mouse_capture_is_disabled_when_debug_features_are_enabled() {
        let mut world = World::new();
        assert!(should_capture_mouse(&world));

        world.register(DebugFeatures {
            enabled: false,
            overlay_visible: false,
            overlay_mode: Default::default(),
        });
        assert!(should_capture_mouse(&world));

        let mut world_debug = World::new();
        world_debug.register(DebugFeatures {
            enabled: true,
            overlay_visible: true,
            overlay_mode: Default::default(),
        });
        assert!(!should_capture_mouse(&world_debug));
    }
}

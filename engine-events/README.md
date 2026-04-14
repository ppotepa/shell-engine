# engine-events

Shared engine event types and the per-frame event queue.

## Purpose

`engine-events` defines the typed events passed between input handling, the game
loop, audio triggers, scene lifecycle systems, and other runtime subsystems.

Keeping these types in a small crate lets multiple systems share the same event
contract without pulling in larger engine modules.

## Key types

- `EngineEvent` — runtime events such as:
  - `Tick`
  - `KeyDown { key: KeyEvent, repeat: bool }` — key pressed (SDL2 key-down)
  - `KeyUp { key: KeyEvent }` — key released (SDL2 key-up)
  - `MouseMoved { x: f32, y: f32 }` — cursor moved (output-space coords)
  - `MouseButtonDown { button: MouseButton, x: f32, y: f32 }`
  - `MouseButtonUp { button: MouseButton, x: f32, y: f32 }`
  - `SceneLoaded`
  - `SceneTransition`
  - `AudioCue`
  - `OutputResized`
  - `Quit`
- `InputEvent` — input-only sub-enum; use this in systems that only care about
  keyboard and mouse events (GUI, camera, scripting). Produced by
  `EngineEvent::as_input_event()`.
  - `KeyDown { key, repeat }`
  - `KeyUp { key }`
  - `MouseMoved { x: f32, y: f32 }`
  - `MouseDown { button: MouseButton, x: f32, y: f32 }`
  - `MouseUp { button: MouseButton, x: f32, y: f32 }`
  - `FocusLost`
- `MouseButton` — typed enum `{ Left, Right, Middle }` (replaces stringly-typed
  button strings)
- `EventQueue` — frame-local queue with `push`, `drain`, and `is_empty`

## Fan-out pattern

`engine/src/systems/scene_lifecycle/mod.rs` calls `EngineEvent::as_input_event()`
on every event during `classify_events` and collects the results into
`LifecycleEvents::input_events: Vec<InputEvent>`. This is then passed as a
unified slice to `GuiSystem::update` and future input consumers without having to
re-classify the same events per subsystem.

## Working with this crate

- prefer extending `EngineEvent` here instead of creating ad-hoc side channels,
- keep variants high-level and engine-facing rather than backend-specific,
- when adding new input event kinds, also add the corresponding `InputEvent`
  variant and update `as_input_event()`,
- when adding new event kinds, update all producers and consumers in the same change.

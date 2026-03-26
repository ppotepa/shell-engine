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
  - `KeyPressed`
  - `MouseMoved`
  - `SceneLoaded`
  - `SceneTransition`
  - `AudioCue`
  - `TerminalResized`
  - `Quit`
- `EventQueue` — frame-local queue with `push`, `drain`, and `is_empty`

## Working with this crate

- prefer extending `EngineEvent` here instead of creating ad-hoc side channels,
- keep variants high-level and engine-facing rather than backend-specific,
- when adding new event kinds, update all producers and consumers in the same change.

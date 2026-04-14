# engine-gui

Declarative GUI widget model, hit-testing, and runtime state for in-engine UI.

## Purpose

`engine-gui` is the domain layer for all in-engine GUI (HUD overlays, settings
panels, menus). It is intentionally stateless in its system layer — all mutable
state lives in `GuiRuntimeState` so callers decide lifetime and ownership.

**Responsibilities:**

- Define widget types (`GuiWidgetDef`): `Slider`, `Button`, `Toggle`, `Panel`.
- Track per-widget runtime state (`GuiRuntimeState`, `GuiWidgetState`).
- Process `engine_events::InputEvent` slices and update hit-test state, hover,
  pressed, clicked, and value fields (`GuiSystem`).

**Non-responsibilities:**

- Rendering — handled by `engine-compositor` via `Panel`/`Vector`/`Text` sprites.
- Rhai scripting — handled by `engine-behavior`'s `ScriptGuiApi`.
- Layout resolution — handled by Taffy inside `engine-compositor`.

## Key types

| Type | Location | Purpose |
|------|----------|---------|
| `GuiWidgetDef` | `widget.rs` | Authored widget definitions (Slider, Button, Toggle, Panel) |
| `GuiRuntimeState` | `state.rs` | Mutable per-frame widget state (values, hover, pressed, clicked) |
| `GuiWidgetState` | `state.rs` | Per-widget flag/value bag |
| `GuiSystem` | `system.rs` | Stateless update system — call `update(widgets, state, events)` once per frame |

## Input contract

`GuiSystem::update` accepts `&[engine_events::InputEvent]`. The caller (scene
lifecycle) passes the unified input event slice produced by
`EngineEvent::as_input_event()` — no separate mouse/keyboard split needed.

- Mouse events update hover, drag, pressed, clicked, and slider values.
- Keyboard events are accepted but currently passed through (stub for future
  focused-widget keyboard input).
- Mouse coordinates are `f32` (output-space pixels, matching SDL2 output coords).
- `MouseButton` is the typed `engine_events::MouseButton` enum (`Left`, `Right`,
  `Middle`) — no stringly-typed button strings.

## Deprecated surface

`GuiInputEvent` (`events.rs`) is kept as a `#[deprecated]` type for backward
compatibility. New code must use `engine_events::InputEvent` directly.

## Integration with scenes

Scenes declare a `gui:` block in YAML listing widget definitions. The scene
runtime in `engine-scene-runtime` owns a `GuiRuntimeState` instance and calls
`GuiSystem::update` each frame via `lifecycle_controls::update_gui`. Rhai scripts
read widget state through `ScriptGuiApi` (`gui.slider_value("id")`,
`gui.button_clicked("id")`, `gui.mouse_x/y`, etc.).

## Adding a widget type

1. Add a variant to `GuiWidgetDef` (`widget.rs`).
2. Implement `id()`, `bounds()`, and `initial_value()` for the variant.
3. Handle it in `GuiSystem::update` (`system.rs`).
4. Add YAML authoring support in `engine-authoring` and update the schema.
5. Add `ScriptGuiApi` accessor in `engine-behavior/src/scripting/gui.rs` if
   needed by scripts.

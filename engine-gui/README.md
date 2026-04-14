# engine-gui

Declarative GUI widget model, hit-testing, and runtime state for in-engine UI.

## Purpose

`engine-gui` is the domain layer for all in-engine GUI (HUD overlays, settings
panels, menus). It is intentionally stateless in its system layer — all mutable
state lives in `GuiRuntimeState` so callers decide lifetime and ownership.

**Responsibilities:**

- Define the `GuiControl` trait and concrete control types (`SliderControl`,
  `ButtonControl`, `ToggleControl`, `PanelControl`).
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
| `GuiControl` | `control.rs` | Trait for polymorphic widget input handling (`on_mouse_down`, `on_drag`, `on_mouse_up`, `visual_sync`) |
| `SliderControl` | `control.rs` | Slider: drag-to-value, handle sprite positioning |
| `ButtonControl` | `control.rs` | Button: click detection |
| `ToggleControl` | `control.rs` | Toggle: on/off state flip |
| `PanelControl` | `control.rs` | Panel: passive container, hover only |
| `WidgetRect` | `control.rs` | Hit-test bounds helper |
| `VisualSync` | `control.rs` | Engine-level sprite positioning data (sprite alias + offset_x) |
| `GuiRuntimeState` | `state.rs` | Mutable per-frame widget state (values, hover, pressed, clicked) |
| `GuiWidgetState` | `state.rs` | Per-widget flag/value bag |
| `GuiSystem` | `system.rs` | Stateless update system — call `update(widgets, state, events)` once per frame |
| `GuiWidgetDef` | `widget.rs` | Legacy authored widget enum (kept for backward compat) |

## GuiControl trait

Each widget type implements `GuiControl`:

```rust
pub trait GuiControl: Send + Sync {
    fn id(&self) -> &str;
    fn bounds(&self) -> WidgetRect;
    fn initial_value(&self) -> f64;
    fn on_mouse_down(&self, x: f32, y: f32, state: &mut GuiWidgetState) -> bool;
    fn on_drag(&self, x: f32, y: f32, state: &mut GuiWidgetState) -> bool;
    fn on_mouse_up(&self, x: f32, y: f32, state: &mut GuiWidgetState) -> bool;
    fn visual_sync(&self, state: &GuiWidgetState) -> Option<VisualSync>;
}
```

`GuiSystem::update` dispatches generically through `&[Box<dyn GuiControl>]` —
no match-on-variant branching. Adding a new widget type only requires
implementing the trait.

`visual_sync()` returns positioning data that the engine applies to scene sprites
automatically (e.g., slider handle `offset_x`). This replaces manual Rhai-side
`scene.set("handle", "position.x", ...)` calls.

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

`GuiWidgetDef` (`widget.rs`) is the legacy widget enum. The runtime now uses
`Box<dyn GuiControl>` trait objects; `GuiWidgetDef` is kept for backward
compatibility during migration.

## Integration with scenes

Scenes declare a `gui:` block in YAML listing widget definitions. The scene
runtime in `engine-scene-runtime` converts each `SceneGuiWidgetDef` into a
`Box<dyn GuiControl>` at construction time. `GuiSystem::update` is called each
frame via `lifecycle_controls::update_gui`. After behaviors run, the engine calls
`sync_widget_visuals()` which reads `visual_sync()` from each widget and applies
sprite offsets (e.g., slider handle positioning) automatically.

Rhai scripts read widget state through `ScriptGuiApi`:
- `gui.slider_value(id)`, `gui.toggle_on(id)`, `gui.button_clicked(id)`
- `gui.has_change()`, `gui.changed_widget()`
- `gui.widget_hovered(id)`, `gui.widget_pressed(id)`
- `gui.set_widget_value(id, val)` — programmatic value change
- `gui.set_panel_visible(id, bool)` — panel visibility
- `gui.mouse_x`, `gui.mouse_y`, `gui.mouse_left_down`

## Adding a widget type

1. Create a new struct implementing `GuiControl` in `control.rs`.
2. Add the conversion in `engine-scene-runtime/src/construction.rs`
   (`scene_gui_widget_to_control`).
3. Add YAML authoring support in `engine-core/src/scene/model.rs`
   (`SceneGuiWidgetDef`) and `engine-authoring` compile path.
4. Update schema generation.
5. Add `ScriptGuiApi` accessor in `engine-behavior/src/scripting/gui.rs` if
   needed by scripts.
6. If the widget needs engine-level sprite positioning, implement `visual_sync()`
   to return `Some(VisualSync { ... })`.

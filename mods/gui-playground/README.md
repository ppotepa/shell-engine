# GUI Playground Mod

Interactive test-bench for the engine GUI widget system, now styled as a compact tool panel instead of a raw debug scene.

## Running

```bash
SHELL_ENGINE_MOD_SOURCE=mods/gui-playground cargo run -p app
```

Or on Windows:

```powershell
$env:SHELL_ENGINE_MOD_SOURCE="mods/gui-playground"; cargo run -p app
```

## What it exercises

| Widget | Interaction | What you can verify |
|--------|-------------|---------------------|
| `slider` | Drag three RGB channels | Handle sync, fill-bar growth, value labels, live swatch updates |
| `toggle` | Click hex / swatch / invert flags | Boolean state, text restyling, visibility changes |
| `button` | Click reset / random | Programmatic widget updates and event logging |
| `text-input` | Type a label and press `Enter` | Keyboard focus, mirrored text, submit detection |
| `number-input` | Type density and press `Enter` | Numeric parsing, min/max clamp, step snap, submit detection |
| `dropdown` | Open compact mode picker and choose an item | Popup state, trigger label sync, current-option highlighting |
| `panel` | Scene cards and input shells | Reusable container visuals for tool-style HUDs |

## Scene structure

The scene is split into two authored UI columns:

- Left column:
  - `COLOR MIXER`
  - `TOGGLES + ACTIONS`
  - `INPUTS + SELECT`
- Right column:
  - `MIX RESULT`
  - `STATE`
  - `EVENT LOG`

This is intentionally closer to a real editor/tool HUD than the earlier “list of controls on screen” approach.

## Visual behaviors

- RGB sliders tint a five-row swatch and update `HEX` / `RGB` readouts.
- Toggle labels restyle themselves to show on/off state.
- The dropdown uses:
  - a boxed trigger
  - a live label
  - a chevron-like arrow (`v` / `^`)
  - a popup list with active-option marker (`>`)
- Input fields are rendered as authored boxed shells with text mirrored by GUI runtime.
- The right column mirrors:
  - hovered widget
  - pressed widget
  - mouse position
  - last changed widget
  - current text/number/dropdown state
- Event log captures slider changes, toggles, button clicks, and submitted input values.

## Architecture

The playground keeps the intended engine split:

1. `scene.yml`
   Defines logical GUI widgets in `gui.widgets`.
2. `layers/*.yml`
   Define authored visuals only: cards, labels, slider tracks, popup shells, readouts.
3. `main.rhai`
   Reads `gui.*` state and applies high-level visual updates via live scene
   handles (`scene.object(...).set(...)` / `runtime.scene.objects.find(...).set(...)`).
4. Engine GUI runtime
   Owns hit-testing, focus, widget state, dropdown open/close, and text/number entry semantics.

The renderer is not widget-aware in a mod-specific way. The mod just binds authored sprites to generic engine GUI controls.

## Rhai API exercised

- `gui.slider_value(id)`
- `gui.button_clicked(id)`
- `gui.toggle_on(id)`
- `gui.selected_index(id)`
- `gui.widget_open(id)`
- `gui.widget_hovered(id)`
- `gui.widget_pressed(id)`
- `gui.text(id)`
- `gui.submitted(id)`
- `gui.number_value(id)`
- `gui.set_widget_value(id, value)`
- `gui.mouse_x`
- `gui.mouse_y`
- `gui.mouse_left_down`

And scene-side mutation examples:

- `scene.object(id).set("text.content", ...)`
- `scene.object(id).set("text.fg", ...)`
- `scene.object(id).set("vector.points", ...)`
- `scene.object(id).set("visible", ...)`

## Why this mod exists

This mod is the practical validation surface for GUI v1:

- base controls are real engine widgets, not fake sprite-only affordances
- dropdown/text/number inputs are exercised in a real scene
- the authored layout demonstrates how to build a reusable tool panel UI with current engine primitives

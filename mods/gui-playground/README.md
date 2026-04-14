# GUI Playground Mod

Interactive test-bench for the engine GUI widget system.

## Running

```bash
SHELL_QUEST_MOD_SOURCE=mods/gui-playground cargo run -p app
```

Or on Windows:
```powershell
$env:SHELL_QUEST_MOD_SOURCE="mods/gui-playground"; cargo run -p app
```

## What it tests

| Widget | Controls | Feedback |
|--------|----------|----------|
| **Slider** (×3) | Drag R/G/B sliders (0–255) | Handle moves, value text updates, color swatch reacts |
| **Toggle** (×3) | Click Show Hex / Show Swatch / Invert Colors | Checkbox indicator, panel visibility, color inversion |
| **Button** (×2) | Click Reset All / Randomize | Sliders reset via `gui.set_widget_value()`, click counter |
| **Panel** | Toggles control swatch & hex visibility | Panel show/hide |

## Layout

```
┌─ INPUTS ──────────────┬─ OUTPUTS ─────────────────────┐
│ GUI PLAYGROUND        │ ─── OUTPUT ──────────────────  │
│ ─── SLIDERS ────────  │ ┌────────────┐ HEX: #8080C8   │
│ R ──────●──────  128  │ │  swatch    │ RGB: 128,128,200│
│ G ──────●──────  128  │ └────────────┘                 │
│ B ──────●──────  128  │ ─── STATE ───────────────────  │
│ ─── TOGGLES ────────  │ Hover:   slider-r              │
│ [✓] Show Hex          │ Pressed: ---                   │
│ [✓] Show Swatch       │ Mouse:   342, 186              │
│ [ ] Invert Colors     │ Changed: slider-r              │
│ ─── BUTTONS ────────  │ LMB:     up                    │
│ [RESET ALL] [RANDOMIZE│ ─── EVENT LOG ───────────────  │
│ Clicks: 3  Last: reset│ slider R -> 128                │
│                       │ toggle hex -> ON               │
│                       │ btn RESET clicked              │
└───────────────────────┴────────────────────────────────┘
```

## Rhai API exercised

- `gui.slider_value(id)` — read slider
- `gui.button_clicked(id)` — detect click
- `gui.toggle_on(id)` — read toggle state
- `gui.has_change()` / `gui.changed_widget()` — change tracking
- `gui.widget_hovered(id)` / `gui.widget_pressed(id)` — hover/press state
- `gui.set_widget_value(id, val)` — programmatic value set (**new**)
- `gui.set_panel_visible(id, bool)` — panel visibility (via toggle)
- `gui.mouse_x` / `gui.mouse_y` / `gui.mouse_left_down` — mouse state
- `scene.set(id, "text.content", ...)` — dynamic text
- `scene.set(id, "text.fg", ...)` — dynamic color
- `scene.set(id, "style.bg", ...)` — dynamic panel bg
- `scene.set(id, "position.x", ...)` — slider handle movement
- `scene.set(id, "visible", ...)` — sprite visibility

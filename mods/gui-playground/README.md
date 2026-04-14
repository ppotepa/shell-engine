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
| **Slider** (×3) | Drag R/G/B sliders (0–255) | Handle moves (engine-level), fill track grows, value text updates, color swatch reacts |
| **Toggle** (×3) | Click Show Hex / Show Swatch / Invert Colors | Checkbox indicator, panel visibility, color inversion |
| **Button** (×2) | Click Reset All / Randomize | Sliders reset via `gui.set_widget_value()`, persistent click counter |
| **Panel** | Toggles control swatch & hex visibility | Panel show/hide |

## Visual features

- **Fill track bars** — colored vector sprites that resize dynamically via `vector.points` each frame
- **Channel-tinted handles** — handles glow R/G/B on hover/press, gray when idle
- **Dynamic fill intensity** — fill bar color ramps from dim to full brightness proportionally
- **5-row color swatch** — enlarged block-char preview with the mixed RGB color
- **Hex readout tinted** with the active mixed color
- **Change detection** — Rhai skips redundant `scene.set` calls using `local` prev values
- **Persistent state** — click count, event log, and previous values survive across frames via `local`

## Layout

```
┌─ INPUTS ──────────────┬─ OUTPUTS ─────────────────────┐
│ GUI PLAYGROUND        │ ─── OUTPUT ──────────────────  │
│ ─── SLIDERS ────────  │ ┌────────────┐ HEX: #8080C8   │
│ R ▓▓▓▓▓▓●──────  128  │ │  swatch    │ RGB: 128,128,200│
│ G ▓▓▓▓▓▓●──────  128  │ │  (5 rows)  │                 │
│ B ▓▓▓▓▓▓●──────  128  │ └────────────┘                 │
│ ─── TOGGLES ────────  │ ─── STATE ───────────────────  │
│ [✓] Show Hex          │ Hover:   slider-r              │
│ [✓] Show Swatch       │ Pressed: ---                   │
│ [ ] Invert Colors     │ Mouse:   342, 186              │
│ ─── BUTTONS ────────  │ Changed: slider-r              │
│ [RESET ALL] [RANDOMIZE│ LMB:     up                    │
│ Clicks: 3  Last: reset│ ─── EVENT LOG ───────────────  │
│                       │ slider R -> 128                │
│                       │ toggle hex -> ON               │
│                       │ btn RESET clicked              │
└───────────────────────┴────────────────────────────────┘
```

## Architecture

The playground uses the **two-layer** scene model:

1. **`scene.yml`** — `gui:` block declares logical widgets (sliders with `handle` refs, toggles, buttons)
2. **Layer YAMLs** — visual sprites: tracks, handles, fill bars, labels, swatch, readouts
3. **`main.rhai`** — behavior script reads `gui.*` APIs, updates text/color/visibility
4. **Engine-level sync** — `GuiControl::visual_sync()` positions slider handles automatically

## Rhai API exercised

- `gui.slider_value(id)` — read slider
- `gui.button_clicked(id)` — detect click
- `gui.toggle_on(id)` — read toggle state
- `gui.has_change()` / `gui.changed_widget()` — change tracking
- `gui.widget_hovered(id)` / `gui.widget_pressed(id)` — hover/press state
- `gui.set_widget_value(id, val)` — programmatic value set
- `gui.set_panel_visible(id, bool)` — panel visibility (via toggle)
- `gui.mouse_x` / `gui.mouse_y` / `gui.mouse_left_down` — mouse state
- `scene.set(id, "text.content", ...)` — dynamic text
- `scene.set(id, "text.fg", ...)` — dynamic color
- `scene.set(id, "vector.points", [[x1,y1],[x2,y2]])` — dynamic fill bars
- `scene.set(id, "visible", ...)` — sprite visibility

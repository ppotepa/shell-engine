# Main Mod — Shell Quest Content

## Scene Map

All scenes in shell-quest (main mod):

| ID | Title | Duration | Purpose | Key Effects |
|----|-------|----------|---------|------------|
| `00.intro.logo` | Intro Logo | 7000ms | Title card | CRT-on, shine, flash/whiteout |
| `01.intro.date` | Intro Date | 9780ms | Date card | Scanlines fade |
| `02.intro.boot` | Intro Boot | 12290ms | Linus boot animation | Fade, scanlines, fade-out |
| `03.intro.lab-enter` | Lab Enter | 3160ms | Lab fade in/out | Fade in/out, pause |
| `04.intro.difficulty-select` | Difficulty Select | 5000ms | Menu + portraits | 4× PostFX, neon-edge-glow |
| `05.intro.cpu-on` | CPU On | ~8000ms | CPU boot sequence | CRT effects, terminal output |
| `06.prologue.` | Prologue (series) | various | Main game prologue | Dynamic behaviors, Rhai |

## Content Structure

```
mods/shell-quest/
├── mod.yaml                         Mod manifest
├── scenes/
│   ├── 00-intro-logo/
│   │   ├── scene.yml
│   │   └── layers/main.yml
│   ├── 01-intro-date/
│   │   ├── scene.yml
│   │   └── layers/main.yml
│   └── ... (4 more scenes)
├── objects/
│   ├── difficulty-menu.yml
│   ├── cpu.yml
│   └── ... (other game objects)
├── behaviors/
│   ├── menu-select.yml
│   ├── hint-system.yml
│   └── ... (custom behaviors)
├── effects/
│   ├── crt-custom.yml
│   └── ... (mod-specific effects)
├── assets/
│   ├── 3d/ (OBJ files, portraits)
│   ├── sounds/ (WAV, MP3, OGG)
│   ├── gifs/ (animated sprites)
│   └── sprites/ (PNG sprites)
└── docs/
    ├── features.md (feature list)
    ├── scripts.md (Rhai API reference)
    └── ... (mod documentation)
```

## Custom Behaviors

**menu-highlight** — Dynamic menu item highlighting
- Attached to menu text sprites
- Sets `style.fg` based on selection state
- Selected: bright color (#88ccff)
- Unselected: dim color (#1a2244)

**hint-system** — Contextual hints during gameplay
- Triggered by scene events
- Displays temporary text overlay
- Fades out after duration

## Custom Effects

**neon-edge-glow** — Cyberpunk aesthetic
- 3-ring volumetric spillover
- Breathing pulse (progress × speed)
- Parameters: color, intensity, alpha, speed, easing
- Applied to difficulty menu 3D portraits

## Asset Organization

### 3D Assets (assets/3d/)
- `dvorak-2.obj` — Full mesh (~1.4M faces)
- `dvorak-2-portrait.obj` — Portrait version
- `dvorak-2-wire.obj` — Wireframe derivative
- Other character portraits (simplified OBJ files)

**Rendering**: Engine caps OBJ at 50k faces. Larger models fall back to wireframe.

### Sounds (assets/sounds/)
- `daisy-short.wav` — Splash screen audio (~5s)
- `boot-sequence.wav` — Boot animation audio
- `menu-select.wav` — Menu interaction SFX
- Format: WAV, MP3, OGG supported

### Animated GIFs (assets/gifs/)
- Boot sequence animations
- Loading spinners
- Transition effects

### Static Sprites (assets/sprites/)
- UI elements (buttons, panels)
- Text overlays
- Background images

## Key Scenes Deep-Dive

### Scene 04: Difficulty Select

Heaviest scene (4 PostFX passes):
1. CRT underlay
2. CRT distortion
3. CRT scan glitch
4. CRT ruby (color) effect

Features:
- 3D portrait models (snap lighting, neon glow)
- Menu with dynamic highlighting
- On Enter: portrait rotates (yaw 0→320°, 500ms ease-out)
- On Leave: intense neon flash + fade-out

Benchmark: stresses PostFX and OBJ rendering.

### Scene 00: Logo

Splash screen:
- Title card with shine effect
- Flash transition (white + fade-out)
- Optional whiteout glow
- Duration: 7000ms (compressed from original 8.5s)

## Sidecar Integration

Scene `05.intro.cpu-on` interacts with C# sidecar:
- Emulates Linus Torvalds boot sequence
- Displays simulated terminal output
- Responsive to player input (affects prologue difficulty)

Communication via `engine-io` IPC bridge.

## Visual Style

- **Colors**: Cyberpunk neon palette (#88ccff, #ff00ff, etc.)
- **Effects**: Heavy CRT, scanlines, glitch for retro-terminal aesthetic
- **Typography**: Monospace font (supports Unicode box-drawing)
- **Resolution**: Optimized for 379×109 (halfblock-based, fits most terminals)

## Extending the Mod

To add new scenes:
1. Create `scenes/<name>/scene.yml`
2. Define stages, layers, sprites
3. Validate via `cargo run -p schema-gen -- --check`
4. Reference in `scenes/<name>/layers/main.yml` (import shared assets)
5. Link in prologue scene transitions

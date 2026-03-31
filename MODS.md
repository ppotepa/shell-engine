# Shell Quest Mod System

## Overview

Mods are self-contained content packages loaded by the engine at startup.
A mod can be an unpacked directory or a `.zip` archive. The engine selects
which mod to load via:

- `--mod-source` CLI flag, or
- `SHELL_QUEST_MOD_SOURCE` environment variable.

If neither is set, the default mod (`mods/shell-quest`) is used.

## Shell Quest (Main Mod)

The primary game content. Contains all intro sequences, menus, gameplay
scenes, assets, and the C# sidecar for the simulated CognitOS terminal.

### Structure

```
mods/shell-quest/
+-- mod.yaml
+-- assets/
|   +-- images/
|   +-- fonts/
|   +-- 3d/
|   +-- audio/
|   +-- linus/
|   +-- raw/                  gitignored staging area
+-- objects/
+-- behaviors/
+-- scenes/
|   +-- 00-intro-logo/
|   +-- 01-intro-date/
|   +-- 02-intro-boot/
|   +-- 03-intro-lab-enter/
|   +-- 04-difficulty-select/
|   +-- 05-intro-cpu-on/
|   +-- 06-intro-login/
|   +-- 3d/
|   +-- mainmenu/
+-- os/cognitOS/              C# sidecar (simulated MinixOS)
+-- schemas/
+-- docs/
```

### Scene Flow

| Scene | ID                       | Effects                  | Notes                  |
|-------|--------------------------|--------------------------|------------------------|
| 00    | 00.intro.logo            | CRT-on, shine, flash     | Splash animation       |
| 01    | 01.intro.date            | Scanlines                | Static date display    |
| 02    | 02.intro.boot            | Fade-in, scanlines       | BIOS boot sequence     |
| 03    | 03.intro.lab-enter       | Fade-in/out              | Environment setup      |
| 04    | 04.difficulty-select     | 4x PostFX, 3D portraits | Menu with OBJ renders  |
| 05    | 05.intro.cpu-on          | Fade-in                  | CPU power-on sequence  |
| 06    | 06.intro.login           | Terminal-shell scripted  | Dual-prompt pattern    |

### Special Features

- Prerender pass for 3D scenes (OBJ model rasterization).
- Scripted terminal-shell with dual-prompt input pattern.
- IPC bridge to C# sidecar (`os/cognitOS/`) for CognitOS simulation.

## Shell Quest Tests (Benchmark Mod)

Automated testing variant of the main mod. All user-input triggers are
replaced with timeouts so scenes advance without interaction.

Assets, behaviors, objects, and schemas are symlinked back to `mods/shell-quest/`.

### Running

```bash
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10
```

### Timeline Per Loop

| Scene | Duration | Trigger        | Compression |
|-------|----------|----------------|-------------|
| 00    | ~1680ms  | timeout 600ms  | 4.2x        |
| 01    | ~1900ms  | timeout 400ms  | 5.1x        |
| 02    | ~2180ms  | timeout 200ms  | 5.6x        |
| 03    | ~1120ms  | timeout 200ms  | 2.8x        |
| 04    | ~2550ms  | timeout 2000ms | 2.0x        |
| Total | ~9430ms  |                | 3.9x        |

Scene 04 loops back to 00 for continuous benchmarking.

## Playground (Dev Mod)

Development sandbox with reference scenes for isolated feature testing.
Contains scenes for terminal-shell, 3d-scene, terminal-size-test,
rhai-lab, rhai-time, and many more.

### Running

```bash
SHELL_QUEST_MOD_SOURCE=mods/playground cargo run -p app
```

Navigation: Esc returns to the playground menu (does not quit the app).
Use Ctrl+C for hard quit.

## Creating a Custom Mod

### Minimum Structure

```
mods/my-mod/
+-- mod.yaml
+-- scenes/
    +-- hello/
        +-- scene.yml
        +-- layers/
            +-- main.yml
```

### mod.yaml

```yaml
name: my-mod
version: "0.1.0"
description: "My custom mod"
entrypoint: /scenes/hello/scene.yml
terminal:
  min_colours: 256
  min_width: 120
  min_height: 30
  render_size: 120x30
  presentation_policy: stretch
```

Use `render_size` for the authored in-memory canvas and `presentation_policy`
for how that canvas is shown on the real terminal/window:

- `stretch` fills the available output area,
- `fit` preserves aspect ratio with letterboxing,
- `strict` keeps 1:1 cells and centers/crops if needed.

### Running

```bash
cargo run -p app -- --mod-source=mods/my-mod
```

## Mod Asset Loading

- Paths use a leading `/`, resolved relative to the mod root.
- The same paths work for both directory and zip-packaged mods.
- `assets/raw/` is a gitignored staging area for work-in-progress assets.
- Named mod behaviors are loaded from top-level `behaviors/*.yml`; those YAML wrappers may point at external Rhai via `src`.

### Path Resolution Example

```
/assets/images/logo.png  -->  mods/my-mod/assets/images/logo.png  (directory)
/assets/images/logo.png  -->  my-mod.zip!/assets/images/logo.png  (zip archive)
```

# Shell Quest - Inspired by One Lone Coder https://www.youtube.com/javidx9

status : 22-03-2026

plot : started doing some plot/script related work, digging some internet, trying to make it as immersive as possible by looking up as many historical details
eastereggs: ;)

**optimizations** : rendering optimizations, there are no regressions that are related to 3d drawing itself, currently the optimizations are my primary focus, 
some stuff is not rendered properly as im reworking the PRERENDERING pipelines

**gpu and paralelization** will be trying to offload as much work as possible to GPU and possibly paralelize few areas, currently we are single CPU bound with
little to ono optimizations so every time we render to terminal this is just costy as terminal is just another layer we have to translate into"

**effects and shaders** since it was solely a proof of concept from the start this is another costy feature, that requires a lot of optimization possibly prerendering too, 
im fairly satisfided with how it all aligns so far, but starting kind of to regret not sticking with gpu accel from the start

**postfx**: a lot of focus went into finding and researching the CRT look and feel, as most of the game is aiming to be set withing the terminal  it is a key POSTFX

**engine**: separated a lot of concernes regarding 3d rendering, currently it is possible to create a small 3d scene and prerender it at a lower cost, 
but there are some issues with camera near/z, and some vertexes appear z-flipped

**sound** i was 100% sure that in will need an audio server to play audio, but it does not matter as long as audio drivers are loaded (thought working via terminal wold limit that in some ways
playing sounds works now, there is playground demo for that

**generated a lot of md files are part of current optimization process unfortunately i want to keep it this way as sometimes i just lose context when working with agensts**

**C# side car is kinda working right now, user is able to navigate and do basic stuff**

## about this project

after ~20 years of coding, regular programming lost its appeal — but building something new still brings spark. this engine is slow going right now, but it's being built as a reusable foundation for future projects.

shell quest runs entirely in the terminal with inherent limitations. the aesthetic is **lo-fi by design** due to budget constraints, paired with **cyber-noire cutscene graphics**. gameplay is terminal-native, where bash commands become your spells and powers. difficulty levels affect simulated machine resources (ram, cpu). the world is open-ended using real state machines under the hood, styled as retro dos text-adventure.

core philosophy: lean on **freedom** in design. no closed ecosystems. awkward in small talk, comfortable in building systems.
decided to remove **RED** colour from the intro cutscenee, as RED was bad
this is a work-in-progress playground.

---

## Demo

![Shell Quest demo](./docs/demo.gif)

Shell Quest is a terminal game + engine for learning shell stuff by actually using commands.

You run YAML-based scenes, solve quests, and get old-school terminal effects on top.

## Getting started

1. Install Rust (1.75+)
2. Clone the repository:
```bash
git clone https://github.com/ppotepa/shell-quest.git
cd shell-quest
```
3. Run the game:
```bash
cargo run -p app
```
4. Run the editor (optional):
```bash
cargo run -p editor
```

## Architecture

shell quest is a two-process system:
- **engine (rust)** — renderer, scene runtime, systems, compositor
- **sidecar (c#)** — simulated os (cognitOS) with shell, filesystem, ftp

the engine spawns the sidecar when entering a terminal scene. they talk via json lines on stdin/stdout through the `engine-io` crate. the sidecar manages all gameplay state: login, shell commands, ftp transfers, quest progress.

difficulty selection affects simulated hardware — cpu speed, ram, nic bandwidth, disk space. everything reads from `MachineSpec`, nothing is hardcoded.

## Repo map

- `app/` — launcher
- `engine/` — runtime loop, systems, renderer
- `engine-core/` — shared model, metadata, built-in effects
- `engine-authoring/` — yaml compile/normalize/schema pipeline
- `engine-io/` — ipc bridge between engine and sidecar processes
- `editor/` — tui authoring editor
- `mods/shell-quest/` — main game mod
  - `os/cognitOS/` — c# simulated minix sidecar
  - `docs/` — quest scripts, design docs
- `mods/` — other content mods
- `schemas/` — shared base schemas
- `docs/` — static docs site + api generation
- `tools/` — asset pipeline, devtool cli, schema-gen

## Documentation

### Core Authoring
- **[scene-centric-authoring.md](scene-centric-authoring.md)** — Complete YAML authoring contract (scenes, layers, sprites, timeline)
- **[timeline-architecture.md](timeline-architecture.md)** — Sprite timeline system, visibility validation, architecture constraints
- **[authoring.md](authoring.md)** — Metadata-first pipeline, effects, postFX, presets

### Features
- **[obj-lighting.md](obj-lighting.md)** — 3D OBJ lighting: directional, point lights, orbit, snap teleport, cel shading
- **[terminal-hud-authoring.md](terminal-hud-authoring.md)** — Terminal UI widgets (window, terminal-input, scroll-list)
- **[inputs.md](inputs.md)** — Scene input profiles (menu, obj-viewer, terminal-shell)
- **[assets.md](assets.md)** — Asset loading, mod structure, packaging

### Tooling
- **[AGENTS.md](AGENTS.md)** — Build commands, schema generation, devtool CLI
- **[editor.md](editor.md)** — TUI editor architecture and usage
- **[logging.md](logging.md)** — Debug logging and overlay system

### Sidecar & Gameplay
- **[engine-io/README.md](engine-io/README.md)** — IPC protocol between engine and sidecar
- **[cognitOS README](mods/shell-quest/os/cognitOS/README.md)** — Simulated OS sidecar architecture
- **[quest scripts](mods/shell-quest/docs/scripts.md)** — Prologue design doc (ftp upload puzzle, 1991)

### Advanced
- **[audio-ipc-prototype.md](audio-ipc-prototype.md)** — Audio IPC design notes
- **[carousel-menu-object.md](carousel-menu-object.md)** — Reusable menu carousel behavior
- **[cutout.md](cutout.md)** — Cutscene compilation and filters


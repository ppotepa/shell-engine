# Shell Quest - Inspired by One Lone Coder https://www.youtube.com/javidx9
---

## changelog

### 24-03-2026

- **splash** : new splash screen
- **optimizations** : tried to optimize the engine as much as possible failed, rolled back plenty of changes, will be doing them more carefully and gradually
- **graphics** : planning on rework of the difficulty menu
- **sidecar** : will be rewritten in **rust** with some improvements
- **audio** : played around with audio, and experimented a little with the final background theme - i think all themes will be simulating 90s machines including floppy, hd operations, modem in simulated manner

### 23-03-2026

- **optimizations** : rendering optimizations, there are no regressions that are related to 3d drawing itself, currently the optimizations are my primary focus, some stuff is not rendered properly as im reworking the PRERENDERING pipelines
- **gpu and paralelization** : will be trying to offload as much work as possible to GPU and possibly paralelize few areas, currently we are single CPU bound with little to no optimizations so every time we render to terminal this is just costly as terminal is just another layer we have to translate into
- **effects and shaders** : since it was solely a proof of concept from the start this is another costly feature, that requires a lot of optimization possibly prerendering too, im fairly satisfied with how it all aligns so far, but starting kind of to regret not sticking with gpu accel from the start
- **postfx** : a lot of focus went into finding and researching the CRT look and feel, as most of the game is aiming to be set within the terminal it is a key POSTFX
- **engine** : separated a lot of concerns regarding 3d rendering, currently it is possible to create a small 3d scene and prerender it at a lower cost, but there are some issues with camera near/z, and some vertexes appear z-flipped
- **sound** : i was 100% sure that i will need an audio server to play audio, but it does not matter as long as audio drivers are loaded (thought working via terminal would limit that in some ways) playing sounds works now, there is playground demo for that
- **C# sidecar** : is kinda working right now, user is able to navigate and do basic stuff
-*plot* : started doing some plot/script related work, digging some internet, trying to make it as immersive as possible by looking up as many historical details
eastereggs: ;)

---

## about this project

after ~20 years of coding, regular programming lost its appeal — but building something new still brings spark. this engine is slow going right now, but it's being built as a reusable foundation for future projects.

shell quest runs entirely in the terminal with inherent limitations. the aesthetic is **lo-fi by design** due to budget constraints, paired with **cyber-noire cutscene graphics**. gameplay is terminal-native, where bash commands become your spells and powers. difficulty levels affect simulated machine resources (ram, cpu). the world is open-ended using real state machines under the hood, styled as retro dos text-adventure.

core philosophy: lean on **freedom** in design. no closed ecosystems.
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

### Local Knowledge Hubs (subsystem navigation)
- **[/app/README.AGENTS.MD](app/README.AGENTS.MD)** — CLI flags, app startup, configuration
- **[/editor/README.AGENTS.MD](editor/README.AGENTS.MD)** — Editor subsystem, architecture, hot-reload
- **[/engine/README.AGENTS.MD](engine/README.AGENTS.MD)** — Runtime systems, optimization status, benchmarking
- **[/engine-core/README.AGENTS.MD](engine-core/README.AGENTS.MD)** — Core model, scene structure, strategy traits
- **[/mods/shell-quest/README.AGENTS.MD](mods/shell-quest/README.AGENTS.MD)** — Main mod content, scenes, assets
- **[/mods/shell-quest-tests/README.AGENTS.MD](mods/shell-quest-tests/README.AGENTS.MD)** — Test mod structure, benchmarking
- **[/tools/README.AGENTS.MD](tools/README.AGENTS.MD)** — Development tools, runners, schema validation
- **[/schemas/README.AGENTS.MD](schemas/README.AGENTS.MD)** — YAML schema system, generation, validation


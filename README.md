# Shell Quest - Inspired by One Lone Coder https://www.youtube.com/javidx9

status : 24-03-2026

plot : started doing some plot/script related work, digging some internet, trying to make it as immersive as possible by looking up as many historical details
eastereggs: ;)

---

## changelog

### 24-03-2026

- **rendering** : diagnosed and rolled back ~20 async rendering optimization commits — visual regressions confirmed (flickering, text artifacts, stale frames, resolution degradation). root cause: async render thread + dirty-region compositing + buffer ownership handoff unstable when combined. all experimental flags preserved in `pipeline_flags.rs` for future one-by-one re-introduction
- **splash** : kept the new YAML-driven splash screen on rolled-back codebase — cherry-picked splash.rs, scene.yml, SVG/PNG assets, added `base64`/`roxmltree` deps
- **diagnostics** : identified 15 root causes across 3 themes (generation tracking, dirty region inconsistencies, viewport caching), documented 20 missing integration tests
- **docs** : updated README, 3D rendering documentation overhaul across subsystem files

### 23-03-2026

- **rendering** : full-day push on async rendering pipeline — ~20 commits covering dirty-region tracking, halfblock partial compositing, async render thread with buffer ownership handoff, virtual buffer presentation optimization
- **postfx** : PostFX quality profiles, modular pipeline restructuring
- **3d** : Scene3D atlas memory management, OBJ prerender parallelization
- **engine** : threaded-render feature flag, prepared-frame infrastructure, pipeline flags system
- **note** : all worked in isolation but combined introduced visual regressions — work preserved in git reflog and stash

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

### Design & Patterns (docs/)
- **[carousel-menu-object.md](docs/carousel-menu-object.md)** — Reusable menu carousel behavior
- **[cutout.md](docs/cutout.md)** — Cutscene compilation and filters
- **[state-machine.md](docs/state-machine.md)** — CognitOS state machine and simulated OS architecture
- **[gameplay.001.md](docs/gameplay.001.md)** — Prologue quest design (1991 MINIX setting)
- **[agents.yaml.md](docs/agents.yaml.md)** — YAML authoring schema reference


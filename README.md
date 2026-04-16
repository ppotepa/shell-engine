# Shell Engine - Inspired by One Lone Coder https://www.youtube.com/javidx9

## about this project

after ~20 years of coding, regular programming lost its appeal — but building something new still brings spark. this engine is slow going right now, but it's being built as a reusable foundation for future projects.

shell quest runs entirely in the terminal with inherent limitations. the aesthetic is **lo-fi by design** due to budget constraints, paired with **cyber-noire cutscene graphics**. gameplay is terminal-native, where bash commands become your spells and powers. difficulty levels affect simulated machine resources (ram, cpu). the world is open-ended using real state machines under the hood, styled as retro dos text-adventure.

core philosophy: lean on **freedom** in design. no closed ecosystems.
this is a work-in-progress playground.

---

## Demo

![Shell Engine demo](./docs/demo.gif)

Shell Engine is a terminal game + engine for learning shell stuff by actually using commands.

You run YAML-based scenes, solve quests, and get old-school terminal effects on top.

## Getting started

1. Install Rust (1.75+)
2. Clone the repository:
```bash
git clone https://github.com/ppotepa/shell-engine.git
cd shell-engine
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
- `mods/shell-engine/` — main game mod
  - `os/cognitOS/` — c# simulated minix sidecar
  - `docs/` — quest scripts, design docs
- `mods/` — other content mods
- `schemas/` — shared base schemas
- `docs/` — static docs site + api generation
- `tools/` — asset pipeline, devtool cli, schema-gen

## Documentation

### Root guides

- **[README.md](README.md)** — project overview and navigation
- **[ARCHITECTURE.md](ARCHITECTURE.md)** — repository structure, dependency graph, systems, rendering pipeline
- **[AUTHORING.md](AUTHORING.md)** — authored scene contract, assets, sprites, effects, PostFX, Rhai
- **[BENCHMARKING.md](BENCHMARKING.md)** — benchmark workflow and regression capture
- **[OPTIMIZATIONS.md](OPTIMIZATIONS.md)** — optimization flags, strategy pattern, invariants

### Local technical READMEs

Technical details are being moved into directory-local README files so knowledge
lives next to the code it describes.

- `app/README.AGENTS.MD` — launcher flow and CLI configuration
- `editor/README.AGENTS.MD` — editor architecture and hot reload
- `engine/README.AGENTS.MD` — runtime orchestration and system order
- `engine-core/README.md` and `engine-core/README.AGENTS.MD` — shared model layer
- `engine-asset/README.md` — scene and asset repository abstraction
- `engine-behavior/README.md` — behavior runtime and Rhai integration
- `engine-compositor/README.md` — composition, rendering, PostFX, prerender
- `engine-scene-runtime/README.md` — mutable scene runtime and control routing
- `mods/shell-engine/README.AGENTS.MD` — main mod structure and content
- `mods/shell-engine-tests/README.AGENTS.MD` — automated test mod and benchmarking usage
- `tools/README.AGENTS.MD` — tooling entry points
- `schemas/README.AGENTS.MD` — schema generation and validation

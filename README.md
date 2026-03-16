# Shell Quest

Shell Quest is an interactive shell game and engine that lets you **learn the Linux shell while playing**. Work through scripted quests, solve terminal puzzles, and watch cinematic effects rendered directly inside your terminal.

## Why Shell Quest?

- **Learn by doing** – every quest is a terminal challenge that teaches practical shell commands.
- **Puzzle-driven gameplay** – progress by reading YAML-driven scenes, wiring behaviors, and executing the right commands.
- **Cinematic terminal renderer** – lightning, CRT glitches, transitions, and sprite animations all share the same extensible effect pipeline.

## Key features

- **Terminal-native renderer** that blasts ANSI frames with double-buffered diffs for smooth animation.
- **Data-driven mods** authored entirely in YAML: scenes, layers, sprites, effects, behaviors, and transitions.
- **Effect metadata + targeting** so every effect declares which objects (scene, layer, bitmap, text) it supports.
- **Built-in editor** with live preview scenes, effect browser, and parameter inspectors.
- **Extensible runtime** composed of `engine`, `engine-core`, and `app`, ready for additional mods or tooling.

## Repository layout

- `app/` – CLI launcher that boots the engine with a selected mod.
- `engine/` – runtime orchestration (game loop, compositor, renderer, behaviors, services).
- `engine-core/` – reusable effect implementations, math utilities, and shared metadata.
- `editor/` – terminal editor with the penguin preview scene and effect browser.
- `mods/` – data packs; `mods/shell-quest` is the reference campaign.
- `docs/` – developer manual plus the embedded [`demo.webm`](docs/demo.webm) capture.

## Getting started

1. Install Rust (1.75+).
2. Clone the repository:
   ```bash
   git clone https://github.com/ppotepa/shell-quest.git
   cd shell-quest
   ```
3. Run the reference mod:
   ```bash
   cargo run -p app
   ```
4. Optional: launch the editor live preview:
   ```bash
   cargo run -p editor
   ```

## Documentation & demo

- Developer docs live in [`docs/`](docs/) (open `docs/index.html` in a browser).
- A short gameplay capture is available at [`docs/demo.webm`](docs/demo.webm) and embedded on the docs landing page.

Learn shell commands while conquering quests!

# Shell Quest

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

## Repo map

- `app/` - launcher
- `engine/` - runtime loop, systems, renderer
- `engine-core/` - shared effects and core logic
- `editor/` - TUI editor
- `mods/` - game data (`mods/shell-quest` is the main one)
- `docs/` - docs + demo clip

## Documentation & demo

- Demo video: [▶ Obejrzyj demo](./docs/demo.mp4)

<div align="center">
  <video src="./docs/demo.mp4" controls preload="metadata" poster="./docs/assets/imported/shell-quest-terminal.png">
    Demo video
  </video>
</div>

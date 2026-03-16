# Shell Quest

## Demo

[![Watch demo](https://img.youtube.com/vi/sD9wEXBITmM/hqdefault.jpg)](https://www.youtube.com/watch?v=sD9wEXBITmM&autoplay=1)

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
- `docs/` - docs + static docs site

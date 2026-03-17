# Shell Quest - Inspired by One Lone Coder https://www.youtube.com/javidx9

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

## Dev helper CLI (`devtool`)

`devtool` is a small workspace utility for scaffolding and schema workflow.

Examples:
```bash
cargo run -p devtool -- new mod my-mod
cargo run -p devtool -- new scene my-mod intro-logo --id my-mod.intro-logo
cargo run -p devtool -- new effect my-mod intro-logo flash --builtin lightning-flash --duration 180
cargo run -p devtool -- schema refresh --all-mods
cargo run -p devtool -- schema check --all-mods
```

Shortcut wrapper:
```bash
./devtool.sh new mod my-mod
```

## Repo map

- `app/` - launcher
- `engine/` - runtime loop, systems, renderer
- `engine-core/` - shared effects and core logic
- `editor/` - TUI editor
- `mods/` - game data (`mods/shell-quest` is the main one)
- `docs/` - docs + static docs site

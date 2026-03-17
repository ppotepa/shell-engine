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

## YAML-first authoring workflow

`./refresh-schemas.sh` is the main authoring refresh entrypoint.

Run it after changing mod content, scene ids, templates, objects, effects, sprites, or assets:
```bash
./refresh-schemas.sh
```

Strict drift check:
```bash
cargo run -p schema-gen -- --all-mods --check
```

What the refresh flow does:

- scans every mod under `mods/`
- rebuilds per-mod authoring overlays in `mods/{mod}/schemas/*.yaml`
- rebuilds `mods/{mod}/schemas/catalog.yaml` with dynamic authoring enums
- keeps `mod.yaml`, `scene.yml`, and scene partial schemas aligned with current content

Important schema layout:

- global base schemas stay in `schemas/*.yaml`
- dynamic per-mod overlays live in `mods/{mod}/schemas/*.yaml`
- `mods/{mod}/mod.yaml` points to local `./schemas/mod.yaml`
- scene roots and partials point to local per-mod overlays such as `schemas/scenes.yaml`, `schemas/layers.yaml`, `schemas/sprites.yaml`

Current generated authoring suggestions include:

- `mod.yaml.entrypoint` from real discoverable scene paths
- scene routing in `next`, `menu-options[].next`, `menu-options[].scene`, `to`
- object references in `objects[].ref` and `objects[].use`
- object prefab logic in `object.logic.behavior` and `object.logic.params`
- sprite template usage in `sprite.use`
- input target selection in `input.obj-viewer.sprite_id`
- sprite asset values in `source` and `font`
- embedded effect names/params in scene, layer, and sprite stages
- behavior target fields such as `params.target` / `params.sprite_id`

Recommended daily flow:

1. Scaffold or edit YAML with `devtool` and normal editor workflow.
2. Run `./refresh-schemas.sh`.
3. Re-open or continue editing YAML with updated completions.
4. Use `schema-gen --check` in CI or before pushing when you want to verify no generated schema drift remains.

Notes:

- `devtool new mod`, `devtool new scene`, and `devtool new effect` already refresh the touched mod schemas for you
- every new dynamic authoring surface should follow the same pipeline: collector -> `catalog.yaml` -> per-mod overlay -> regression test

## Repo map

- `app/` - launcher
- `engine/` - runtime loop, systems, renderer
- `engine-core/` - shared effects and core logic
- `editor/` - TUI editor
- `mods/` - game data (`mods/shell-quest` is the main one)
- `docs/` - docs + static docs site

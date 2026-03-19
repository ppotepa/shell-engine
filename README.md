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
   Run playground mod explicitly:
```bash
cargo run -p app -- --mod-source mods/playground
```
4. Run the editor (optional):
```bash
cargo run -p editor
```

## Project docs

- `authoring.md` - metadata-first rollout and tooling direction
- `assets.md` - asset, sprite, object, and source model guide
- `editor.md` - editor architecture and refactor direction
- `docs/scene-centric-authoring.md` - scene/layer/object authoring model
- `docs/carousel-menu-object.md` - centered menu composition pattern (object + runtime behavior)
- `docs/cutout.md` - aktualne działanie efektu cutout i diagnostyka YAML

## How the project fits together

The repository is split into a few clear domains:

- `engine-core/` = shared scene model, builtin effects, and authoring/runtime data types
- `engine-authoring/` = scene package assembly, authored YAML compilation, schema overlay generation
- `engine/` = runtime loading, repositories, render pipeline, compositor, startup checks
- `editor/` = YAML-first TUI editor and preview tooling
- `mods/` = authored game data and generated per-mod schemas

In practice the flow is:

1. author YAML in `mods/<mod>/`
2. refresh generated schema overlays
3. `engine-authoring` assembles/normalizes authored content
4. runtime loads the compiled scene model
5. editor should preview the same compiled result, not invent a parallel format

## Current authoring model

The preferred direction is now **scene-centric**:

- `scene.yml` orchestrates flow and explicit layer order
- `layers/*.yml` hold visual composition
- `objects/*.yml` hold reusable prefabs

For package scenes, prefer explicit layer references such as:

```yaml
layers:
  - ref: main
```

instead of relying only on implicit package merge order.

Current scene composition also supports:

- `stages-ref` for reusable lifecycle presets (`/stages/*.yml`)
- `cutscene-ref` for frame-timed cutscene expansion
- `menu-options` with `to` routing alias
- `sprite-defaults` on scene/layer/sprite scopes
- `frame-sequence` sprite expansion for PNG sequence animation
- `logic` na scenie (`native` / `script`) oraz auto-detekcję sidecarów logiki (`*.rhai`, `*.logic.rhai`, `*.logic.yml`)

## Runtime facts: menu + logic

- `menu-options` są obsługiwane wyłącznie gdy scena jest w `on_idle` i `on_idle.trigger` ma wartość `any-key`.
- Przejście menu działa przez: klawisze opcji (np. `1`, `2`), `Enter` (aktywna pozycja), strzałki (nawigacja).
- Scena może ładować logikę skryptową Rhai przez:
  - `logic: { type: script, src: ./plik.rhai }`,
  - albo przez sidecar wykrywany automatycznie przy scenie (bez bloku `logic`).
- Zakres komend emitowanych przez `rhai-script`:
  - `visibility` (`target`, `visible`)
  - `offset` (`target`, `dx`, `dy`)

### Rhai sidecar naming

Auto-detekcja sidecarów logiki:

- dla scen pakietowych `scenes/<name>/scene.yml`:
  - `<name>.rhai`
  - `scene.rhai`
  - `<name>.logic.rhai`
  - `scene.logic.rhai`
  - `<name>.logic.yml`
  - `scene.logic.yml`
- dla scen single-file `scenes/<name>.yml`:
  - `<name>.rhai`
  - `<name>.logic.rhai`
  - `<name>.logic.yml`

### YAML diagnostics cheat sheet

| Komunikat walidacji | Najczęstsza przyczyna | Co poprawić |
| --- | --- | --- |
| `Property stretch-to-area is not allowed.` | Klucz wstawiony do węzła, który nie dopuszcza tej właściwości | Sprawdź schema dla bieżącego kontekstu (`scene`/`layer`/`sprite`/`object`) i przenieś klucz do poprawnego miejsca |
| `Value is not divisible by 0.05000000074505806.` | Parametr ma ograniczenie `multipleOf: 0.05` | Użyj wartości w krokach `0.05` (`0.10`, `0.15`, `0.20`...) |
| Menu nie reaguje na `Enter`/strzałki | `on_idle.trigger` nie jest `any-key` | Ustaw `stages.on_idle.trigger: any-key` lub użyj `stages-ref` z takim triggerem |
| `next/to` nie przechodzi do sceny | Brak docelowego `id` lub literówka w referencji | Sprawdź `menu-options[].to` / `next` i istnienie sceny docelowej |

### Minimal menu scene (working)

```yaml
# scenes/menu/scene.yml
id: my-menu
title: My Menu
bg: black
layers:
  - ref: main
stages-ref: /stages/anykey-loop-1200-fade-250-180.yml
next: my-scene-a
menu-options:
  - { key: "1", label: "SCENE A", to: my-scene-a }
  - { key: "2", label: "SCENE B", to: my-scene-b }
```

```yaml
# scenes/menu/layers/main.yml
- name: menu
  z_index: 0
  sprites:
    - { type: text, content: "MY MENU", at: ct, y: -6, font: "generic:1", fg: white }
    - { type: text, content: "[1] SCENE A", at: cc, y: -1, font: "generic:1", fg: silver }
    - { type: text, content: "[2] SCENE B", at: cc, y: 1, font: "generic:1", fg: gray }
```

## Practical engine limits to remember

These limits matter when designing content and editor features:

- image alpha is thresholded, not smoothly blended
- image sprites now support static PNG and animated GIF playback through the same pixel-buffer render path
- 3D mesh support is currently **OBJ**, not glTF/glb
- there is no dedicated `3D -> offscreen buffer -> projected reflection texture` pipeline yet

So if you want a reflected 3D element today, the realistic options are:

- author it as an `obj` sprite,
- pre-render it to images,
- or add a new engine feature rather than expecting glTF reflection support to already exist

For authored animated cutscenes today:

- use an `image` sprite with a `.gif` source
- the engine decodes GIF frames and renders them through the normal terminal pixel buffer
- timing follows the GIF frame delays
- current playback behavior loops automatically and does not yet expose dedicated controls like `once`, `hold-last-frame`, or manual playhead control

## Dev helper CLI (`devtool`)

`devtool` is a workspace utility for authoring scaffolds and schema workflow.

Examples:
```bash
cargo run -p devtool -- create mod my-mod
cargo run -p devtool -- create scene --mod my-mod intro-logo --id my-mod.intro-logo
cargo run -p devtool -- create layer --mod my-mod --scene intro-logo overlay
cargo run -p devtool -- create sprite ./logo.png --mod my-mod --scene intro-logo --layer main --width 18
cargo run -p devtool -- edit sprite --mod my-mod --scene intro-logo --layer main --id logo --x -3 --y "oscillate(-1,1,1200ms)" --width 24
cargo run -p devtool -- create effect --mod my-mod --scene intro-logo flash --builtin lightning-flash --duration 180
cargo run -p devtool -- schema refresh --all-mods
cargo run -p devtool -- schema check --all-mods
```

Shortcut wrapper:
```bash
./devtool.sh create mod my-mod
```

Raw frame sequence -> GIF helper:
```bash
python tools/devtool/stop_animation_converter.py frame1.png frame2.png ... \
  --output mods/shell-quest/assets/images/intro/cutscene.gif
```

Retro cel-shaded style preset (lower palette, stronger edges):
```bash
python tools/devtool/stop_animation_converter.py frame1.png frame2.png ... \
  --output mods/shell-quest/assets/images/intro/cutscene_cellshaded.gif \
  --fps 4 --colors 48 --contrast 1.2 --sharpness 1.35
```

## Interactive terminal shell (scene input profile)

Scenes can now declare an interactive command shell that writes into normal text sprites.
This reuses existing scene layers (background + output panel + prompt line), so UI can be narrow/wide/fullscreen depending on authored sprite layout.

```yaml
input:
  terminal-shell:
    prompt-sprite-id: terminal-prompt
    output-sprite-id: terminal-output
    prompt-prefix: "λ "
    max-lines: 120
    banner: ["connected: shell-node", "try: help, ls, status"]
    commands:
      - name: status
        output: ["power: online", "hull: 92%"]
```

Built-ins: `help`, `clear`, `ls`, `pwd`, `echo`, `whoami`.

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

- `devtool create mod`, `devtool create scene`, `devtool create layer`, `devtool create sprite`, and `devtool create effect` already refresh the touched mod schemas for you
- `devtool create sprite` copies the source image into `assets/images/` and appends an image sprite into the target layer (`main/main` by default)
- `devtool edit sprite` updates common sprite fields (`at`, `x`, `y`, `width`, `height`) in-place and also refreshes schemas for the touched mod
- `mods/*/assets/raw/` is treated as local staging space for source frames/raw inputs and is ignored by git; commit converted assets outside `raw/`
- `devtool new ...` still works as a backward-compatible alias for `devtool create ...`
- every new dynamic authoring surface should follow the same pipeline: collector -> `catalog.yaml` -> per-mod overlay -> regression test

## Repo map

- `app/` - launcher
- `engine/` - runtime loop, systems, renderer
- `engine-authoring/` - scene compiler, package assembly, schema overlay generation
- `engine-core/` - shared effects and core logic
- `editor/` - TUI editor
- `mods/` - game data (`mods/shell-quest` is the main one)
- `docs/` - docs + static docs site

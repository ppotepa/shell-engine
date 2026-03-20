# AGENTS.md

## 1) Repo shape

- `app/` launcher
- `engine/` runtime systems and render pipeline
- `engine-core/` shared model, metadata, built-in effects
- `engine-authoring/` YAML compile/normalize/schema pipeline
- `editor/` TUI authoring tool
- `mods/` content mods
- `schemas/` shared base schemas

Scenes are loaded as:

- single YAML file (`scenes/*.yml`),
- scene package (`scenes/<name>/scene.yml` + partials).

Asset loading supports unpacked mod dirs and zip-packaged mods.

## 2) Editor architecture (short map)

`editor/src`:

- `app.rs` terminal lifecycle and main editor loop
- `cli.rs` CLI options (`--mod-source`)
- `domain/` scene/effect/asset indexes and diagnostics
- `io/` file scanning and YAML IO
- `input/` key mapping and commands
- `state/` app state
- `ui/` draw/layout/focus/filter/theme

Editor uses model and metadata from `engine-core` + `engine-authoring`.

## 3) Tooling commands

Schema generation:

```bash
cargo run -p schema-gen -- --all-mods
```

Schema drift check:

```bash
cargo run -p schema-gen -- --all-mods --check
```

Helper:

```bash
./refresh-schemas.sh
```

Run app:

```bash
cargo run -p app
```

Run editor:

```bash
cargo run -p editor
```

Run playground mod:

```bash
SHELL_QUEST_MOD_SOURCE=mods/playground cargo run -p app
```

Run playground mod with debug helpers:

```bash
SHELL_QUEST_MOD_SOURCE=mods/playground cargo run -p app -- --debug-feature
```

Run shell-quest with debug helpers:

```bash
cargo run -p app -- --debug-feature
```

Debug overlay keys (when `--debug-feature` is active):
- **F1** — toggle Stats overlay (scene id, virtual size, last Rhai errors)
- **~** / **`** — toggle Logs overlay (last N runtime log entries)
- **F3 / F4** — prev/next scene

Core tests:

```bash
cargo test -p engine
cargo test -p engine-core
cargo test -p engine-authoring
```

## 4) Authoring invariants

- Preserve runtime system order unless explicitly changing architecture.
- Keep resolver correctness for layer/sprite ordering.
- Apply scene `virtual-size-override` on transitions.
- Keep virtual buffer in sync with terminal resize (`max-available` policy).
- Keep stage progression stable for empty/0ms steps.
- Reset per-frame behavior runtime state before behavior execution.
- Keep compatibility with existing mod YAML structure.
- **Rhai multiline strings: always use backtick templates** (`` `...` ``), never `"...\n..."`.
- **ScriptError command** is emitted on Rhai compile/runtime failure — consumed by `behavior_system` into `DebugLogBuffer`.

## 5) Change playbook

When changing scene model or fields:

- update `engine-core` model,
- update `engine-authoring` compile/normalize path,
- update schema surfaces,
- update runtime consumption.

When changing effect params:

- update effect metadata,
- update schema generation,
- update editor consumption.

When changing render/compositor:

- verify compositor + renderer + virtual buffer interactions.

When changing transitions/lifecycle:

- verify scoped reset behavior,
- verify scene loader reference resolution.

When changing Rhai script API (scope variables, commands):

- update `BehaviorContext` in `engine/src/behavior.rs`,
- update scope push block in `RhaiScriptBehavior::update`,
- update `scene-centric-authoring.md` section 13,
- add regression test in `behavior::tests`.

When adding new debug/diagnostic features:

- push to `DebugLogBuffer` via `BehaviorCommand::ScriptError` or direct `world.get_mut::<DebugLogBuffer>()`,
- keep overlay render O(rows × cols),
- do not read `run.log` from disk per frame.

## 6) Preferred working style

- Keep changes minimal, local, and type-safe.
- Avoid hidden fallback behavior.
- Validate with existing crate tests after code changes.

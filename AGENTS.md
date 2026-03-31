# AGENTS.md

## 0) Documentation Roadmap

**Start here:** Read the root-level consolidated docs first, then drill into subsystem READMEs as needed.

### Root-Level Consolidated Docs (Main Directory)

- **[ARCHITECTURE.md](ARCHITECTURE.md)** — Repository structure, dependency graph, per-frame systems, strategy pattern, scene model, buffer architecture, halfblock rendering, timeline system, input system, logging, schema system, editor design, change playbook
- **[AUTHORING.md](AUTHORING.md)** — Metadata-first approach, mod structure, asset system, sprite types, scene contract, PostFX pipeline, OBJ lighting, terminal HUD, Rhai scripting, compilation pipeline, author checklist, daily workflow
- **[MODS.md](MODS.md)** — Shell Quest main mod, shell-quest-tests benchmark mod, playground dev mod, creating custom mods
- **[BENCHMARKING.md](BENCHMARKING.md)** — Quick start, optimization flags, test mod specs, frame capture regression testing, benchmark scenarios and reports
- **[OPTIMIZATIONS.md](OPTIMIZATIONS.md)** — CLI flags, strategy pattern, 20 optimization implementations, summary stats, key invariants, usage examples

### Subsystem-Local READMEs (Each Directory)

Each subsystem has a focused `README.AGENTS.MD` for deep dives:

- **[app/README.AGENTS.MD](app/README.AGENTS.MD)** — CLI flags, startup flow, configuration
- **[editor/README.AGENTS.MD](editor/README.AGENTS.MD)** — Editor architecture, hot-reload, subsystems
- **[engine/README.AGENTS.MD](engine/README.AGENTS.MD)** — Runtime systems, optimization status, benchmarking
- **[engine-core/README.AGENTS.MD](engine-core/README.AGENTS.MD)** — Scene model, buffer management, strategy traits
- **[engine-*/README.md](engine-3d/README.md)** — crate-specific READMEs (purpose, key types, dependencies, usage)
- **[mods/shell-quest/README.AGENTS.MD](mods/shell-quest/README.AGENTS.MD)** — Content structure, scenes, assets
- **[mods/shell-quest-tests/README.AGENTS.MD](mods/shell-quest-tests/README.AGENTS.MD)** — Test mod, benchmarking, looping
- **[tools/README.AGENTS.MD](tools/README.AGENTS.MD)** — Benchmark runners, frame capture, schema tools
- **[schemas/README.AGENTS.MD](schemas/README.AGENTS.MD)** — Schema generation, validation, drift checking

**Workflow:** Read consolidated docs first for breadth, then `README.AGENTS.MD` / `engine-*/README.md` for depth.

## 1) Repo shape

- `app/` launcher
- `engine/` runtime systems and render pipeline
- `engine-core/` shared model, metadata, built-in effects
- `engine-authoring/` YAML compile/normalize/schema pipeline
- `engine-io/` transport-agnostic IPC bridge (sidecar communication)
- `editor/` TUI authoring tool
- `mods/` content mods
  - `mods/shell-quest/os/cognitOS/` C# sidecar (simulated MinixOS)
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

Run startup validation for a mod:

```bash
cargo run -p app -- --mod-source=mods/playground --check-scenes
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

Build C# sidecar:

```bash
cd mods/shell-quest/os/cognitOS
dotnet build -c Release
```

Benchmarking:

```bash
# Baseline 10-second benchmark (splash auto-skipped)
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10

# With all optimizations
cargo run -p app -- --mod-source=mods/shell-quest-tests --bench 10 --opt

# Aggregate reports to CSV
python3 collect-benchmarks.py
```

Optimization flags:

| Flag | What it does |
|------|-------------|
| `--opt-comp` | Compositor: layer scratch skip, dirty-region halfblock |
| `--opt-diff` | DirtyRegionDiff instead of full buffer scan |
| `--opt-present` | Hash-based static frame skip |
| `--opt-skip` | Unified frame-skip oracle (prevents flickering) |
| `--opt-rowdiff` | Row-level dirty skip in diff scan |
| `--opt` | All of the above |

## 4) Authoring invariants

Refer to **[AUTHORING.md](AUTHORING.md)** for comprehensive authoring contract. Key points below.

- Preserve runtime system order unless explicitly changing architecture.
- Keep resolver correctness for layer/sprite ordering.
- Apply scene `virtual-size-override` on transitions.
- Keep authored render size and output presentation semantics distinct:
  fixed `render_size` defines the in-memory canvas; `match-output` /
  `max-available` are only for intentionally output-tracking scenes.
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
- update the Rhai scripting section in `AUTHORING.md`,
- add regression test in `behavior::tests`.

When changing audio sequencing or generated synth cues:

- update `engine-audio-sequencer`,
- update `engine/src/audio_sequencer.rs`,
- update startup checks in `engine-mod/src/startup/checks/audio_sequencer.rs`,
- update `AUTHORING.md` audio section,
- verify `cargo run -p app -- --mod-source=mods/<mod> --check-scenes`.

When adding new debug/diagnostic features:

- push to `DebugLogBuffer` via `BehaviorCommand::ScriptError` or direct `world.get_mut::<DebugLogBuffer>()`,
- keep overlay render O(rows × cols),
- do not read `run.log` from disk per frame.

When adding new optimization flags:

- add `--opt-*` CLI flag in `app/src/main.rs`,
- add field to `PipelineFlags` (`engine/src/pipeline_flags.rs`),
- add field to `EngineConfig` (`engine/src/lib.rs`),
- implement as Strategy trait (see `engine/src/strategy/`),
- wire into `PipelineStrategies::from_flags()`,
- include in `--opt` umbrella flag,
- add benchmark comparison before/after,
- update `OPTIMIZATIONS.md`.

**When adding sprite timing fields**:

- Add fields to `Sprite` variants in `engine-core/src/scene/sprite.rs`,
- Add metadata in `engine-core/src/scene/metadata.rs`,
- Update schema in `schemas/scene.schema.yaml`,
- Update validation in `engine-authoring/src/validate/mod.rs` (if timing-related),
- Update `AUTHORING.md` section 9 and `ARCHITECTURE.md` section 8.

## 6) Sidecar & gameplay changes

When adding new shell commands:

- add `Commands/<Name>Command.cs` implementing `ICommand`,
- register in `Program.cs` commands array,
- if command changes `SessionMode`, handle in `AppHost.HandleSubmit` switch,
- update `mods/shell-quest/docs/scripts.md` if gameplay-relevant.

When changing difficulty levels:

- update `Difficulty` enum in `Core/Difficulty.cs`,
- update `MachineSpec.FromDifficulty()` switch,
- update difficulty table in `docs/scripts.md`,
- verify boot sequence reads spec (`BootSequence.cs`).

When changing sidecar IPC protocol:

- update `IoRequest`/`IoEvent` enums in `engine-io/src/lib.rs`,
- update engine consumption in `engine/src/systems/engine_io.rs`,
- update C# parsing in `Program.cs` / `Protocol.cs`,
- update `engine-io/README.md`.

When changing virtual filesystem:

- update `IVirtualFileSystem` or `IMutableFileSystem` in `State/VirtualFileSystem.cs`,
- update `SeedEpochFiles()` if adding new game files,
- verify path resolution in `CommandContext.ResolvePath()`.

## 7) Preferred working style

- Keep changes minimal, local, and type-safe.
- Avoid hidden fallback behavior.
- Validate with existing crate tests after code changes.

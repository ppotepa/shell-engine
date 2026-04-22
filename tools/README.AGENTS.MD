# Tools — developer utilities

## Purpose

`tools/` contains helper utilities used while authoring, validating, and
benchmarking Shell Engine content. Some are Rust binaries in the workspace and
some are standalone scripts.

## Main entries

### `schema-gen/`

Rust CLI for generating or checking per-mod YAML schema fragments and shared
effect schemas.

Primary commands:

```bash
cargo run -p schema-gen -- --all-mods
cargo run -p schema-gen -- --all-mods --check
cargo run -p schema-gen -- --mod mods/playground
```

Use it whenever scene/effect schema surfaces change.

### `devtool/`

Rust authoring helper CLI for scaffolding and editing mod content.

It currently supports:

- creating mods,
- creating scene packages,
- creating layer files,
- creating sprite scaffolds,
- creating effect scaffolds,
- editing sprite declarations,
- refreshing/checking schema fragments for selected mods,
- printing an architecture snapshot report for scene contract, world-model,
  controller-defaults, legacy camera input, and object-ref migration status.

It also keeps yaml-language-server schema fragments in sync after scaffold/edit operations.

### Benchmark and capture scripts

Repository-root scripts such as `benchmark.py` and `collect-benchmarks.py`
work together with direct `cargo run -p app ... --capture-frames` commands and
benchmark reports.

Use these when measuring optimization impact or checking visual regressions.

### Asset helpers

- `simplify_glb.py` — simplify heavy GLB assets before converting or shipping
- `glb_to_obj.py` — conversion helper for 3D asset preparation
- `ttf-rasterizer/` — font-related helper toolchain

## Working with this directory

- keep generated artifacts out of committed docs unless they are intended outputs,
- prefer documenting tool inputs/outputs and integration points over implementation trivia,
- if a tool mutates mod content, mention whether it also refreshes schemas,
- when changing schema generation behavior, update both `tools/README.AGENTS.MD`
  and `schemas/README.AGENTS.MD`.

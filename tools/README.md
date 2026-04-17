# tools

Developer utilities for authoring, schema maintenance, asset preparation, and
benchmark workflows.

## Purpose

This directory groups helper tools that are not part of the runtime itself but
support day-to-day development.

## Important tools

- `schema-gen/` — generate or check mod-local schema fragments
- `devtool/` — scaffold and edit mod content from the command line
- `simplify_glb.py` — reduce polygon count for heavy GLB assets
- `ttf-rasterizer/` — font tooling

Repository-root scripts like `benchmark.py` and `collect-benchmarks.py` are
also part of the overall tooling workflow even though they sit outside this
directory. Frame capture currently uses direct
`cargo run -p app ... --capture-frames` commands rather than checked-in wrapper
scripts.

## Usage

```bash
# refresh schema fragments for all mods
cargo run -p schema-gen -- --all-mods

# check schema drift
cargo run -p schema-gen -- --all-mods --check

# inspect devtool commands
cargo run -p devtool -- --help
```

For deeper operational notes, see `tools/README.AGENTS.MD`.

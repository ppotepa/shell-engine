# engine-authoring

YAML compile, normalize, and schema pipeline for scene authoring.

## Purpose

The largest engine crate. Compiles raw YAML scene files into validated,
normalized scene models consumed by the runtime. Also generates JSON
schemas for editor autocompletion and validation. Handles scene packages,
partial includes, and repository-level operations across mods.

## Key Types

- `SceneCompiler` — transforms raw YAML into fully resolved `Scene` models
- `Normalizer` — applies defaults, expands shorthand, and validates fields
- `SchemaGenerator` — produces JSON schemas from scene/effect metadata
- `Repository` — indexes all scenes and assets across a mod source

## Dependencies

- `engine-core` — scene model, effect metadata, and strategy traits
- `anyhow` — error propagation during compilation
- `serde` / `serde_yaml` — YAML parsing and serialization

## Usage

```bash
# Generate schemas for all mods
cargo run -p schema-gen -- --all-mods

# Check for schema drift
cargo run -p schema-gen -- --all-mods --check
```

The editor and CLI tools use this crate to load, validate, and
transform scene YAML.

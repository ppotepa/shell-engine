# schemas

Shared generated YAML schemas for authoring.

## Purpose

`schemas/` contains the repository-level schema files used by YAML authoring
workflows, validation, and editor intellisense. These are generated from the
current authoring metadata and shared across mods.

## Typical workflow

```bash
cargo run -p schema-gen -- --all-mods
cargo run -p schema-gen -- --all-mods --check
```

Per-mod schema fragments are generated into each mod’s own `schemas/`
directory.

## Related docs

- `schemas/README.AGENTS.MD` for generation details and invariants
- `tools/README.AGENTS.MD` for schema tooling

# engine-authoring

YAML authoring pipeline for scenes, packages, validation, and schema metadata.

## Purpose

`engine-authoring` is the shared content-ingest layer for authored Shell Quest
data. It parses YAML documents, assembles scene packages, normalizes authored
input, validates it, and exposes schema-generation metadata consumed by tooling.

## Key modules

- `compile` — compile authored inputs into runtime-ready scene data
- `document` — YAML document handling helpers
- `package` — scene package assembly
- `repository` — repository/mod scanning and authored asset discovery
- `schema` — schema metadata and generation support
- `validate` — validation rules and diagnostics

## Main export

- `AuthoringResult<T>`

## Integration points

- `tools/schema-gen` uses this crate to write shared and mod-local YAML schemas
- `devtool` relies on it for scaffold/edit-aware schema refresh
- `editor` and runtime loading paths depend on its compile/validation behavior

## Working with this crate

- this is the source of truth for authored content interpretation,
- if authoring fields or normalization rules change, update schemas and nearby docs in the same change,
- keep generated-schema discussions aligned with the current YAML workflow, not the older JSON-only descriptions.

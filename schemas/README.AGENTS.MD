# Schemas — generated YAML authoring support

## Purpose

`schemas/` contains shared schema files used for YAML authoring validation and
editor intellisense. These files are generated from the current scene/effect
metadata and are consumed by mods and tooling.

## What lives here

- shared effect schemas like `effect.schema.yaml` and `effect-params.schema.yaml`
- core mod/schema files such as `mod.schema.yaml`, `scene.schema.yaml`, and related fragment schemas
- `generated/` outputs used by authoring workflows

Per-mod schema fragments are generated inside each mod’s `schemas/` directory.

## Generation workflow

Use:

```bash
cargo run -p schema-gen -- --all-mods
```

or:

```bash
./refresh-schemas.sh
```

Use check mode in validation flows:

```bash
cargo run -p schema-gen -- --all-mods --check
```

## Working with schemas

- do not hand-edit generated outputs unless the workflow explicitly expects it,
- if a scene/effect/model field changes, regenerate schemas in the same change,
- keep root shared schemas and mod-local schema fragments in sync,
- when authoring metadata changes, update both generation code and the docs that
  describe the public contract.

## Integration points

- `engine-authoring` defines the metadata and schema-generation logic
- `tools/schema-gen` writes the files
- `devtool` refreshes mod-local schema fragments after scaffold/edit operations
- mods reference generated schema files via yaml-language-server comments

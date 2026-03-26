# engine-runtime

Runtime settings parsed from mod manifests and environment overrides.

## Purpose

`engine-runtime` owns the small but important layer that turns authored
`mod.yaml` terminal/runtime settings into concrete runtime configuration used by
the app and engine.

It is intentionally narrow: this crate does not run the game loop; it defines
how runtime options such as virtual-buffer usage, virtual size, and renderer
mode overrides are interpreted.

## Key types

- `RuntimeSettings` — resolved runtime options used at startup and resize time
- `VirtualPolicy` — how virtual rendering should react to terminal constraints
- `parse_virtual_size_str()` — shared helper for CLI/config parsing

## What it does

- reads terminal settings from the manifest `terminal` block,
- accepts both kebab-case and snake_case YAML keys,
- applies environment overrides such as:
  - `SHELL_QUEST_USE_VIRTUAL_BUFFER`
  - `SHELL_QUEST_VIRTUAL_SIZE`
  - `SHELL_QUEST_VIRTUAL_POLICY`
  - `SHELL_QUEST_RENDERER_MODE`
- resolves `max-available` virtual size against the current terminal dimensions.

## Working with this crate

- keep parsing behavior backward-compatible when possible,
- if manifest field names change, support both forms during migrations,
- keep renderer-mode parsing aligned with `engine-core::scene::SceneRenderedMode`,
- when runtime settings change, update launcher docs and mod authoring docs in
  the same change.

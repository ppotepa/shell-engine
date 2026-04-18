# engine-runtime

Runtime settings parsed from mod manifests and environment overrides.

## Purpose

`engine-runtime` owns the small but important layer that turns authored
`mod.yaml` display/runtime settings into concrete runtime configuration used by
the app and engine.

It is intentionally narrow: this crate does not run the game loop; it defines
how runtime options such as world/UI render sizes and presentation policy are interpreted.

## Key types

- `RuntimeSettings` — resolved runtime options used at startup and presentation time
- `RenderSize` — authored size token (`Fixed`, `MatchOutput`, `FitWidth`)
- `PresentationPolicy` — how authored rendering maps into the active output buffer
- `BufferLayout` — explicit world/UI/output dimensions derived from runtime settings
- `parse_render_size_str()` — shared helper for CLI/config parsing

## What it does

- reads display settings from the manifest `display` block,
- accepts both kebab-case and snake_case YAML keys,
- applies environment overrides such as:
  - `SHELL_ENGINE_WORLD_RENDER_SIZE`
  - `SHELL_ENGINE_UI_RENDER_SIZE`
  - `SHELL_ENGINE_UI_LAYOUT_SIZE`
  - `SHELL_ENGINE_PRESENTATION_POLICY`
- resolves separate world/UI/layout sizes and still supports `match-output` /
  `max-available` when a target should track the current output dimensions.
- supports display policies:
  - `strict` — 1:1 with centered crop/pad,
  - `fit` — preserve aspect ratio with letterboxing,
  - `stretch` — fill the whole output buffer by resampling.

## Working with this crate

- keep world/UI/layout semantics explicit and non-overlapping,
- when runtime settings change, update launcher docs and mod authoring docs in
  the same change.

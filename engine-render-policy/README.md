# engine-render-policy

Renderer-mode and font-resolution policy helpers.

## Purpose

`engine-render-policy` contains small shared policy functions used to resolve
effective renderer mode and font specification choices from scene data,
overrides, and sprite sizing rules.

## Main functions

- `resolve_renderer_mode()`
- `resolve_font_spec()`
- `resolve_text_font_spec()`

## What it handles

- forced renderer-mode overrides,
- generic font-mode derivation from render mode,
- text font derivation from sprite size presets,
- `font: default` alias resolution via mod-level `terminal.default_font`,
- normalization of named and generic font-mode aliases.

## Working with this crate

- keep these helpers pure and deterministic,
- preserve compatibility with authored font specs and existing aliases,
- if renderer-mode semantics change, update both these helpers and the authoring docs.

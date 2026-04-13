# engine-render-policy

Font-resolution policy helpers.

## Purpose

`engine-render-policy` contains small shared policy functions used to resolve
effective font specification choices from scene data, overrides, and sprite
sizing rules.

## Main functions

- `resolve_font_spec()`
- `resolve_text_font_spec()`

## What it handles

- generic font-mode derivation,
- text font derivation from sprite size presets,
- `font: default` alias resolution via mod-level `terminal.default_font`,
- normalization of named and generic font-mode aliases.

## Working with this crate

- keep these helpers pure and deterministic,
- preserve compatibility with authored font specs and existing aliases.

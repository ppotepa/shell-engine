# engine-mod

Mod manifest loading and startup validation.

## Purpose

`engine-mod` loads `mod.yaml` from either an unpacked mod directory or a zipped
mod package and validates that the configured scene entrypoint exists before the
runtime starts.

## Main responsibilities

- detect whether a mod source is a directory or `.zip` archive,
- read and parse `mod.yaml`,
- validate required manifest structure,
- verify that the configured scene entrypoint exists in that source,
- expose startup helpers used before scene compilation begins.

## Main exports

- `load_mod_manifest()` — load and validate manifest YAML
- `startup` — startup validation helpers

## Working with this crate

- keep directory and zip behavior aligned,
- preserve good error reporting because this crate is part of the user-facing startup path,
- if manifest requirements change, update authoring docs and example mods in the same change.

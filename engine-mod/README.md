# engine-mod

Mod manifest loading and startup validation.

## Purpose

`engine-mod` loads `mod.yaml` from either an unpacked mod directory or a zipped
mod package and provides startup validation before runtime boot.

## Main responsibilities

- detect whether a mod source is a directory or `.zip` archive,
- read and parse `mod.yaml`,
- validate required manifest structure,
- verify that the configured scene entrypoint exists in that source,
- run startup checks for scene graph, levels, Rhai scripts, effect names,
  image/font assets, display settings, and audio sequencer data,
- enforce lightweight Rhai script policy checks during `--check-scenes`
  for banned legacy patterns and oversized scene entrypoints without
  modular imports,
- expose startup helpers used before scene compilation begins.

## Main exports

- `load_mod_manifest()` — load and validate manifest YAML
- `startup::StartupRunner` — orchestrated startup check pipeline
- `startup::StartupReport` — warning/error aggregate for CLI output

## Working with this crate

- keep directory and zip behavior aligned,
- preserve good error reporting because this crate is part of the user-facing startup path,
- keep `--check-scenes` validation deterministic and fast (CI-friendly),
- keep Rhai script policy checks focused and actionable rather than growing
  a generic lint framework,
- if manifest requirements change, update authoring docs and example mods in the same change.

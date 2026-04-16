# playground

Development sandbox mod for experiments.

## Purpose

`playground/` is the mod intended for trying new scenes, renderer ideas,
effects, and authoring experiments without disturbing the main game content.

## Contents

- `mod.yaml` — manifest and runtime defaults
- `assets/` — local experimental assets
- `objects/` — reusable authored objects
- `scenes/` — sandbox scenes
- `schemas/` — generated mod-local schema fragments
- `stages/` — stage definitions

## Typical usage

```bash
SHELL_ENGINE_MOD_SOURCE=mods/playground cargo run -p app
```

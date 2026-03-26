# mods/shell-quest — main game mod

## Purpose

This is the primary playable content mod. It contains the authored scenes,
objects, assets, behaviors, schemas, and sidecar integration content that make
up the Shell Quest experience.

## Directory layout

- `mod.yaml` — mod manifest and terminal/runtime defaults
- `scenes/` — authored scene files and scene packages
- `objects/` — shared authored object fragments
- `behaviors/` — mod-defined behavior definitions
- `assets/` — images, 3D assets, sounds, fonts, and other content
- `schemas/` — mod-local generated schema fragments
- `os/cognitOS/` — C# sidecar project used by terminal/OS scenes

## How to work with this mod

- treat it as the source of truth for real gameplay content and visual style,
- prefer scene packages for complex scenes so layers/effects/objects stay split,
- refresh mod-local schemas after changing scene/effect authoring surfaces,
- use this mod for real gameplay validation, not for automation-only benchmarks,
- keep sidecar-dependent scenes aligned with the IPC contract documented in `engine-io` and the C# sidecar README.

## Important runtime assumptions

- the manifest currently opts into virtual buffer rendering with `virtual_size: max-available`,
- the entrypoint is `/scenes/00-intro-logo/scene.yml`,
- authored scene transitions and intro pacing here are the “real” player-facing versions,
- large 3D assets should be kept compatible with the renderer’s practical face budget.

## Related docs

- root `AUTHORING.md` for authored content rules
- `mods/shell-quest-tests/README.AGENTS.MD` for the benchmark/test variant
- `mods/shell-quest/os/cognitOS/README.md` for sidecar details

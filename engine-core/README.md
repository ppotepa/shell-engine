# engine-core

Shared foundational types for the whole workspace.

## Purpose

`engine-core` is the base layer used by nearly every other crate. It owns the
data types and low-level helpers that should not depend on engine orchestration.

That includes:

- the scene model consumed at runtime,
- resolved scene-level `view-profile` data contracts (`LightingProfile`,
  `SpaceEnvironmentProfile`, `ViewProfile`, `ResolvedViewProfile`),
- buffer and cell types,
- asset source abstractions,
- effect and authoring metadata,
- game object and scene-runtime snapshot types,
- the shared `World` container and access helpers.

## Key Types

- `Scene`, `Layer`, `Sprite` — normalized runtime scene model
- `Buffer`, `Cell` — double-buffered terminal/virtual framebuffer types
- `World` — shared type-indexed resource container
- `GameObject` / `GameObjectKind` — runtime object graph types
- `TargetResolver`, `ObjectRuntimeState`, `RawKeyEvent`, `SidecarIoFrameState` — shared scene-runtime data shapes
- effect metadata and authoring catalogs used by compilers and editors

## Working with this crate

When you add or change a cross-cutting model type, this is usually the right
home. But keep pure data here: orchestration, scene lifecycle flow, behavior
execution, and rendering algorithms belong in higher crates.

If you change scene fields or public runtime types, also update:

- `engine-authoring` compilation and normalization,
- schema generation,
- runtime consumers such as `engine-scene-runtime` and `engine-compositor`,
- editor-facing metadata if authoring surfaces changed.

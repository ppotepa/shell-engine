# engine-3d

Shared 3D scene definitions, resolution, and prerender support.

## Purpose

`engine-3d` owns the data and helper layers needed to work with authored 3D
scene content. It does not own the full compositor pipeline; instead it
provides the reusable 3D-specific pieces that other runtime/rendering crates
consume.

## Key modules

- `scene3d_format` — authored `Scene3DDefinition` structures
- `scene3d_resolve` — asset resolution and reference binding
- `scene3d_atlas` — atlas/cache structures for resolved 3D scenes
- `obj_prerender` — prerender helpers for expensive 3D content
- `obj_frame_cache` — cached prerender frame support

## Main exports

- `Scene3DDefinition`
- `Scene3DAtlas`
- `Scene3DAssetResolver`
- `resolve_scene3d_refs()`

## Working with this crate

- keep authored format and resolution logic separate from renderer orchestration,
- if Scene3D schema or reference rules change, update authoring/schema surfaces too,
- heavy 3D rendering policy still belongs in compositor/runtime crates, not here.

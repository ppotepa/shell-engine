# engine-compositor — composition and rendering internals

## Purpose

`engine-compositor` owns scene composition after the engine has extracted the
resources it needs from `World`. It contains:

- compositor dispatch,
- layer traversal and sprite rendering,
- text/image/OBJ/Scene3D drawing helpers,
- layout helpers for panel/grid/flex content,
- PostFX passes,
- OBJ warmup and prerender helpers,
- Scene3D atlas prerendering.

The engine-side compositor system should stay a thin wrapper around this crate.

## Main modules

- `compositor.rs` — `dispatch_composite()` entry point
- `layer_compositor.rs` / `sprite_renderer.rs` — layer traversal and sprite drawing
- `text_render.rs`, `image_render.rs`, `obj_render.rs` — per-sprite-type rendering
- `obj_loader.rs` — OBJ parsing, `mesh_to_obj_mesh()` conversion, `cube-sphere://N` URI handling
- `layout/` — measurement and placement helpers
- `systems/postfx/` — compiled PostFX pass execution
- `prerender.rs`, `scene3d_prerender.rs`, `warmup.rs` — scene preparation helpers

## Procedural mesh URIs

`get_or_load_obj_mesh` in `obj_render.rs` intercepts URI-scheme paths before
falling back to file loading. Currently supported:

| URI | Generator | Example |
|-----|-----------|---------|
| `cube-sphere://N` | `engine_mesh::primitives::cube_sphere(N)` | `cube-sphere://64` |

Meshes generated this way are converted via `obj_loader::mesh_to_obj_mesh` and
cached in `OBJ_MESH_CACHE` under the URI string. To add a new scheme, add a
branch in `get_or_load_obj_mesh` before the file-load fallback.

## Working with this crate

When changing compositor behavior:

- keep engine/world orchestration outside this crate,
- preserve object region reporting for targeted effects and behavior consumers,
- update measurement helpers together with the renderer that consumes them,
- preserve dirty-region correctness through PostFX.

When changing prerender logic, return data for engine-side scoped registration
rather than mutating `World` directly.

## Invariants

- PostFX must preserve the combined dirty region across pass swaps.
- OBJ prerender and Scene3D atlas generation are optional accelerators, not the
  only render path.

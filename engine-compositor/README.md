# engine-compositor

Scene composition, sprite rendering, layout, PostFX, and prerender helpers.

## Purpose

`engine-compositor` owns the rendering work between scene runtime state and the
final back buffer. It contains:

- scene composition dispatch,
- layer traversal and sprite rendering,
- text, image, OBJ, and Scene3D rendering helpers,
- layout measurement and grid/flex helpers,
- PostFX passes,
- OBJ warmup and prerender helpers,
- Scene3D atlas prerendering.

The engine crate should only extract world resources and call into this crate.

## Key modules

- `compositor` — composition entry points such as `dispatch_composite()`
- `layer_compositor` / `sprite_renderer` — layer traversal and sprite drawing
- `text_render`, `image_render`, `obj_render` — sprite-type-specific renderers
- `obj_loader` — OBJ parsing, `mesh_to_obj_mesh()` conversion, `cube-sphere://N` URI handling
- `layout` — measurement and placement helpers for panel, grid, and flex content
- `systems::postfx` — PostFX pipeline
- `prerender`, `scene3d_prerender`, `warmup` — preparation helpers used before scene activation
- `provider` / `access` — decoupling traits for engine integration

## Procedural mesh URIs

`get_or_load_obj_mesh` intercepts URI-scheme `mesh-source` values before falling
back to file loading. Currently supported:

| URI | Generator |
|-----|-----------|
| `cube-sphere://N` | `engine_mesh::primitives::cube_sphere(N)` |

Generated meshes are converted to `ObjMesh` via `obj_loader::mesh_to_obj_mesh`
and cached in `OBJ_MESH_CACHE` under the URI string.
To add a new scheme add a branch in `get_or_load_obj_mesh` before the file-load fallback.

## Working with this crate

When changing compositor behavior:

- keep world-specific orchestration out of this crate,
- preserve object-region reporting because behaviors and targeted effects depend on it,
- preserve dirty-region information through the full PostFX chain,
- keep prerender helpers pure: return data for engine-side registration instead of mutating `World`.

When changing rendering for a sprite type, update both dimensions/measurement
logic and the actual draw path.

## Important invariants

- PostFX must preserve or expand the combined dirty region rather than
  accidentally narrowing it away.
- OBJ prerender and Scene3D atlas generation are optional accelerators; engine
  wrappers remain responsible for scoped resource registration.

## Integration points

- `engine` calls `dispatch_composite()` from `compositor_system`
- `engine-scene-runtime` supplies object states, target resolver, and camera state
- `engine-asset` and `engine-3d` provide asset loading and 3D scene parsing inputs
- `engine-pipeline` strategy traits select diff/layer behavior

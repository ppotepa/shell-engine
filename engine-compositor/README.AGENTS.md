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

- `compositor.rs` — `dispatch_composite()` and halfblock packing entry points
- `layer_compositor.rs` / `sprite_renderer.rs` — layer traversal and sprite drawing
- `text_render.rs`, `image_render.rs`, `obj_render.rs` — per-sprite-type rendering
- `layout/` — measurement and placement helpers
- `systems/postfx/` — compiled PostFX pass execution
- `prerender.rs`, `scene3d_prerender.rs`, `warmup.rs` — scene preparation helpers

## Working with this crate

When changing compositor behavior:

- keep engine/world orchestration outside this crate,
- preserve object region reporting for targeted effects and behavior consumers,
- update measurement helpers together with the renderer that consumes them,
- verify halfblock packing against virtual-buffer assumptions,
- preserve dirty-region correctness through PostFX.

When changing prerender logic, return data for engine-side scoped registration
rather than mutating `World` directly.

## Invariants

- PostFX must preserve the combined dirty region across pass swaps.
- Halfblock packing must match the renderer’s physical-cell expectations.
- OBJ prerender and Scene3D atlas generation are optional accelerators, not the
  only render path.

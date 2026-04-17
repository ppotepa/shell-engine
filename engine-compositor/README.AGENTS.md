# engine-compositor — frame assembly

## Purpose

`engine-compositor` assembles frame output after the engine has extracted the
resources it needs from `World`.

It should own:

- layer/frame preparation,
- composition entry points,
- adapter-level dispatch into 2D and 3D render crates,
- PostFX application,
- data-oriented prerender orchestration.

It should not own:

- 2D sprite raster internals,
- 3D raster/shading internals,
- generated-world mesh building,
- Scene3D work-item rendering kernels.

## Main modules

- `compositor.rs` — top-level composition entry points
- `scene_compositor.rs` — frame input structs and layer prep
- `prepared_frame.rs` — prepared 2D/3D frame inputs
- `provider.rs` / `access.rs` — engine integration seams
- `prerender.rs` — prerender orchestration surface
- `systems/postfx` — PostFX execution

## Working rules

- keep engine/world orchestration outside this crate,
- keep 2D raster logic in `engine-render-2d`,
- keep 3D raster/prerender logic in `engine-render-3d`,
- preserve object region reporting for targeted effects and behavior consumers,
- preserve dirty-region correctness through the full frame assembly chain.

## Invariants

- PostFX must preserve the combined dirty region across pass swaps.
- Prepared-frame paths must stay equivalent to direct dispatch behavior.
- Prerender outputs are accelerators, not the only render path.

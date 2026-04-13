# engine-render

Shared rendering abstractions and helper utilities.

## Purpose

`engine-render` provides cross-crate rendering primitives that are not tied to
the main engine orchestrator or to a single backend implementation.

It defines the backend-facing traits used to present frames and also hosts
shared helpers used by extracted renderers, including rasterization and asset
loading support.

## Key types and modules

- `RenderBackend` — trait for presenting composed frames
- `OutputBackend` — live runtime backend contract for diffed cell output
- `RenderFrame` — frame payload passed to a backend
- `RenderCaps`, `ColorDepth`, `PresentMode` — backend capability and present semantics
- `rasterizer` — shared text/font rasterization helpers
- `generic` — renderer-agnostic helper utilities
- `image_loader` / `font_loader` — asset loading helpers used by renderer code

## Integration points

- `engine-render-sdl2` implements an SDL2 `OutputBackend`
- `engine-compositor` uses shared rasterization and loader helpers
- the engine runtime presents final buffers through a boxed `OutputBackend`

## Working with this crate

- keep this layer backend-agnostic,
- prefer putting reusable rendering helpers here rather than back into `engine`,
- if a helper requires world access or scene orchestration, it likely belongs in
  `engine-compositor` or `engine`, not here.

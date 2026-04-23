# engine-render

Shared rendering abstractions and helper utilities.

## Purpose

`engine-render` provides cross-crate rendering primitives that are not tied to
the main engine orchestrator or to a single backend implementation.

It defines backend-facing presentation traits for the `software|hardware` split
with a `FrameSubmission`-first contract for renderer submission paths. It also
hosts shared helpers used by extracted renderers, including rasterization and
asset loading support.

## Key types and modules

- `RenderBackend` — legacy software-present trait for composed `Buffer` frames
- `PresentationBackend` — shared backend lifecycle contract
- `SoftwareRendererBackend` — software backend contract
- `HardwareRendererBackend` — hardware backend contract
- `FrameSubmission` — primary backend-neutral frame envelope submitted by runtime renderer paths
- `PreparedWorld`, `PreparedUi`, `PreparedOverlay` — backend-neutral payload parts carried by `FrameSubmission`
- `RenderFrame` — legacy frame payload used by older backend APIs
- `RenderCaps`, `ColorDepth`, `PresentMode` — backend capability and present semantics
- `rasterizer` — shared text/font rasterization helpers
- `generic` — renderer-agnostic helper utilities
- `font_loader` — font asset loading helper used by renderer code (image decode/cache lives in `engine-asset`)

## Integration points

- engine runtime renderer systems submit `FrameSubmission` first through `RendererBackend::submit_frame`
- `HardwareRendererBackend::submit_frame` is the preferred hardware contract entrypoint
- `engine-compositor` uses shared rasterization and loader helpers
- the runtime can select `software` or `hardware` backend paths at startup

## Naming notes

- `RenderBackendKind` is the canonical backend family selector used by runtime startup.
- `RendererBackend` and `RenderBackend` remain in this crate as compatibility-era traits
  for existing software pipeline wiring.
- renderer integration should use `FrameSubmission` as the only hardware-facing submission envelope.

## Working with this crate

- keep this layer backend-agnostic,
- prefer putting reusable rendering helpers here rather than back into `engine`,
- if a helper requires world access or scene orchestration, it likely belongs in
  `engine-compositor` or `engine`, not here.

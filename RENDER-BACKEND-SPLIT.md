# Render Backend Split

> Status: Historical transition document.
>
> Active source of truth for current runtime migration is:
> [wgpu.migration.md](wgpu.migration.md)
>
> This file describes an earlier split strategy where software stayed a
> long-lived fallback. Current migration target is full `winit + wgpu` runtime.

This document lays out a pragmatic software/hardware split for the render stack
without changing the current repo structure up front.

The goal is not to rewrite the engine around a GPU path immediately. The goal is
to separate responsibilities so the existing software renderer keeps working
while a hardware backend can be introduced behind stable seams.

## Current Shape

The present structure already gives us most of the needed boundaries:

- `engine-render` holds shared render traits, frame types, and backend-agnostic helpers.
- `engine-render-2d` owns reusable 2D draw logic.
- `engine-render-3d` owns reusable 3D domain logic, software raster helpers, and 3D effect kernels.
- `engine-compositor` assembles the final frame from layers, sprites, and postfx.
- `engine-render-sdl2` was the SDL2 presentation backend in the legacy path.

That means the software path is already separated enough to be treated as the
baseline implementation. The hardware path should be introduced as a parallel
implementation, not as a rewrite of the current pipeline.

## Recommended Boundaries

Keep the current crates as the software path and add hardware-specific crates
only where they make the ownership clear.

### Shared layer

`engine-render`

- backend traits
- frame and capability types
- present-mode and output metadata
- backend-neutral helpers that both paths can use

### Software layer

`engine-render-2d`

- text, layout, vector, and image rendering helpers
- panel/container primitives
- CPU-side 2D draw logic

`engine-render-3d`

- CPU-side 3D scene and sprite rendering
- software raster helpers
- Scene3D prerender/runtime-store logic
- shading, atmosphere, terrain, and biome kernels

`engine-compositor`

- frame assembly
- layer ordering
- sprite dispatch
- post-processing that still belongs to the CPU path

`engine-render-sdl2`

- software frame presentation
- SDL window/input integration
- clipboard, sizing, and host-side presentation behavior

### Hardware layer

If and when the GPU path is added, use a separate crate boundary for the
hardware-specific work. A good split is:

- `engine-render-hw` for hardware frame contracts, resource lifetime, and GPU
  presentation orchestration
- backend-specific crates such as `engine-render-wgpu` or
  `engine-render-vulkan` if a concrete API needs its own adapter

The important rule is that the hardware layer should not absorb compositor or
scene orchestration concerns. It should consume prepared render data, not own
the scene model.

## What Stays In Software

Keep the following on the software path for now:

- UI and HUD rendering
- text layout and rasterized font output
- debug overlays
- vector overlays
- immediate-mode compositor assembly
- CPU postfx that already operate on the composed buffer
- all existing SDL2 presentation behavior
- CPU 3D raster fallback and any logic that depends on direct buffer access

This is the safest boundary because these systems already depend on the
existing buffer model and on backend-agnostic helpers that are stable today.

## What Moves To Hardware

The hardware path should own work that only makes sense once a GPU backend is
available:

- swapchain and surface management
- GPU resource creation and lifetime tracking
- buffer/texture upload strategy
- shader or pipeline setup
- hardware draw call encoding
- GPU-side post-processing passes
- present scheduling specific to the hardware API

Where possible, the hardware path should consume prepared scene data and render
items rather than reaching back into scene loading or behavior execution.

## Practical Split Rule

Use this rule to decide ownership:

- If the code needs direct access to the engine buffer, keep it in software.
- If the code only needs prepared render inputs, it can be shared or moved to
  hardware.
- If the code knows about windows, surfaces, swapchains, or GPU resources, it
  belongs in hardware-facing crates.
- If the code knows about scenes, stages, behaviors, or mod content, it does
  not belong in the hardware backend.

## Suggested Migration Stages

### Stage 1: Lock The Contract

Keep the current software path as the default and make the backend boundary
explicit.

- keep `engine-render` as the shared contract crate
- keep software rendering in the current crates
- ensure hardware-facing types stay narrow and serializable
- prefer prepared frame inputs over direct scene access

Outcome:

- the engine loop still runs exactly as it does today
- the hardware path can be scaffolded without affecting shipping behavior

### Stage 2: Extract Hardware Plumbing

Introduce the hardware backend crate(s) and move only GPU plumbing there.

- add hardware surface and resource management
- add hardware frame submission APIs
- keep scene composition and CPU raster untouched
- keep software and hardware present paths separate but parallel

Outcome:

- hardware backend code can compile and evolve without forcing compositor churn
- software remains the fallback path

### Stage 3: Move Prepared Inputs

Push more of the draw decision into shared prepared data.

- formalize the minimal render-item contracts consumed by both backends
- move any duplicated frame preflight into shared preparation code
- keep software as the reference implementation for correctness

Outcome:

- the hardware path gets enough structure to draw the same content
- the software path remains the regression baseline

### Stage 4: Split The Hot Path

Move the hot presentation path to hardware where it clearly pays off.

- GPU postfx
- heavy sprite and mesh presentation
- resource upload deduplication
- surface present and frame pacing

Keep software implementations for:

- fallback mode
- diagnostics
- platforms where hardware support is unavailable or undesirable

Outcome:

- the engine gains a genuine hardware path without losing portability
- software rendering remains available as a stable fallback

## Migration Guidelines

- Move one seam at a time.
- Keep the software backend behavior stable until the hardware path proves it can
  match output and timing closely enough.
- Do not push scene semantics into hardware crates.
- Avoid splitting the compositor before you have a replacement for its output
  contract.
- Keep feature flags or startup selection simple while both paths exist.
- Treat software rendering as the reference for image correctness until the
  hardware path has its own regression coverage.

## What Not To Do

- Do not mix scene loading and GPU resource management in the same crate.
- Do not let hardware code reach into mod-specific data structures.
- Do not remove the software path before the hardware path is complete enough to
  support debugging and fallback.
- Do not split the current 2D/3D software crates just to mirror GPU concepts.
  The existing crates already match the current ownership model.

## Summary

The safest split is:

- shared contracts in `engine-render`
- software drawing in `engine-render-2d`, `engine-render-3d`, and
  `engine-compositor`
- presentation in `engine-render-sdl2`
- hardware-specific work in new hardware-facing crate(s) that consume prepared
  render data

That keeps the current engine stable, gives the hardware path a clean boundary,
and avoids dragging scene/runtime concerns into backend code.

## Current Status (2026-04-23)

Crate-by-crate status for the split:
- `engine-render`: `90%` (split traits active; final legacy contract cleanup remains)
- `engine`: `98%` (backend selection and live hardware runtime loop active)
- `engine-render-sdl2`: `95%` (software fallback path stable)
- `engine-compositor`: `90%` (software ownership clear; hardware bypass path active for world render)
- `engine-render-3d`: `90%` (prepared layer active; final backend-neutral packet polish remains)
- `engine-render-2d`: `90%` (software path stable; hardware UI seam still finalizing)
- `engine-runtime`: `95%` (canonical settings flow active; minor compatibility cleanup remains)
- `app`: `95%` (backend switch and canonical size path active)
- `launcher`: `95%` (backend forwarding + canonical size parsing active)
- `editor`: `85%` (migration wording aligned; runtime split work intentionally limited)
- `engine-render-wgpu`: `98%` (live `winit` + `wgpu` runtime/present path active)
- `engine-platform-winit`: `98%` (platform window + live hardware input bridge active)

Overall split progress (rough): **99% complete / 1% remaining**.

## Tranche Details (app + launcher)

App feature split (`software-backend` / `hardware-backend`):
- Runtime backend selection is active via `--render-backend software|hardware`.
- Documentation now tracks compile-time split as the next step; feature-gated build selection is not fully enforced yet.
- Current execution reality: software path is stable fallback; hardware path runs on a live `winit`/`wgpu` loop with hardware input bridge.

Launcher wiring:
- Launcher accepts `--render-backend` and passes it through to app runtime.
- Launcher manifest parsing now reads canonical `display.world_render_size`.
- Legacy compatibility is preserved via alias fallback from `render_size`.

Current tranche focus:
- [x] backend bootstrap extraction from `engine::run`
- [x] renderer contract hook in runtime loop
- [x] explicit renderer branch (`software` vs `hardware`)
- [x] `engine-render-wgpu` scaffold crate

## Cleanup Checklist

- [x] Introduce explicit backend split language (`software|hardware`) in render docs
- [x] Keep software path as default and stable fallback
- [x] Keep backend choice in launcher/runtime, not scene or mod YAML
- [x] Wire app/launcher runtime backend switch end-to-end
- [x] Align launcher parsing to canonical `world_render_size` with legacy alias
- [ ] Add compile-time feature gating in app/runtime (`software-backend`, `hardware-backend`)
- [ ] Move remaining SDL-specific wording behind "software backend" phrasing
- [ ] Remove stale contract references and legacy duplicate abstractions
- [x] Extract backend bootstrap from `engine::run`
- [x] Hook renderer contract into runtime loop (`software` and `hardware`)
- [x] Split renderer branch in hot path
- [x] Land hardware bootstrap crates (`engine-platform-winit`, `engine-render-wgpu`)
- [x] Separate world render from software compositor in hardware path
- [ ] Final contract cleanup for backend-neutral frame packet interfaces

## Tranche Checklist (Sync)

- [x] Backend bootstrap extraction
- [x] Renderer contract hook
- [x] Renderer branch split
- [x] `engine-render-wgpu` scaffold crate

## Explicit Blockers: True `winit+wgpu` Integration

1. Finalize backend-neutral render packet contract between scene preparation and GPU submission.
2. Close warning/deprecation cleanup in hardware runtime modules.
3. Finalize long-term overlay/HUD parity path across both backends.

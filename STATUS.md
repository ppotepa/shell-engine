# STATUS

Last update: 2026-04-23
Owner: Codex session

## Current Progress

- This status is tracked by binary migration gates (DONE / NOT DONE / NOT VERIFIED), not percentages.
- Completed in latest wave:
  - `app` default feature switched to `hardware-backend`.
  - `engine` default feature switched to `hardware-backend`.
  - CLI default backend is `hardware`.
  - SDL2 runtime path is removed from active/default wiring (`app` + `launcher` launch hardware unless software is explicitly requested).
  - K1 follow-up: explicit `StartupOutputSetting::Sdl2` variant usage is removed from active code paths; startup scene checks now use `StartupOutputSetting::Compatibility`.
  - `FrameSubmission` seam remains active in runtime (`submit_frame` first, software fallback on submit failure).
- Remaining core blockers:
  - SDL2/software compatibility gates remain (`software-backend` wiring, `cfg(feature = "sdl2")` compatibility code, and legacy alias parsing such as `"sdl2"` in startup token parsing).
  - runtime/compositor still carries `is_pixel_backend` compatibility branches.

## Validation Snapshot (2026-04-23)

- `app` tests: GREEN (`cargo test -p app` on 2026-04-23).
- `launcher` tests: GREEN (`cargo test -p launcher` on 2026-04-23).
- `engine-runtime` tests: GREEN (`cargo test -p engine-runtime` on 2026-04-23).
- `engine-render-wgpu` tests: GREEN (`cargo test -p engine-render-wgpu` on 2026-04-23).
- `engine` library tests (`cargo test -p engine --lib`): GREEN (validated passing on 2026-04-23).

Gate impact (dated):
- migration test gate is satisfied on 2026-04-23 for current library targets, including `cargo test -p engine --lib`.

Targeted blocker validations (2026-04-23):
- launcher SDL helper removal: `rg -n "SDL2_LIB_DIR|SDL2_INCLUDE_DIR|SDL2\\.dll|sdl2-config|inject_sdl2_rustflags|ensure_sdl2_dll" launcher/src` returns no helper-code matches.
- explicit `StartupOutputSetting::Sdl2` variant usage: `rg -n -F "StartupOutputSetting::Sdl2" app/src engine/src launcher/src engine-mod/src` returns no matches.
- workspace membership check: `rg -n "engine-render-sdl2" --glob "Cargo.toml"` matches only `engine-render-sdl2/Cargo.toml` crate metadata and an `engine/Cargo.toml` comment (no root workspace member entry).

## H4 Engine-Lib Failure Buckets (H1/H2/H3)

Closure rule:
- Do not mark any bucket as fixed until validated by main branch `cargo test -p engine --lib`.

| Bucket | Current status | Closure status |
|---|---|---|
| H1 | CLOSED (validated by `cargo test -p engine --lib` on 2026-04-23) | CLOSED |
| H2 | CLOSED (validated by `cargo test -p engine --lib` on 2026-04-23) | CLOSED |
| H3 | CLOSED (validated by `cargo test -p engine --lib` on 2026-04-23) | CLOSED |

## Source Of Truth Scope

This file is only a quick status snapshot.
Canonical migration scope, phases, and exit criteria live in:
- [wgpu.migration.md](wgpu.migration.md)

If there is any conflict between this file and legacy roadmap docs, `wgpu.migration.md`
takes precedence.

## Milestones (WGPU Scope)

1. Done: initial hardware path bootstrapped
- backend selection wired (`software|hardware`)
- live `winit` window runtime exists
- hardware input bridge exists

2. Done: submission seam introduced (compatibility stage)
- `PreparedWorld` / `PreparedUi` / `PreparedOverlay` / `FrameSubmission` are present in `engine-render`
- runtime submits `FrameSubmission` first
- fallback to `present_frame` remains active when hardware submit fails

3. Done: hardware-first defaults landed
- `app/Cargo.toml` default is `hardware-backend`
- `engine/Cargo.toml` default is `hardware-backend`
- app CLI `--render-backend` default is `hardware`

4. In progress: hardware renderer contract cleanup
- keep submit path intact while removing compatibility-only `HardwareFrame` bridging from active hot paths

5. In progress: full SDL2 compatibility removal gates
- remove remaining SDL2/software compatibility branches/tooling and `is_pixel_backend` fallback paths

## Active Blockers

- `engine-render` still carries compatibility conversion (`FrameSubmission <-> HardwareFrame`).
- runtime/compositor still depends on `is_pixel_backend` compatibility state.
- SDL2/software compatibility wiring is still present (`software-backend`, `cfg(feature = "sdl2")`, legacy alias parsing tokens).

## Next Execution Batch

1. M1 Contract adoption cutover
- acceptance: `cargo check -p engine -p engine-render` and no new runtime-side `HardwareFrame` construction.
2. M2 Native wgpu present path
- acceptance: `engine-render-wgpu` consumes `FrameSubmission` directly in runtime path without re-materializing `HardwareFrame`, and `cargo check -p engine-render-wgpu` passes.
3. M3 Runtime compatibility cleanup
- acceptance: no SDL2/software compatibility branches remain in active runtime path (`is_pixel_backend`, `cfg(feature = "sdl2")`, and legacy software-compat wiring removed from hot paths).

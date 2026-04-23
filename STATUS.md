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
  - `software-backend` name usage is removed from active non-doc code paths (`rg -n --hidden --glob '!target/**' --glob '!**/*.md' "software-backend" .` returns no matches).
  - K1 follow-up: explicit `StartupOutputSetting::Sdl2` variant usage is removed from active code paths; startup scene checks now use `StartupOutputSetting::Compatibility`.
  - K2 follow-up: legacy startup output aliases are removed (`engine-mod/src/output_backend.rs` asserts `StartupOutputSetting::parse("sdl2") == None`).
  - Deprecated `--sdl-window-ratio` / `--sdl-pixel-scale` / `--no-sdl-vsync` aliases are removed from active CLI code paths (`rg -n "sdl-window-ratio|sdl-pixel-scale|no-sdl-vsync" app/src launcher/src` returns no matches).
  - `is_pixel_backend` compatibility usage is removed from active runtime/policy code paths (`rg -n "is_pixel_backend" engine-runtime/src engine-render-policy/src` returns no matches).
  - Compat-only runtime flags are active in `app` (`rg -n "compat-" app/src launcher/src` matches `app/src/main.rs` for `--compat-window-ratio`, `--compat-pixel-scale`, `--no-compat-vsync`).
  - `FrameSubmission` seam remains active in runtime (`submit_frame` first, software fallback on submit failure).
- Remaining core blockers:
  - no active renderer bridge blocker remains for `FrameSubmission` contract adoption.

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
- startup alias removal evidence: `rg -n -F 'parse("sdl2")' engine-mod/src/output_backend.rs` shows test assertion `StartupOutputSetting::parse("sdl2") == None`.
- no active `sdl2` feature-gate wiring in runtime/app/launcher code: `rg -n -F 'feature = "sdl2"' engine/src app/src launcher/src engine-mod/src engine/Cargo.toml app/Cargo.toml launcher/Cargo.toml` returns no matches.
- active non-doc `software-backend` usage: `rg -n --hidden --glob '!target/**' --glob '!**/*.md' "software-backend" .` returns no matches.
- active `is_pixel_backend` usage in runtime/policy paths: `rg -n "is_pixel_backend" engine-runtime/src engine-render-policy/src` returns no matches.
- active engine renderer path uses `FrameSubmission` directly (`engine/src/systems/renderer.rs`).
- renderer crates no longer contain `HardwareFrame` / `present_hardware_frame` bridge symbols: `rg -n "HardwareFrame|present_hardware_frame\\(" engine-render/src/lib.rs engine-render-wgpu/src/lib.rs` returns no matches.
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

4. Done: hardware renderer contract cleanup
- submit path is primary (`FrameSubmission`) and legacy `HardwareFrame` bridge symbols are removed from active renderer crates

5. Done: active SDL2 compatibility removal gates
- deprecated `--sdl-*` CLI aliases removed from active `app`/`launcher` code paths
- `is_pixel_backend` compatibility usage removed from active `engine-runtime`/`engine-render-policy` paths

## Active Blockers

- no active blockers in renderer contract cleanup lane; remaining migration closure items are outside this bridge-removal scope (verification artifacts/perf gates).

## Next Execution Batch

1. M1 Contract adoption cutover (DONE)
- acceptance: `cargo check -p engine -p engine-render` and no new runtime-side `HardwareFrame` construction.
2. M2 Native wgpu present path (DONE)
- acceptance met: no `HardwareFrame` / `present_hardware_frame` symbols in active `engine-render` + `engine-render-wgpu` paths; `cargo check -p engine-render -p engine-render-wgpu` passes.
3. M3 Runtime compatibility cleanup (DONE)
- acceptance met: `rg -n "is_pixel_backend" engine-runtime/src engine-render-policy/src` returns no matches; `rg -n "sdl-window-ratio|sdl-pixel-scale|no-sdl-vsync" app/src launcher/src` returns no matches.

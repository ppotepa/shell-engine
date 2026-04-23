# WGPU Migration Plan

## Purpose

This document is the source of truth for the current migration scope:

- target runtime: `winit + wgpu`
- target presentation path: GPU-first
- target outcome: remove SDL2 from the active runtime path
- target architecture for this phase: backend-neutral seams required for a real hardware runtime

This document is intentionally narrow. Wider refactor topics (full plugin model,
Unity-like workflow expansion, broad repo productization) are deferred until the
runtime cutover is complete and stable.

## Scope

## In Scope

1. Full `winit + wgpu` runtime path.
2. Windowing, input, surface, present, resize, and cursor handling.
3. Replacing software-era renderer assumptions where they block hardware.
4. SDL2 removal from active runtime, build defaults, launcher flow, and docs.
5. Crate-boundary cleanup only where required by items 1-4.

## Out of Scope

1. Full plugin architecture and native dynamic plugin ABI.
2. Full Unity-like authoring/editor workflow.
3. Broad gameplay/ECS redesign not required by runtime cutover.
4. Generic cleanup not needed for `winit+wgpu` migration.

## Current Reality

Verified snapshot (2026-04-23), based on current workspace code:

| Area | Status | Notes |
|---|---|---|
| Runtime defaults (`app`, `engine`) | DONE | both crates default to `hardware-backend` |
| CLI backend default | DONE | `--render-backend` defaults to `hardware` |
| Backend-neutral submission seam | DONE | `PreparedWorld` / `PreparedUi` / `PreparedOverlay` / `FrameSubmission` exist and are used by engine renderer |
| Engine present flow | DONE (compat mode) | engine submits `FrameSubmission` first, then falls back to software `present_frame` on submit failure |
| Platform runtime crate split | IN_PROGRESS | `engine-platform-winit` has lifecycle/cursor/event translation runtime helpers and tests |
| Hardware presenter implementation | IN_PROGRESS | `engine-render-wgpu` no longer depends on `pixels`, but still uses compatibility `HardwareFrame` bridging |
| SDL2 detach from optional paths | IN_PROGRESS | SDL2 runtime bootstrap is removed from active/default wiring; optional software compatibility paths remain |
| Launcher SDL tooling removal | DONE | validated by grep: no SDL helper setup symbols remain in `launcher/src` (`SDL2_LIB_DIR`, `SDL2_INCLUDE_DIR`, DLL copy, rustflags injection helpers) |

Completion for final SDL2 compatibility removal is tracked only by binary checklist gates below (not percentages).

## Validation Snapshot (2026-04-23)

Recorded validation status for this migration checkpoint:

| Scope | Status | Date | Notes |
|---|---|---|---|
| `app` tests | GREEN | 2026-04-23 | `cargo test -p app` passed |
| `launcher` tests | GREEN | 2026-04-23 | `cargo test -p launcher` passed |
| `engine-runtime` tests | GREEN | 2026-04-23 | `cargo test -p engine-runtime` passed |
| `engine-render-wgpu` tests | GREEN | 2026-04-23 | `cargo test -p engine-render-wgpu` passed |
| `engine` library tests (`cargo test -p engine --lib`) | GREEN | 2026-04-23 | `cargo test -p engine --lib` passed |

## H4 Engine-Lib Failure Buckets (H1/H2/H3)

Closure rule:
- Do not mark any bucket fixed until main branch `cargo test -p engine --lib` validates it.

| Bucket | Current status | Closure status |
|---|---|---|
| H1 | CLOSED (validated by `cargo test -p engine --lib` on 2026-04-23) | CLOSED |
| H2 | CLOSED (validated by `cargo test -p engine --lib` on 2026-04-23) | CLOSED |
| H3 | CLOSED (validated by `cargo test -p engine --lib` on 2026-04-23) | CLOSED |

## Audit Snapshot

Landed:
- `app` default feature is now `hardware-backend`.
- `engine` default feature is now `hardware-backend`.
- app CLI default backend is `hardware`.
- `engine-render-wgpu` exists and runs a live window runtime.
- `app` and `launcher` support runtime backend selection.
- `engine` has backend bootstrap branching.
- `engine-platform-winit` exists as a seed crate.
- `engine-render` exports `PreparedWorld`, `PreparedUi`, `PreparedOverlay`, and `FrameSubmission`.
- `engine/src/systems/renderer.rs` now submits `FrameSubmission` first and falls back to `present_frame` on submit failure.

Still blocking completion:
- `engine-render-wgpu` still relies on compatibility conversion (`FrameSubmission <-> HardwareFrame`) in the active path.
- `engine-render` still keeps compatibility conversion (`FrameSubmission <-> HardwareFrame`) and `Buffer`-centric legacy traits.
- `engine-runtime` / compositor flow still carries `is_pixel_backend` compatibility branches.
- SDL/software compatibility surfaces still exist (`software-backend` wiring, `cfg(feature = "sdl2")` compatibility code, and legacy startup alias parsing for `"sdl2"`/`"sdl"`).

## Architectural Rules

1. `winit` owns platform runtime concerns only.
2. `wgpu` owns rendering only.
3. `engine` communicates through traits and prepared packets, not `wgpu` types.
4. `Buffer` is not a universal frame contract.
5. scene/gameplay/authoring crates remain backend-agnostic.
6. every migration shim must have an explicit deletion step.

## End-State Shape

```text
mods/scenes/assets
        |
        v
authoring/mod/asset crates
        |
        v
scene/gameplay/runtime crates
        |
        v
render extraction
        |
        +--> PreparedWorld / PreparedUi / PreparedOverlay
        |
        v
hardware renderer trait
        |
        v
engine-render-wgpu
        |
        v
engine-platform-winit + native window
```

## Phase Plan

### P0 - Scope Lock + Baseline Freeze

Goal:
- lock scope to `winit+wgpu` only
- create measurable baseline before deep refactor

Changes:
- freeze this document as canonical runtime migration source
- mark SDL2 as transition-only and removal target
- capture baseline metrics: startup time, avg frame ms, p95 frame ms, resize latency

Exit criteria:
- no new out-of-scope tasks in active migration tracker
- baseline artifact exists and is linked from this document

Validation:
- `cargo check -p app -p engine -p engine-render -p engine-render-wgpu -p engine-platform-winit`

### P1 - Backend Contracts Hard Split

Goal:
- separate hardware contracts from software `Buffer` assumptions

Changes:
- refactor `engine-render` to one coherent hardware-capable contract set
- add neutral payloads: `PreparedWorld`, `PreparedUi`, `PreparedOverlay`, `FrameSubmission`
- degrade `Buffer` to temporary software-compat layer only

Exit criteria:
- `engine` talks to renderer only through new contracts
- `engine-render-wgpu` no longer requires `Buffer` as frame input

Validation:
- `cargo check -p engine-render -p engine -p engine-render-wgpu`

### P2 - Platform Runtime In `engine-platform-winit`

Goal:
- move platform lifecycle ownership out of renderer crate

Changes:
- implement real window/event runtime in `engine-platform-winit`
- support focus, resize, cursor lock/hide, relative mouse
- standardize event translation into `engine-events`

Exit criteria:
- `engine-platform-winit` is the sole owner of event loop/window policy
- renderer no longer owns platform lifecycle

Validation:
- `cargo check -p engine-platform-winit -p engine-events -p engine`

### P3 - Real WGPU Renderer (No `pixels`)

Goal:
- replace presenter shim with full `wgpu` renderer lifecycle

Changes:
- remove `pixels` dependency
- implement adapter/device/queue/surface lifecycle
- handle surface loss/outdated/resize without process restart

Exit criteria:
- `engine-render-wgpu` renders/presents without `pixels`
- no software fallback in hardware hot path

Validation:
- `cargo check -p engine-render-wgpu`
- renderer integration tests: init/resize/present/surface-loss recovery

### P4 - World Extraction To Neutral Packet

Goal:
- separate 3D world preparation from software raster output

Changes:
- `engine-render-3d` emits neutral world packet (`PreparedWorld v1`)
- include camera, visible mesh refs, materials, lights, terrain chunks
- feed packet directly into hardware renderer

Exit criteria:
- world submission path is `engine -> PreparedWorld -> engine-render-wgpu`
- no `Buffer` dependency in world packet

Validation:
- `cargo check -p engine-render-3d -p engine-render-wgpu -p engine`

### P5 - UI/HUD/Debug Packet Path

Goal:
- remove remaining software dependence for overlays/HUD/debug

Changes:
- replace `is_pixel_backend` branches with capability/profile-driven behavior
- add `PreparedUi` and `PreparedOverlay`
- migrate HUD/debug presentation to hardware path

Exit criteria:
- HUD/debug operate on hardware runtime
- no global branching by `is_pixel_backend`

Validation:
- `cargo check -p engine-runtime -p engine-render-2d -p engine`

### P6 - SDL2 Runtime/Build/Launcher Removal

Goal:
- complete runtime cutover

Changes:
- remove SDL2 defaults from `engine` and `app`
- remove `engine-render-sdl2` from active runtime/workspace path
- remove SDL2 setup/doctor/copy-DLL launcher flows
- update docs to runtime reality

Exit criteria:
- default `cargo run -p app` boots `winit+wgpu`
- build/run no longer require SDL2 environment

Validation:
- `cargo check -p app -p engine -p launcher` in environment without SDL2 dev libs

### P7 - Post-Cutover Cleanup + Hardening

Goal:
- remove migration debt left by cutover

Changes:
- delete dead compatibility shims and backend aliases
- simplify bootstrap to one primary runtime path
- finalize naming/docs consistency

Exit criteria:
- no active temporary migration shim in runtime hot path
- docs and runtime behavior are consistent

Validation:
- workspace checks + regression benchmark vs P0 baseline

## Crate-By-Crate Backlog

| Crate | Status | Required work (1 line) |
|---|---|---|
| `engine` | `IN_PROGRESS` | Keep hardware-default path, then remove remaining SDL2-gated bootstrap/startup compatibility and `is_pixel_backend` assumptions. |
| `engine-render` | `IN_PROGRESS` | Finalize neutral renderer contracts and replace `HardwareFrame` placeholder with real packet contracts. |
| `engine-render-wgpu` | `IN_PROGRESS` | Keep hardware window runtime path, then remove compatibility `HardwareFrame` bridging and finish native GPU lifecycle ownership. |
| `engine-platform-winit` | `IN_PROGRESS` | Promote lifecycle/event translation helpers into sole platform runtime owner used by engine runtime. |
| `engine-runtime` | `IN_PROGRESS` | Complete capability-descriptor transition and remove `is_pixel_backend` compatibility usage in hot paths. |
| `engine-render-3d` | `NOT_STARTED` | Move from `Render3dOutput { color: Buffer }` shape to backend-neutral world packet output. |
| `engine-render-2d` | `NOT_STARTED` | Stop direct `Buffer`/pixel-backend assumptions and emit neutral UI packet data for hardware backend. |
| `launcher` | `IN_PROGRESS` | Remove SDL-specific setup/linking/env/doctor flows and keep backend choice as runtime switch only. |
| `app` | `IN_PROGRESS` | Keep hardware default and prune remaining software-compat CLI alias surface where no longer needed. |
| `engine-mod` | `IN_PROGRESS` | Keep backend-neutral `StartupOutputSetting::Compatibility`; legacy SDL alias parsing (`"sdl2"`/`"sdl"`) is still present. |
| `engine-core` | `BLOCKER` | Isolate/remove SDL-era `PixelCanvas` semantics so core types stop forcing software pixel model globally. |

Dependency corrections relevant to migration:
- `engine` now has `default = ["hardware-backend"]` (SDL2 remains available via optional `software-backend`/`sdl2` features).
- `app` now has `default = ["hardware-backend"]`.
- `engine-render-wgpu` depends on `winit` (no `pixels` dependency).
- `engine-platform-winit` now contains runtime lifecycle/event translation helpers with tests, but is not yet the sole platform lifecycle owner.
- `app` scene checks now request `StartupOutputSetting::compatibility_default()`; explicit `StartupOutputSetting::Sdl2` variant usage is not present in active code.
- `engine-mod` keeps legacy startup string aliases (`"sdl2"`/`"sdl"`) mapped to `StartupOutputSetting::Compatibility`.
- `launcher` SDL-specific setup/linker/doctor helper flows were removed; launcher still retains backend-selection UX and software-backend feature arguments for compatibility/deprecation messaging.
- workspace no longer includes `engine-render-sdl2` as a member; the crate directory still exists.

## SDL2 Removal Checklist (Binary, Testable)

Status rule:
- `DONE` only if verification passes exactly.
- otherwise `NOT DONE`.

| ID | Item | Verification | Done condition |
|---|---|---|---|
| SDL-00 | remove SDL2 from active/default runtime wiring | inspect `engine/src/backend_bootstrap.rs`, `app/Cargo.toml`, `launcher/src/cli.rs` | default runtime path routes to hardware only; software requires explicit opt-in |
| SDL-01 | remove `engine-render-sdl2` from workspace | `rg -n "engine-render-sdl2" Cargo.toml` | no workspace member entry |
| SDL-02 | remove SDL2 runtime deps | `rg -n "\\bsdl2\\b" engine app launcher engine-render* */Cargo.toml` | no `sdl2` dep in active runtime crates |
| SDL-03 | remove SDL2 defaults in features | inspect `engine/Cargo.toml`, `app/Cargo.toml` | defaults no longer include SDL/software backend |
| SDL-04 | remove SDL2 runtime branches from active path | `rg -n "sdl2|software-backend" engine/src app/src launcher/src` + active bootstrap review | no default/active runtime branch routes through SDL2 |
| SDL-05 | remove SDL2 launcher setup flows | `rg -n "SDL2_LIB_DIR|SDL2_INCLUDE_DIR|SDL2.dll" launcher/src` | no SDL setup/copy/linker logic |
| SDL-06 | remove SDL2 operational docs | `rg -n "SDL2|sdl2" README.md ARCHITECTURE.md AUTHORING.md STATUS.md` | no doc implies SDL2 required |
| SDL-07 | hardware default run works | `cargo run -p app` | app boots via `winit+wgpu` without SDL2 env |
| SDL-08 | clean build without SDL2 libs | `cargo check --workspace` in clean env | check passes |
| SDL-09 | remove remaining SDL2 compatibility code | `rg -n "feature = \\\"sdl2\\\"|software-backend|is_pixel_backend|\\\"sdl2\\\"|\\\"sdl\\\"" engine/src app/src launcher/src engine-mod/src` | no SDL2 compatibility branches/tokens remain |

### Checklist Snapshot (2026-04-23)

| ID | Status | Evidence |
|---|---|---|
| SDL-00 | DONE | default feature/backend selections and `backend_bootstrap` route active runtime to hardware path |
| SDL-01 | DONE | root `Cargo.toml` workspace members do not include `engine-render-sdl2` |
| SDL-02 | NOT DONE | active crates still contain SDL2/software feature wiring |
| SDL-03 | DONE | `app/Cargo.toml` and `engine/Cargo.toml` both default to `hardware-backend` |
| SDL-04 | DONE | active/default runtime bootstrap no longer routes through SDL2; software backend is explicit compatibility path |
| SDL-05 | DONE | `rg -n "SDL2_LIB_DIR|SDL2_INCLUDE_DIR|SDL2\\.dll|sdl2-config|inject_sdl2_rustflags|ensure_sdl2_dll" launcher/src` has no helper-code matches |
| SDL-06 | NOT DONE | docs still include SDL2 runtime guidance in active docs |
| SDL-07 | NOT VERIFIED | defaults are hardware-first, but an explicit `cargo run -p app` verification result is not recorded in this document |
| SDL-08 | NOT VERIFIED | clean-room build without SDL2 libs has not been recorded as passing |
| SDL-09 | NOT DONE | SDL2 compatibility code remains in `engine`/`app`/`launcher`/`engine-mod` (`feature = "sdl2"`, `software-backend`, `is_pixel_backend`, legacy `"sdl2"` alias parsing) |

### Validation Gate Snapshot (2026-04-23)

| Gate | Status | Date | Evidence note |
|---|---|---|---|
| crate test gate: `app` | DONE | 2026-04-23 | `cargo test -p app` passed |
| crate test gate: `launcher` | DONE | 2026-04-23 | `cargo test -p launcher` passed |
| crate test gate: `engine-runtime` | DONE | 2026-04-23 | `cargo test -p engine-runtime` passed |
| crate test gate: `engine-render-wgpu` | DONE | 2026-04-23 | `cargo test -p engine-render-wgpu` passed |
| crate test gate: `engine --lib` | DONE | 2026-04-23 | `cargo test -p engine --lib` passed |

## Performance Gates (Binary, Testable)

Fail-fast:
- missing measurement artifact = `NOT DONE`.

Required benchmark artifact:
- `benchmarks/wgpu-migration/<phase>/report.json`
- fields: `avg_fps`, `p95_frame_ms`, `p99_frame_ms`, `cpu_copy_mb_s`, `resize_recover_ms`, `surface_loss_recover_ms`

| ID | Gate | Verification | Pass threshold |
|---|---|---|---|
| PERF-01 | baseline exists per phase | report exists for `baseline` and current phase | both exist |
| PERF-02 | FPS regression bounded | compare current vs baseline `avg_fps` | `current >= 0.90 * baseline` |
| PERF-03 | frame-time regression bounded | compare `p95_frame_ms` | `current <= 1.15 * baseline` |
| PERF-04 | tail latency bounded | compare `p99_frame_ms` | `current <= 1.20 * baseline` |
| PERF-05 | no software copy in hardware hot path | instrument `cpu_copy_mb_s` | `cpu_copy_mb_s == 0` steady-state world render |
| PERF-06 | resize recovery bounded | resize storm test | `resize_recover_ms <= 250` |
| PERF-07 | surface-loss recovery bounded | forced surface-loss test | `surface_loss_recover_ms <= 500` |
| PERF-08 | no unbounded alloc churn | allocation telemetry over 60s | no monotonic growth after warmup |
| PERF-09 | input-to-photon sanity | latency capture | median latency regression <= 20% vs baseline |

Stop-ship:
- any failed `PERF-*` gate blocks phase closure.

## Definition Of Done (Binary Release Gate)

All items must be `DONE` simultaneously:

| ID | Requirement | Verification | Done condition |
|---|---|---|---|
| DOD-01 | default runtime is `winit+wgpu` | `cargo run -p app` + startup log | no SDL/software runtime selected |
| DOD-02 | hardware path is GPU-native | code/runtime trace | no software presenter on world render path |
| DOD-03 | backend-neutral engine boundary | API review | engine-facing traits do not expose `wgpu`/`winit` types |
| DOD-04 | platform/render separation | dependency/API review | platform lifecycle in `engine-platform-winit`, GPU rendering in `engine-render-wgpu` |
| DOD-05 | SDL2 removed from build+launch | workspace checks + launcher run | no SDL2 deps/env/setup required |
| DOD-06 | UI/HUD/debug parity | parity checklist + tests | core overlay/HUD functionality works on hardware path |
| DOD-07 | FPS input parity | acceptance tests | WASD + freelook + cursor lock/hide stable |
| DOD-08 | docs/runtime consistent | doc check | docs describe production runtime as `winit+wgpu` |
| DOD-09 | migration tests green | required test suite | all pass |
| DOD-10 | performance gates green | PERF-01..09 | all pass |

### DOD Snapshot (2026-04-23)

| DOD ID | Status | Date | Note |
|---|---|---|---|
| DOD-09 | NOT DONE | 2026-04-23 | crate-level gates listed in this document are green, but full migration-suite closure is still gated by remaining SDL2 compatibility and DOD items |

## Risks (Binary Tracking)

Status:
- `OPEN` when trigger true and mitigation evidence missing
- `CLOSED` when trigger false or mitigation evidence validated

| Risk ID | Risk | Trigger | Mitigation evidence | Close condition |
|---|---|---|---|---|
| R-01 | false progress reporting | status claims without DOD evidence | each status update includes DOD snapshot | DOD snapshot present in all status reports |
| R-02 | renderer/platform coupling relapse | renderer owns event loop/window policy | API review shows platform APIs only in platform crate | ownership split validated |
| R-03 | `Buffer` leakage into hardware contracts | hardware contract references `Buffer` | interface checks compile without `Buffer` in hardware path | no `Buffer` in engine-facing hardware contracts |
| R-04 | SDL2 reintroduction | new SDL deps/flags/docs appear | CI grep guard for SDL tokens | guard active and green |
| R-05 | UI/debug parity lag | missing overlay/HUD on hardware | parity matrix + tests | parity matrix all green |
| R-06 | FPS input regressions | freelook/cursor tests fail | automated acceptance tests for mouse capture + WASD | tests stable across target platforms |
| R-07 | hidden perf regressions | no benchmark artifact | required phase report + PERF gates | PERF-01..09 pass |
| R-08 | docs/runtime drift | docs and runtime behavior diverge | doc consistency check in CI | check enforced and green |

## Suggested Execution Order

1. run **P0** immediately and freeze scope + baseline
2. run **P1** as architecture gate before deeper implementation
3. run **P2** and **P3** in parallel with one sync point:
   - P2 finalizes platform/event contract
   - P3 consumes that contract in renderer
4. run **P4** after P3 has stable present/resize/surface-loss
5. run **P5** in parallel with late P4 where possible
6. run **P6** only after P4+P5 functional parity
7. run **P7** as mandatory closure stage

## Immediate Next Tasks

1. prune remaining SDL/software compatibility wiring in active runtime crates (`software-backend`, `cfg(feature = "sdl2")`, `is_pixel_backend` branches).
2. decide and execute removal timing for legacy startup alias parsing (`"sdl2"`/`"sdl"`) in `engine-mod`.
3. remove `runtime_settings.is_pixel_backend` compatibility dependency from runtime/compositor/renderer paths.
4. remove remaining `FrameSubmission <-> HardwareFrame` compatibility bridge usage from active render path contracts.
5. record verification artifacts for `cargo run -p app` hardware default boot and clean-room `cargo check --workspace` without SDL2 libs.

## Next Actionable Milestones

### M1 - Contract Adoption Cutover

Goal:
- make `FrameSubmission` the required engine-to-renderer path for hardware runtime.

Acceptance criteria:
- `engine/src/systems/renderer.rs` submits through `submit_frame` without creating compatibility-only payloads in system code.
- compatibility mapping stays isolated inside `engine-render` trait defaults only.
- `cargo check -p engine -p engine-render` passes.

### M2 - WGPU Native Present Path

Goal:
- finish native GPU presenter ownership in `engine-render-wgpu` without compatibility bridge payload re-materialization.

Acceptance criteria:
- `engine-render-wgpu` runtime path consumes `FrameSubmission` directly without re-materializing `HardwareFrame`.
- presenter still accepts engine submissions and presents frames after resize.
- `cargo check -p engine-render-wgpu` passes and resize smoke test succeeds.

### M3 - Runtime Compatibility Cleanup

Goal:
- complete removal of remaining runtime compatibility shims tied to SDL/software legacy paths.

Acceptance criteria:
- `app` default feature is hardware path (not `software-backend`).
- explicit `StartupOutputSetting::Sdl2` variant usage is absent in active code paths.
- `cargo check -p app` passes.

## Deferred Follow-Up (After This Migration)

Not tracked in this file for now:
- full plugin taxonomy and registry system
- broader service interface model
- full asset-handle generalization
- Unity-like workflow expansion
- large repo cleanup beyond runtime migration needs

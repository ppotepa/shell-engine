# Jira Handoff: Final 3D Refactor Closure

## Ticket

- Title: Finalize 2D/3D ownership split and remove remaining compatibility command debt
- Type: Epic / implementation handoff
- Priority: High
- Status: Ready for next agent

## Why this still exists

The repo is close to the intended end state, but not there yet.

What is already true:

- `engine-render-2d` owns 2D rendering.
- Most 3D typed scene graph and prerender ownership already sits in `engine-render-3d`.
- A large part of mutation handling already flows through typed `SceneMutationRequest -> SceneMutation`.
- Internal gameplay/emitter producers have already been moved off many old command paths.

What is not yet true:

- `engine-compositor` still owns one last meaningful piece of 3D domain render logic.
- Old command compatibility shapes still exist in engine-facing paths.
- `SetProperty` is still used as a fallback in places where typed requests are incomplete or lack runtime state.

This means the architectural intent is visible in the codebase, but the final boundaries are not fully enforced yet.

## Target outcome

After this ticket is done, the repo should satisfy all of these:

- `engine-compositor` only assembles frame output and does not own domain 3D render logic.
- Scene3D prerender no longer depends on a compositor-owned callback.
- Old command shapes `SceneSpawn` / `SceneDespawn` are removed or reduced to an explicit short-lived shim with no active producers.
- Typed mutation flow is the default for all supported script/API paths.
- Remaining `SetProperty` fallback is either:
  - removed, or
  - explicitly limited to low-level unsupported raw command usage, not normal engine API usage.

## Scope

This ticket covers only the remaining unfinished architectural work.

It does **not** include:

- new gameplay features,
- new atmosphere visuals,
- performance tuning beyond what is required by the refactor,
- changes to mod content unless tests/fixtures need updating.

## Remaining workstreams

### 1. Remove the last compositor-owned Scene3D prerender callback

This is the highest-value remaining architectural seam.

#### Current problem

`engine/src/systems/scene3d_prerender.rs` currently calls:

- `engine_compositor::render_scene3d_work_item`

That function is currently defined in:

- `engine-compositor/src/scene_clip_render_adapter.rs`

and re-exported from:

- `engine-compositor/src/lib.rs`

This means `engine-compositor` still owns part of the Scene3D prerender execution path, which violates the target architecture.

#### Files involved

- `engine/src/systems/scene3d_prerender.rs`
- `engine-compositor/src/scene_clip_render_adapter.rs`
- `engine-compositor/src/lib.rs`
- `engine-render-3d/src/prerender/scene3d_prerender.rs`
- `engine-render-3d/src/prerender/render_item.rs`
- `engine-render-3d/src/pipeline/renderer.rs`
- likely also `engine-compositor/src/obj_render.rs`
- likely also `engine-compositor/src/obj_render_helpers.rs`

#### What to do

Move the effective ownership of `render_scene3d_work_item` out of `engine-compositor`.

There are two acceptable implementation directions:

1. Preferred:
   - move the callback implementation itself into `engine-render-3d`,
   - make `engine/src/systems/scene3d_prerender.rs` depend directly on `engine-render-3d`,
   - remove the compositor re-export.

2. Transitional but acceptable only if it actually reduces ownership:
   - move the remaining technical raster helpers from compositor into `engine-render-3d`,
   - then rebuild the callback there,
   - then delete the compositor-owned version.

#### Constraint

Do not solve this by just adding another wrapper or alias. The ownership must actually move.

#### Definition of done

- `engine/src/systems/scene3d_prerender.rs` no longer imports a Scene3D work-item renderer from `engine-compositor`.
- `engine-compositor/src/lib.rs` no longer re-exports `render_scene3d_work_item`.
- `engine-compositor/src/scene_clip_render_adapter.rs` no longer owns the reusable Scene3D prerender callback implementation.
- Scene3D prerender still works and build/test checks pass.

### 2. Move remaining low-level 3D raster helpers out of compositor

This is the structural work required to make step 1 clean.

#### Current problem

The following functionality still lives in `engine-compositor`:

- `render_obj_to_canvas`
- `render_obj_to_shared_buffers`
- `virtual_dimensions`
- `blit_color_canvas`

Current ownership is effectively spread across:

- `engine-compositor/src/obj_render.rs`
- `engine-compositor/src/obj_render_helpers.rs`

These are low-level 3D technical helpers, not frame assembly responsibilities.

#### Why this matters

As long as these functions live in compositor, any Scene3D prerender callback or 3D rendering path tends to drag compositor back into domain ownership.

#### Files involved

- `engine-compositor/src/obj_render.rs`
- `engine-compositor/src/obj_render_helpers.rs`
- `engine-compositor/src/obj_render_adapter.rs`
- `engine-compositor/src/generated_world_render_adapter.rs`
- `engine-compositor/src/scene_clip_render_adapter.rs`
- `engine-render-3d/src/pipeline/renderer.rs`
- `engine-render-3d/src/prerender/render_item.rs`
- potentially new files in `engine-render-3d` if needed

#### What to do

Move these helpers into `engine-render-3d` under a location that matches their actual role:

- render execution helpers into `engine-render-3d::pipeline`
- shared technical canvas/blit helpers into `engine-render-3d::pipeline` or `engine-render-3d::prerender`

Then update compositor adapters to consume those helpers from `engine-render-3d`.

#### Constraint

Do not just duplicate the functions and keep both copies. There should be one source of truth.

#### Definition of done

- compositor adapters no longer depend on compositor-owned low-level 3D raster helpers.
- those helpers live under `engine-render-3d`.
- `engine-compositor` keeps orchestration only.

### 3. Remove or isolate old command shapes `SceneSpawn` and `SceneDespawn`

#### Current problem

The old command variants still exist in the public command surface:

- `engine-api/src/commands.rs`

They are also still consumed in runtime:

- `engine-scene-runtime/src/behavior_runner.rs`

and still emitted in at least one engine system:

- `engine/src/systems/visual_binding.rs`

There may also be tests asserting the old behavior in runtime.

#### Current producer status

A large part of internal producers has already been migrated:

- `engine-behavior/src/scripting/ephemeral.rs`
- `engine-behavior/src/scripting/gameplay_impl.rs`

These now emit typed `ApplySceneMutation` requests for spawn/despawn in the migrated paths.

#### What to do

Audit all remaining active producers and consumers of:

- `BehaviorCommand::SceneSpawn`
- `BehaviorCommand::SceneDespawn`

Then choose one of these two outcomes:

1. Preferred final outcome:
   - remove both variants from `BehaviorCommand`,
   - migrate all remaining producers and runtime handling to `ApplySceneMutation`.

2. If full removal is too invasive for one pass:
   - keep them only as a strict compatibility shim,
   - ensure no active engine producers still emit them except explicitly marked compatibility edges,
   - document that status clearly in `3drefactor.md`.

#### Files involved

- `engine-api/src/commands.rs`
- `engine-scene-runtime/src/behavior_runner.rs`
- `engine/src/systems/visual_binding.rs`
- `engine-scene-runtime/src/lib.rs`
- `engine-behavior/src/lib.rs`
- any tests that still expect old command shapes

#### Definition of done

Minimum acceptable:

- no normal engine producer emits `SceneSpawn` / `SceneDespawn`,
- runtime still behaves correctly,
- compatibility status is explicit.

Preferred:

- the variants are removed entirely from the command surface.

### 4. Finish typed mutation convergence for script-facing setters

#### Current problem

There are still `SetProperty` fallbacks in:

- `engine-api/src/scene/api.rs`

Current remaining sites are around:

- `engine-api/src/scene/api.rs:255`
- `engine-api/src/scene/api.rs:296`
- `engine-api/src/scene/api.rs:444`

Some of these are legitimate because typed mutation currently needs runtime state to calculate deltas.

#### Important nuance

Not every fallback is wrong.

There are two distinct cases:

1. Safe to convert now:
   - when the API has enough runtime state or snapshot state to compute the typed mutation.

2. Not safe to convert with current mutation shapes:
   - when only a raw property assignment is known and there is no reliable state for delta computation.

This distinction must be preserved. Do not force a wrong typed conversion.

#### What to do

Audit each remaining `SetProperty` fallback and classify it:

- `convert now`
- `needs a new typed request shape`
- `intentionally low-level raw fallback`

If a new typed request shape is needed, add it cleanly instead of overloading delta-based `Set2dProps`.

Good example already implemented:

- `position.x/y` and `offset.x/y` now convert to typed `Set2dProps` when state exists.

#### Files involved

- `engine-api/src/scene/api.rs`
- `engine-api/src/commands.rs`
- `engine-api/src/scene/mutation.rs`
- `engine-scene-runtime/src/request_adapter.rs`
- `engine-scene-runtime/src/behavior_runner.rs`
- potentially `engine-core/src/render_types/*` or mutation types if new shapes are needed

#### Definition of done

- Normal high-level scripting APIs default to typed mutations whenever enough information exists.
- Remaining raw `SetProperty` fallback is intentionally narrow and documented.
- No new string-path branches are added to runtime semantics.

### 5. Reconcile runtime compatibility handling in `engine-scene-runtime`

#### Current problem

Runtime still contains compatibility branches in:

- `engine-scene-runtime/src/behavior_runner.rs`
- `engine-scene-runtime/src/render3d_state.rs`
- `engine-scene-runtime/src/materialization.rs`

This is partly expected, but the remaining shape needs to be cleaned up.

#### Specific points to inspect

- `BehaviorCommand::SetProperty` handling in `engine-scene-runtime/src/behavior_runner.rs`
- `BehaviorCommand::SceneSpawn` / `SceneDespawn` handling in the same file
- compatibility property application in `engine-scene-runtime/src/render3d_state.rs`
- direct sprite field mutation helpers in `engine-scene-runtime/src/materialization.rs`

#### What to do

After steps 3 and 4, reduce this runtime surface so that:

- typed mutation path is the obvious primary path,
- compatibility code is only a thin adapter,
- there is no second fully-fledged runtime branch growing in parallel.

#### Definition of done

- runtime mutation handling clearly converges on typed APIs,
- compatibility logic is thin and bounded,
- `3drefactor.md` can honestly mark the convergence item done.

## Explicit non-goals

Do not do these in this ticket unless absolutely required to unblock the core refactor:

- redesign planet visuals,
- introduce new authoring syntax,
- change mod content semantics,
- broad performance tuning unrelated to ownership cleanup,
- opportunistic cleanup outside the touched architecture seams.

## Recommended implementation order

Do this in order. The order matters.

1. Move the low-level 3D raster helpers from compositor to `engine-render-3d`.
2. Rebuild or move `render_scene3d_work_item` so Scene3D prerender no longer depends on compositor.
3. Remove compositor re-export and direct engine dependency on compositor for Scene3D prerender.
4. Finish migrating remaining internal spawn/despawn producers and decide whether to delete old command variants.
5. Audit and narrow remaining `SetProperty` fallbacks.
6. Clean runtime compatibility handling after the command surface is settled.
7. Update `3drefactor.md` with exact final status.

## Risks

### Risk 1: fake refactor via wrappers

It is easy to move code by adding more wrappers while keeping true ownership unchanged.

Avoid this.

The real question is: which crate owns the implementation and the dependency direction?

### Risk 2: incorrect typed conversion for position/offset paths

Converting raw property writes into delta-based mutations without correct state will create wrong runtime behavior.

Do not convert those blindly.

### Risk 3: breaking mod compatibility by deleting commands too early

If `SceneSpawn`, `SceneDespawn`, or `SetProperty` are removed from the public surface too early, old paths may break.

If full removal is not safe in one pass, isolate and document the shim explicitly instead of pretending it is gone.

### Risk 4: compositor still owning hidden 3D logic

Even if top-level exports are gone, ownership is still wrong if compositor still contains the actual reusable 3D implementation.

Check implementation location, not just imports.

## Verification checklist

Run these at minimum:

```powershell
cargo test -p engine-api --lib -- --nocapture
cargo test -p engine-behavior --lib -- --nocapture
cargo test -p engine-scene-runtime --lib -- --nocapture
cargo check -p engine-compositor
cargo check -p engine-render-3d
cargo check -p engine
```

Also run targeted grep audits:

```powershell
rg -n "render_scene3d_work_item" engine-compositor/src engine-render-3d/src engine/src
rg -n "SceneSpawn|SceneDespawn" engine-api/src engine-behavior/src engine-scene-runtime/src engine/src
rg -n "BehaviorCommand::SetProperty|SetProperty \\\\{" engine-api/src engine-behavior/src engine-scene-runtime/src engine/src
rg -n "legacy" engine-authoring/src/compile/render_scene.rs engine-compositor engine-render-3d engine-scene-runtime engine-render-2d engine-core
```

If scene checks are still part of the active validation policy, also run:

```powershell
cargo run -p app -- --mod-source=mods/planet-generator --check-scenes
cargo run -p app -- --mod-source=mods/playground --check-scenes
```

## Files most likely to change

- `engine-compositor/src/lib.rs`
- `engine-compositor/src/scene_clip_render_adapter.rs`
- `engine-compositor/src/obj_render.rs`
- `engine-compositor/src/obj_render_helpers.rs`
- `engine-render-3d/src/pipeline/renderer.rs`
- `engine-render-3d/src/prerender/render_item.rs`
- `engine-render-3d/src/prerender/scene3d_prerender.rs`
- `engine/src/systems/scene3d_prerender.rs`
- `engine-api/src/commands.rs`
- `engine-api/src/scene/api.rs`
- `engine-api/src/scene/mutation.rs`
- `engine-scene-runtime/src/behavior_runner.rs`
- `engine-scene-runtime/src/request_adapter.rs`
- `engine-scene-runtime/src/render3d_state.rs`
- `engine-scene-runtime/src/materialization.rs`
- `engine/src/systems/visual_binding.rs`
- `engine-behavior/src/scripting/ephemeral.rs`
- `engine-behavior/src/scripting/gameplay_impl.rs`
- `engine-behavior/src/lib.rs`
- `3drefactor.md`

## Current known remaining blockers at handoff time

All blockers listed below were resolved by subsequent work in this session.

- ~~`engine-compositor/src/lib.rs:34` still re-exports `render_scene3d_work_item`.~~ → **Resolved (Streams 1+2)**: re-export removed; `engine-render-3d::prerender` now owns it.
- ~~`engine-compositor/src/scene_clip_render_adapter.rs:97` still owns that callback implementation.~~ → **Resolved (Streams 1+2)**: implementation moved to `engine-render-3d/src/raster.rs`.
- ~~`engine/src/systems/scene3d_prerender.rs:30` still imports the callback from compositor.~~ → **Resolved (Streams 1+2)**: now imports from `engine_render_3d::prerender`.
- ~~`engine-api/src/commands.rs` still contains `SetProperty`, `SceneSpawn`, and `SceneDespawn`.~~ → **Resolved (Streams 3+4)**: `SceneSpawn` and `SceneDespawn` removed. `SetProperty` remains as a narrow, intentional, documented fallback.
- ~~`engine-scene-runtime/src/behavior_runner.rs` still handles `SetProperty`, `SceneSpawn`, and `SceneDespawn`.~~ → **Resolved (Streams 3+4)**: `SceneSpawn`/`SceneDespawn` arms removed. `SetProperty` arm kept as a bounded, typed-first fallback.
- ~~`engine/src/systems/visual_binding.rs:36` still emits `SceneDespawn`.~~ → **Resolved (Streams 3+4)**: uses `BehaviorCommand::ApplySceneMutation { DespawnObject }`.
- ~~`engine-api/src/scene/api.rs` still contains narrow `SetProperty` fallback sites that need final classification.~~ → **Resolved (Streams 3+4 / PR5)**: all three sites classified and documented as intentional fallbacks for unsupported typed-mutation paths.

## Expected update discipline

While working this ticket:

- update `3drefactor.md` every time a meaningful step is completed,
- do not add any identifier with `legacy` in the name,
- do not create a permanent dual execution path,
- do not leave TODO comments as the only form of documentation,
- keep the end-state honest: if something remains as compatibility, mark it explicitly.

## Final acceptance

This ticket is done only when the repo is in a state where a reviewer can say:

- compositor assembles frames but does not own 3D render-domain implementation,
- typed mutation path is the real primary path,
- remaining compatibility surface is either gone or deliberately tiny and documented,
- `3drefactor.md` matches reality.

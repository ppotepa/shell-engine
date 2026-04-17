# Left After 3D Refactor Audit

## Verdict

`3drefactor.md` currently has no open checkboxes.

The implementation work described by the refactor is largely complete in code.
What is still left is now very narrow:

- one architectural policy decision around bounded `SetProperty` compatibility.

This means the answer is not "major refactor still open".
The answer is "core refactor landed; remaining work is cleanup and policy closure".

## Confirmed Done In Code

These items were checked against the current repository state and do not appear
to be open anymore:

- Scene3D prerender ownership is in `engine-render-3d`, not `engine-compositor`.
  Evidence:
  - `engine/src/systems/scene3d_prerender.rs`
  - `engine-render-3d/src/prerender/render_item.rs`
- `BehaviorCommand::SceneSpawn` and `BehaviorCommand::SceneDespawn` are gone.
  Evidence:
  - `engine-api/src/commands.rs`
  - `engine-scene-runtime/src/behavior_runner.rs`
  - `engine/src/systems/visual_binding.rs`
- Low-level OBJ / Scene3D raster ownership has been moved out of compositor.
  Evidence:
  - `engine-render-3d/src/raster.rs`
  - deleted compositor-local render files visible in git status
- `engine-compositor` is operating mainly as assembly/orchestration with
  adapter modules, not as the owner of the raster pipeline.
  Evidence:
  - `engine-compositor/src/provider.rs`
  - `engine-compositor/src/lib.rs`
- 3D feature ownership is feature-gated instead of hardwired everywhere.
  Evidence:
  - `engine-compositor/Cargo.toml`
  - `engine/Cargo.toml`

## What Is Actually Left

### 1. Decide the final status of `SetProperty`

This is the only remaining architectural question that is not fully closed.

#### Current state

`SetProperty` still exists, but it is now a bounded typed-first compatibility
fallback rather than a second runtime pipeline.

Live sites:

- `engine-api/src/commands.rs`
- `engine-api/src/scene/api.rs`
- `engine-scene-runtime/src/behavior_runner.rs`
- `engine-behavior/src/lib.rs`
- `engine-behavior/src/scripting/gameplay_impl.rs`

Most important fallback points:

- `engine-api/src/scene/api.rs:258`
- `engine-api/src/scene/api.rs:300`
- `engine-api/src/scene/api.rs:451`
- `engine-scene-runtime/src/behavior_runner.rs:544`

#### What must be decided

Choose one of two end states:

1. Keep `SetProperty` as an explicit low-level compatibility escape hatch.
2. Remove it entirely and finish typed coverage for every remaining supported
   path.

#### If you keep it

Then the refactor is effectively done, but `3drefactor.md` should stop claiming
"retire string-path setters" in an absolute sense. It should instead say:

- typed mutations are the default path,
- raw `SetProperty` remains intentionally narrow,
- unsupported paths still go through the compatibility layer.

#### If you remove it

Then there is still real implementation work left:

- add typed request shapes for remaining unsupported paths,
- remove raw `BehaviorCommand::SetProperty`,
- remove the runtime compat converter in
  `engine-scene-runtime/src/behavior_runner.rs`,
- remove API fallback emission in `engine-api/src/scene/api.rs`,
- update tests in `engine-behavior` and `engine-scene-runtime`.

#### Why it matters

This is the only place where "refactor complete" still depends on policy, not
just implementation state.

### 2. Optional stricter cleanup around compositor adapters

This is not clearly required, but it is the only remaining area someone could
still argue about.

Current modules still present in compositor:

- `engine-compositor/src/obj_render_adapter.rs`
- `engine-compositor/src/generated_world_render_adapter.rs`
- `engine-compositor/src/scene_clip_render_adapter.rs`

Current evidence suggests these are orchestration/dispatch modules, not owners
of 3D raster logic. That is probably acceptable.

If you want a stricter interpretation of "compositor only assembles":

- push more of the adapter surface into `engine-render-3d`,
- keep compositor limited to prepared-frame assembly and delegation only.

This is a design tightening step, not a proven blocker from the current audit.

## Recommended Final Classification

### Core refactor status

- Complete enough in implementation.

### Real open work

- final decision on permanent `SetProperty` compatibility,
- optional stricter adapter boundary cleanup.

## Recommended Next Actions

1. Decide whether `SetProperty` is allowed to remain as a narrow escape hatch.
2. If yes:
   - update docs and close the refactor.
3. If no:
   - open one more focused task only for removing the remaining raw property
     fallback.

## Bottom Line

If the acceptance criterion is:

- "Has the 2D/3D subsystem split landed and is compositor no longer the owner of
  3D render logic?"

then the answer is effectively yes.

If the acceptance criterion is:

- "Has every last string-path compatibility escape hatch been removed?"

then no, one bounded compatibility layer still remains by design.

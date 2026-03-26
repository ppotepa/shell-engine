# engine-frame

Frame identity and freshness checks for threaded rendering.

## Purpose

`engine-frame` provides `FrameTicket`, the shared identity token used to track
which simulation frame a rendered result belongs to and whether it is still
safe to present.

This crate is small by design, but it protects an important correctness
boundary: stale render-thread frames must not be shown after scene-generation
changes such as transitions or resizes.

## Key type

- `FrameTicket`
  - `sim_frame_id` — monotonically increasing simulation frame counter
  - `scene_generation` — bumped when scene identity changes

## Important semantics

- `matches_generation()` checks cross-scene freshness
- `is_newer_than()` compares tickets within a generation
- `is_acceptable()` only checks frame ordering; generation filtering must happen
  before calling it

## Working with this crate

- keep the semantics explicit and simple,
- if freshness rules change, update the presenter/game-loop call sites together,
- be careful not to hide generation checks inside the wrong helper; the current
  split is intentional.

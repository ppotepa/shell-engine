# 3D Refactor Checklist

Status: `DONE (current ownership split scope)`
Last updated: `2026-04-18`
Owner: `engine/render/runtime`

## 1. Governance / Architecture Rules

- [x] Do not keep permanent compatibility layers in renderer/compositor internals.
- [x] Treat planets as one producer of 3D data (renderer remains domain-agnostic).
- [x] Keep 3D domain logic out of `engine-compositor`.
- [x] Runtime mutation path converges on typed APIs with API-edge translation for supported `scene.set(...)` keys.

## 2. Crate Ownership

- [x] `engine-render-2d` owns 2D rendering.
- [x] `engine-render-3d` owns 3D rendering.
- [x] `engine-compositor` assembles frame output.
- [x] `engine-scene-runtime` owns runtime state/mutations.
- [x] `engine-worldgen` owns worldgen + mesh build key policy.

## 3. 3D/2D Separation (Current)

- [x] Scene environment rendering (`starfield`, `primary-star glare`) moved to `engine-render-3d::scene::environment`.
- [x] View lighting resolution is shared in `engine-render-3d::pipeline::resolve_view_lighting`.
- [x] Shared world LOD source rewrite lives in `engine-render-3d::pipeline::apply_world_lod_to_source`.
- [x] `SceneClip` rendering moved from compositor adapter to `engine-render-3d::pipeline::render_scene_clip_sprite_to_buffer`.
- [x] `Obj` rendering moved from compositor adapter to `engine-render-3d::pipeline::render_obj_sprite_to_buffer`.
- [x] `GeneratedWorld` rendering moved from compositor adapter to `engine-render-3d::pipeline::render_generated_world_sprite_to_buffer`.
- [x] Generated-world profile synthesis moved to `engine-render-3d::pipeline::build_generated_world_render_profile`.
- [x] 3D delegate finalization is unified in compositor provider (`finalize_sprite` now applied consistently after region-returning 3D pipeline calls, including `SceneClip`).
- [x] Removed compositor-local 3D adapters:
  - `engine-compositor/src/obj_render_adapter.rs`
  - `engine-compositor/src/generated_world_render_adapter.rs`
  - `engine-compositor/src/scene_clip_render_adapter.rs`

## 4. Remaining Work

- [x] Remove residual 3D-specific wiring from `engine-compositor` docs/comments and keep it as pure frame assembler.
- [x] Final pass over crate docs/changelog alignment after this migration lands.
- [x] Run full startup validation on active mods after integration commit.

## 5. Validation Snapshot

- [x] `cargo check -p engine-render-3d -p engine-compositor -p engine`
- [x] `cargo test -p engine-render-3d`
- [x] `cargo run -p app -- --mod-source=mods/lighting-playground --check-scenes`
- [x] `cargo run -p app -- --mod-source=mods/planet-generator --check-scenes`

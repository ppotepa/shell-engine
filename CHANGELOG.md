# Changelog

Daily progress updates for Shell Engine development.

## 18-04-2026

**Viewport clipping investigation + compositor buffer-pool fix** ✅
- **engine-compositor**: fixed world/UI scratch-buffer sizing in `buffer_pool.rs` — `acquire()` no longer clamps requested buffer size to pool defaults (`512x256`), preventing 3D viewport crop/stretch artifacts at larger authored world resolutions.
- **engine-compositor**: release policy now drops oversized transient buffers instead of retaining them in the pool, keeping memory usage bounded while preserving correctness.
- **engine-compositor/tests**: added regression coverage for non-clamped acquire behavior and oversized-buffer release policy.
- **mods/lighting-playground**: corrected world layer classification (`ui: false`) so 3D world content always stays on world pass semantics.
- **engine-scene-runtime**: strengthened orbit safe-distance estimation (aspect-aware + sphere angular fit) to reduce edge clipping under zoom-heavy camera motion.

**3D ownership cleanup: compositor as frame assembler only** ✅
- **engine-render-3d**: took ownership of Obj / GeneratedWorld / SceneClip sprite render paths via pipeline entry points (`render_obj_sprite_to_buffer`, `render_generated_world_sprite_to_buffer`, `render_scene_clip_sprite_to_buffer`) and shared generated-world profile synthesis.
- **engine-compositor**: removed compositor-local 3D render adapters and now delegates directly to `engine-render-3d` from provider path, with consistent `finalize_sprite` handling across all region-returning 3D calls.
- **docs**: synced `3drefactor.md`, `engine-compositor` crate docs, and runtime docs wording to the typed-mutation boundary contract.
- **validation**: `cargo check -p engine-render-3d -p engine-compositor -p engine`; `cargo run -p app -- --mod-source=mods/planet-generator --check-scenes`; `cargo run -p app -- --mod-source=mods/lighting-playground --check-scenes`.

**Cloud-heavy LOD cap retune** ✅
- **engine-worldgen**: tightened `recommended_subdivisions_cap_for_lod` caps (`112/80/60/48/32`) to reduce CPU surface raster cost on `world://` meshes under screen-space LOD.
- **perf**: cloud-heavy benchmark (`mods/asteroids` bench-cloud, 6s, `--opt`) improved from ~16.6 FPS to ~18.3 FPS; compositor avg dropped (~54.8ms -> ~50.9ms), with lower average tri count.

**Cloud cadence smoothing follow-up** ✅
- **engine-render-3d**: tuned cloud cadence/stale policy in generated-world renderer (shorter update intervals, tighter stale-cache window, lighter cloud2 mesh source) to reduce perceived cloud lag while keeping software path cost bounded.
- **perf**: follow-up cloud-heavy bench reached ~18.8 FPS class with lower compositor and renderer averages; tri count dropped again versus earlier cloud-heavy baseline.

**Generated-world surface path: remove redundant RGB->RGBA copy** ✅
- **engine-render-3d**: `generated_world_renderer` now renders surface directly to RGBA and skips intermediate RGB canvas conversion in the hot path.
- **perf/metrics**: `3D convert` phase is now `0.0us` in cloud-heavy benchmark report (`20260418-114554`), confirming the copy stage is removed.

**Adaptive cloud render-scale tuning for large viewports** ✅
- **engine-render-3d**: cloud1/cloud2 offscreen render scales now adapt to viewport area (`generated_world_renderer`) instead of using fixed defaults only, reducing cloud pass cost on larger 3D sprites while preserving small-viewport quality.
- **perf**: cloud-heavy smoke (`20260418-114805`) keeps FPS class stable with lower cloud pass cost (`3D cloud1`/`3D cloud2`) and no new conversion overhead.

**Atmosphere halo pass: temporal reuse + tighter ROI** ✅
- **engine-render-3d**: `raster.rs` now reuses computed halo pixels between nearby motion states (`halo_temporal_key` + material signature) and tightens halo nearest-distance work to a circular ROI with an additional radial cutoff in final scan.
- **validation**: `cargo check -p engine-render-3d` and halo regression test (`atmosphere_halo_paints_pixels_outside_the_planet_silhouette`) pass.

**Planet-generator perf presets doc refresh** ✅
- **mods/planet-generator**: README now includes practical performance presets (`Balanced`, `Look-dev`, `Fast iteration`) and a benchmark smoke command aligned with current optimization workflow.

**3D renderer elasticity step: Halo pass extraction** ✅
- **engine-render-3d**: moved atmosphere halo implementation out of `raster.rs` into reusable effect-pass module (`effects/passes/halo.rs`) with typed `HaloPassParams` and dedicated temporal-key helper (`halo_temporal_key_from_obj_params`).
- **engine-render-3d**: `raster.rs` now orchestrates halo through pass invocation instead of owning full halo internals, reducing planet-specific coupling in raster core.
- **validation**: `cargo check -p engine-render-3d -p engine` and halo regression test (`atmosphere_halo_paints_pixels_outside_the_planet_silhouette`) pass.

**3D renderer elasticity step: Surface pass extraction** ✅
- **engine-render-3d**: moved Gouraud terrain/surface raster paths (`rasterize_triangle_gouraud`, `rasterize_triangle_gouraud_rgba`) from `raster.rs` into `effects/passes/surface.rs`.
- **engine-render-3d**: `raster.rs` now treats planet-surface shading as a pass dependency instead of embedding the full terrain/biome/crater logic inline.
- **validation**: `cargo check -p engine-render-3d` and halo regression test still pass after extraction.

**3D renderer elasticity step: Planet param mapping extraction** ✅
- **engine-render-3d**: moved `ObjRenderParams -> PlanetBiomeParams/PlanetTerrainParams` mapping out of `raster.rs` into `effects/passes/planet_params.rs`.
- **engine-render-3d**: `raster.rs` now consumes pass-level parameter builders (`build_biome_params`, `build_terrain_extra_params`) instead of owning planet-profile projection details inline.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer elasticity step: Halo orchestration extraction** ✅
- **engine-render-3d**: added `apply_obj_halo_from_params(...)` in `effects/passes/halo.rs`; this now owns atmospheric halo gating + param projection from `ObjRenderParams`.
- **engine-render-3d**: `render_obj_to_canvas` in `raster.rs` now invokes one pass-level function for halo instead of embedding halo-mapping logic inline.
- **tests/cleanup**: removed raster-local halo wrapper and switched the halo regression test to call the pass API (`apply_halo_pass` + `HaloPassParams`) directly.

**3D renderer elasticity step: RGB post-pass chain seam** ✅
- **engine-render-3d**: added `effects/passes/postprocess.rs` with `apply_rgb_post_passes(...)` and `RgbPostPassMetrics` as a single orchestration seam for RGB post effects.
- **engine-render-3d**: `raster.rs` now consumes post-pass metrics (`halo_us`) from the chain instead of directly wiring a single effect call.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer stage split: shared projection stage** ✅
- **engine-render-3d**: added `pipeline/stages/project.rs` with `project_vertices_into(...)`, `ProjectionStageInput`, `ProjectionStageConfig`, and explicit terrain-noise policies.
- **engine-render-3d**: three duplicated projection paths in `raster.rs` now share one stage implementation, with only policy differences left at the call site (`SurfaceOrDisplacement` vs `SurfaceUnlessSoftCloudsOrDisplacement`, smooth normals on/off, parallel threshold).
- **architecture**: this is the first true stage seam in the 3D pipeline (`project`) and reduces direct planet-specific projection logic in raster orchestration.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer stage split: shared face classification stage** ✅
- **engine-render-3d**: added `pipeline/stages/classify.rs` with `classify_and_sort_faces_into(...)` and typed `FaceClassificationConfig`.
- **engine-render-3d**: the three duplicated face-selection/depth-sort blocks in `raster.rs` now share one stage implementation; call sites only provide policy (`backface_cull`, `depth_sort_faces`, min projected area, max faces).
- **architecture**: `raster.rs` now has explicit `project -> classify` seams, making the remaining shading/raster passes easier to isolate next.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer stage split: shared shading preparation stage** ✅
- **engine-render-3d**: added `pipeline/stages/shade.rs` with reusable `prepare_gouraud_faces_into(...)`, `prepare_flat_faces_into(...)`, and typed `FlatShadingStageContext`.
- **engine-render-3d**: duplicated face shading/pre-raster preparation logic in `raster.rs` now goes through shared stage helpers for shared-buffer RGB, regular RGB, and RGBA render paths.
- **architecture**: pipeline now has explicit `project -> classify -> shade-prep` seams; the remaining large responsibility in `raster.rs` is mostly the actual raster execution path and output-specific orchestration.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer stage split: shared raster execution stage** ✅
- **engine-render-3d**: added `pipeline/stages/raster_exec.rs` with reusable thin execution helpers for Gouraud RGB, flat RGB, strip-parallel Gouraud RGB, and strip-parallel Gouraud RGBA paths.
- **engine-render-3d**: `raster.rs` no longer manually expands most execution loops after shading prep; call sites now pass typed raster contexts (`GouraudRgbRasterContext`, `GouraudRgbaRasterContext`) into stage helpers.
- **architecture**: pipeline now has explicit `project -> classify -> shade-prep -> raster-exec -> postprocess` seams, leaving `raster.rs` primarily as high-level orchestration and metrics glue.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer stage split: shared frame shading context** ✅
- **engine-render-3d**: added `pipeline/stages/frame_context.rs` so per-frame lighting, tone, terrain, and biome inputs are assembled once instead of being rebuilt independently in each raster path.
- **engine-render-3d**: `raster.rs` now consumes `FrameShadingContext` to build flat-shading inputs plus Gouraud RGB/RGBA raster contexts, removing another large block of duplicated param plumbing.
- **architecture**: the remaining work in `raster.rs` shifts further toward orchestration, with frame-domain inputs prepared up front and handed to later stages.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer cleanup: RGBA path now reuses shared Gouraud prep** ✅
- **engine-render-3d**: the RGBA cloud/soft-alpha render path now calls `prepare_gouraud_faces_into(...)` instead of keeping an extra inline parallel face-prep block in `raster.rs`.
- **architecture**: Gouraud face preparation is now owned by one stage helper across shared-buffer RGB, standard RGB, and RGBA paths, reducing drift risk between output formats.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer stage split: shared projection setup helper** ✅
- **engine-render-3d**: extended `pipeline/stages/project.rs` with `project_mesh_with_viewport(...)` and `ProjectionPoseConfig`, so yaw/pitch/roll, clip rows, viewport clipping, and vertex projection are prepared through one stage seam.
- **engine-render-3d**: both `render_obj_to_canvas` and `render_obj_to_rgba_canvas` now delegate shared projection setup instead of rebuilding that block inline; the remaining difference is explicit pose policy (`animated yaw/camera look` on RGB, fixed pose on RGBA).
- **architecture**: this pushes `raster.rs` further toward orchestration and makes future unification of RGB/RGBA setup substantially simpler.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer stage split: shared Gouraud visibility prep** ✅
- **engine-render-3d**: added `pipeline/stages/gouraud.rs` with `prepare_visible_gouraud_faces_into(...)`, which now owns the common `classify -> Gouraud face prep` flow.
- **engine-render-3d**: the RGBA path now uses one higher-level stage seam for visible-face selection plus Gouraud shading prep instead of spelling those steps inline in `raster.rs`.
- **architecture**: this is the first combined stage above the low-level seams and is the direct foundation for a future shared RGB/RGBA render core.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer cleanup: shared pooled-buffer and finalization helpers** ✅
- **engine-render-3d**: `raster.rs` now uses shared helpers for projected/sorted/shaded/depth/canvas pool acquisition and release, instead of repeating thread-local buffer plumbing in each public render entrypoint.
- **engine-render-3d**: RGB and RGBA paths now share explicit `finish_*_canvas_render(...)` finalization for stats, metrics, projection buffer release, and grading.
- **architecture**: this reduces entrypoint duplication and makes the remaining RGB/RGBA divergence much more clearly about render policy, not pool bookkeeping.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**3D renderer cleanup: smooth RGB now uses shared Gouraud visibility prep** ✅
- **engine-render-3d**: the smooth-shaded RGB path now also uses `prepare_visible_gouraud_faces_into(...)`, matching the RGBA path on the combined `classify -> Gouraud prep` seam.
- **architecture**: Gouraud rendering across RGB and RGBA now diverges mainly at raster execution/output format, not at visible-face selection and shading preparation.
- **validation**: `cargo check -p engine-render-3d` and `cargo test -p engine-render-3d` pass.

**Dual-resolution UI/world render path** ✅
- **engine-runtime**: introduced explicit world-vs-final buffer layout (`world_width/world_height` + `render_width/render_height`) and `display.world_render_size` / `display.ui_render_size` / `display.ui_layout_size`.
- **engine / compositor**: added split-pass composition path (WorldOnly -> upscale -> UiOnly) using compositor pass filtering, preserving renderer/domain separation.
- **engine / renderer**: FPS HUD generic-font scale now follows the UI render/layout split to keep overlays readable on denser UI targets.
- **schemas/docs/mods**: added `world_render_size` / `ui_render_size` / `ui_layout_size` to `schemas/mod.schema.yaml`, documented the contract in `AUTHORING.md`, and enabled the split in active mods (`planet-generator`, `lighting-playground`, `gui-playground`).
- **validation**: `cargo check -p engine-runtime -p engine`, `cargo check -p engine-compositor -p engine`, `cargo check -p app`, targeted engine tests, and `--check-scenes` for gui/planet mods.

## 17-04-2026

**3D view-profile foundation** ✅
- **engine-core**: added scene-level `view` contract with reusable `profile`, `lighting-profile`, and `space-environment-profile` references plus typed `LightingProfile` / `SpaceEnvironmentProfile` / `ViewProfile` models as the first foundation for hierarchical 3D scene look selection.
- **engine-authoring**: added authored document stubs for `lighting-profile`, `space-environment-profile`, and `view-profile` so the authoring layer has explicit reusable profile types.
- **engine-scene-runtime / engine-api / engine / engine-compositor / engine-asset / engine-render-3d**: runtime now stores a resolved scene view profile, compositor/render adapters receive that resolved contract instead of a raw ambient-float seam, scene background can now fall back to `space-environment-profile.background_color`, compositor now renders deterministic scene-level `starfield` and `primary_star_glare` passes from `space-environment-profile`, typed API/runtime mutations support switching `view-profile`, `lighting-profile`, and `space-environment-profile` without new string-path branches, scene repositories now hydrate asset-backed profiles from conventional mod paths or explicit YAML paths, `engine-render-3d` now consumes scene-level `exposure` / `gamma` / `tonemap` / `shadow_contrast` as 3D grading on rendered output, scene-level `SetLightingParam` / `SetSpaceEnvironmentParam` overrides now apply as dedicated runtime overlays after view-profile resolution, and `lighting-profile.night_glow_scale` / `lighting-profile.haze_night_leak` now drive reusable night-side atmosphere behavior without mod-specific logic.
- **schemas/docs/tests**: extended `schemas/scene.schema.yaml`, added profile asset schema stubs, updated `AUTHORING.md`, rewrote `lightning.impl.md` around the new `view-profile -> lighting/environment` composition model, and added initial resolution/runtime/background/asset-hydration/grading tests plus deterministic compositor regression hashes for built-in scene view presets.

**App launch & compositor stability** ✅
- **app**: startup pixel scale no longer defaults to a fixed `8` multiplier; it now auto-resolves from `display.render_size` when CLI scale is unset, preventing oversized benchmark/dev windows on multi-monitor setups.
- **engine-compositor**: frame assembly marks layers with any 3D sprite and forces scratch compositor path for them, preventing black stripe/ghost artifacts in mixed 2D+3D scenes.
- **verification**: `cargo check -p app` succeeds after changes; pending full visual smoke + benchmark on `mods/planet-generator` and `mods/asteroids` after window/scratch fixes.

**2D regression safeguards** ✅
- **engine**: added 2D-only scene regression tests proving that the new scene pipeline does not schedule 3D preparers, scene3d atlas/runtime stores, or obj prerender state for pure-2D scenes.
- **docs**: updated `3drefactor.md` and `left.md` with closed DoD entries and remaining follow-up scope.
- **tests**: executed targeted verification with:
  - `cargo test -p engine scene_pipeline_2d_only_does_not_schedule_3d_preparation_steps -- --nocapture`
  - `cargo test -p engine composite_2d_only_scene_runs_without_3d_world_resources -- --nocapture`
- Extended this pass with a 3D-path guard test:
  - `scene_pipeline_3d_prerender_scene_schedules_obj_prepass_state` (verifies prerenderable `type: obj` scenes register obj-prerender state during prepare).

**Planet generator perf stability pass** ✅
- **engine-asset**: bounded generated render-mesh cache (`RENDER_MESH_CACHE`) with LRU-style eviction (cap: 64) to prevent unbounded `world://...` cache growth during slider-heavy sessions
- **mods/planet-generator**: reduced per-frame mutation pressure by gating `obj.rotation-speed`, `obj.ambient`, `obj.light.*`, and full `obj.atmo.*` push block behind change thresholds
- **validation**: `cargo check -p app` and benchmark smoke on `mods/planet-generator`

**3D ownership split closed + docs sync** ✅
- **engine-render-3d**: now owns the moved raster path, Scene3D prerender work-item flow, generated-world rendering internals, and the final 3D-side seams consumed by compositor
- **engine-compositor**: reduced to frame assembly, prepared-frame orchestration, PostFX, and adapter-level delegation into `engine-render-2d` / `engine-render-3d`
- **engine-api / engine-scene-runtime / engine-behavior**: typed-first scene mutation flow consolidated end-to-end; spawn/despawn command variants removed; supported `scene.set(...)` paths are translated at the API edge and unsupported paths no longer enqueue runtime mutation work
- **spatial/docs**: added `UNITS.md` (engine unit model: `screen_px`, `virtual_px`, `wu`, `m/km`) and synced root documentation to current architecture
- **planet perf**: generated-world cloud/atmosphere optimization pass closed (cloud cadence/reuse, reduced cloud mesh sources, one-heavy-cloud-refresh-per-frame guard, startup surface LOD ramp)
- **benchmarks**: added cloud-heavy scenario `mods/asteroids/scenes/bench-cloud/scene.yml`; stabilized run reached ~23.8 FPS class with compositor ~36ms avg on 10s bench
- **docs**: refreshed root docs, mod docs, compositor/render/worldgen docs, and runtime mutation docs to match the current architecture and bundled mods
- **validation**: `cargo check -p engine`, `cargo check -p engine-render-3d`, benchmark smoke runs on `mods/asteroids` + `mods/planet-generator`

**Ambient floor + free-look surface mode** ✅
- **engine-core**: added optional scene lighting config (`scene.lighting.ambient-floor`) and free-look camera surface controls schema + runtime defaults.
- **engine-compositor / engine-render-3d / engine-scene-runtime**: threaded `ambient_floor` from scene data to rasterizer ambient floor clamp with no renderer-specific coupling to planet-generator.
- **mods/planet-generator**: enabled free-look surface mode in `scenes/main/scene.yml` and documented control behavior in mod README.
- **docs**: updated `AUTHORING.md` and `schemas/scene.schema.yaml` for new scene lighting and free-look controls, keeping compatibility with legacy camelCase/hyphenated keys.
- **validation**: `cargo check -p engine-compositor -p engine-render-3d -p engine -p engine-render-sdl2 -p engine-scene-runtime -p engine-core` and `cargo test -p engine-scene-runtime --lib`.

## Format Guidelines

Each day should follow this structure:
- **Header**: `## DD-MM-YYYY` (date of work)
- **Title**: Brief summary of primary focus
- **Entries**: List changes by subdomain (only include subdomains that were touched)
  - Format: `**subdomain**: one-liner description`
  - Examples: `**splash**`, `**optimizations**`, `**graphics**`, `**sidecar**`, `**audio**`, `**engine**`, `**docs**`
  - Omit subdomains if no work was done that day
- **Result** (optional): Summary of outcome or impact

Example:
```
## 25-03-2026

**Documentation consolidation complete** ✅
- **docs**: consolidated 26 scattered files into 5 focused docs + 20 crate READMEs
- **testing**: verified all 204 engine tests passing (zero regressions)
- **result**: 69% doc reduction (26 → 8 root files), clear navigation hierarchy

## 24-03-2026

**Splash & optimization focus**
- **splash**: new splash screen design
- **optimizations**: attempted aggressive optimization; rolled back to apply gradually
- **graphics**: planning difficulty menu rework
- **sidecar**: will be rewritten in Rust
- **audio**: 90s machine simulation experiments (floppy, HD, modem sounds)
```

Keep entries minimalistic (one-liner per subdomain). Move detailed feature specs to [Unreleased] section below.

---

**`GuiControl` trait refactor + GUI playground mod** ✅
- **engine-gui**: new `GuiControl` trait (`control.rs`) with polymorphic dispatch replacing monolithic `GuiWidgetDef` enum; concrete types `SliderControl`, `ButtonControl`, `ToggleControl`, `PanelControl`; `GuiSystem::update` now dispatches via `&[Box<dyn GuiControl>]` trait methods — no more match-on-variant; `VisualSync` struct enables engine-level sprite positioning (slider handles)
- **engine-scene-runtime**: `gui_widgets` field changed to `Vec<Box<dyn GuiControl>>`; construction converts `SceneGuiWidgetDef → Box<dyn GuiControl>` via `scene_gui_widget_to_control()`; new `sync_widget_visuals()` method applies `visual_sync()` + `TargetResolver` alias lookup to position sprites automatically
- **engine-api**: added `BehaviorCommand::SetGuiValue { widget_id, value }` for programmatic slider value changes
- **engine-behavior**: added `gui.set_widget_value(id, val)` to `ScriptGuiApi` Rhai surface; emits `SetGuiValue` command
- **engine-core**: added `handle` (sprite reference) and `hit-padding` fields to `SceneGuiWidgetDef::Slider`
- **engine**: `behavior_system` calls `sync_widget_visuals()` after `reset_frame_state()` + behavior updates — fixes frame-order wipe bug where slider handles disappeared each frame
- **gui-playground mod**: new test-bench mod (`mods/gui-playground`) with RGB color mixer (3 sliders, 3 toggles, 2 buttons), real-time fill track bars via dynamic `vector.points`, channel-tinted handles, persistent state via `local`, 5-row color swatch, hex/RGB readout, state monitor, and event log
- **docs**: updated READMEs for engine-gui, engine-scene-runtime, engine-behavior, engine; added `gui` API section to SCRIPTING-API.md; updated ARCHITECTURE.md change playbook
- **result**: new widget types only need to implement `GuiControl` trait — no changes to `GuiSystem`, scene runtime, or engine orchestration; 88 tests pass, all 3 mods validate

---

## 15-04-2026

**engine-compositor refactor: obj_render split + engine-worldgen + engine-render-3d** ✅
- **engine-compositor**: extracted `obj_render.rs` monolith into `obj_render/` submodule tree (`mesh_source`, `params`, `setup`, `terrain_eval`); world URI parsing + mesh building moved to `engine-worldgen`; render-domain math moved to `engine-render-3d`; added `MAX_OBJ_FACE_RENDER = 250_000` safety cap; now uses `Render3dPipeline` trait from `engine-render-3d`
- **engine-worldgen**: new crate owning `world://` URI parsing, all base-sphere selection (`cube`/`uv`/`tetra`/`octa`/`icosa`), world mesh building, per-vertex elevation displacement, per-face biome/altitude coloring, and canonical URI serialization for cache keys
- **engine-render-3d**: new crate centralizing shared 3D render-domain logic — geometry helpers, procedural effect kernels (planet atmosphere/biome/terrain signals), shading/color-space utilities, and the `Render3dPipeline` seam trait; keeps compositor focused on orchestration + buffer composition
- **engine-terrain**: added `WorldBase` enum (`Cube`/`Uv`/`Tetra`/`Octa`/`Icosa`) with serde `lowercase` rename; added `base: WorldBase` field to `WorldGenParams`; updated crate root doc comment and re-export
- **engine-mesh**: added `poly_sphere` module with `tetra_sphere`, `octa_sphere`, `icosa_sphere` generators (recursive polyhedron subdivision → normalized unit sphere); re-exported from `primitives/mod.rs`
- **engine-core**: added `world-base` YAML field (`world_gen_base: Option<String>`) to `Sprite::Obj`
- **engine-scene-runtime**: added `world.base` Rhai property path (valid values: `"cube"`, `"uv"`, `"tetra"`, `"octa"`, `"icosa"`); added 5 atmosphere Rhai property paths: `obj.atmo.color`, `obj.atmo.strength`, `obj.atmo.rim_power`, `obj.atmo.haze_strength`, `obj.atmo.haze_power`; added all 6 new fields to the `Sprite::Obj` destructure block
- **engine-mod**: added `GuiWidgetBindingsCheck` startup check (`gui_widget_bindings.rs`) that validates GUI widget registrations; registered in both `StartupRunner::default()` and app `run_scene_checks()`
- **planet-generator**: minor YAML adjustments to `hud-panel.yml`, `hud-sliders.yml`, `hud-tabs.yml`, and `scene.yml` following the GUI slider + tab click migration

## 14-04-2026

**Unified `world://` URI — biome planet pipeline via `engine-terrain`** ✅
- **engine-terrain**: added `WorldShape`, `WorldColoring`, `WorldGenParams` enums/struct to `params.rs`; new `coloring.rs` module with `biome_color(Biome) → [u8;3]` (10-biome Earth palette) and `altitude_color(f32) → [u8;3]` gradient; all exported from crate root
- **engine-mesh**: re-exported `compute_smooth_normals` from crate root (was pub but not in lib.rs)
- **engine-compositor**: added `engine-terrain` dep; wired `world://` URI handler in `get_or_load_obj_mesh`; `build_world_mesh` bridges the full pipeline: `engine_terrain::generate()` → 512×256 heightmap → `cube_sphere(N)` geometry → per-vertex elevation displacement → `compute_smooth_normals` → per-face biome/altitude coloring → `ObjMesh` cached by full parameterized URI; `sprite_renderer.rs` builds effective URI from `world_gen_*` fields
- **engine-core**: added 11 `world_gen_*` fields to `Sprite::Obj` (world-shape, world-coloring, world-seed, world-ocean-fraction, world-continent-scale/warp/octaves, world-mountain-scale/strength, world-moisture-scale, world-displacement-scale)
- **engine-scene-runtime**: added `world.*` Rhai property paths (seed, ocean_fraction, continent_scale/warp/octaves, mountain_scale/strength, moisture_scale, displacement_scale, coloring) — any change rebuilds the URI key → cache miss → planet regenerated
- **terrain-playground earth-planet**: migrated from `earth-sphere://32` (altitude gradient) to `world://32` (full biome pipeline); HUD panel replaced 12 terrain-mesh params with 6 world params: seed, ocean%, continent size, coast chaos, mountain strength, displacement

**Planet generator mod + extended world:// params** ✅
- **engine-terrain**: added 4 new fields to `PlanetGenParams`: `mountain_ridge_octaves` (u8, default 5), `ice_cap_strength` (f64, default 1.0), `lapse_rate` (f64, default 0.6), `rain_shadow` (f64, default 0.35); wired into `climate.rs` (replacing hardcoded constants) and `elevation.rs` (`ridged_fbm` octave count); added `LAST_PLANET_STATS` global cache + `last_planet_stats()` pub fn; added `forest_fraction` + `grassland_fraction` to `PlanetStats`
- **engine-core**: added 5 new `Sprite::Obj` fields: `world_gen_mountain_ridge_octaves`, `world_gen_ice_cap_strength`, `world_gen_lapse_rate`, `world_gen_rain_shadow`, `world_gen_subdivisions`
- **engine-compositor**: `sprite_renderer.rs` extended URI builder for all 4 new params + subdivisions; `obj_render.rs` `parse_world_params_from_uri` parses `mroct`, `ice`, `lapse`, `rainshadow` keys
- **engine-scene-runtime**: added 5 new `world.*` Rhai property paths: `mountain_ridge_octaves`, `ice_cap_strength`, `lapse_rate`, `rain_shadow`, `subdivisions`; added 5 fields to `Sprite::Obj` destructure
- **engine-behavior**: added `engine-terrain` dep; new `scripting/world.rs` module registers `planet_last_stats()` Rhai function returning biome coverage map (ocean, shallow, desert, grassland, forest, cold, mountain)
- **planet-generator mod**: new standalone mod with a full-screen planet viewer and tabbed HUD: 4 tabs (Continents / Mountains / Climate / Visual) × 5-6 sliders each; 7 presets (Earth/Mars/Ocean/Desert/Ice/Volcanic/Archipelago) via F1–F7; R=randomize, Delete=reset; live stats bar reading `planet_last_stats()` for ocean/forest/desert/snow% coverage; sun azimuth+elevation→ light direction math; orbit camera (Ctrl+F)

**engine-core / engine-compositor / engine-scene-runtime ambient & lighting** ✅
- **engine-core**: added `ambient: Option<f32>` field to `Sprite::Obj` (was previously silently ignored in YAML; now properly deserialized and exposed)
- **engine-compositor**: wire `ambient` from `Sprite::Obj` through `ObjRenderParams` (falls back to `0.15` if not set)
- **engine-scene-runtime**: added `obj.ambient`, `obj.light.x`, `obj.light.y`, `obj.light.z`, and `obj.rotation-speed` runtime property paths in `materialization.rs`; added `ambient` + `light_direction_x/y/z` to the destructure block so scripts can adjust lighting at runtime
- **terrain-playground**: added 3rd scene `terrain-sphere` — cube-sphere://32 mesh with full orbit-camera (-85°/+85° pitch), smooth shading, noise surface mode, 5-param HUD panel (ambient, rotation speed, light X/Y/Z), and Rhai script wiring all `obj.*` paths; updated menu with 4th item "SPHERE TERRAIN" (shortcut [3])

**Build performance: lld linker + incremental compilation re-enabled** ✅
- **build**: switched dev linker from MSVC `link.exe` to `rust-lld` (bundled with Rust) via `.cargo/config.toml` — 2–4× faster linking on large workspaces, no extra install required
- **build**: re-enabled incremental compilation (`incremental = true`) in dev profile — was disabled due to old Windows NTFS hard-link warning; warning is now non-fatal (copies instead of links), net benefit over full recompile
- **docs**: `.cargo/config.toml` comments updated to document rationale for both changes

**Unified input event architecture** ✅
- **engine-events**: renamed `KeyPressed` → `KeyDown { key, repeat }` and `KeyReleased` → `KeyUp { key }`; mouse coords changed from `u16` to `f32` (output-space); `button: String` replaced by typed `MouseButton` enum; added `InputEvent` sub-enum and `EngineEvent::as_input_event()` for fan-out
- **engine-gui**: `GuiSystem::update` now accepts `&[engine_events::InputEvent]` instead of `&[GuiInputEvent]`; mouse coords are `f32`; `drag_button` uses typed `MouseButton`; keyboard events accepted (pass-through stub); `GuiInputEvent` kept as `#[deprecated]` alias
- **engine-render-sdl2**: `map_mouse_to_output` returns `(f32, f32)`; `map_mouse_button` returns `MouseButton` enum; all `EngineEvent` emissions updated
- **engine/scene_lifecycle**: `classify_events` calls `as_input_event()` for every event, collecting `input_events: Vec<InputEvent>`; old separate `mouse_moves/buttons_down/up` vectors removed; `handle_gui_mouse_events` replaced by `handle_gui_input_events`; free-look/3D-mouse helpers use `(f32, f32)`; test helpers updated
- **engine/game_loop**: match arm updated to `EngineEvent::KeyDown { key, .. }`
- **editor**: `scene_run.rs` updated to push `EngineEvent::KeyDown { key, repeat: false }`
- **engine-behavior**: `BehaviorContext.mouse_x/y` changed to `f32`; `ScriptGuiApi.mouse_x/y` cast to `rhai::INT` via `as rhai::INT` (no Rhai script changes needed)
- **engine-gui**: created `README.md` documenting the crate's role, widget types, and input contract
- **docs**: updated `engine-events/README.md`, `engine-render-sdl2/README.md`, `engine-behavior/README.md`, `ARCHITECTURE.md` (section 9 input pipeline diagram + change playbook row)
- **result**: keyboard and mouse events now flow through a single `InputEvent` slice to all consumers; `GuiInputEvent` fully deprecated; zero Rhai script changes required

**Resolution slider + throttled parameter updates** ✅
- **planet-generator**: added RESOLUTION slider (16/32/64/128 subdivisions) to Visual tab for mesh quality control; throttled `world.*` param pushes to 500ms intervals to prevent blocking the render thread during continuous slider adjustment; visual-only params (rotation, lighting) still update every frame
- **engine-scene-runtime**: fixed `as_u64()` type mismatch for `mountain_ridge_octaves` and `subdivisions` — Rhai sends all numbers as float; added `as_f64().map(|f| f as u64)` fallback

**Build reliability: git-aware rebuild tracking** ✅
- **app**: added `build.rs` that watches `.git/HEAD` and `.git/index`; embeds `BUILD_GIT_HASH` env var so cargo detects commits/staging; startup banner now shows `ShellEngine [hash] initialized`

**Documentation update** ✅
- **docs**: added `engine-terrain/README.md` (full pipeline, params, modules, integration); added world generation section to `ARCHITECTURE.md` (section 13, pipeline diagram, Rhai property table, change playbook row); added `world://` URI authoring guide + param table to `AUTHORING.md`; added `planet_last_stats()` to `SCRIPTING-API.md`; added planet-generator mod to `MODS.md`; updated `engine-mesh/README.md`, `engine-behavior/README.AGENTS.md`, `engine-core/README.AGENTS.MD`, `app/README.AGENTS.MD`, `AGENTS.md`

## 13-04-2026


**SDL2-only migration complete — terminal renderer fully removed** ✅
- **engine**: removed `SceneRenderedMode` enum and `rendered_mode` field from Scene; removed `HalfblockPacker`, `FullScanPacker`, `DirtyRegionPacker` strategies; removed `ratatui` and `crossterm` dependencies from all crates
- **engine-render**: removed `DisplaySink`/`DisplayFrame` traits; `CellPatch` renamed to `GlyphPatch` in SDL2 renderer
- **engine-render-policy**: removed `resolve_renderer_mode()`; `renderer-mode` and `force-renderer-mode` config fields gone
- **engine-runtime**: removed `SHELL_ENGINE_RENDERER_MODE` env var and `SceneRenderedMode` parsing
- **engine-pipeline**: removed `HalfblockPacker` from `PipelineStrategies`; `renderer-mode locking` flag removed
- **engine-compositor**: halfblock packing path removed; `convert_to_terminal_colors` renamed to `convert_rgba_to_rgb_samples`; `terminal_crt` effect renamed to `crt-filter`
- **editor**: terminal TUI (ratatui/crossterm) replaced with SDL2 stub; terminal launcher menu replaced with stdin-based menu
- **docs**: updated ARCHITECTURE.md, AGENTS.md, ARCH.MD, OPTIMIZATIONS.md, AUTHORING.md, BENCHMARKING.md, and all subsystem READMEs to reflect SDL2-only architecture

## 12-04-2026

**Terminal output backend removed — SDL2-only, display manifest rename** ✅
- **engine**: deleted `engine-render-terminal` and `engine-terminal` crates; SDL2 is now the only renderer backend; no feature flags or runtime selection remain
- **engine-mod**: replaced `terminal_caps.rs` with `display_config.rs`; `target_fps_from_manifest()` now reads from `display:` block
- **engine-error**: renamed `TerminalRequirementsNotMet` → `DisplayRequirementsNotMet`
- **engine-runtime**: manifest parsing updated; `terminal:` block renamed to `display:` in all parsing paths and tests
- **app**: removed `resolve_startup_output()` and manifest-driven backend selection; `StartupOutputSetting::Sdl2` is hardcoded
- **launcher**: removed `--sdl2`/`--output` CLI args, `sdl2: bool` flag from `LaunchFlags`, SDL2 toggle from menu; always passes `engine/sdl2` feature; reads `display.render_size` instead of `terminal.*`
- **mods**: all `mod.yaml` manifests updated — `terminal:` → `display:`, `output: sdl2`/`output_backend:` lines removed
- **schemas**: `schemas/mod.schema.yaml` and all per-mod generated schemas regenerated; `output_backend` property removed; `terminal` → `display` throughout
- **docs**: `ARCHITECTURE.md`, `ARCH.MD`, `BENCHMARKING.md`, `OPTIMIZATIONS.md`, `MODS.md`, `app/README.AGENTS.MD`, `engine-render*/README.md`, `engine-mod/README.md` updated to reflect SDL2-only stack
- **result**: zero terminal-backend references in source or schemas; `cargo build --workspace --features sdl2` clean; all mod scene checks pass

## 11-04-2026

**Asteroids realism-first hybrid orbital rework (radius/atmosphere/HUD)** ✅
- **mods/asteroids**: replaced fixed-radius orbit lock with live orbital state (`radius`, `vrad`, `vfwd`, `vright`) driven by body catalog gravity (`gravity_mu`) and geodesic tangent transport
- **mods/asteroids**: added atmosphere model fields in `catalogs/bodies.yaml` (`atmosphere_top`, `atmosphere_dense_start`, `atmosphere_drag_max`) and wired drag/heat into flight dynamics
- **mods/asteroids**: introduced reentry failure logic (thermal tick damage, severe reentry kill path, surface-impact death) integrated with existing lives/game-over flow
- **mods/asteroids**: rebuilt orbital telemetry HUD from 1-row to 2-row pilot panel: `ALT`, `STATUS`, `TSPD`, `RSPD`, `HEAT`, `VXY`
- **mods/asteroids**: steering pass updated to target yaw-rate response with stronger side-slip trim under thrust; chase camera + ship scene placement now follow live orbital altitude (`radius * SCENE_SCALE`)
- **docs**: updated `MODS.md` Asteroids section for the hybrid orbital model, atmosphere/reentry behavior, updated layer stack, and current feel parameters
- **validation**: repeated `--check-scenes` and runtime `--start-scene ... --bench 1` smoke runs pass after the rework

**Asteroids orbital control complete: geodesic transport, RCS gimbal, in-game feel tuning** ✅
- **mods/asteroids**: orbital flight model — `sn/sf/sr` sphere normal/forward/right basis vectors; yaw rotates `sf/sr` around `sn`; translation via geodesic transport (Rodrigues rotation of `sn` and `sf` per frame)
- **mods/asteroids**: input separation — `turn_left/right` (yaw via RCS), `strafe_left/right` (lateral), `thrust/brake` (prograde/retro) now independent (no heading-derived physics)
- **mods/asteroids**: RCS VFX pipeline (`mods/asteroids/scripts/rcs.rhai`) — 4-emitter system (main/bow/port/starboard); rotation couple with yaw rate feedback; rotation brake on release; linear trim corrections; settling puff at rest
- **mods/asteroids**: main engine 3-phase profile — ignition (hot white/cyan), transition (mid cyan), sustain (blue burn), fade (cool tail); driven by thrust hold/release timers
- **mods/asteroids**: camera inertial lag — `cam_n` (instant), `cam_up` (exponential smoothing, τ=0.68s), sway linked to yaw rate; gimbal feel without disconnection
- **mods/asteroids**: critical bug fixes — dt cap removed (physics no longer freezes on FPS drops); camera 1-frame lag eliminated; bullet carries full tangential velocity
- **mods/asteroids**: feel tuning applied — `YAW_ACCEL: 1.8→1.95` (snappier), `YAW_DAMP: 2.4→2.6` (faster settle), `CAM_UP_TAU: 0.58→0.68` (heavier gimbal), `CAM_SWAY_GAIN: 0.24→0.28` (banking cues)
- **validation**: `--check-scenes` passes all 9 startup checks; no build errors
- **result**: Asteroids orbital flight is now cinematic (5-min orbit, 450px radius, 9.42 px/s baseline), responsive (snappy yaw, independent controls), and visually rich (multi-phase engine VFX, planet gimbal camera, RCS torque clarity)

**Asteroids orbital rendering, live Scene3D clips, and planet authoring refresh** ✅
- **engine-compositor**: added `Scene3DRuntimeStore` live clip path — bare Scene3D clip ids like `solar-orbit` now render against current elapsed time instead of requiring pre-baked `clip-0..N` frame selection
- **engine-3d / schemas**: Scene3D authoring extended with point-light `falloff_constant`, orbit/clip tween properties, and structured transform coverage for richer large-scale background scenes
- **engine / authoring**: OBJ sprites gained planet-oriented biome shading controls (`polar-ice-*`, `desert-*`, `atmo-*`, `night-light-*`) plus transparent thresholded cloud overlays; added `lens-blur` post-FX
- **engine-behavior / engine-game**: emitter catalogs now support worker-thread particle physics, collisions, orbital gravity attractors, and palette-driven lifetime ramps
- **mods/asteroids**: game scene rebuilt around orbital flight (prograde / retro / strafe), cockpit-follow planet rendering, layered stars → planet → gameplay canvas composition, and bottom-centre orbital telemetry HUD
- **result**: Asteroids now plays as a cinematic orbital dogfight with cockpit-follow planet rendering and much richer planet / particle authoring controls

---

## 08-04-2026

**Asteroids HUD: transparent panels, 3-layer background, retro pixel-art hearts** ✅
- **engine**: `sprite_renderer.rs` — panel `bg` and shadow default from `DarkGrey` → `Color::Reset`; `set_panel_cell` skips writes when bg is `Reset`, making HUD corner panels fully transparent
- **engine**: `generic.rs` — added `♥` glyph (5×7 pixel bitmap) to built-in generic font; available in `generic:2` (standard) and `generic:3` (large/2× scale) modes; removes need for vector polygon hearts
- **mods/asteroids**: `stars-layer.yml` (z=0) — 22 text-sprite star field (5 gold accent `*`, 17 dim `.`) as background plane
- **mods/asteroids**: `planets-layer.yml` (z=1) — 3 closed vector polygon circles (large planet, small planet, moon) using `@palette.planet_body/rim` bindings
- **mods/asteroids**: all 3 palettes (`neon`, `classic`, `teal`) — added `planet_body` and `planet_rim` keys
- **mods/asteroids**: `hud-grid.yml` — replaced smooth vector polygon hearts with 3 retro pixel-art text sprites (`font: "generic:3"`, `scale-x/y: 2.0`, 24×28 px each); properly centred in 142×38 px lives-panel inner area (y=11, x=23/64/105)
- **mods/asteroids**: `scene.yml` — added layer refs: stars → planets → hud draw order (z-sorted background stack)
- **docs**: `docs/layout/hud-design.svg` — design mockup with 3-layer exploded isometric view, composited HUD preview, and component specs
- **result**: HUD panels are fully transparent (game field visible through corners); star field + planet background composited behind gameplay; lives display is chunky retro pixel-art ♥ icons



**Engine-first scripting migration: components, palette, HUD bindings, optimizations** ✅
- **engine**: `LinearBrake` engine component — physics deceleration fully handled engine-side; removes braking logic from Rhai
- **engine**: `ThrusterRamp` engine component — moves entire thrust ramp state machine (burst/wave/auto-brake) from Rhai to Rust
- **engine**: `world.after_ms` / `world.timer_fired` — replaces manual `-16.0` timer decrements
- **engine**: `world.spawn_from_heading` — atomic bullet spawn with heading/offset/speed; removes sin/cos trig from scripts
- **engine**: `world.heading_drift` — projects velocity onto ship axes; removes manual cross/dot product trig
- **engine**: `collision.enters(a, b)` typed pair API — eliminates `tag_has` boilerplate from collision dispatch
- **engine**: `frame_ms` exposed to Rhai scope — replaces hardcoded `16.0` VFX timing constants
- **engine**: `PrefabTemplate.fg_colour` + `default_tags` — color and tags applied at spawn; removes per-spawn `world.set` and `tags: [...]` args from scripts
- **engine**: `wrappable: true` in prefab catalog — `world.enable_wrap` calls removed from all spawn helpers
- **engine**: YAML `@palette.<key>` bindings in sprite `fg`/`bg` fields — engine resolves on palette change; no Rhai color push needed
- **engine**: YAML `@game_state.<path>` bindings in sprite `content` fields — HUD text updated engine-side via `game.set()`; removes 3 cache-diff blocks from scripts
- **engine**: `ParticleColorRamp` component — color+radius ramp applied engine-side per emitter; removes per-frame ramp update loops from Rhai
- **engine**: palette-aware emitter color ramps via `palette.particles("ramp_name")` in YAML emitter config
- **engine**: indexed palette access — `palette.color_at(n)`, `palette.key_at(n)`, `palette.colors_len()`, `palette.color_keys()`, `palette.color_values()`
- **perf**: 7 optimizations (o1–o7) — throttle ramp, lazy palette re-read, frame_ms constant, collision filter short-circuit, heading vector cache, drift debounce, tag-query fastpath
- **mods/asteroids**: extracted ship RCS VFX into `scripts/rcs.rhai`; neutralised all mod-specific names in engine code
- **docs**: updated `SCRIPTING-API.md` with AngularBody, LinearBrake, ThrusterRamp, heading helpers, frame_ms
- **result**: `game-loop.rhai` down ~40% in LOC; all color/wrap/tag/HUD boilerplate moved to YAML + engine; 103 tests pass; scene checks pass

## 29-03-2026 (E7: Final cleanup + collision filtering)

**E7: Final cleanup + collision filtering migration complete** ✅
- **mods/asteroids**: replaced manual collision dispatch with filtered collision APIs — `world.collisions_between("ship", "asteroid")` and `world.collisions_between("bullet", "asteroid")`
- **mods/asteroids**: removed 11 LOC from collision dispatch (~57 → ~28 per collision type); eliminated nested if-chains checking `hit.contains("a")`/`hit.contains("b")`
- **mods/asteroids**: removed `despawn_entity_visual()` helper — E1 auto-despawn now handles visual cleanup on `world.despawn()` calls
- **mods/asteroids**: modularized shared helpers into `scripts/asteroids-shared.rhai` — `crack_duration_ms()`, `fragment_heading_offset()`, `heading32_to_rad()`, visual helpers now imported via `import "asteroids-shared" as h;`
- **engine-behavior**: fixed Rhai module resolver initialization — `RhaiScriptBehavior::from_params()` now calls `init_rhai_engine()` to apply module resolver; `init_behavior_system()` called on app startup
- **app**: added `init_behavior_system(&mod_source)` call before scene checks to ensure Rhai module resolution works correctly
- **result**: ~35 LOC removed across game-loop + render-sync; asteroids-game-loop.rhai: 921 → 886 LOC (-3.8%); modular script structure ready for extensibility; all 62 behavior tests pass; scene checks pass; collisions work correctly (bullets kill asteroids, ship dies on impact)

## 29-03-2026
- **mods/asteroids**: migrated from manual fixed-point physics (`x_fp`, `dx_fp` manual integration) to engine PhysicsBody2D 
- **mods/asteroids**: replaced asteroids/bullet/smoke position integration with engine physics step (SimpleEulerIntegration: velocity + drag + max_speed)
- **mods/asteroids**: simplified bullet/smoke loops: now read positions from Transform2D, wrap toroidally, skip manual `x += dx; y += dy` integration
- **mods/asteroids**: toroid wrapping now handled consistently: script wraps engine-integrated positions (collision detection still via WrapStrategy::None + manual wrap)
- **mods/asteroids**: smoke drag (0.96 factor per frame) mapped to PhysicsBody2D drag coefficient (0.04); all velocity parameters now float instead of fixed-point integers
- **result**: ~30 LOC removed from physics update loops; all entity types (asteroids/bullets/smoke) use consistent PhysicsBody2D integration; preflight validation passes; visual behavior unchanged (acceptance tests OK)

**Scripting modernization (A1-A4 continuation)**
- **engine**: A1 — auto-despawn visuals on `world.despawn(id)` and `entity.despawn()`; multi-visual binding via `world.bind_visual()` and `VisualBinding.additional_visuals`
- **engine**: A2 — unified `world.spawn_visual(kind, template, data)` atomic spawn (entity + visual + binding + transform + collider in one call)
- **engine**: A3 — `visual_sync_system` auto-copies Transform2D → scene position.x/y after behavior step, before compositor
- **engine**: A4 — Rhai `FileModuleResolver` for shared script modules; `import "module" as m;` resolves from `{mod}/scripts/`
- **engine**: added `entity.get_b()` alias for `get_bool()`, `entity.despawn()` method, `world.bind_visual()` function
- **engine**: legacy cleanup — removed unused `ScriptTimerApi`, `ScriptPrefabApi`, `ScriptSpawnerApi` stubs; removed `input.is_down()` duplicate; removed dead `rhai_map_to_json`
- **engine**: marked 6 asteroids-specific geometry functions (ship_points, asteroid_points, etc.) for extraction to mod-level shared module
- **mods**: asteroids entity-ref migration — replaced all session-map bulk reads/writes with `session_ref.get_i/set` (world.get 18→8, world.set 17→7)
- **docs**: rewrote `scripting.md` as canonical 832-line contract + enhancement roadmap (107 Rhai functions, 7 component types, 7 implementation tasks, target 1091→330 LOC)
- **docs**: updated engine-behavior, engine-game, engine README module docs for new APIs
- **result**: E1-E4 scripting migration tasks complete; engine-side infra ready for E5-E7 (physics, audio, rendering tasks in dependent mods)

## 29-03-2026 (earlier)

**Audio sequencing, Asteroids modularization, and startup validation**
- **audio**: added YAML-driven audio sequencer with semantic SFX bank, song library, and synth note-sheet generation from `audio/synth/`
- **audio**: switched Asteroids to synth-first cue playback with in-memory generated tones plus scene-driven menu/game/highscore song playback
- **engine**: inserted audio sequencer tick into frame loop and exposed `audio.event`, `audio.cue`, `audio.play_song`, and `audio.stop_song` to Rhai
- **engine**: exposed typed gameplay Rhai API (transform/physics/collider/lifetime, collision buffer) and wired collision events into behavior context
- **engine**: added `world.set_visual(...)` Rhai API plus runtime visual cleanup queue for lifetime-based despawns
- **engine**: collision system now applies toroidal wrap bounds from active render buffer dimensions
- **startup**: added `--check-scenes` runner with scene graph, level config, Rhai script, font/image, and audio sequencer checks
- **authoring**: mod behaviors now support external Rhai via `src`; Asteroids gameplay/render logic moved out of inline YAML wrappers
- **launcher**: `./menu` now persists SDL2, audio, splash-skip, scene-check, and release flags; audio defaults on and release launches show cargo build progress
- **mods**: added `mods/asteroids` showcase mod with levels, dynamic runtime entities, synth audio, and SDL-oriented launcher flow
- **mods**: Asteroids gameplay migrated to component-backed spawns (transform/physics/collider/lifetime) and collision-buffer handling
- **docs**: refreshed architecture/authoring/mod/runtime docs for scene checks, synth audio, behavior `src`, and current launcher flow

---

## 28-03-2026

**SDL splash unification, readability pass, and startup controls**
- **splash**: unified startup splash flow across terminal and SDL2; removed backend divergence
- **splash**: added dedicated SDL splash presentation mode (aspect-preserving fit) plus centered scale handling
- **splash**: improved timeline behavior so authored splash stages (including fade) are not cut by short audio
- **splash**: added mod-level splash config in `mod.yaml` (`splash.enabled`, `splash.scene`) with safe fallback to engine default
- **schemas**: extended `mod.schema.yaml` and mod overlay schema generator with splash config support
- **engine-render-sdl2**: splash letterbox clear now matches splash background instead of hard black
- **testing**: added splash config parser tests and verified engine/app compile paths

---

## 27-03-2026

**SDL2 rendering optimizations & font pipeline** 🚀
- **engine-render-sdl2**: implemented pixel-buffer rasterizer (streaming texture, single DMA upload per frame)
- **engine-render-sdl2**: added shade character anti-aliasing (░▒▓█ → blended fg/bg at 25/50/75/100%)
- **engine-render-sdl2**: FNV-1a static frame skip for flicker-free rendering
- **engine-compositor**: added `scale-x`/`scale-y` fields to text sprites with nearest-neighbor blitting
- **engine-render-policy**: backend-aware font resolution — SDL2 auto-selects `:raster` mode for named fonts
- **engine-runtime**: propagated `is_pixel_backend` flag through compositor pipeline (8-file threading)
- **testing**: 25 compositor tests pass, new font policy tests added, headless SDL smoke test passes
- **result**: font rendering now backend-specific; SDL gets shade glyphs + stretch capability

---

## 26-03-2026

**Crate rebalancing complete (28 commits)** 🏗️
- **architecture**: extracted engine into 15 sub-crates: `engine-core`, `engine-pipeline`, `engine-mod`, `engine-render-terminal`, `engine-compositor`, `engine-behavior`, `engine-scene-runtime`, `engine-asset`, and more
- **design**: domain `XxxAccess` traits (BufferAccess, GameStateAccess, AssetAccess, AnimatorAccess, EventAccess, DebugAccess, RuntimeAccess, AudioAccess) enable decoupled provider impls
- **engine-core**: moved World, AssetRoot, AssetCache, GameState, runtime data types, and color system
- **color**: decoupled Color type from crossterm dependency (migrated to engine-core)
- **testing**: verified zero regressions — all 204 engine tests passing post-refactor
- **result**: 15 focused crates with clear boundaries; terminal renderer now isolated in engine-render-terminal; orphan rule satisfied via newtype wrappers

---

## 25-03-2026

**Documentation consolidation complete** ✅
- **docs**: consolidated 26 scattered files into 5 focused docs + 20 crate READMEs
- **docs**: added CHANGELOG format guidelines for standardized daily reporting
- **testing**: verified all 204 engine tests passing (zero regressions)
- **result**: 69% doc reduction (26 → 8 root files), clear navigation hierarchy

**Input regression fix** 🔧
- **tests**: restored input handling in test scene (trigger: any-key instead of timeout)
- **ui**: verified lightning background effects render during on_idle phase
- **testing**: confirmed all 204 tests still passing post-fix

---

## 24-03-2026

**Splash screen refresh & optimization experiments**
- **splash**: new splash screen design
- **optimizations**: attempted aggressive optimization; rolled back changes to apply more gradually
- **graphics**: planning difficulty menu rework
- **sidecar**: will be rewritten in Rust with improvements
- **audio**: experimented with 90s machine simulation (floppy, HD, modem sounds)

---

## 23-03-2026

**Rendering pipeline & architectural improvements**
- **optimizations**: rendering pass refactored; no regressions on 3D drawing; prerendering pipelines under revision
- **gpu & parallelization**: researching GPU offload; currently single-CPU bound; terminal is another render layer
- **effects & shaders**: proof-of-concept shaders require optimization; considering GPU acceleration
- **postfx**: heavy focus on CRT look/feel (key visual for terminal aesthetic)
- **engine**: separated 3D rendering concerns; prerender now possible at lower cost; some z-flip vertex issues
- **sound**: audio works via rodio without needing server; playground demo available
- **C# sidecar**: basic navigation and commands working
- **plot**: started quest design work; researching historical details for immersion

---

## [Unreleased] — Prologue & Feature Implementations

### Added

- **Prologue architecture**: Difficulty enum (5 levels), MachineSpec hardware config, per-difficulty resource scaling
- **Shell commands**: cd, pwd, cp (with disk space checks), ftp (FTP session mode)
- **FTP client**: Full simulation with ASCII/binary modes, DNS, transfer delays, discovery puzzle
- **Mutable filesystem**: IMutableFileSystem interface, ZipVirtualFileSystem overlay, boot file seeding
- **Quest tracking**: QuestState (FtpTransferMode, UploadAttempted, BackupMade, UploadSuccess)
- **Timeline validation**: Compile-time sprite timing validation (appear_at_ms checks, disappear_at_ms validation)
- **Snap lighting**: light-point-snap-hz fields for instant lighting jumps (difficulty menu 3D portraits)
- **Neon edge glow**: New builtin effect with 3-ring spillover and breathing pulse
- **Menu highlight behavior**: Dynamic per-item styling (bright selected, dim unselected)
- **Difficulty animation**: Portrait rotation + forward lean on confirm, periodic glitch flashes, neon cycles
- **Strategy optimization**: 9 traits with safe/optimized implementations; CLI flags (--opt-comp, --opt-present, etc)
- **Benchmark system**: --bench flag with per-frame sampling, scene breakdown, CSV reports
- **Test mod**: shell-engine-tests with compressed scenes (~9.4s loop, all timeouts, no user input)
- **Frame capture**: --capture-frames with binary comparison for regression testing

### Fixed

- **Visual regressions**: Transparency on timed sprites, image ghosting, CRT artifacts, animation flicker
- **Boot sequence**: Fixed sprite leak, scene timing, GIF duration (10530ms), realistic I/O delays
- **Scene cleanup**: Verified world.clear_scoped() properly isolates scenes

### Changed

- **Timeline semantics**: Sprite timing is absolute (scene-relative), not layer-relative
- **Snap vs Orbit**: Snap takes priority when both lighting modes specified

---

## Testing Status

- **Engine**: 204 tests passing ✓
- **Engine-authoring**: 73 tests passing (includes timeline validation)
- **Engine-core**: 79 tests passing

---

## Documentation

See **[ARCHITECTURE.md](ARCHITECTURE.md)**, **[AUTHORING.md](AUTHORING.md)**, **[MODS.md](MODS.md)**, **[OPTIMIZATIONS.md](OPTIMIZATIONS.md)**, **[AGENTS.md](AGENTS.md)** for comprehensive reference.

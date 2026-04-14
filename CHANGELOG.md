# Changelog

Daily progress updates for Shell Quest development.

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

## 14-04-2026

**Sphere terrain scene + `obj.ambient` / `obj.light.*` runtime property paths** ✅
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

## 13-04-2026


**SDL2-only migration complete — terminal renderer fully removed** ✅
- **engine**: removed `SceneRenderedMode` enum and `rendered_mode` field from Scene; removed `HalfblockPacker`, `FullScanPacker`, `DirtyRegionPacker` strategies; removed `ratatui` and `crossterm` dependencies from all crates
- **engine-render**: removed `DisplaySink`/`DisplayFrame` traits; `CellPatch` renamed to `GlyphPatch` in SDL2 renderer
- **engine-render-policy**: removed `resolve_renderer_mode()`; `renderer-mode` and `force-renderer-mode` config fields gone
- **engine-runtime**: removed `SHELL_QUEST_RENDERER_MODE` env var and `SceneRenderedMode` parsing
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
- **Test mod**: shell-quest-tests with compressed scenes (~9.4s loop, all timeouts, no user input)
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

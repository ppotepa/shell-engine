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

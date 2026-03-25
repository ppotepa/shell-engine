# Changelog

Daily progress updates for Shell Quest development.

---

## 25-03-2026

**Documentation consolidation complete** ✅
- Consolidated 26 scattered markdown files → 5 focused root docs + 20 crate READMEs
- Created ARCHITECTURE.md (352 lines), AUTHORING.md (260 lines), MODS.md (154 lines), OPTIMIZATIONS.md (100 lines)
- Rewrote BENCHMARKING.md consolidating frame capture + test mod specs
- Updated AGENTS.md navigation model (two-tier: consolidated + subsystem)
- Archived 13 source docs in docs/ARCHIVED/, deleted 9 refactor history files
- **Result**: 69% doc reduction (26 → 8 files), zero test regressions (204/204 passing)

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

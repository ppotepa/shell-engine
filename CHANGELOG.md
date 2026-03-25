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

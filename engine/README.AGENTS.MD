# Engine — Runtime Systems & Optimization

## Strategy Pattern Architecture

All pipeline optimizations use trait-based strategy dispatch. No boolean branching in hot loops.

### PipelineStrategies Container

```rust
pub struct PipelineStrategies {
    pub diff:      Box<dyn DiffStrategy>,
    pub layer:     Box<dyn LayerCompositor>,
    pub halfblock: Box<dyn HalfblockPacker>,
    pub present:   Box<dyn VirtualPresenter>,
    pub flush:     Box<dyn TerminalFlusher>,
    pub display:   Box<dyn DisplaySink>,
}
```

Registered once in `World` at startup. All systems call trait methods.

### Per-Frame Systems (11 total)

| System | Time Tracking | Purpose |
|--------|---------------|---------|
| input | input_us | Crossterm event polling |
| lifecycle | lifecycle_us | Scene transitions, event drain |
| animator | animator_us | Stage/step advancement |
| hot_reload | hot_reload_us | Dev file scanning |
| engine_io | engine_io_us | Sidecar IPC bridge |
| behavior | behavior_us | Rhai script execution |
| audio | audio_us | Audio playback tick |
| compositor | compositor_us | Layer blitting + sprite rendering |
| postfx | postfx_us | Post-processing effects |
| renderer | renderer_us | Diff + flush pipeline |
| sleep | sleep_us | Frame budget sleep |

## Optimization Re-enablement Status

| Flag | Status | Implementation |
|------|--------|-----------------|
| `--opt-comp` | ✅ Working | Layer scratch skip, dirty-region halfblock packing |
| `--opt-present` | ✅ Working | Hash-based static frame skip, disables postfx cache |
| `--opt-diff` | ⚠️ Experimental | DirtyRegionDiff instead of FullScanDiff |
| `--opt-skip` | ✅ Working | FrameSkipOracle prevents redundant frames |
| `--opt-rowdiff` | ✅ Working | Row-level dirty tracking, skip unchanged rows |

All flags gate implementations in `PipelineStrategies::from_flags()`.

## Benchmark System

**FrameSample** (per-frame):
- `scene_id: String` — current scene
- `frame_us` — total wall-clock time
- All 11 system timings
- `diff_cells`, `dirty_cells`, `total_cells`
- `write_ops` — buffer mutation count

**BenchResults**:
- `samples: Vec<FrameSample>` — all per-frame data
- `fps` statistics (avg, min, max, p50, p95, p99)
- Per-metric stats (frame_us, compositor_us, etc.)
- Per-scene breakdown (computed via `compute_per_scene()`)

**Report output** (reports/benchmark/YYYYMMDD-HHMMSS.txt):
- Score formula: `fps.avg × 10 + (1M / frame.p50) × 5 − frame.p99 / 100`
- ASCII budget chart
- Buffer pipeline stats
- SCENE BREAKDOWN table

## Render Pipeline & Invariants

### Buffer Management

- **Double-buffer** architecture: back-buffer (current), front-buffer (last frame)
- **Dirty tracking**: dirty region bounding box + per-row dirty bits
- **Diff strategies**:
  - FullScanDiff: compare all cells (safe, slow)
  - DirtyRegionDiff: compare only dirty region (~90% savings)
  - RowSkipDiff: row-level dirty skip (even faster)

### Key Invariants

1. **Render thread safety**: Last accepted ticket MUST reset on scene transition
2. **Dirty region preservation**: PostFX pass chain preserves combined dirty region
3. **Fill() dirty region**: Preserve dirty bits when clearing buffer
4. **Frame skip oracle**: Skip redundant frames by content hash, never create artifacts

### Scene Transition Reset

When `scene_generation` bumps:
- `last_accepted_ticket = FrameTicket { generation: new_gen, sim_frame_id: 0 }`
- Ensures all render-thread frames re-evaluate `is_acceptable()`
- Otherwise permanent rejection of all frames

### PostFX Dirty Region

Combined dirty region must carry through entire PostFX pass chain:
1. Compositor sets dirty region
2. Each PostFX pass: AND dirty region with effect bounds
3. Final result: merged dirty region for renderer

## Per-Frame Flow

```
1. Input system polls events
2. Scene lifecycle processes transitions, drains events
3. Animator advances stages/steps
4. Hot-reload checks for file changes
5. Engine-IO exchanges messages with sidecar
6. Behavior system executes Rhai + built-in behaviors
7. Audio system ticks playback
8. Compositor blits layers → combined buffer
9. PostFX pipeline applies effects on dirty region
10. Renderer: DiffStrategy(back, front) → flush to terminal
11. Sleep to maintain frame budget
```

Optional benchmarking: sample all timings + capture scene_id.

## Buffer Dirty Tracking

```rust
pub struct Buffer {
    cells: Vec<Cell>,
    dirty_min: usize,     // Top-left dirty region
    dirty_max: usize,     // Bottom-right dirty region
    dirty_rows: BitSet,   // Per-row dirty flags
    last_diff_count: usize,
    write_count: usize,
}
```

- `dirty_rows` enables RowSkipDiff strategy
- Preserves across compositing pass
- Reset at scene transition or manual clear

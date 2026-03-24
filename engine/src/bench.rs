//! Benchmark mode — collects per-frame metrics for every engine pipeline stage,
//! renders a results screen in-game, and writes a full report to `reports/benchmark/`.

use std::time::{Duration, Instant};

// ── Per-frame sample ────────────────────────────────────────────────

/// Raw timing sample collected each frame (all values in **microseconds**).
#[derive(Clone, Default)]
pub struct FrameSample {
    // top-level wall time
    pub frame_us:      f32,
    // per-system
    pub input_us:      f32,
    pub lifecycle_us:  f32,
    pub animator_us:   f32,
    pub hot_reload_us: f32,
    pub engine_io_us:  f32,
    pub behavior_us:   f32,
    pub audio_us:      f32,
    pub compositor_us: f32,
    pub postfx_us:     f32,
    pub renderer_us:   f32,
    pub sleep_us:      f32,
    // buffer/pipeline counters (per-frame)
    pub diff_cells:    u32,
    pub dirty_cells:   u32,
    pub total_cells:   u32,
    pub write_ops:     u64,
}

// ── Accumulator ─────────────────────────────────────────────────────

/// Lives as a `World` resource while `--bench` is active.
pub struct BenchmarkState {
    pub duration: Duration,
    pub opt_comp: bool,
    pub opt_present: bool,
    pub opt_diff: bool,
    start: Instant,
    samples: Vec<FrameSample>,
    /// Set once the results screen has been rendered.
    pub results_shown: bool,
    results_shown_at: Option<Instant>,
}

impl BenchmarkState {
    pub fn new(duration_secs: f32, opt_comp: bool, opt_present: bool, opt_diff: bool) -> Self {
        let cap = (duration_secs * 120.0) as usize;
        Self {
            duration: Duration::from_secs_f32(duration_secs),
            opt_comp,
            opt_present,
            opt_diff,
            start: Instant::now(),
            samples: Vec::with_capacity(cap),
            results_shown: false,
            results_shown_at: None,
        }
    }

    pub fn push(&mut self, sample: FrameSample) { self.samples.push(sample); }

    pub fn time_up(&self) -> bool { self.start.elapsed() >= self.duration }

    pub fn should_quit(&self) -> bool {
        self.results_shown_at
            .map(|t| t.elapsed() >= Duration::from_secs(5))
            .unwrap_or(false)
    }

    pub fn mark_results_shown(&mut self) {
        if !self.results_shown {
            self.results_shown = true;
            self.results_shown_at = Some(Instant::now());
        }
    }

    pub fn results(&self) -> BenchResults {
        BenchResults::compute(&self.samples, self.opt_comp, self.opt_present, self.opt_diff)
    }
}

// ── Computed results ────────────────────────────────────────────────

/// Statistics for a single metric across all sampled frames.
#[derive(Clone, Default)]
pub struct MetricStats {
    pub avg: f32,
    pub min: f32,
    pub max: f32,
    pub p50: f32,
    pub p95: f32,
    pub p99: f32,
    pub total: f64,
}

impl MetricStats {
    fn from_samples(raw: &[f32]) -> Self {
        if raw.is_empty() {
            return Self::default();
        }
        let mut sorted = raw.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = sorted.len();
        let pct = |p: f32| -> f32 {
            let idx = ((p / 100.0) * (n - 1) as f32).round() as usize;
            sorted[idx.min(n - 1)]
        };
        let total: f64 = raw.iter().map(|v| *v as f64).sum();
        Self {
            avg: (total / n as f64) as f32,
            min: sorted[0],
            max: sorted[n - 1],
            p50: pct(50.0),
            p95: pct(95.0),
            p99: pct(99.0),
            total,
        }
    }
}

pub struct BenchResults {
    pub total_frames: usize,
    pub score: u32,
    // active opt flags
    pub opt_comp: bool,
    pub opt_present: bool,
    pub opt_diff: bool,
    // frame-level
    pub frame:      MetricStats,
    pub fps:        MetricStats,
    // per-system (microseconds)
    pub input:      MetricStats,
    pub lifecycle:  MetricStats,
    pub animator:   MetricStats,
    pub hot_reload: MetricStats,
    pub engine_io:  MetricStats,
    pub behavior:   MetricStats,
    pub audio:      MetricStats,
    pub compositor: MetricStats,
    pub postfx:     MetricStats,
    pub renderer:   MetricStats,
    pub sleep:      MetricStats,
    // buffer pipeline
    pub diff_cells: MetricStats,
    pub dirty_cells: MetricStats,
    pub total_cells: f32,
    pub write_ops: MetricStats,
}

impl BenchResults {
    fn compute(samples: &[FrameSample], opt_comp: bool, opt_present: bool, opt_diff: bool) -> Self {
        let n = samples.len();
        let extract = |f: fn(&FrameSample) -> f32| -> Vec<f32> {
            samples.iter().map(f).collect()
        };

        let frame_us = extract(|s| s.frame_us);
        let fps_vals: Vec<f32> = frame_us
            .iter()
            .map(|&us| if us > 0.0 { 1_000_000.0 / us } else { 0.0 })
            .collect();

        let frame   = MetricStats::from_samples(&frame_us);
        let fps     = MetricStats::from_samples(&fps_vals);

        let input      = MetricStats::from_samples(&extract(|s| s.input_us));
        let lifecycle  = MetricStats::from_samples(&extract(|s| s.lifecycle_us));
        let animator   = MetricStats::from_samples(&extract(|s| s.animator_us));
        let hot_reload = MetricStats::from_samples(&extract(|s| s.hot_reload_us));
        let engine_io  = MetricStats::from_samples(&extract(|s| s.engine_io_us));
        let behavior   = MetricStats::from_samples(&extract(|s| s.behavior_us));
        let audio      = MetricStats::from_samples(&extract(|s| s.audio_us));
        let compositor = MetricStats::from_samples(&extract(|s| s.compositor_us));
        let postfx     = MetricStats::from_samples(&extract(|s| s.postfx_us));
        let renderer   = MetricStats::from_samples(&extract(|s| s.renderer_us));
        let sleep      = MetricStats::from_samples(&extract(|s| s.sleep_us));

        let diff_cells_v: Vec<f32> = samples.iter().map(|s| s.diff_cells as f32).collect();
        let dirty_cells_v: Vec<f32> = samples.iter().map(|s| s.dirty_cells as f32).collect();
        let write_ops_v: Vec<f32> = samples.iter().map(|s| s.write_ops as f32).collect();
        let diff_cells  = MetricStats::from_samples(&diff_cells_v);
        let dirty_cells = MetricStats::from_samples(&dirty_cells_v);
        let write_ops   = MetricStats::from_samples(&write_ops_v);
        let total_cells = samples.first().map(|s| s.total_cells as f32).unwrap_or(0.0);

        // Score: higher is better. Dominated by avg FPS, penalised by variance.
        let score = (fps.avg * 10.0
            + (1_000_000.0 / frame.p50.max(1.0)) * 5.0
            - frame.p99 / 100.0)
            .max(0.0) as u32;

        Self {
            total_frames: n,
            score,
            opt_comp, opt_present, opt_diff,
            frame, fps,
            input, lifecycle, animator, hot_reload, engine_io,
            behavior, audio, compositor, postfx, renderer, sleep,
            diff_cells, dirty_cells, total_cells, write_ops,
        }
    }

    // ── report text ────────────────────────────────────────────────

    /// Generate a full multi-line plain-text report.
    pub fn report_text(&self) -> String {
        let mut r = String::with_capacity(4096);
        r.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        r.push_str("║              SHELL QUEST ENGINE — BENCHMARK REPORT          ║\n");
        r.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        // Optimization flags
        r.push_str("── CONFIGURATION ─────────────────────────────────────────────\n");
        let flag = |b: bool| if b { "ON" } else { "off" };
        r.push_str(&format!("  --opt-comp ........ {}\n", flag(self.opt_comp)));
        r.push_str(&format!("  --opt-present ..... {}\n", flag(self.opt_present)));
        r.push_str(&format!("  --opt-diff ........ {}\n", flag(self.opt_diff)));
        r.push('\n');

        r.push_str(&format!("  SCORE .............. {}\n", self.score));
        r.push_str(&format!("  TOTAL FRAMES ....... {}\n\n", self.total_frames));

        r.push_str("── FPS ────────────────────────────────────────────────────────\n");
        Self::fmt_metric(&mut r, "FPS", &self.fps, "");
        r.push('\n');

        r.push_str("── FRAME TIME (us) ────────────────────────────────────────────\n");
        Self::fmt_metric(&mut r, "Frame", &self.frame, "us");
        r.push('\n');

        r.push_str("── SYSTEM BREAKDOWN (us) ──────────────────────────────────────\n");
        let systems: &[(&str, &MetricStats)] = &[
            ("Input",       &self.input),
            ("Lifecycle",   &self.lifecycle),
            ("Animator",    &self.animator),
            ("HotReload",   &self.hot_reload),
            ("EngineIO",    &self.engine_io),
            ("Behavior",    &self.behavior),
            ("Audio",       &self.audio),
            ("Compositor",  &self.compositor),
            ("PostFX",      &self.postfx),
            ("Renderer",    &self.renderer),
            ("Sleep",       &self.sleep),
        ];
        for (name, stat) in systems {
            Self::fmt_metric(&mut r, name, stat, "us");
        }
        r.push('\n');

        // Budget breakdown (% of avg frame)
        r.push_str("── BUDGET BREAKDOWN (% of avg frame) ─────────────────────────\n");
        let total = self.frame.avg.max(1.0);
        for (name, stat) in systems {
            let pct = stat.avg / total * 100.0;
            let bar_len = (pct / 2.0).round() as usize; // 50 chars = 100%
            let bar: String = "█".repeat(bar_len.min(50));
            r.push_str(&format!("  {:<12} {:>5.1}%  {}\n", name, pct, bar));
        }
        r.push('\n');

        // Buffer pipeline stats
        r.push_str("── BUFFER PIPELINE ───────────────────────────────────────────\n");
        r.push_str(&format!("  Total cells ........ {:.0}\n", self.total_cells));
        Self::fmt_metric(&mut r, "Diff cells", &self.diff_cells, "");
        Self::fmt_metric(&mut r, "Dirty cells", &self.dirty_cells, "");
        Self::fmt_metric(&mut r, "Write ops", &self.write_ops, "");
        if self.total_cells > 0.0 {
            let dirty_pct = self.dirty_cells.avg / self.total_cells * 100.0;
            let diff_pct  = self.diff_cells.avg / self.total_cells * 100.0;
            r.push_str(&format!("  Avg dirty coverage . {:.1}%\n", dirty_pct));
            r.push_str(&format!("  Avg diff coverage .. {:.1}%\n", diff_pct));
        }
        r.push('\n');

        r
    }

    fn fmt_metric(r: &mut String, name: &str, s: &MetricStats, unit: &str) {
        r.push_str(&format!(
            "  {:<12}  avg={:>8.1}{}  min={:>8.1}{}  max={:>8.1}{}  p50={:>8.1}{}  p95={:>8.1}{}  p99={:>8.1}{}\n",
            name, s.avg, unit, s.min, unit, s.max, unit, s.p50, unit, s.p95, unit, s.p99, unit
        ));
    }
}

// ── In-game results screen ──────────────────────────────────────────

/// Render benchmark results onto the buffer in large font.
pub fn render_bench_results(buf: &mut crate::buffer::Buffer, results: &BenchResults) {
    use crate::rasterizer::generic::{rasterize_generic, generic_dimensions};
    use crossterm::style::Color;
    use engine_core::scene::sprite::TextTransform;

    let bg = Color::Rgb { r: 10, g: 10, b: 20 };
    buf.fill(bg);

    let w = buf.width;
    let _h = buf.height;
    let t = TextTransform::None;
    let green  = Color::Rgb { r: 0, g: 255, b: 120 };
    let gold   = Color::Rgb { r: 255, g: 220, b: 50 };
    let silver = Color::Rgb { r: 180, g: 180, b: 200 };
    let dim    = Color::Rgb { r: 100, g: 100, b: 120 };

    // Title — scale 2 (12×14)
    let title = "BENCHMARK";
    let (tw, _) = generic_dimensions(title, 2);
    rasterize_generic(title, 2, green, w.saturating_sub(tw) / 2, 1, buf, &t);

    // Score — scale 3 (18×21)
    let score_text = format!("{}", results.score);
    let (sw, _) = generic_dimensions(&score_text, 3);
    rasterize_generic(&score_text, 3, gold, w.saturating_sub(sw) / 2, 16, buf, &t);

    // FPS line — scale 1
    let fps_line = format!(
        "AVG {:.0} FPS   P50 {:.1}us   P99 {:.1}us   {} FRAMES",
        results.fps.avg, results.frame.p50, results.frame.p99, results.total_frames
    );
    let (fw, _) = generic_dimensions(&fps_line, 1);
    rasterize_generic(&fps_line, 1, silver, w.saturating_sub(fw) / 2, 39, buf, &t);

    // System breakdown — scale 1, two columns
    let systems: &[(&str, &MetricStats)] = &[
        ("COMP", &results.compositor),
        ("REND", &results.renderer),
        ("BHV",  &results.behavior),
        ("PFX",  &results.postfx),
        ("IO",   &results.engine_io),
        ("ANIM", &results.animator),
    ];
    let mut y = 50;
    for chunk in systems.chunks(3) {
        let line = chunk
            .iter()
            .map(|(n, s)| format!("{}: {:.0}us", n, s.avg))
            .collect::<Vec<_>>()
            .join("   ");
        let (lw, _) = generic_dimensions(&line, 1);
        rasterize_generic(&line, 1, dim, w.saturating_sub(lw) / 2, y, buf, &t);
        y += 9;
    }
}

// ── Report file writer ──────────────────────────────────────────────

/// Write the report to `reports/benchmark/<timestamp>.txt` and return the path.
pub fn write_report(results: &BenchResults) -> std::io::Result<std::path::PathBuf> {
    use std::fs;
    use std::io::Write;

    let dir = std::path::PathBuf::from("reports/benchmark");
    fs::create_dir_all(&dir)?;

    let ts = chrono_lite_timestamp();
    let path = dir.join(format!("{ts}.txt"));
    let mut f = fs::File::create(&path)?;
    f.write_all(results.report_text().as_bytes())?;
    f.flush()?;
    Ok(path)
}

/// Cheap timestamp without pulling in chrono — `YYYYMMDD-HHMMSS` UTC.
fn chrono_lite_timestamp() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Manual UTC decomposition (no leap-second precision needed for filenames).
    let days = secs / 86400;
    let day_secs = secs % 86400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;

    // Days since epoch → year/month/day (simplified Gregorian).
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}{mo:02}{d:02}-{h:02}{m:02}{s:02}")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut y = 1970;
    loop {
        let ylen = if is_leap(y) { 366 } else { 365 };
        if days < ylen { break; }
        days -= ylen;
        y += 1;
    }
    let leap = is_leap(y);
    let mdays = [31, if leap {29} else {28}, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mo = 0;
    for (i, &ml) in mdays.iter().enumerate() {
        if days < ml { mo = i as u64 + 1; break; }
        days -= ml;
    }
    (y, mo, days + 1)
}

fn is_leap(y: u64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

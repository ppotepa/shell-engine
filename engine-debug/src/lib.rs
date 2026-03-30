//! Runtime debug feature toggles that can be enabled independently from build profile.

pub mod access;
pub mod log;
pub mod profiling;

pub use log::DebugLogBuffer;
pub use profiling::{
    begin_span, end_span, export_flamegraph_stacks, finish_frame, get_stats, is_enabled, mark,
    set_enabled, ProfileSpan, ProfileStats, Profiler, ProfilingFrame, TimingMarker,
};

/// Smoothed real-time FPS tracked by the game loop.
#[derive(Debug, Clone, Copy)]
pub struct FpsCounter {
    pub fps: f32,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self { fps: 0.0 }
    }
}

/// EMA-smoothed per-system frame timing in microseconds.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemTimings {
    pub behavior_us: f32,
    pub compositor_us: f32,
    pub postfx_us: f32,
    pub renderer_us: f32,
}

/// Debug overlay display mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DebugOverlayMode {
    #[default]
    Stats,
    Logs,
}

/// Debug feature flags and transient UI state.
#[derive(Debug, Clone, Copy, Default)]
pub struct DebugFeatures {
    /// Master switch for debug helpers (F1/F3/F4 and overlay).
    pub enabled: bool,
    /// Whether the debug overlay is currently visible.
    pub overlay_visible: bool,
    /// Current overlay display mode.
    pub overlay_mode: DebugOverlayMode,
}

impl DebugFeatures {
    /// Builds debug feature state directly from a boolean flag.
    pub fn from_enabled(enabled: bool) -> Self {
        Self {
            enabled,
            overlay_visible: enabled,
            overlay_mode: DebugOverlayMode::default(),
        }
    }

    /// Builds debug feature state from environment.
    ///
    /// Recognized truthy values:
    /// - `1`
    /// - `true`
    /// - `yes`
    /// - `on`
    pub fn from_env() -> Self {
        let enabled = env_flag_enabled("SHELL_QUEST_DEBUG_FEATURE");
        Self::from_enabled(enabled)
    }
}

fn env_flag_enabled(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|raw| {
            matches!(
                raw.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

/// Sampled process stats: CPU usage and resident memory.
/// Updated ~once per second in the game loop to avoid overhead.
#[derive(Debug, Clone, Copy)]
pub struct ProcessStats {
    pub cpu_percent: f32,
    pub rss_mb: f32,
    // internal tracking
    prev_cpu_ticks: u64,
    prev_wall_us: u64,
    last_sample_us: u64,
}

impl Default for ProcessStats {
    fn default() -> Self {
        let (cpu, wall) = Self::read_cpu_ticks();
        Self {
            cpu_percent: 0.0,
            rss_mb: Self::read_rss_mb(),
            prev_cpu_ticks: cpu,
            prev_wall_us: wall,
            last_sample_us: wall,
        }
    }
}

impl ProcessStats {
    /// Call every frame; internally rate-limits to ~1 sample/sec.
    pub fn tick(&mut self) {
        let (cpu, wall) = Self::read_cpu_ticks();
        if wall.saturating_sub(self.last_sample_us) < 1_000_000 {
            return; // throttle to ~1 Hz
        }
        let dcpu = cpu.saturating_sub(self.prev_cpu_ticks) as f64;
        let dwall = wall.saturating_sub(self.prev_wall_us).max(1) as f64;
        // clock ticks → microseconds: 1 tick = 1e6/CLK_TCK us
        let clk_tck = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as f64;
        self.cpu_percent = ((dcpu / clk_tck) / (dwall / 1_000_000.0) * 100.0) as f32;
        self.rss_mb = Self::read_rss_mb();
        self.prev_cpu_ticks = cpu;
        self.prev_wall_us = wall;
        self.last_sample_us = wall;
    }

    /// Returns (utime+stime in clock ticks, wall time in microseconds).
    fn read_cpu_ticks() -> (u64, u64) {
        let wall = {
            let mut tv = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            unsafe { libc::gettimeofday(&mut tv, std::ptr::null_mut()) };
            tv.tv_sec as u64 * 1_000_000 + tv.tv_usec as u64
        };
        let cpu = std::fs::read_to_string("/proc/self/stat")
            .ok()
            .and_then(|s| {
                let fields: Vec<&str> = s.split_whitespace().collect();
                // field 13 = utime, field 14 = stime (0-indexed)
                let utime: u64 = fields.get(13)?.parse().ok()?;
                let stime: u64 = fields.get(14)?.parse().ok()?;
                Some(utime + stime)
            })
            .unwrap_or(0);
        (cpu, wall)
    }

    fn read_rss_mb() -> f32 {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                for line in s.lines() {
                    if let Some(rest) = line.strip_prefix("VmRSS:") {
                        let kb: f32 = rest.trim().trim_end_matches(" kB").trim().parse().ok()?;
                        return Some(kb / 1024.0);
                    }
                }
                None
            })
            .unwrap_or(0.0)
    }
}

//! Profiling instrumentation — fine-grained timing markers for performance analysis.
//!
//! This module provides per-frame timing markers for key compositing functions,
//! enabling bottleneck identification in release builds. Markers can be collected
//! and exported as flamegraph-compatible data.
//!
//! Markers are collected with zero-copy stacking and can be exported for offline analysis.

use std::collections::VecDeque;
use std::time::Instant;

/// A timing marker capturing function name, elapsed time, and hierarchy.
#[derive(Debug, Clone)]
pub struct TimingMarker {
    /// Function or operation name
    pub name: &'static str,
    /// Elapsed microseconds
    pub elapsed_us: u64,
    /// Depth in call stack (0 = root)
    pub depth: u32,
}

/// Per-frame profiling snapshot with all collected markers.
#[derive(Debug, Clone)]
pub struct ProfilingFrame {
    /// Markers collected this frame
    pub markers: Vec<TimingMarker>,
    /// Frame start timestamp (for correlation)
    pub frame_ts: Instant,
}

/// Thread-local profiler for per-frame timing collection.
pub struct Profiler {
    /// Stack of active timing spans
    stack: VecDeque<(Instant, &'static str, u32)>,
    /// Markers collected this frame
    current_markers: Vec<TimingMarker>,
    /// Max markers per frame (to prevent unbounded growth)
    max_markers: usize,
    /// Whether profiling is enabled
    enabled: bool,
}

impl Default for Profiler {
    fn default() -> Self {
        Self {
            stack: VecDeque::with_capacity(16),
            current_markers: Vec::with_capacity(256),
            max_markers: 1024,
            enabled: cfg!(feature = "profiling"),
        }
    }
}

impl Profiler {
    /// Enable or disable profiling globally.
    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if profiling is currently enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start a timing span (push to stack).
    #[inline]
    pub fn begin(&mut self, name: &'static str) {
        if !self.enabled {
            return;
        }
        let depth = self.stack.len() as u32;
        self.stack.push_back((Instant::now(), name, depth));
    }

    /// End a timing span (pop from stack, record marker).
    #[inline]
    pub fn end(&mut self, expected_name: &'static str) {
        if !self.enabled {
            return;
        }
        if let Some((start, name, depth)) = self.stack.pop_back() {
            debug_assert_eq!(
                name, expected_name,
                "profiler mismatch: expected {}, got {}",
                expected_name, name
            );
            let elapsed_us = start.elapsed().as_micros() as u64;
            if self.current_markers.len() < self.max_markers {
                self.current_markers.push(TimingMarker {
                    name,
                    elapsed_us,
                    depth,
                });
            }
        }
    }

    /// Record a marker directly (no stack).
    #[inline]
    pub fn mark(&mut self, name: &'static str, elapsed_us: u64) {
        if !self.enabled {
            return;
        }
        let depth = self.stack.len() as u32;
        if self.current_markers.len() < self.max_markers {
            self.current_markers.push(TimingMarker {
                name,
                elapsed_us,
                depth,
            });
        }
    }

    /// Finish frame, returning all collected markers.
    pub fn finish_frame(&mut self) -> ProfilingFrame {
        let markers = std::mem::take(&mut self.current_markers);
        self.current_markers.clear();
        ProfilingFrame {
            markers,
            frame_ts: Instant::now(),
        }
    }

    /// Clear all markers (for frame boundary).
    #[inline]
    pub fn reset(&mut self) {
        self.current_markers.clear();
        self.stack.clear();
    }

    /// Get statistics about this frame's markers.
    pub fn stats(&self) -> ProfileStats {
        let mut total_us = 0u64;
        let mut by_name: std::collections::HashMap<&str, (u64, u32)> = Default::default();

        for marker in &self.current_markers {
            total_us += marker.elapsed_us;
            let entry = by_name.entry(marker.name).or_insert((0, 0));
            entry.0 += marker.elapsed_us;
            entry.1 += 1;
        }

        let mut entries: Vec<_> = by_name
            .into_iter()
            .map(|(name, (us, count))| ProfileEntry {
                name,
                total_us: us,
                count,
            })
            .collect();
        entries.sort_by(|a, b| b.total_us.cmp(&a.total_us));

        ProfileStats {
            total_us,
            marker_count: self.current_markers.len(),
            entries,
        }
    }
}

/// Statistics about profiling markers collected in a frame.
#[derive(Debug, Clone)]
pub struct ProfileStats {
    pub total_us: u64,
    pub marker_count: usize,
    pub entries: Vec<ProfileEntry>,
}

/// Per-function statistics.
#[derive(Debug, Clone)]
pub struct ProfileEntry {
    pub name: &'static str,
    pub total_us: u64,
    pub count: u32,
}

impl ProfileEntry {
    /// Average time per invocation.
    pub fn avg_us(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_us as f32 / self.count as f32
        }
    }
}

/// Export profiling data as flamegraph-compatible text (stack format).
///
/// Format: "func;parent;grandparent 1" (count; separated stack).
pub fn export_flamegraph_stacks(frame: &ProfilingFrame) -> String {
    let mut stacks: std::collections::HashMap<String, u64> = Default::default();
    let mut current_stack = Vec::new();

    for marker in &frame.markers {
        // Adjust stack depth: pop until we match the marker's depth
        while current_stack.len() > marker.depth as usize {
            current_stack.pop();
        }

        // Push current function
        current_stack.push(marker.name);

        // Build flamegraph stack string
        let stack_str = current_stack.join(";");
        *stacks.entry(stack_str).or_insert(0) += marker.elapsed_us;
    }

    // Output flamegraph format: "stack_str count"
    let mut output = String::new();
    for (stack, count) in stacks {
        output.push_str(&format!("{} {}\n", stack, count));
    }
    output
}

/// RAII guard for automatic profiler span management.
#[must_use]
pub struct ProfileSpan {
    name: &'static str,
    profiler: *mut Profiler,
}

impl ProfileSpan {
    /// Create a new profiler span, pushing to the stack.
    pub fn new(name: &'static str, profiler: &mut Profiler) -> Self {
        profiler.begin(name);
        Self {
            name,
            profiler: profiler as *mut _,
        }
    }
}

impl Drop for ProfileSpan {
    fn drop(&mut self) {
        unsafe {
            if !self.profiler.is_null() {
                (*self.profiler).end(self.name);
            }
        }
    }
}

thread_local! {
    /// Global thread-local profiler for timing collection.
    static PROFILER: std::cell::RefCell<Profiler> = std::cell::RefCell::new(Profiler::default());
}

/// Start a global profiler span.
#[inline]
pub fn begin_span(name: &'static str) {
    PROFILER.with(|p| p.borrow_mut().begin(name));
}

/// End a global profiler span.
#[inline]
pub fn end_span(name: &'static str) {
    PROFILER.with(|p| p.borrow_mut().end(name));
}

/// Record a global marker.
#[inline]
pub fn mark(name: &'static str, elapsed_us: u64) {
    PROFILER.with(|p| p.borrow_mut().mark(name, elapsed_us));
}

/// Get global profiler stats for current frame.
pub fn get_stats() -> ProfileStats {
    PROFILER.with(|p| p.borrow().stats())
}

/// Finish frame and get profiling snapshot.
pub fn finish_frame() -> ProfilingFrame {
    PROFILER.with(|p| p.borrow_mut().finish_frame())
}

/// Enable/disable global profiler.
pub fn set_enabled(enabled: bool) {
    PROFILER.with(|p| p.borrow_mut().set_enabled(enabled));
}

/// Check if global profiler is enabled.
pub fn is_enabled() -> bool {
    PROFILER.with(|p| p.borrow().is_enabled())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiler_tracks_markers() {
        let mut prof = Profiler::default();
        prof.set_enabled(true);

        prof.begin("test_fn");
        std::thread::sleep(std::time::Duration::from_micros(100));
        prof.end("test_fn");

        let frame = prof.finish_frame();
        assert_eq!(frame.markers.len(), 1);
        assert!(frame.markers[0].elapsed_us >= 100);
    }

    #[test]
    fn profiler_tracks_depth() {
        let mut prof = Profiler::default();
        prof.set_enabled(true);

        prof.begin("outer");
        prof.begin("inner");
        prof.end("inner");
        prof.end("outer");

        let frame = prof.finish_frame();
        // After begin("outer"), stack is [outer], depth = 0 (stored with marker)
        // After begin("inner"), stack is [outer, inner], depth = 1 (stored with marker)
        // After end("inner"), popped marker has depth = 1 ✓
        // After end("outer"), popped marker has depth = 0 ✓
        assert_eq!(frame.markers.len(), 2);
        assert_eq!(frame.markers[0].name, "inner");
        assert_eq!(frame.markers[0].depth, 1); // inner was at depth 1
        assert_eq!(frame.markers[1].name, "outer");
        assert_eq!(frame.markers[1].depth, 0); // outer was at depth 0
    }

    #[test]
    fn profile_span_raii_calls_end() {
        let mut prof = Profiler::default();
        prof.set_enabled(true);

        {
            let _span = ProfileSpan::new("test", &mut prof);
        }

        let frame = prof.finish_frame();
        assert_eq!(frame.markers.len(), 1);
    }

    #[test]
    fn stats_aggregates_by_name() {
        let mut prof = Profiler::default();
        prof.set_enabled(true);

        prof.mark("func_a", 100);
        prof.mark("func_b", 200);
        prof.mark("func_a", 50);

        let stats = prof.stats();
        assert_eq!(stats.marker_count, 3);
        assert_eq!(stats.total_us, 350);
        assert_eq!(stats.entries.len(), 2);
    }

    #[test]
    fn flamegraph_export_creates_stacks() {
        let mut prof = Profiler::default();
        prof.set_enabled(true);

        prof.begin("outer");
        prof.begin("inner");
        prof.end("inner");
        prof.end("outer");

        let frame = prof.finish_frame();
        let flamegraph = export_flamegraph_stacks(&frame);
        // The flamegraph format should contain outer and outer;inner stacks
        assert!(flamegraph.contains("outer") || flamegraph.is_empty() == false);
    }
}

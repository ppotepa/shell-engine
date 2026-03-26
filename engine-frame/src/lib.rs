//! `FrameTicket` — the single source of truth for frame identity across all thread boundaries.
//!
//! Every submission to the render thread, every response, and every present decision
//! carries the same `FrameTicket`. Freshness checks compare tickets rather than individual
//! counters scattered across callers.
//!
//! # Fields
//! - `sim_frame_id`: monotonically increasing counter, incremented once per game-loop iteration.
//! - `scene_generation`: bumped on every scene transition or terminal resize. Any render result
//!   whose generation does not match the current one is cross-scene stale and must be discarded.

/// Identity token for a single simulation frame.
///
/// Passed into `RenderFrameRequest`, echoed back in `RenderFrameResponse`/`RenderResult`,
/// and compared in the presenter to reject stale results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameTicket {
    /// Monotonically increasing per-frame counter (1-based; 0 = "no frame").
    pub sim_frame_id: u64,
    /// Incremented on scene transition or terminal resize.
    pub scene_generation: u64,
}

impl FrameTicket {
    /// Returns `true` if this ticket belongs to the same scene generation as `current`.
    #[inline]
    pub fn matches_generation(&self, current: &FrameTicket) -> bool {
        self.scene_generation == current.scene_generation
    }

    /// Returns `true` if this ticket is strictly newer than `other` within the same generation.
    #[inline]
    pub fn is_newer_than(&self, other: &FrameTicket) -> bool {
        self.scene_generation == other.scene_generation && self.sim_frame_id > other.sim_frame_id
    }

    /// Returns `true` if this ticket should be accepted as the next presented frame.
    ///
    /// Accepts when the frame ID is strictly newer than the last accepted ticket.
    /// Generation check is NOT performed here — the caller (`apply_render_thread_result`)
    /// already rejects cross-generation frames via `matches_generation` before calling this.
    #[inline]
    pub fn is_acceptable(&self, last_accepted: &FrameTicket) -> bool {
        self.sim_frame_id > last_accepted.sim_frame_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_frame_accepted() {
        let last = FrameTicket {
            sim_frame_id: 5,
            scene_generation: 1,
        };
        let next = FrameTicket {
            sim_frame_id: 6,
            scene_generation: 1,
        };
        assert!(next.is_acceptable(&last));
    }

    #[test]
    fn same_frame_not_accepted() {
        let t = FrameTicket {
            sim_frame_id: 5,
            scene_generation: 1,
        };
        assert!(!t.is_acceptable(&t));
    }

    #[test]
    fn cross_generation_accepted_if_newer() {
        // Generation filtering is the caller's responsibility via matches_generation().
        // is_acceptable only checks sim_frame_id ordering.
        let last = FrameTicket {
            sim_frame_id: 100,
            scene_generation: 1,
        };
        let newer = FrameTicket {
            sim_frame_id: 101,
            scene_generation: 2,
        };
        assert!(newer.is_acceptable(&last));
    }
}

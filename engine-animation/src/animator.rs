//! Animation state tracking.

/// Which lifecycle stage the scene is currently in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SceneStage {
    #[default]
    OnEnter,
    OnIdle,
    OnLeave,
    Done,
}

/// Per-scene animation state. Scoped — reset on each scene transition.
#[derive(Debug, Default, Clone)]
pub struct Animator {
    pub stage: SceneStage,
    pub step_idx: usize,
    pub elapsed_ms: u64,
    pub stage_elapsed_ms: u64,
    pub scene_elapsed_ms: u64,
    pub next_scene_override: Option<String>,
    pub menu_selected_index: usize,
}

impl Animator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Progress of the current step as 0.0..=1.0.
    pub fn step_progress(&self, step_duration_ms: u64) -> f32 {
        if step_duration_ms == 0 {
            return 0.0;
        }
        (self.elapsed_ms as f32 / step_duration_ms as f32).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animator_default() {
        let animator = Animator::new();
        assert_eq!(animator.stage, SceneStage::OnEnter);
        assert_eq!(animator.step_idx, 0);
        assert_eq!(animator.elapsed_ms, 0);
    }

    #[test]
    fn test_step_progress_zero_duration() {
        let animator = Animator::new();
        assert_eq!(animator.step_progress(0), 0.0);
    }

    #[test]
    fn test_step_progress_clamped() {
        let mut animator = Animator::new();
        animator.elapsed_ms = 5000;
        let progress = animator.step_progress(1000);
        assert_eq!(progress, 1.0); // Clamped to max
    }
}

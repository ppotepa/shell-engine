//! Animation system — processes animator tick and returns scene transitions.
//!
//! Integration: engine-animation is decoupled from World and engine-core.
//! The engine calls `animator_system()` once per frame with a provider that implements AnimatorProvider.

use engine_core::scene::Scene;

use crate::animator::Animator;

/// Trait providing access to animator state and scene runtime.
pub trait AnimatorProvider {
    fn scene(&self) -> Option<Scene>;
    fn animator(&self) -> Option<&Animator>;
    fn animator_mut(&mut self) -> Option<&mut Animator>;
}

pub fn animator_system<T: AnimatorProvider>(provider: &mut T, tick_ms: u64) -> Option<String> {
    // Extract only the primitives tick_animator needs — avoids cloning the entire Scene.
    let tick_data = {
        let Some(scene) = provider.scene() else {
            return None;
        };
        let Some(animator) = provider.animator() else {
            return None;
        };
        if animator.stage == crate::SceneStage::Done {
            return None;
        }
        let stage_def = match &animator.stage {
            crate::SceneStage::OnEnter => &scene.stages.on_enter,
            crate::SceneStage::OnIdle => &scene.stages.on_idle,
            crate::SceneStage::OnLeave => &scene.stages.on_leave,
            crate::SceneStage::Done => return None,
        };
        // Pass all step durations so tick_animator_primitives can skip consecutive
        // zero-duration steps in a single tick.
        let step_durs: Vec<u64> = stage_def.steps.iter().map(|s| s.duration_ms()).collect();
        let step_dur = step_durs.get(animator.step_idx).copied().unwrap_or(0);
        (
            stage_def.steps.len(),
            step_dur,
            step_durs,
            stage_def.looping,
            scene.next.clone(),
        )
    };
    let (step_count, step_dur, step_durs, stage_looping, next_scene) = tick_data;

    let transition = {
        let Some(animator) = provider.animator_mut() else {
            return None;
        };
        tick_animator_primitives(
            animator,
            step_count,
            step_dur,
            &step_durs,
            stage_looping,
            next_scene,
            tick_ms,
        )
    };

    transition
}

fn tick_animator_primitives(
    animator: &mut Animator,
    step_count: usize,
    step_dur: u64,
    step_durs: &[u64],
    stage_looping: bool,
    next_scene: Option<String>,
    tick_ms: u64,
) -> Option<String> {
    animator.elapsed_ms += tick_ms;
    animator.stage_elapsed_ms += tick_ms;
    animator.scene_elapsed_ms += tick_ms;

    // Empty stages (step_count == 0) or instant steps (step_dur == 0) should advance immediately.
    let step_done = (step_count == 0 || step_dur == 0) || animator.elapsed_ms >= step_dur;
    if !step_done {
        return None;
    }

    // Advance through consecutive zero-duration steps in one tick.
    let mut new_idx = animator.step_idx;
    while new_idx < step_durs.len() && step_durs[new_idx] == 0 {
        new_idx += 1;
    }
    animator.step_idx = new_idx;
    animator.elapsed_ms = 0;

    // If stage done, advance to next stage.
    if animator.step_idx >= step_count {
        animator.stage = match animator.stage {
            crate::SceneStage::OnEnter => crate::SceneStage::OnIdle,
            crate::SceneStage::OnIdle => {
                if stage_looping {
                    animator.step_idx = 0;
                    animator.elapsed_ms = 0;
                    return None;
                }
                crate::SceneStage::OnLeave
            }
            crate::SceneStage::OnLeave => crate::SceneStage::Done,
            crate::SceneStage::Done => crate::SceneStage::Done,
        };
        animator.step_idx = 0;
        animator.elapsed_ms = 0;
    }

    if animator.stage == crate::SceneStage::Done {
        if let Some(scene_id) = next_scene {
            return Some(scene_id);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::scene::{
        Scene, Stage, Step, StageTrigger, SceneStages, SceneAudio, SceneUi, SceneInput,
        SceneRenderedMode,
    };

    struct TestAnimatorProvider {
        animator: Option<Animator>,
        scene: Option<Scene>,
    }

    impl AnimatorProvider for TestAnimatorProvider {
        fn scene(&self) -> Option<Scene> {
            self.scene.clone()
        }
        fn animator(&self) -> Option<&Animator> {
            self.animator.as_ref()
        }
        fn animator_mut(&mut self) -> Option<&mut Animator> {
            self.animator.as_mut()
        }
    }

    fn minimal_scene() -> Scene {
        Scene {
            id: "test".to_string(),
            title: "Test".to_string(),
            cutscene: false,
            target_fps: None,
            rendered_mode: SceneRenderedMode::default(),
            virtual_size_override: None,
            bg_colour: None,
            stages: SceneStages {
                on_enter: Stage::default(),
                on_idle: Stage::default(),
                on_leave: Stage::default(),
            },
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            ui: SceneUi::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            input: SceneInput::default(),
            postfx: Vec::new(),
            next: None,
            prerender: false,
        }
    }

    #[test]
    fn animator_system_processes_without_error() {
        let mut scene = minimal_scene();
        scene.stages.on_enter = Stage {
            trigger: StageTrigger::None,
            steps: vec![Step {
                effects: Vec::new(),
                duration: Some(100),
            }],
            looping: false,
        };

        let mut provider = TestAnimatorProvider {
            animator: Some(Animator::new()),
            scene: Some(scene),
        };

        let result = animator_system(&mut provider, 50);
        assert_eq!(result, None);
    }

    #[test]
    fn zero_duration_steps_advance_immediately() {
        let mut scene = minimal_scene();
        scene.stages.on_enter = Stage {
            trigger: StageTrigger::None,
            steps: vec![
                Step {
                    effects: Vec::new(),
                    duration: Some(0),
                },
                Step {
                    effects: Vec::new(),
                    duration: Some(0),
                },
            ],
            looping: false,
        };
        let mut provider = TestAnimatorProvider {
            animator: Some(Animator::new()),
            scene: Some(scene),
        };

        animator_system(&mut provider, 1);

        assert_eq!(
            provider.animator.as_ref().unwrap().stage,
            crate::SceneStage::OnIdle
        );
    }
}

//! Animation system — processes animator tick and returns scene transitions.
//!
//! Integration: engine-animation is decoupled from World and engine-core.
//! The engine calls `animator_system()` once per frame with a provider that implements AnimatorProvider.

use engine_core::scene::{Scene, StageTrigger};

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
        let scene = provider.scene()?;
        let animator = provider.animator()?;
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
            scene.stages.on_idle.trigger.clone(),
            animator
                .next_scene_override
                .clone()
                .or_else(|| scene.next.clone()),
        )
    };
    let (step_count, step_dur, step_durs, stage_looping, idle_trigger, next_scene) = tick_data;

    let transition = {
        let animator = provider.animator_mut()?;
        tick_animator_primitives(
            animator,
            step_count,
            step_dur,
            &step_durs,
            stage_looping,
            &idle_trigger,
            next_scene,
            tick_ms,
        )
    };

    transition
}

fn next_stage(stage: &crate::SceneStage) -> crate::SceneStage {
    match stage {
        crate::SceneStage::OnEnter => crate::SceneStage::OnIdle,
        crate::SceneStage::OnIdle => crate::SceneStage::OnLeave,
        crate::SceneStage::OnLeave | crate::SceneStage::Done => crate::SceneStage::Done,
    }
}

#[allow(clippy::too_many_arguments)]
fn tick_animator_primitives(
    animator: &mut Animator,
    step_count: usize,
    step_dur: u64,
    step_durs: &[u64],
    stage_looping: bool,
    idle_trigger: &StageTrigger,
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

    // Advance to next step, then skip any consecutive zero-duration steps in one tick.
    let mut step_idx = animator.step_idx;
    loop {
        let next_step = step_idx + 1;
        if next_step < step_count {
            step_idx = next_step;
            // If next step also has zero duration, keep going in same tick.
            if step_durs.get(step_idx).copied().unwrap_or(1) == 0 {
                continue;
            }
        }
        break;
    }

    if step_idx > animator.step_idx {
        // Advanced one or more steps but still within the stage.
        animator.step_idx = step_idx;
        animator.elapsed_ms = 0;
        // Check if we've consumed all steps.
        if animator.step_idx + 1 < step_count
            || step_durs.get(animator.step_idx).copied().unwrap_or(1) != 0
        {
            return None;
        }
    } else if animator.step_idx + 1 < step_count {
        // Normal advance to next step.
        animator.step_idx += 1;
        animator.elapsed_ms = 0;
        return None;
    }

    // All steps done — check for loop or stage transition.
    let should_loop = stage_looping
        || matches!(
            (&animator.stage, idle_trigger),
            (crate::SceneStage::OnIdle, StageTrigger::AnyKey)
        );
    if should_loop {
        animator.step_idx = 0;
        animator.elapsed_ms = 0;
        return None;
    }

    animator.stage = next_stage(&animator.stage);
    animator.step_idx = 0;
    animator.elapsed_ms = 0;
    animator.stage_elapsed_ms = 0;

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
        Scene, SceneAudio, SceneInput, SceneStages, SceneUi, Stage, StageTrigger, Step,
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
            space: Default::default(),
            celestial: Default::default(),
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
            palette_bindings: Vec::new(),
            game_state_bindings: Vec::new(),
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
    fn nonzero_steps_advance_through_stage() {
        let mut scene = minimal_scene();
        scene.stages.on_enter = Stage {
            trigger: StageTrigger::None,
            steps: vec![
                Step {
                    effects: Vec::new(),
                    duration: Some(200),
                },
                Step {
                    effects: Vec::new(),
                    duration: Some(400),
                },
                Step {
                    effects: Vec::new(),
                    duration: Some(100),
                },
            ],
            looping: false,
        };
        let mut provider = TestAnimatorProvider {
            animator: Some(Animator::new()),
            scene: Some(scene),
        };

        // Step 0 (200ms): tick 150ms — still in step 0
        animator_system(&mut provider, 150);
        assert_eq!(provider.animator.as_ref().unwrap().step_idx, 0);
        assert_eq!(
            provider.animator.as_ref().unwrap().stage,
            crate::SceneStage::OnEnter
        );

        // Tick 60ms more (total 210ms) — should advance to step 1
        animator_system(&mut provider, 60);
        assert_eq!(provider.animator.as_ref().unwrap().step_idx, 1);

        // Step 1 (400ms): tick 410ms — should advance to step 2
        animator_system(&mut provider, 410);
        assert_eq!(provider.animator.as_ref().unwrap().step_idx, 2);

        // Step 2 (100ms): tick 110ms — should transition to OnIdle
        animator_system(&mut provider, 110);
        assert_eq!(
            provider.animator.as_ref().unwrap().stage,
            crate::SceneStage::OnIdle
        );
    }

    #[test]
    fn any_key_trigger_forces_idle_looping() {
        let mut scene = minimal_scene();
        scene.stages.on_idle = Stage {
            trigger: StageTrigger::AnyKey,
            steps: vec![Step {
                effects: Vec::new(),
                duration: Some(100),
            }],
            looping: false, // looping is false, but any-key trigger should force looping
        };
        let mut provider = TestAnimatorProvider {
            animator: Some(Animator {
                stage: crate::SceneStage::OnIdle,
                ..Animator::new()
            }),
            scene: Some(scene),
        };

        // Complete the on_idle step — should loop back, not transition to OnLeave
        animator_system(&mut provider, 110);
        assert_eq!(
            provider.animator.as_ref().unwrap().stage,
            crate::SceneStage::OnIdle
        );
        assert_eq!(provider.animator.as_ref().unwrap().step_idx, 0);
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

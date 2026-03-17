use crate::events::EngineEvent;
#[cfg(test)]
use crate::scene::Scene;
use crate::scene::StageTrigger;
use crate::services::EngineWorldAccess;
use crate::world::World;

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
#[derive(Debug, Default)]
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

pub fn animator_system(world: &mut World, tick_ms: u64) {
    // Extract only the primitives tick_animator needs — avoids cloning the entire Scene.
    let tick_data = {
        let Some(runtime) = world.scene_runtime() else {
            return;
        };
        let Some(animator) = world.animator() else {
            return;
        };
        if animator.stage == SceneStage::Done {
            return;
        }
        let scene = runtime.scene();
        let stage_def = match &animator.stage {
            SceneStage::OnEnter => &scene.stages.on_enter,
            SceneStage::OnIdle => &scene.stages.on_idle,
            SceneStage::OnLeave => &scene.stages.on_leave,
            SceneStage::Done => return,
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
        let Some(animator) = world.animator_mut() else {
            return;
        };
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

    if let Some(to_scene_id) = transition {
        if let Some(queue) = world.events_mut() {
            queue.push(EngineEvent::SceneTransition { to_scene_id });
        }
    }
}

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

    // Advance through consecutive zero-duration steps in one tick.
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
            (SceneStage::OnIdle, StageTrigger::AnyKey)
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
    if animator.stage == SceneStage::Done {
        return next_scene;
    }

    None
}

#[cfg(test)]
fn tick_animator(animator: &mut Animator, scene: &Scene, tick_ms: u64) -> Option<String> {
    if animator.stage == SceneStage::Done {
        return None;
    }
    let Some((step_count, step_dur, stage_looping)) =
        stage_runtime(scene, &animator.stage, animator.step_idx)
    else {
        return None;
    };
    let stage_def = match &animator.stage {
        SceneStage::OnEnter => &scene.stages.on_enter,
        SceneStage::OnIdle => &scene.stages.on_idle,
        SceneStage::OnLeave => &scene.stages.on_leave,
        SceneStage::Done => return None,
    };
    let step_durs: Vec<u64> = stage_def.steps.iter().map(|s| s.duration_ms()).collect();
    tick_animator_primitives(
        animator,
        step_count,
        step_dur,
        &step_durs,
        stage_looping,
        &scene.stages.on_idle.trigger,
        animator
            .next_scene_override
            .clone()
            .or_else(|| scene.next.clone()),
        tick_ms,
    )
}

#[cfg(test)]
fn stage_runtime(scene: &Scene, stage: &SceneStage, step_idx: usize) -> Option<(usize, u64, bool)> {
    let stage_def = match stage {
        SceneStage::OnEnter => &scene.stages.on_enter,
        SceneStage::OnIdle => &scene.stages.on_idle,
        SceneStage::OnLeave => &scene.stages.on_leave,
        SceneStage::Done => return None,
    };
    let step_dur = stage_def
        .steps
        .get(step_idx)
        .map(|st| st.duration_ms())
        .unwrap_or(0);
    Some((stage_def.steps.len(), step_dur, stage_def.looping))
}

fn next_stage(stage: &SceneStage) -> SceneStage {
    match stage {
        SceneStage::OnEnter => SceneStage::OnIdle,
        SceneStage::OnIdle => SceneStage::OnLeave,
        SceneStage::OnLeave | SceneStage::Done => SceneStage::Done,
    }
}

#[cfg(test)]
mod tests {
    use super::{tick_animator, Animator, SceneStage};
    use crate::scene::{
        Scene, SceneAudio, SceneRenderedMode, SceneStages, Stage, Step, TermColour,
    };

    fn scene_with_stages(
        on_enter: Stage,
        on_idle: Stage,
        on_leave: Stage,
        next: Option<&str>,
    ) -> Scene {
        Scene {
            id: "s".to_string(),
            title: "Scene".to_string(),
            cutscene: true,
            target_fps: None,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages {
                on_enter,
                on_idle,
                on_leave,
            },
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            input: Default::default(),
            next: next.map(str::to_string),
        }
    }

    #[test]
    fn advances_from_enter_to_idle_after_stage_steps_finish() {
        let scene = scene_with_stages(
            Stage {
                trigger: crate::scene::StageTrigger::None,
                steps: vec![Step {
                    effects: Vec::new(),
                    duration: Some(100),
                }],
                looping: false,
            },
            Stage::default(),
            Stage::default(),
            None,
        );
        let mut animator = Animator::new();

        let transition = tick_animator(&mut animator, &scene, 100);

        assert!(transition.is_none());
        assert_eq!(animator.stage, SceneStage::OnIdle);
        assert_eq!(animator.step_idx, 0);
        assert_eq!(animator.elapsed_ms, 0);
    }

    #[test]
    fn emits_transition_when_leave_stage_finishes_and_next_exists() {
        let scene = scene_with_stages(
            Stage::default(),
            Stage::default(),
            Stage {
                trigger: crate::scene::StageTrigger::None,
                steps: vec![Step {
                    effects: Vec::new(),
                    duration: Some(50),
                }],
                looping: false,
            },
            Some("next-scene"),
        );
        let mut animator = Animator {
            stage: SceneStage::OnLeave,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        };

        let transition = tick_animator(&mut animator, &scene, 50);

        assert_eq!(transition.as_deref(), Some("next-scene"));
        assert_eq!(animator.stage, SceneStage::Done);
        assert_eq!(animator.step_idx, 0);
        assert_eq!(animator.elapsed_ms, 0);
    }

    #[test]
    fn uses_next_scene_override_when_present() {
        let scene = scene_with_stages(
            Stage::default(),
            Stage::default(),
            Stage {
                trigger: crate::scene::StageTrigger::None,
                steps: vec![Step {
                    effects: Vec::new(),
                    duration: Some(10),
                }],
                looping: false,
            },
            Some("default-next"),
        );
        let mut animator = Animator {
            stage: SceneStage::OnLeave,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: Some("override-next".to_string()),
            menu_selected_index: 0,
        };

        let transition = tick_animator(&mut animator, &scene, 10);

        assert_eq!(transition.as_deref(), Some("override-next"));
    }

    #[test]
    fn empty_enter_stage_advances_immediately() {
        let scene = scene_with_stages(
            Stage::default(), // empty stage
            Stage::default(),
            Stage::default(),
            None,
        );
        let mut animator = Animator::new();

        let transition = tick_animator(&mut animator, &scene, 1);

        assert!(transition.is_none());
        assert_eq!(animator.stage, SceneStage::OnIdle);
    }

    #[test]
    fn zero_duration_steps_advance_immediately() {
        let scene = scene_with_stages(
            Stage {
                trigger: crate::scene::StageTrigger::None,
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
            },
            Stage::default(),
            Stage::default(),
            None,
        );
        let mut animator = Animator::new();

        let transition = tick_animator(&mut animator, &scene, 1);

        assert!(transition.is_none());
        assert_eq!(animator.stage, SceneStage::OnIdle);
    }

    #[test]
    fn timeout_idle_stage_advances_to_leave_when_duration_finishes() {
        let scene = scene_with_stages(
            Stage::default(),
            Stage {
                trigger: crate::scene::StageTrigger::Timeout,
                steps: vec![Step {
                    effects: Vec::new(),
                    duration: Some(5_000),
                }],
                looping: false,
            },
            Stage::default(),
            Some("next-scene"),
        );
        let mut animator = Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        };

        let transition = tick_animator(&mut animator, &scene, 5_000);

        assert!(transition.is_none());
        assert_eq!(animator.stage, SceneStage::OnLeave);
        assert_eq!(animator.step_idx, 0);
        assert_eq!(animator.elapsed_ms, 0);
    }

    #[test]
    fn any_key_idle_stage_stays_idle_after_step_finishes() {
        let scene = scene_with_stages(
            Stage::default(),
            Stage {
                trigger: crate::scene::StageTrigger::AnyKey,
                steps: vec![Step {
                    effects: Vec::new(),
                    duration: Some(1),
                }],
                looping: false,
            },
            Stage::default(),
            Some("next-scene"),
        );
        let mut animator = Animator {
            stage: SceneStage::OnIdle,
            step_idx: 0,
            elapsed_ms: 0,
            stage_elapsed_ms: 0,
            scene_elapsed_ms: 0,
            next_scene_override: None,
            menu_selected_index: 0,
        };

        let transition = tick_animator(&mut animator, &scene, 1);

        assert!(transition.is_none());
        assert_eq!(animator.stage, SceneStage::OnIdle);
    }
}

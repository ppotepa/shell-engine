use crate::world::World;

/// Which lifecycle stage the scene is currently in.
#[derive(Debug, Clone, PartialEq, Default)]
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
    pub scene_elapsed_ms: u64,
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
    use crate::events::{EngineEvent, EventQueue};
    use crate::scene::Scene;

    // Snapshot state needed to avoid holding immutable borrows across mutable calls.
    let snapshot = {
        let a = match world.get::<Animator>() { Some(a) => a, None => return };
        let s = match world.get::<Scene>()    { Some(s) => s, None => return };

        let (step_count, step_dur, stage_looping) = match &a.stage {
            SceneStage::OnEnter => {
                let stage = &s.stages.on_enter;
                let dur = stage.steps.get(a.step_idx).map(|st| st.duration_ms()).unwrap_or(0);
                (stage.steps.len(), dur, stage.looping)
            }
            SceneStage::OnIdle => {
                let stage = &s.stages.on_idle;
                let dur = stage.steps.get(a.step_idx).map(|st| st.duration_ms()).unwrap_or(0);
                (stage.steps.len(), dur, stage.looping)
            }
            SceneStage::OnLeave => {
                let stage = &s.stages.on_leave;
                let dur = stage.steps.get(a.step_idx).map(|st| st.duration_ms()).unwrap_or(0);
                (stage.steps.len(), dur, stage.looping)
            }
            SceneStage::Done => return,
        };
        let idle_trigger = s.stages.on_idle.trigger.clone();
        let next_scene   = s.next.clone();
        (a.stage.clone(), a.step_idx, a.elapsed_ms, step_count, step_dur, stage_looping, idle_trigger, next_scene)
    };

    let (stage, step_idx, elapsed_ms, step_count, step_dur, stage_looping, idle_trigger, next_scene) = snapshot;

    if let Some(a) = world.get_mut::<Animator>() {
        a.elapsed_ms += tick_ms;
        a.scene_elapsed_ms += tick_ms;
    }

    let new_elapsed = elapsed_ms + tick_ms;
    let step_done   = step_dur > 0 && new_elapsed >= step_dur;

    if step_done {
        let next_step = step_idx + 1;
        if next_step < step_count {
            if let Some(a) = world.get_mut::<Animator>() {
                a.step_idx   = next_step;
                a.elapsed_ms = 0;
            }
        } else {
            // Stage looping: wrap back to step 0 instead of advancing stage.
            // on_idle with any-key/timeout trigger also loops — game_loop drives the transition.
            let should_loop = stage_looping || matches!(
                (&stage, &idle_trigger),
                (SceneStage::OnIdle, crate::scene::StageTrigger::AnyKey)
                | (SceneStage::OnIdle, crate::scene::StageTrigger::Timeout)
            );
            if should_loop {
                if let Some(a) = world.get_mut::<Animator>() {
                    a.step_idx   = 0;
                    a.elapsed_ms = 0;
                }
            } else {
                let next_stage = match &stage {
                    SceneStage::OnEnter => SceneStage::OnIdle,
                    SceneStage::OnIdle  => SceneStage::OnLeave,
                    SceneStage::OnLeave => SceneStage::Done,
                    SceneStage::Done    => SceneStage::Done,
                };
                if let Some(a) = world.get_mut::<Animator>() {
                    a.stage      = next_stage.clone();
                    a.step_idx   = 0;
                    a.elapsed_ms = 0;
                }
                if next_stage == SceneStage::Done {
                    if let Some(id) = next_scene {
                        if let Some(q) = world.get_mut::<EventQueue>() {
                            q.push(EngineEvent::SceneTransition { to_scene_id: id });
                        }
                    }
                }
            }
        }
    }
}

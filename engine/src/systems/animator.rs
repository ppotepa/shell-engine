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

use crate::effects::utils::math::TICK_MS;

pub fn animator_system(world: &mut World) {
    use crate::events::{EngineEvent, EventQueue};
    use crate::scene::Scene;

    // Snapshot state needed to avoid holding immutable borrows across mutable calls.
    let snapshot = {
        let a = match world.get::<Animator>() { Some(a) => a, None => return };
        let s = match world.get::<Scene>()    { Some(s) => s, None => return };

        let (step_count, step_dur, looping) = match &a.stage {
            SceneStage::OnEnter => {
                let stage = &s.stages.on_enter;
                let dur = stage.steps.get(a.step_idx).map(|st| st.duration_ms()).unwrap_or(0);
                let lp  = stage.steps.get(a.step_idx).and_then(|st| st.effects.first()).map(|e| e.looping).unwrap_or(false);
                (stage.steps.len(), dur, lp)
            }
            SceneStage::OnIdle => {
                let stage = &s.stages.on_idle;
                let dur = stage.steps.get(a.step_idx).map(|st| st.duration_ms()).unwrap_or(0);
                let lp  = stage.steps.get(a.step_idx).and_then(|st| st.effects.first()).map(|e| e.looping).unwrap_or(false);
                (stage.steps.len(), dur, lp)
            }
            SceneStage::OnLeave => {
                let stage = &s.stages.on_leave;
                let dur = stage.steps.get(a.step_idx).map(|st| st.duration_ms()).unwrap_or(0);
                let lp  = stage.steps.get(a.step_idx).and_then(|st| st.effects.first()).map(|e| e.looping).unwrap_or(false);
                (stage.steps.len(), dur, lp)
            }
            SceneStage::Done => return,
        };
        let idle_trigger = s.stages.on_idle.trigger.clone();
        let next_scene   = s.next.clone();
        (a.stage.clone(), a.step_idx, a.elapsed_ms, step_count, step_dur, looping, idle_trigger, next_scene)
    };

    let (stage, step_idx, elapsed_ms, step_count, step_dur, looping, _idle_trigger, next_scene) = snapshot;

    if let Some(a) = world.get_mut::<Animator>() {
        a.elapsed_ms += TICK_MS;
        a.scene_elapsed_ms += TICK_MS;
    }

    let new_elapsed = elapsed_ms + TICK_MS;
    let step_done   = !looping && step_dur > 0 && new_elapsed >= step_dur;

    if step_done {
        let next_step = step_idx + 1;
        if next_step < step_count {
            if let Some(a) = world.get_mut::<Animator>() {
                a.step_idx   = next_step;
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

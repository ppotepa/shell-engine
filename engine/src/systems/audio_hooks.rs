use std::collections::HashSet;

use crate::events::{EngineEvent, EventQueue};
use crate::scene::{AudioCue, Scene};
use crate::systems::animator::{Animator, SceneStage};
use crate::world::World;

/// Runtime state for one-shot audio cue emission.
#[derive(Debug, Default)]
pub struct AudioHookState {
    emitted: HashSet<String>,
}

pub fn audio_hooks_system(world: &mut World) {
    let scene = match world.get::<Scene>() {
        Some(scene) => scene.clone(),
        None => return,
    };
    let animator = match world.get::<Animator>() {
        Some(animator) => animator,
        None => return,
    };
    let stage = animator.stage.clone();
    let scene_elapsed_ms = animator.scene_elapsed_ms;

    let cues = cues_for_stage(&scene, &stage);
    if cues.is_empty() {
        return;
    }

    let Some(state) = world.get_mut::<AudioHookState>() else {
        return;
    };
    let mut newly_emitted = Vec::new();
    for cue in cues {
        if scene_elapsed_ms < cue.at_ms || cue.cue.trim().is_empty() {
            continue;
        }
        let key = cue_key(&scene.id, &stage, cue);
        if state.emitted.insert(key) {
            newly_emitted.push(EngineEvent::AudioCue {
                cue: cue.cue.clone(),
                volume: cue.volume,
            });
        }
    }

    if let Some(queue) = world.get_mut::<EventQueue>() {
        for ev in newly_emitted {
            queue.push(ev);
        }
    }
}

fn cues_for_stage<'a>(scene: &'a Scene, stage: &SceneStage) -> &'a [AudioCue] {
    match stage {
        SceneStage::OnEnter => &scene.audio.on_enter,
        SceneStage::OnIdle => &scene.audio.on_idle,
        SceneStage::OnLeave => &scene.audio.on_leave,
        SceneStage::Done => &[],
    }
}

fn cue_key(scene_id: &str, stage: &SceneStage, cue: &AudioCue) -> String {
    format!("{scene_id}:{stage:?}:{}:{}", cue.at_ms, cue.cue)
}

#[cfg(test)]
mod tests {
    use super::{audio_hooks_system, AudioHookState};
    use crate::events::{EngineEvent, EventQueue};
    use crate::scene::{
        AudioCue, Layer, LayerStages, Scene, SceneAudio, SceneRenderedMode, SceneStages, TermColour,
    };
    use crate::systems::animator::Animator;
    use crate::systems::animator::SceneStage;
    use crate::world::World;

    #[test]
    fn emits_audio_cue_once_when_scene_time_reaches_threshold() {
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            audio: SceneAudio {
                on_enter: vec![AudioCue {
                    at_ms: 100,
                    cue: "thunder_far_01".to_string(),
                    volume: Some(0.7),
                }],
                on_idle: Vec::new(),
                on_leave: Vec::new(),
            },
            layers: vec![Layer {
                name: "l".to_string(),
                z_index: 0,
                visible: true,
                stages: LayerStages::default(),
                sprites: Vec::new(),
            }],
            next: None,
        };
        let mut world = World::new();
        world.register(EventQueue::new());
        world.register_scoped(scene);
        world.register_scoped(Animator {
            stage: SceneStage::OnEnter,
            step_idx: 0,
            elapsed_ms: 0,
            scene_elapsed_ms: 100,
        });
        world.register_scoped(AudioHookState::default());

        audio_hooks_system(&mut world);
        audio_hooks_system(&mut world);

        let events = world.get_mut::<EventQueue>().expect("queue").drain();
        let audio: Vec<_> = events
            .into_iter()
            .filter_map(|e| match e {
                EngineEvent::AudioCue { cue, volume } => Some((cue, volume)),
                _ => None,
            })
            .collect();
        assert_eq!(audio.len(), 1);
        assert_eq!(audio[0].0, "thunder_far_01");
        assert_eq!(audio[0].1, Some(0.7));
    }
}

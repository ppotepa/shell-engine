//! Scene audio behavior: fires cues at scheduled timestamps.

use std::collections::HashSet;

use engine_core::logging;

use crate::{BehaviorCommand, Behavior, BehaviorContext, GameObject, Scene, emit_audio, EmittedCueKey};

/// Fires scene-level audio cues at their scheduled `at_ms` timestamps.
#[derive(Default)]
pub struct SceneAudioBehavior {
    emitted: HashSet<EmittedCueKey>,
}

impl Behavior for SceneAudioBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let cues = crate::cues_for_stage(scene, &ctx.stage);
        if !cues.is_empty() {
            logging::debug(
                "engine.audio.behavior",
                format!(
                    "scene={} stage={:?} elapsed={}ms cues={} emitted={}",
                    scene.id,
                    ctx.stage,
                    ctx.scene_elapsed_ms,
                    cues.len(),
                    self.emitted.len()
                ),
            );
        }
        for cue in cues {
            if ctx.scene_elapsed_ms < cue.at_ms || cue.cue.trim().is_empty() {
                continue;
            }
            let key = (
                scene.id.clone(),
                object.id.clone(),
                ctx.stage,
                cue.at_ms,
                cue.cue.clone(),
            );
            if self.emitted.insert(key) {
                logging::info(
                    "engine.audio.behavior",
                    format!("emitting audio cue='{}' volume={:?}", cue.cue, cue.volume),
                );
                emit_audio(commands, cue.cue.clone(), cue.volume);
            }
        }
    }
}

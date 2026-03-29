//! Song sequencer system: advances active song and queues resolved cues.

use crate::audio::AudioCommand;
use crate::audio_sequencer::AudioSequencerState;
use crate::services::EngineWorldAccess;
use crate::world::World;

/// Advances active song timeline and enqueues generated cue hits.
pub fn audio_sequencer_system(world: &mut World, dt_ms: u64) {
    let now_ms = world
        .animator()
        .map(|animator| animator.scene_elapsed_ms)
        .unwrap_or(0);

    let hits = {
        let Some(sequencer) = world.get_mut::<AudioSequencerState>() else {
            return;
        };
        sequencer.tick_song(dt_ms, now_ms)
    };
    if hits.is_empty() {
        return;
    }

    let Some(audio_runtime) = world.audio_runtime_mut() else {
        return;
    };
    for (cue, gain) in hits {
        audio_runtime.queue(AudioCommand {
            cue,
            volume: Some(gain),
        });
    }
}

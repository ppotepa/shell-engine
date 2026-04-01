//! Audio cue and event API surface.

use std::sync::{Arc, Mutex};

use rhai::Engine as RhaiEngine;

use crate::BehaviorCommand;

/// Script-facing audio API for playing cues, events, and songs.
#[derive(Clone)]
pub struct ScriptAudioApi {
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptAudioApi {
    /// Create a new audio API wrapper with the given command queue.
    pub fn new(queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
        Self { queue }
    }

    /// Play an audio cue with optional volume override.
    pub fn cue(&mut self, cue: &str, volume: Option<f32>) -> bool {
        let cue = cue.trim();
        if cue.is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::PlayAudioCue {
            cue: cue.to_string(),
            volume,
        });
        true
    }

    /// Play an audio event with optional gain scale override.
    pub fn event(&mut self, event: &str, gain: Option<f32>) -> bool {
        let event = event.trim();
        if event.is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::PlayAudioEvent {
            event: event.to_string(),
            gain,
        });
        true
    }

    /// Play a music track by ID.
    pub fn play_song(&mut self, song_id: &str) -> bool {
        let song_id = song_id.trim();
        if song_id.is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::PlaySong {
            song_id: song_id.to_string(),
        });
        true
    }

    /// Stop the currently playing music track.
    pub fn stop_song(&mut self) -> bool {
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::StopSong);
        true
    }
}

/// Register audio API into the Rhai engine.
pub fn register_audio_api(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptAudioApi>("AudioApi");

    engine.register_fn("cue", |audio: &mut ScriptAudioApi, cue: &str| {
        audio.cue(cue, None)
    });
    engine.register_fn(
        "cue",
        |audio: &mut ScriptAudioApi, cue: &str, volume: rhai::FLOAT| {
            audio.cue(cue, Some(volume as f32))
        },
    );
    engine.register_fn("event", |audio: &mut ScriptAudioApi, event: &str| {
        audio.event(event, None)
    });
    engine.register_fn(
        "event",
        |audio: &mut ScriptAudioApi, event: &str, gain_scale: rhai::FLOAT| {
            audio.event(event, Some(gain_scale as f32))
        },
    );
    engine.register_fn("play_song", |audio: &mut ScriptAudioApi, song_id: &str| {
        audio.play_song(song_id)
    });
    engine.register_fn("stop_song", |audio: &mut ScriptAudioApi| audio.stop_song());
}

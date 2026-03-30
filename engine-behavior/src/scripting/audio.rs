//! Audio domain API: ScriptAudioApi for audio cues and songs, ScriptFxApi for effects.

use std::sync::{Arc, Mutex};

use rhai::{Engine as RhaiEngine, Map as RhaiMap};

use crate::{BehaviorCommand, catalog};
use engine_game::GameplayWorld;

// ── ScriptAudioApi ───────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptAudioApi {
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptAudioApi {
    pub(crate) fn new(queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
        Self { queue }
    }

    fn cue(&mut self, cue: &str, volume: Option<f32>) -> bool {
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

    fn event(&mut self, event: &str, gain: Option<f32>) -> bool {
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

    fn play_song(&mut self, song_id: &str) -> bool {
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

    fn stop_song(&mut self) -> bool {
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::StopSong);
        true
    }
}

// ── ScriptFxApi ──────────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptFxApi {
    world: Option<GameplayWorld>,
    catalogs: Arc<catalog::ModCatalogs>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptFxApi {
    pub(crate) fn new(
        world: Option<GameplayWorld>,
        catalogs: Arc<catalog::ModCatalogs>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self {
            world,
            catalogs,
            queue,
        }
    }

    fn emit(&mut self, effect_name: &str, _args: RhaiMap) -> bool {
        let effect_name = effect_name.trim();
        if effect_name.is_empty() {
            return false;
        }
        // Check if effect exists in catalog
        if !self.catalogs.effects.contains_key(effect_name) {
            return false;
        }
        // In a full implementation, we would emit the effect to the gameplay world.
        // For now, return success if the effect is registered.
        true
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptAudioApi>("AudioApi");
    engine.register_type_with_name::<ScriptFxApi>("FxApi");

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

    engine.register_fn(
        "emit",
        |fx: &mut ScriptFxApi, effect_name: &str, args: RhaiMap| fx.emit(effect_name, args),
    );
}

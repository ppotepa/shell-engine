//! Audio sequencer bridge:
//! resolves semantic audio events into concrete cue ids.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use engine_audio_sequencer::{
    load_sfx_bank, synthesize_note_sheets, validate_song_file, SfxEventRuntime, SongFile,
    SongRuntime,
};
use engine_core::logging;

/// Runtime bridge for optional `audio/sfx.yaml`.
#[derive(Debug, Default)]
pub struct AudioSequencerState {
    sfx_runtime: Option<SfxEventRuntime>,
    songs: BTreeMap<String, SongFile>,
    active_song: Option<SongRuntime>,
}

impl AudioSequencerState {
    pub fn synthesize_note_sheets_if_any(
        mod_source: &Path,
    ) -> Option<HashMap<String, (u32, Vec<i16>)>> {
        match synthesize_note_sheets(mod_source) {
            Ok(map) => {
                if !map.is_empty() {
                    logging::info(
                        "engine.audio.sequencer",
                        format!("generated {} synth cues from /audio/synth", map.len()),
                    );
                }
                Some(map)
            }
            Err(error) => {
                logging::warn(
                    "engine.audio.sequencer",
                    format!("synth note-sheet generation failed: {error}"),
                );
                None
            }
        }
    }

    /// Loads the optional SFX bank from `<mod>/audio/sfx.(yaml|yml)`.
    pub fn from_mod_source(mod_source: &Path) -> Self {
        let mut state = Self {
            sfx_runtime: None,
            songs: BTreeMap::new(),
            active_song: None,
        };
        let candidates = [
            mod_source.join("audio/sfx.yaml"),
            mod_source.join("audio/sfx.yml"),
        ];
        for path in candidates {
            if !path.is_file() {
                continue;
            }
            match load_sfx_bank(&path) {
                Ok(bank) => {
                    logging::info(
                        "engine.audio.sequencer",
                        format!("loaded sfx bank: {}", path.display()),
                    );
                    state.sfx_runtime = Some(SfxEventRuntime::new(bank));
                }
                Err(error) => logging::warn(
                    "engine.audio.sequencer",
                    format!("cannot load sfx bank '{}': {error}", path.display()),
                ),
            }
            break;
        }
        if state.sfx_runtime.is_none() {
            logging::debug("engine.audio.sequencer", "no sfx bank found under /audio");
        }
        state.songs = load_song_library(mod_source);
        state
    }

    /// Resolves a semantic event id into `(cue_stem, gain)`.
    pub fn resolve_sfx_event(
        &mut self,
        event_id: &str,
        now_ms: u64,
        gain_scale: Option<f32>,
    ) -> Option<(String, f32)> {
        let runtime = self.sfx_runtime.as_mut()?;
        let resolved = runtime.resolve_event(event_id, now_ms, gain_scale)?;
        Some((resolved.cue, resolved.gain))
    }

    /// Immediately resolves and returns a semantic event (for script-triggered SFX).
    pub fn trigger_event(
        &mut self,
        event_id: &str,
        now_ms: u64,
        gain: Option<f32>,
    ) -> Option<(String, f32)> {
        self.resolve_sfx_event(event_id, now_ms, gain)
    }

    pub fn play_song(&mut self, song_id: &str) -> bool {
        let Some(song) = self.songs.get(song_id).cloned() else {
            return false;
        };
        self.active_song = Some(SongRuntime::new(song));
        true
    }

    pub fn stop_song(&mut self) {
        self.active_song = None;
    }

    pub fn active_song_id(&self) -> Option<&str> {
        self.active_song.as_ref().map(SongRuntime::id)
    }

    /// Advances active song and returns concrete `(cue, gain)` hits for this frame.
    pub fn tick_song(&mut self, dt_ms: u64, now_ms: u64) -> Vec<(String, f32)> {
        let mut resolved = Vec::new();
        let Some(active) = self.active_song.as_mut() else {
            return resolved;
        };
        let events = active.tick(dt_ms);
        for event in events {
            if let Some(hit) = self.resolve_sfx_event(&event.event, now_ms, Some(event.gain)) {
                resolved.push(hit);
            }
        }
        resolved
    }
}

fn load_song_library(mod_source: &Path) -> BTreeMap<String, SongFile> {
    let mut songs = BTreeMap::new();
    let songs_root = mod_source.join("audio").join("songs");
    if !songs_root.is_dir() {
        return songs;
    }
    let mut stack = vec![songs_root];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) => {
                logging::warn(
                    "engine.audio.sequencer",
                    format!("cannot read songs directory '{}': {error}", dir.display()),
                );
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !is_yaml_file(&path) {
                continue;
            }
            let raw = match fs::read_to_string(&path) {
                Ok(raw) => raw,
                Err(error) => {
                    logging::warn(
                        "engine.audio.sequencer",
                        format!("cannot read song file '{}': {error}", path.display()),
                    );
                    continue;
                }
            };
            let song: SongFile = match serde_yaml::from_str(&raw) {
                Ok(song) => song,
                Err(error) => {
                    logging::warn(
                        "engine.audio.sequencer",
                        format!("cannot parse song '{}': {error}", path.display()),
                    );
                    continue;
                }
            };
            if let Err(error) = validate_song_file(&song) {
                logging::warn(
                    "engine.audio.sequencer",
                    format!("invalid song '{}': {error}", path.display()),
                );
                continue;
            }
            logging::info(
                "engine.audio.sequencer",
                format!("loaded song '{}' from {}", song.id, path.display()),
            );
            songs.insert(song.id.clone(), song);
        }
    }
    songs
}

fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            let ext = ext.to_ascii_lowercase();
            ext == "yml" || ext == "yaml"
        })
}

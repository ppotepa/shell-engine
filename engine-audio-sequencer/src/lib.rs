use std::collections::{BTreeMap, HashMap};
use std::f32::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SequencerError {
    #[error("io error while reading '{path}': {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("yaml parse error in '{path}': {source}")]
    Yaml {
        path: String,
        source: serde_yaml::Error,
    },
    #[error("invalid audio sequencer data: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SfxBank {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub events: BTreeMap<String, SfxEvent>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SfxEvent {
    #[serde(default = "default_gain")]
    pub gain: f32,
    #[serde(default = "default_polyphony")]
    pub max_polyphony: u32,
    #[serde(default)]
    pub cooldown_ms: u64,
    #[serde(default)]
    pub variants: Vec<SfxVariant>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SfxVariant {
    pub asset: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub gain: Option<f32>,
    #[serde(default)]
    pub pitch_semitones: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SongFile {
    pub id: String,
    #[serde(default = "default_tempo")]
    pub tempo_bpm: f32,
    #[serde(default = "default_signature")]
    pub time_signature: [u8; 2],
    #[serde(default)]
    pub loop_region: Option<LoopRegion>,
    #[serde(default)]
    pub tracks: Vec<SongTrack>,
    #[serde(default)]
    pub patterns: BTreeMap<String, Pattern>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LoopRegion {
    pub start_beat: f32,
    pub end_beat: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SongTrack {
    pub id: String,
    #[serde(default = "default_gain")]
    pub gain: f32,
    #[serde(default)]
    pub pan: f32,
    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub clips: Vec<TrackClip>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TrackClip {
    pub at_beat: f32,
    pub pattern: String,
    #[serde(default = "default_gain")]
    pub gain: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Pattern {
    #[serde(default)]
    pub steps: Vec<PatternStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PatternStep {
    pub at_beat: f32,
    pub event: String,
    #[serde(default)]
    pub gain: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NoteSheetFile {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub sounds: BTreeMap<String, SynthSound>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SynthSound {
    #[serde(default = "default_waveform")]
    pub waveform: String,
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    #[serde(default = "default_gain")]
    pub gain: f32,
    #[serde(default = "default_attack_ms")]
    pub attack_ms: u32,
    #[serde(default = "default_release_ms")]
    pub release_ms: u32,
    #[serde(default)]
    pub notes: Vec<SynthNote>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SynthNote {
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub freq_hz: Option<f32>,
    #[serde(default = "default_note_len_ms")]
    pub len_ms: u32,
    #[serde(default)]
    pub gain: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSfxEvent {
    pub cue: String,
    pub gain: f32,
}

/// Runtime helper that resolves semantic SFX events into concrete cue ids.
///
/// Cue ids are derived from variant asset file stems, which makes this runtime
/// compatible with the existing engine audio backend that indexes cues by stem.
#[derive(Debug, Clone)]
pub struct SfxEventRuntime {
    bank: SfxBank,
    sequence: u64,
    last_play_ms: HashMap<String, u64>,
}

impl SfxEventRuntime {
    pub fn new(bank: SfxBank) -> Self {
        Self {
            bank,
            sequence: 0,
            last_play_ms: HashMap::new(),
        }
    }

    /// Resolves an event id into a concrete cue + gain using deterministic
    /// weighted selection. Returns `None` when the event is unknown, blocked by
    /// cooldown, or cannot be mapped to a cue id.
    pub fn resolve_event(
        &mut self,
        event_id: &str,
        now_ms: u64,
        gain_scale: Option<f32>,
    ) -> Option<ResolvedSfxEvent> {
        let event_id = event_id.trim();
        if event_id.is_empty() {
            return None;
        }
        let event = self.bank.events.get(event_id)?;
        if event.variants.is_empty() {
            return None;
        }

        if event.cooldown_ms > 0
            && self
                .last_play_ms
                .get(event_id)
                .is_some_and(|last| now_ms.saturating_sub(*last) < event.cooldown_ms)
        {
            return None;
        }

        let variant = pick_weighted_variant(&event.variants, self.sequence)?;
        self.sequence = self.sequence.wrapping_add(1);
        self.last_play_ms.insert(event_id.to_string(), now_ms);

        let cue = cue_from_asset_path(&variant.asset)?;
        let gain = event.gain * variant.gain.unwrap_or(1.0) * gain_scale.unwrap_or(1.0).max(0.0);
        Some(ResolvedSfxEvent { cue, gain })
    }

    pub fn has_event(&self, event_id: &str) -> bool {
        self.bank.events.contains_key(event_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SequencedEvent {
    pub event: String,
    pub gain: f32,
}

#[derive(Debug, Clone)]
pub struct SongRuntime {
    song: SongFile,
    beat: f32,
    just_started: bool,
}

impl SongRuntime {
    pub fn new(song: SongFile) -> Self {
        Self {
            song,
            beat: 0.0,
            just_started: true,
        }
    }

    pub fn id(&self) -> &str {
        &self.song.id
    }

    pub fn beat(&self) -> f32 {
        self.beat
    }

    /// Advances the song by `dt_ms` and returns event hits in this frame.
    pub fn tick(&mut self, dt_ms: u64) -> Vec<SequencedEvent> {
        if self.song.tempo_bpm <= 0.0 {
            return Vec::new();
        }
        let seconds = dt_ms as f32 / 1000.0;
        let beats_per_second = self.song.tempo_bpm / 60.0;
        if beats_per_second <= 0.0 {
            return Vec::new();
        }
        let beat_delta = seconds * beats_per_second;
        if beat_delta <= 0.0 {
            return Vec::new();
        }

        let start = self.beat;
        let mut end = start + beat_delta;
        let mut hits = Vec::new();

        if let Some(region) = &self.song.loop_region {
            let loop_start = region.start_beat;
            let loop_end = region.end_beat;
            if loop_end > loop_start {
                let mut cursor = start;
                let mut include_start = self.just_started;
                while end > loop_end {
                    hits.extend(self.collect_hits(cursor, loop_end, include_start));
                    end = loop_start + (end - loop_end);
                    cursor = loop_start;
                    include_start = true;
                }
                hits.extend(self.collect_hits(cursor, end, include_start));
                self.beat = end;
            } else {
                hits.extend(self.collect_hits(start, end, self.just_started));
                self.beat = end;
            }
        } else {
            hits.extend(self.collect_hits(start, end, self.just_started));
            self.beat = end;
        }

        self.just_started = false;
        hits
    }

    fn collect_hits(&self, start: f32, end: f32, include_start: bool) -> Vec<SequencedEvent> {
        let mut out = Vec::new();
        for track in &self.song.tracks {
            if track.mute {
                continue;
            }
            for clip in &track.clips {
                let Some(pattern) = self.song.patterns.get(&clip.pattern) else {
                    continue;
                };
                for step in &pattern.steps {
                    let beat = clip.at_beat + step.at_beat;
                    let in_window = if include_start {
                        beat >= start && beat <= end
                    } else {
                        beat > start && beat <= end
                    };
                    if !in_window {
                        continue;
                    }
                    out.push(SequencedEvent {
                        event: step.event.clone(),
                        gain: track.gain * clip.gain * step.gain.unwrap_or(1.0),
                    });
                }
            }
        }
        out
    }
}

fn pick_weighted_variant(variants: &[SfxVariant], sequence: u64) -> Option<&SfxVariant> {
    let total_weight: u64 = variants
        .iter()
        .map(|variant| u64::from(variant.weight))
        .sum();
    if total_weight == 0 {
        return None;
    }
    let mut cursor = sequence % total_weight;
    for variant in variants {
        let weight = u64::from(variant.weight);
        if cursor < weight {
            return Some(variant);
        }
        cursor -= weight;
    }
    variants.first()
}

fn cue_from_asset_path(asset: &str) -> Option<String> {
    if let Some(rest) = asset.trim().strip_prefix("synth:") {
        let stem = rest.trim();
        return if stem.is_empty() {
            None
        } else {
            Some(stem.to_string())
        };
    }
    let name = asset.trim().replace('\\', "/");
    if name.is_empty() {
        return None;
    }
    let stem = name.rsplit('/').next()?;
    let stem = stem
        .rsplit_once('.')
        .map(|(prefix, _)| prefix)
        .unwrap_or(stem);
    let stem = stem.trim();
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_string())
    }
}

pub fn load_sfx_bank(path: &Path) -> Result<SfxBank, SequencerError> {
    let raw = fs::read_to_string(path).map_err(|source| SequencerError::Io {
        path: path.display().to_string(),
        source,
    })?;
    let bank = serde_yaml::from_str::<SfxBank>(&raw).map_err(|source| SequencerError::Yaml {
        path: path.display().to_string(),
        source,
    })?;
    validate_sfx_bank(&bank)?;
    Ok(bank)
}

pub fn load_song_file(path: &Path) -> Result<SongFile, SequencerError> {
    let raw = fs::read_to_string(path).map_err(|source| SequencerError::Io {
        path: path.display().to_string(),
        source,
    })?;
    let song = serde_yaml::from_str::<SongFile>(&raw).map_err(|source| SequencerError::Yaml {
        path: path.display().to_string(),
        source,
    })?;
    validate_song_file(&song)?;
    Ok(song)
}

/// Loads all `/audio/synth/*.yml|yaml` sheets and renders cues into memory.
///
/// Returns map cue_id → (sample_rate, samples_i16).
pub fn synthesize_note_sheets(
    mod_source: &Path,
) -> Result<HashMap<String, (u32, Vec<i16>)>, SequencerError> {
    let synth_root = mod_source.join("audio").join("synth");
    if !synth_root.is_dir() {
        return Ok(HashMap::new());
    }
    let mut paths = Vec::<PathBuf>::new();
    collect_yaml_files(&synth_root, &mut paths);
    paths.sort();
    if paths.is_empty() {
        return Ok(HashMap::new());
    }

    let mut generated = HashMap::new();
    for path in paths {
        let raw = fs::read_to_string(&path).map_err(|source| SequencerError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let sheet =
            serde_yaml::from_str::<NoteSheetFile>(&raw).map_err(|source| SequencerError::Yaml {
                path: path.display().to_string(),
                source,
            })?;
        for (cue, sound) in sheet.sounds {
            let wav = render_sound_to_pcm16(&sound)?;
            generated.insert(cue, (sound.sample_rate, wav));
        }
    }
    Ok(generated)
}

pub fn validate_sfx_bank(bank: &SfxBank) -> Result<(), SequencerError> {
    for (event_id, event) in &bank.events {
        if event.variants.is_empty() {
            return Err(SequencerError::Invalid(format!(
                "event '{event_id}' has no variants"
            )));
        }
        for (idx, variant) in event.variants.iter().enumerate() {
            if variant.asset.trim().is_empty() {
                return Err(SequencerError::Invalid(format!(
                    "event '{event_id}' variant #{idx} has empty asset"
                )));
            }
            if variant.weight == 0 {
                return Err(SequencerError::Invalid(format!(
                    "event '{event_id}' variant #{idx} has zero weight"
                )));
            }
        }
    }
    Ok(())
}

pub fn validate_song_file(song: &SongFile) -> Result<(), SequencerError> {
    if song.id.trim().is_empty() {
        return Err(SequencerError::Invalid(
            "song id cannot be blank".to_string(),
        ));
    }
    if song.tempo_bpm <= 0.0 {
        return Err(SequencerError::Invalid(format!(
            "song '{}' tempo must be > 0",
            song.id
        )));
    }
    if song.time_signature[0] == 0 || song.time_signature[1] == 0 {
        return Err(SequencerError::Invalid(format!(
            "song '{}' has invalid time signature",
            song.id
        )));
    }
    if let Some(region) = &song.loop_region {
        if !(region.end_beat > region.start_beat) {
            return Err(SequencerError::Invalid(format!(
                "song '{}' loop region end must be > start",
                song.id
            )));
        }
    }
    for track in &song.tracks {
        if track.id.trim().is_empty() {
            return Err(SequencerError::Invalid(format!(
                "song '{}' has track with empty id",
                song.id
            )));
        }
        for clip in &track.clips {
            if !song.patterns.contains_key(&clip.pattern) {
                return Err(SequencerError::Invalid(format!(
                    "song '{}' track '{}' references missing pattern '{}'",
                    song.id, track.id, clip.pattern
                )));
            }
        }
    }
    Ok(())
}

const fn default_tempo() -> f32 {
    120.0
}

const fn default_signature() -> [u8; 2] {
    [4, 4]
}

const fn default_gain() -> f32 {
    1.0
}

const fn default_weight() -> u32 {
    1
}

const fn default_polyphony() -> u32 {
    4
}

const fn default_sample_rate() -> u32 {
    44_100
}

const fn default_attack_ms() -> u32 {
    4
}

const fn default_release_ms() -> u32 {
    8
}

const fn default_note_len_ms() -> u32 {
    80
}

fn default_waveform() -> String {
    "square".to_string()
}

fn collect_yaml_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_yaml_files(&path, out);
            continue;
        }
        let is_yaml = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| {
                let ext = ext.to_ascii_lowercase();
                ext == "yml" || ext == "yaml"
            });
        if is_yaml {
            out.push(path);
        }
    }
}

fn render_sound_to_pcm16(sound: &SynthSound) -> Result<Vec<i16>, SequencerError> {
    if sound.sample_rate < 8_000 || sound.sample_rate > 96_000 {
        return Err(SequencerError::Invalid(format!(
            "invalid sample_rate {} (expected 8000..96000)",
            sound.sample_rate
        )));
    }
    if sound.notes.is_empty() {
        return Err(SequencerError::Invalid(
            "synth sound has no notes".to_string(),
        ));
    }
    let waveform = sound.waveform.trim().to_ascii_lowercase();
    if !matches!(
        waveform.as_str(),
        "sine" | "square" | "triangle" | "saw" | "noise"
    ) {
        return Err(SequencerError::Invalid(format!(
            "unsupported waveform '{}'",
            sound.waveform
        )));
    }

    let mut out = Vec::<i16>::new();
    let mut noise_state = 0x9E37_79B9u32;
    let sr = sound.sample_rate as f32;
    let gain = sound.gain.clamp(0.0, 1.2);

    for note in &sound.notes {
        let freq = if waveform == "noise" {
            note.freq_hz.unwrap_or(0.0).max(0.0)
        } else {
            resolve_note_frequency(note)?
        };
        let len_ms = note.len_ms.max(1);
        let count = ((len_ms as f32 / 1000.0) * sr).round() as usize;
        if count == 0 {
            continue;
        }
        let note_gain = gain * note.gain.unwrap_or(1.0).clamp(0.0, 2.0);
        let attack = ((sound.attack_ms as f32 / 1000.0) * sr).round() as usize;
        let release = ((sound.release_ms as f32 / 1000.0) * sr).round() as usize;
        let phase_inc = (2.0 * PI * freq) / sr;
        let mut phase = 0.0f32;

        for idx in 0..count {
            let env = envelope(idx, count, attack, release);
            let raw = match waveform.as_str() {
                "sine" => phase.sin(),
                "square" => {
                    if phase.sin() >= 0.0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                "triangle" => (2.0 / PI) * phase.sin().asin(),
                "saw" => {
                    let x = phase / (2.0 * PI);
                    2.0 * (x - (x + 0.5).floor())
                }
                "noise" => {
                    noise_state = noise_state
                        .wrapping_mul(1_664_525)
                        .wrapping_add(1_013_904_223);
                    let unit = ((noise_state >> 8) & 0xFFFF) as f32 / 65_535.0;
                    (unit * 2.0) - 1.0
                }
                _ => 0.0,
            };
            let sample = (raw * env * note_gain).clamp(-1.0, 1.0);
            out.push((sample * 32767.0) as i16);
            phase += phase_inc;
            if phase > 2.0 * PI {
                phase -= 2.0 * PI;
            }
        }
    }

    Ok(out)
}

fn resolve_note_frequency(note: &SynthNote) -> Result<f32, SequencerError> {
    if let Some(freq) = note.freq_hz {
        if freq > 0.0 {
            return Ok(freq);
        }
    }
    let Some(name) = note.note.as_deref() else {
        return Err(SequencerError::Invalid(
            "note entry needs 'note' or 'freq_hz'".to_string(),
        ));
    };
    parse_note_hz(name)
        .ok_or_else(|| SequencerError::Invalid(format!("invalid note name '{name}'")))
}

fn parse_note_hz(name: &str) -> Option<f32> {
    let trimmed = name.trim();
    if trimmed.len() < 2 {
        return None;
    }
    let mut chars = trimmed.chars();
    let letter = chars.next()?.to_ascii_uppercase();
    let mut semitone = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };

    let rest: String = chars.collect();
    let (accidental_part, octave_part) =
        if rest.starts_with('#') || rest.starts_with('b') || rest.starts_with('B') {
            (&rest[..1], &rest[1..])
        } else {
            ("", rest.as_str())
        };
    match accidental_part {
        "#" => semitone += 1,
        "b" | "B" => semitone -= 1,
        _ => {}
    }
    let octave: i32 = octave_part.parse().ok()?;
    let midi = (octave + 1) * 12 + semitone;
    let freq = 440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0);
    Some(freq)
}

fn envelope(idx: usize, total: usize, attack: usize, release: usize) -> f32 {
    if total == 0 {
        return 0.0;
    }
    let mut env = 1.0f32;
    if attack > 0 && idx < attack {
        env *= idx as f32 / attack as f32;
    }
    if release > 0 {
        let release_start = total.saturating_sub(release);
        if idx >= release_start {
            let tail = total.saturating_sub(idx) as f32 / release as f32;
            env *= tail.clamp(0.0, 1.0);
        }
    }
    env.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_minimal_song_file() {
        let song = SongFile {
            id: "asteroids.main".to_string(),
            tempo_bpm: 128.0,
            time_signature: [4, 4],
            loop_region: Some(LoopRegion {
                start_beat: 4.0,
                end_beat: 20.0,
            }),
            tracks: vec![SongTrack {
                id: "pulse".to_string(),
                gain: 0.8,
                pan: 0.0,
                mute: false,
                clips: vec![TrackClip {
                    at_beat: 0.0,
                    pattern: "pulses.a".to_string(),
                    gain: 1.0,
                }],
            }],
            patterns: BTreeMap::from([(
                "pulses.a".to_string(),
                Pattern {
                    steps: vec![PatternStep {
                        at_beat: 0.0,
                        event: "music.pulse.hit".to_string(),
                        gain: Some(1.0),
                    }],
                },
            )]),
        };
        let result = validate_song_file(&song);
        assert!(result.is_ok(), "song should validate: {result:?}");
    }

    #[test]
    fn rejects_sfx_event_with_empty_variants() {
        let bank = SfxBank {
            version: 1,
            events: BTreeMap::from([(
                "ui.select".to_string(),
                SfxEvent {
                    gain: 1.0,
                    max_polyphony: 2,
                    cooldown_ms: 0,
                    variants: Vec::new(),
                },
            )]),
        };
        let result = validate_sfx_bank(&bank);
        assert!(result.is_err(), "bank should be invalid");
    }

    #[test]
    fn runtime_resolves_event_to_asset_stem_cue() {
        let mut runtime = SfxEventRuntime::new(SfxBank {
            version: 1,
            events: BTreeMap::from([(
                "ship.shoot".to_string(),
                SfxEvent {
                    gain: 0.8,
                    max_polyphony: 4,
                    cooldown_ms: 0,
                    variants: vec![SfxVariant {
                        asset: "assets/audio/sfx/shoot_01.wav".to_string(),
                        weight: 1,
                        gain: Some(0.5),
                        pitch_semitones: None,
                    }],
                },
            )]),
        });

        let resolved = runtime
            .resolve_event("ship.shoot", 123, Some(0.5))
            .expect("event should resolve");
        assert_eq!(resolved.cue, "shoot_01");
        assert!((resolved.gain - 0.2).abs() < 0.0001);
    }

    #[test]
    fn runtime_honors_event_cooldown() {
        let mut runtime = SfxEventRuntime::new(SfxBank {
            version: 1,
            events: BTreeMap::from([(
                "ui.tick".to_string(),
                SfxEvent {
                    gain: 1.0,
                    max_polyphony: 2,
                    cooldown_ms: 100,
                    variants: vec![SfxVariant {
                        asset: "assets/audio/sfx/tick.wav".to_string(),
                        weight: 1,
                        gain: None,
                        pitch_semitones: None,
                    }],
                },
            )]),
        });

        assert!(runtime.resolve_event("ui.tick", 1_000, None).is_some());
        assert!(runtime.resolve_event("ui.tick", 1_050, None).is_none());
        assert!(runtime.resolve_event("ui.tick", 1_100, None).is_some());
    }

    #[test]
    fn song_runtime_emits_pattern_events_and_loops() {
        let song = SongFile {
            id: "test.loop".to_string(),
            tempo_bpm: 120.0,
            time_signature: [4, 4],
            loop_region: Some(LoopRegion {
                start_beat: 0.0,
                end_beat: 2.0,
            }),
            tracks: vec![SongTrack {
                id: "lead".to_string(),
                gain: 1.0,
                pan: 0.0,
                mute: false,
                clips: vec![TrackClip {
                    at_beat: 0.0,
                    pattern: "p.a".to_string(),
                    gain: 1.0,
                }],
            }],
            patterns: BTreeMap::from([(
                "p.a".to_string(),
                Pattern {
                    steps: vec![
                        PatternStep {
                            at_beat: 0.0,
                            event: "e0".to_string(),
                            gain: Some(1.0),
                        },
                        PatternStep {
                            at_beat: 1.0,
                            event: "e1".to_string(),
                            gain: Some(1.0),
                        },
                    ],
                },
            )]),
        };

        let mut runtime = SongRuntime::new(song);
        // 120 BPM = 2 beats/sec. 600ms advances 1.2 beats => should hit e0 and e1.
        let first = runtime.tick(600);
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].event, "e0");
        assert_eq!(first[1].event, "e1");

        // Next 600ms crosses loop boundary and should hit e0 after wrap.
        let second = runtime.tick(600);
        assert!(second.iter().any(|ev| ev.event == "e0"));
        assert!(runtime.beat() >= 0.0 && runtime.beat() <= 2.0);
    }

    #[test]
    fn parses_note_names() {
        let a4 = parse_note_hz("A4").expect("A4");
        let c5 = parse_note_hz("C5").expect("C5");
        assert!((a4 - 440.0).abs() < 0.1);
        assert!((c5 - 523.25).abs() < 1.0);
    }
}

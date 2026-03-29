//! Validates optional audio sequencer authoring assets under `/audio`.

use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;

use engine_audio_sequencer::{validate_sfx_bank, validate_song_file, SfxBank, SongFile};
use engine_error::EngineError;
use zip::ZipArchive;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check for sequencer assets:
/// - `/audio/sfx.yml` or `/audio/sfx.yaml`
/// - `/audio/songs/**/*.yml|yaml`
///
/// The check is intentionally warning-only so mods can iterate without hard
/// startup failures while authoring audio content.
pub struct AudioSequencerCheck;

impl StartupCheck for AudioSequencerCheck {
    fn name(&self) -> &'static str {
        "audio-sequencer"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let files = discover_audio_yaml_files(ctx.mod_source())?;
        if files.is_empty() {
            report.add_info(
                self.name(),
                "audio sequencer check skipped (no /audio YAML found)",
            );
            return Ok(());
        }

        let mut sfx_bank: Option<SfxBank> = None;
        let mut sfx_files = 0usize;
        let mut song_files = 0usize;

        for file in &files {
            let normalized = file.path.trim_start_matches('/').replace('\\', "/");
            let lowered = normalized.to_ascii_lowercase();

            if lowered == "audio/sfx.yml" || lowered == "audio/sfx.yaml" {
                sfx_files += 1;
                match serde_yaml::from_slice::<SfxBank>(&file.bytes) {
                    Ok(bank) => {
                        if let Err(error) = validate_sfx_bank(&bank) {
                            report.add_warning(
                                self.name(),
                                format!("invalid sfx bank `{}`: {error}", file.path),
                            );
                        } else {
                            for (event, spec) in &bank.events {
                                for variant in &spec.variants {
                                    if is_synth_asset(&variant.asset) {
                                        continue;
                                    }
                                    if !asset_exists(ctx.mod_source(), &variant.asset)? {
                                        report.add_warning(
                                            self.name(),
                                            format!(
                                                "event `{event}` references missing asset `{}`",
                                                variant.asset
                                            ),
                                        );
                                    }
                                }
                            }
                            sfx_bank = Some(bank);
                        }
                    }
                    Err(error) => report.add_warning(
                        self.name(),
                        format!("cannot parse sfx bank `{}`: {error}", file.path),
                    ),
                }
                continue;
            }

            if !lowered.starts_with("audio/songs/") {
                continue;
            }
            song_files += 1;
            match serde_yaml::from_slice::<SongFile>(&file.bytes) {
                Ok(song) => {
                    if let Err(error) = validate_song_file(&song) {
                        report.add_warning(
                            self.name(),
                            format!("invalid song `{}`: {error}", file.path),
                        );
                        continue;
                    }
                    if let Some(bank) = &sfx_bank {
                        let events = song_events(&song);
                        for event in events {
                            if !bank.events.contains_key(&event) {
                                report.add_warning(
                                    self.name(),
                                    format!(
                                        "song `{}` references unknown event `{event}` (missing in sfx bank)",
                                        file.path
                                    ),
                                );
                            }
                        }
                    }
                }
                Err(error) => report.add_warning(
                    self.name(),
                    format!("cannot parse song `{}`: {error}", file.path),
                ),
            }
        }

        if sfx_files > 1 {
            report.add_warning(
                self.name(),
                format!(
                    "multiple sfx banks found ({sfx_files}); expected a single `audio/sfx.yaml`"
                ),
            );
        }

        report.add_info(
            self.name(),
            format!(
                "audio sequencer checked ({} yaml files, {} sfx bank(s), {} song file(s))",
                files.len(),
                sfx_files,
                song_files
            ),
        );
        Ok(())
    }
}

#[derive(Debug)]
struct AudioYamlFile {
    path: String,
    bytes: Vec<u8>,
}

fn discover_audio_yaml_files(mod_source: &Path) -> Result<Vec<AudioYamlFile>, EngineError> {
    if mod_source.is_dir() {
        discover_audio_yaml_files_from_dir(mod_source)
    } else if is_zip_file(mod_source) {
        discover_audio_yaml_files_from_zip(mod_source)
    } else {
        Ok(Vec::new())
    }
}

fn discover_audio_yaml_files_from_dir(
    mod_source: &Path,
) -> Result<Vec<AudioYamlFile>, EngineError> {
    let audio_root = mod_source.join("audio");
    if !audio_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    walk_audio_dir(mod_source, &audio_root, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk_audio_dir(
    mod_source: &Path,
    dir: &Path,
    out: &mut Vec<AudioYamlFile>,
) -> Result<(), EngineError> {
    let entries = fs::read_dir(dir).map_err(|source| EngineError::ManifestRead {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| EngineError::ManifestRead {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            walk_audio_dir(mod_source, &path, out)?;
            continue;
        }
        if !is_yaml_file(&path) {
            continue;
        }
        let rel = normalize_relative_asset_path(mod_source, &path);
        let bytes = fs::read(&path).map_err(|source| EngineError::ManifestRead {
            path: path.clone(),
            source,
        })?;
        out.push(AudioYamlFile { path: rel, bytes });
    }
    Ok(())
}

fn discover_audio_yaml_files_from_zip(
    mod_source: &Path,
) -> Result<Vec<AudioYamlFile>, EngineError> {
    let file = fs::File::open(mod_source).map_err(|source| EngineError::ManifestRead {
        path: mod_source.to_path_buf(),
        source,
    })?;
    let mut archive = ZipArchive::new(file).map_err(|source| EngineError::ZipArchive {
        path: mod_source.to_path_buf(),
        source,
    })?;
    let mut out = Vec::new();
    for idx in 0..archive.len() {
        let mut entry = archive
            .by_index(idx)
            .map_err(|source| EngineError::ZipArchive {
                path: mod_source.to_path_buf(),
                source,
            })?;
        if !entry.is_file() {
            continue;
        }
        let normalized = format!(
            "/{}",
            entry.name().trim_start_matches('/').replace('\\', "/")
        );
        if !normalized.to_ascii_lowercase().starts_with("/audio/") {
            continue;
        }
        if !(normalized.ends_with(".yml") || normalized.ends_with(".yaml")) {
            continue;
        }
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|source| EngineError::ManifestRead {
                path: mod_source.to_path_buf(),
                source,
            })?;
        out.push(AudioYamlFile {
            path: normalized,
            bytes,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn asset_exists(mod_source: &Path, asset: &str) -> Result<bool, EngineError> {
    if is_synth_asset(asset) {
        return Ok(true);
    }
    let normalized = asset.trim_start_matches('/').replace('\\', "/");
    if normalized.trim().is_empty() {
        return Ok(false);
    }

    if mod_source.is_dir() {
        return Ok(mod_source.join(&normalized).is_file());
    }
    if !is_zip_file(mod_source) {
        return Ok(false);
    }

    let file = fs::File::open(mod_source).map_err(|source| EngineError::ManifestRead {
        path: mod_source.to_path_buf(),
        source,
    })?;
    let mut archive = ZipArchive::new(file).map_err(|source| EngineError::ZipArchive {
        path: mod_source.to_path_buf(),
        source,
    })?;
    let needle = normalized.to_ascii_lowercase();
    for idx in 0..archive.len() {
        let entry = archive
            .by_index(idx)
            .map_err(|source| EngineError::ZipArchive {
                path: mod_source.to_path_buf(),
                source,
            })?;
        let name = entry.name().trim_start_matches('/').replace('\\', "/");
        if name.to_ascii_lowercase() == needle {
            return Ok(true);
        }
    }
    Ok(false)
}

fn is_synth_asset(path: &str) -> bool {
    path.trim_start().to_ascii_lowercase().starts_with("synth:")
}

fn song_events(song: &SongFile) -> BTreeSet<String> {
    let mut events = BTreeSet::new();
    for pattern in song.patterns.values() {
        for step in &pattern.steps {
            if !step.event.trim().is_empty() {
                events.insert(step.event.trim().to_string());
            }
        }
    }
    events
}

fn normalize_relative_asset_path(mod_source: &Path, full_path: &Path) -> String {
    let rel = full_path.strip_prefix(mod_source).unwrap_or(full_path);
    format!("/{}", rel.display().to_string().replace('\\', "/"))
}

fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            let ext = ext.to_ascii_lowercase();
            ext == "yml" || ext == "yaml"
        })
}

fn is_zip_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
}

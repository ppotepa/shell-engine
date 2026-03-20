//! Cutscene maker command dispatch.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::input::commands::Command;

use super::{AppState, SidebarItem};

impl AppState {
    pub(super) fn activate_cutscene_maker(&mut self) {
        self.reset_scene_fullscreen_state();
        self.sidebar.active = SidebarItem::Cutscene;
        self.sidebar.visible = true;
        self.focus = super::focus::FocusPane::ProjectTree;
        self.refresh_cutscene_source_folder();
        self.status = self.cutscene_status_message();
    }

    pub(super) fn refresh_cutscene_source_folder(&mut self) {
        self.cutscene.frames.clear();
        self.cutscene.missing_frames.clear();
        self.cutscene.validation_error = None;

        if self.mod_source.is_empty() {
            self.cutscene.source_dir = "assets/raw".to_string();
            self.cutscene.validation_error = Some("Open a mod project first.".to_string());
            return;
        }

        let source_dir = Path::new(&self.mod_source).join("assets/raw");
        self.cutscene.source_dir = source_dir.display().to_string();
        if !source_dir.exists() {
            self.cutscene.validation_error = Some("Missing folder: assets/raw".to_string());
            return;
        }
        if !source_dir.is_dir() {
            self.cutscene.validation_error =
                Some("assets/raw exists but is not a directory".to_string());
            return;
        }

        let mut numbered_frames: BTreeMap<u32, Vec<String>> = BTreeMap::new();
        let mut invalid_named_frames: Vec<String> = Vec::new();
        let mut read_failed = false;

        const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "webp"];

        match fs::read_dir(&source_dir) {
            Ok(entries) => {
                for entry in entries {
                    let Ok(entry) = entry else {
                        read_failed = true;
                        continue;
                    };
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }

                    let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
                        continue;
                    };
                    let ext = ext.to_ascii_lowercase();
                    if !IMAGE_EXTENSIONS.contains(&ext.as_str()) {
                        continue;
                    }

                    let file_name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("<invalid-name>")
                        .to_string();
                    let stem = path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("");
                    match stem.parse::<u32>() {
                        Ok(number) if number > 0 => {
                            numbered_frames.entry(number).or_default().push(file_name);
                        }
                        _ => invalid_named_frames.push(file_name),
                    }
                }
            }
            Err(err) => {
                self.cutscene.validation_error =
                    Some(format!("Cannot read assets/raw folder: {err}"));
                return;
            }
        }

        if read_failed {
            self.cutscene.validation_error =
                Some("Could not read some files from assets/raw".to_string());
            return;
        }

        if !invalid_named_frames.is_empty() {
            invalid_named_frames.sort();
            self.cutscene.validation_error = Some(format!(
                "Invalid frame names (must be numeric): {}",
                invalid_named_frames.join(", ")
            ));
            return;
        }

        if numbered_frames.is_empty() {
            self.cutscene.validation_error = Some(
                "No numbered image frames found in assets/raw (expected 1.png, 2.png, ...)"
                    .to_string(),
            );
            return;
        }

        let mut duplicate_numbers = Vec::new();
        for (number, names) in &numbered_frames {
            if names.len() > 1 {
                duplicate_numbers.push(format!("{number}"));
            }
        }
        if !duplicate_numbers.is_empty() {
            self.cutscene.validation_error = Some(format!(
                "Duplicate frame numbers detected: {}",
                duplicate_numbers.join(", ")
            ));
            return;
        }

        if let Some(max_number) = numbered_frames.keys().copied().max() {
            for expected in 1..=max_number {
                if !numbered_frames.contains_key(&expected) {
                    self.cutscene.missing_frames.push(expected);
                }
            }
        }

        for names in numbered_frames.values_mut() {
            names.sort();
        }
        self.cutscene.frames = numbered_frames
            .values()
            .filter_map(|names| names.first().cloned())
            .collect();

        if !self.cutscene.missing_frames.is_empty() {
            let preview = self
                .cutscene
                .missing_frames
                .iter()
                .take(12)
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            let suffix = if self.cutscene.missing_frames.len() > 12 {
                ", ..."
            } else {
                ""
            };
            self.cutscene.validation_error =
                Some(format!("Missing frame numbers: {preview}{suffix}"));
        }
    }

    pub(super) fn cutscene_status_message(&self) -> String {
        if let Some(err) = &self.cutscene.validation_error {
            return format!("Cutscene Maker: invalid source ({err})");
        }
        if self.cutscene.frames.is_empty() {
            "Cutscene Maker: no frames detected in assets/raw".to_string()
        } else {
            format!(
                "Cutscene Maker: {} frame(s) ready | default {}ms/frame",
                self.cutscene.frames.len(),
                self.cutscene.default_frame_ms
            )
        }
    }

    pub(super) fn handle_cutscene_command(&mut self, cmd: Command) -> bool {
        if self.sidebar.active != SidebarItem::Cutscene {
            return false;
        }

        match cmd {
            Command::TogglePreview => {
                self.refresh_cutscene_source_folder();
                self.status = self.cutscene_status_message();
                true
            }
            _ => false,
        }
    }
}

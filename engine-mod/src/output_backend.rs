//! Startup output selection for the current run.

use serde_yaml::Value;

/// Which output backend was selected for this run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupOutputSetting {
    Sdl2,
}

impl StartupOutputSetting {
    /// Parse from a CLI string like `"sdl2"`.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "sdl2" | "sdl" => Some(Self::Sdl2),
            _ => None,
        }
    }

    /// The engine is SDL2-only, so the manifest no longer controls output selection.
    pub fn from_manifest(_manifest: &Value) -> Result<Option<Self>, String> {
        Ok(None)
    }
}

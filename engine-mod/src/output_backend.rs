//! Startup output backend selection parsed from the mod manifest.

use serde_yaml::Value;

/// Which output backend was selected for this run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupOutputSetting {
    Terminal,
    Sdl2,
    Prompt,
}

impl StartupOutputSetting {
    /// Parse from a CLI string like `"terminal"`, `"sdl2"`, or `"prompt"`.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "terminal" => Some(Self::Terminal),
            "sdl2" | "sdl" => Some(Self::Sdl2),
            "prompt" => Some(Self::Prompt),
            _ => None,
        }
    }

    /// Attempts to parse a preferred output backend from the manifest.
    ///
    /// Returns `Ok(None)` when no preference is declared (caller picks default).
    pub fn from_manifest(manifest: &Value) -> Result<Option<Self>, String> {
        let Some(terminal) = manifest.get("terminal") else {
            return Ok(None);
        };
        let Some(output) = terminal
            .get("output")
            .or_else(|| terminal.get("output-backend"))
            .and_then(Value::as_str)
        else {
            return Ok(None);
        };
        match output.trim().to_ascii_lowercase().as_str() {
            "terminal" => Ok(Some(Self::Terminal)),
            "sdl2" | "sdl" => Ok(Some(Self::Sdl2)),
            other => Err(format!("unknown output backend: {other}")),
        }
    }
}

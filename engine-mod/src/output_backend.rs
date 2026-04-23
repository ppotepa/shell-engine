//! Startup backend selection compatibility shim for the current run.

use serde_yaml::Value;

/// Which startup backend token was selected for this run.
///
/// Transitional note:
/// This enum is kept for compatibility with existing startup wiring.
/// Canonical backend selection is runtime/CLI-owned, not manifest-owned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupOutputSetting {
    /// Backend-neutral compatibility token used by startup validation paths.
    Compatibility,
}

impl StartupOutputSetting {
    /// Returns the backend-neutral compatibility token.
    pub const fn compatibility_default() -> Self {
        Self::Compatibility
    }

    /// Normalizes legacy aliases into the backend-neutral compatibility token.
    pub const fn to_compatibility_token(self) -> Self {
        Self::Compatibility
    }

    /// Parse compatibility aliases from a startup string like `"sdl2"`.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "compat" | "compatibility" | "default" => Some(Self::Compatibility),
            "sdl2" | "sdl" => Some(Self::Compatibility),
            _ => None,
        }
    }

    /// Manifest does not choose the canonical runtime backend in the transition model.
    ///
    /// The startup path accepts backend selection from launch/runtime configuration,
    /// while manifest content remains backend-agnostic.
    pub fn from_manifest(_manifest: &Value) -> Result<Option<Self>, String> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::StartupOutputSetting;

    #[test]
    fn parse_accepts_backend_neutral_aliases() {
        assert_eq!(
            StartupOutputSetting::parse("compatibility"),
            Some(StartupOutputSetting::Compatibility)
        );
        assert_eq!(
            StartupOutputSetting::parse(" default "),
            Some(StartupOutputSetting::Compatibility)
        );
    }

    #[test]
    fn parse_accepts_legacy_sdl_aliases() {
        assert_eq!(
            StartupOutputSetting::parse("sdl2"),
            Some(StartupOutputSetting::Compatibility)
        );
        assert_eq!(
            StartupOutputSetting::parse("SDL"),
            Some(StartupOutputSetting::Compatibility)
        );
    }

    #[test]
    fn compatibility_projection_returns_compatibility_token() {
        assert_eq!(
            StartupOutputSetting::Compatibility.to_compatibility_token(),
            StartupOutputSetting::Compatibility
        );
    }
}

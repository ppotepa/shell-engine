//! Runtime debug feature toggles that can be enabled independently from build profile.

/// Debug overlay display mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DebugOverlayMode {
    #[default]
    Stats,
    Logs,
}

/// Debug feature flags and transient UI state.
#[derive(Debug, Clone, Copy, Default)]
pub struct DebugFeatures {
    /// Master switch for debug helpers (F1/F3/F4 and overlay).
    pub enabled: bool,
    /// Whether the debug overlay is currently visible.
    pub overlay_visible: bool,
    /// Current overlay display mode.
    pub overlay_mode: DebugOverlayMode,
}

impl DebugFeatures {
    /// Builds debug feature state directly from a boolean flag.
    pub fn from_enabled(enabled: bool) -> Self {
        Self {
            enabled,
            overlay_visible: enabled,
            overlay_mode: DebugOverlayMode::default(),
        }
    }

    /// Builds debug feature state from environment.
    ///
    /// Recognized truthy values:
    /// - `1`
    /// - `true`
    /// - `yes`
    /// - `on`
    pub fn from_env() -> Self {
        let enabled = env_flag_enabled("SHELL_QUEST_DEBUG_FEATURE");
        Self::from_enabled(enabled)
    }
}

fn env_flag_enabled(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|raw| {
            matches!(
                raw.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LauncherConfig {
    #[serde(default)]
    pub flags: LaunchFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchFlags {
    #[serde(default)]
    pub skip_splash: bool,
    #[serde(default = "default_true")]
    pub audio: bool,
    #[serde(default)]
    pub check_scenes: bool,
    #[serde(default)]
    pub release: bool,
    #[serde(default)]
    pub dev: bool,
    #[serde(default)]
    pub all_opt: bool,
    #[serde(default)]
    pub render_backend: RenderBackendSetting,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderBackendSetting {
    Software,
    Hardware,
}

impl RenderBackendSetting {
    pub fn as_cli_value(self) -> &'static str {
        match self {
            Self::Software => "software",
            Self::Hardware => "hardware",
        }
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Self::Software => Self::Hardware,
            Self::Hardware => Self::Software,
        };
    }
}

impl Default for RenderBackendSetting {
    fn default() -> Self {
        Self::Hardware
    }
}

fn default_true() -> bool {
    true
}

impl Default for LaunchFlags {
    fn default() -> Self {
        Self {
            skip_splash: false,
            audio: true,
            check_scenes: false,
            release: false,
            dev: false,
            all_opt: false,
            render_backend: RenderBackendSetting::Hardware,
        }
    }
}

pub fn load_config(workspace_root: &Path) -> Result<LauncherConfig> {
    let config_path = workspace_root.join(".se.toml");

    if !config_path.exists() {
        return Ok(LauncherConfig::default());
    }

    let content = fs::read_to_string(&config_path).context("failed to read .se.toml")?;

    toml::from_str(&content).context("failed to parse .se.toml")
}

pub fn save_config(workspace_root: &Path, config: &LauncherConfig) -> Result<()> {
    let config_path = workspace_root.join(".se.toml");

    let content = toml::to_string_pretty(config).context("failed to serialize config")?;

    let header = "# Shell Engine launcher config — auto-generated, safe to edit\n";
    let full = format!("{}{}", header, content);

    fs::write(&config_path, full).context("failed to write .se.toml")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{LaunchFlags, RenderBackendSetting};

    #[test]
    fn launch_flags_default_to_hardware_backend() {
        let flags = LaunchFlags::default();
        assert!(matches!(
            flags.render_backend,
            RenderBackendSetting::Hardware
        ));
    }
}

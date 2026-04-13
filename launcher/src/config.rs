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

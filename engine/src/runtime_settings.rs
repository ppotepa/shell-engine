use serde_yaml::Value;
use std::env;

use crate::scene::SceneRenderedMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualPolicy {
    Strict,
    Fit,
}

impl Default for VirtualPolicy {
    fn default() -> Self {
        Self::Fit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeSettings {
    pub use_virtual_buffer: bool,
    pub virtual_width: u16,
    pub virtual_height: u16,
    pub virtual_policy: VirtualPolicy,
    pub renderer_mode_override: Option<SceneRenderedMode>,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            use_virtual_buffer: false,
            virtual_width: 320,
            virtual_height: 240,
            virtual_policy: VirtualPolicy::Fit,
            renderer_mode_override: None,
        }
    }
}

impl RuntimeSettings {
    pub fn from_manifest(manifest: &Value) -> Self {
        let mut settings = Self::default();

        if let Some(block) = manifest.get("terminal") {
            if let Some(enabled) = block
                .get("use_virtual_buffer")
                .or_else(|| block.get("use-virtual-buffer"))
                .and_then(Value::as_bool)
            {
                settings.use_virtual_buffer = enabled;
            }

            let size = block
                .get("virtual_size")
                .or_else(|| block.get("virtual-size"))
                .and_then(Value::as_str)
                .and_then(parse_virtual_size);
            if let Some((w, h)) = size {
                settings.virtual_width = w;
                settings.virtual_height = h;
            }

            let policy = block
                .get("virtual_policy")
                .or_else(|| block.get("virtual-policy"))
                .and_then(Value::as_str)
                .and_then(parse_virtual_policy);
            if let Some(policy) = policy {
                settings.virtual_policy = policy;
            }

            let renderer_mode = block
                .get("renderer_mode")
                .or_else(|| block.get("renderer-mode"))
                .and_then(Value::as_str)
                .and_then(parse_renderer_mode);
            if renderer_mode.is_some() {
                settings.renderer_mode_override = renderer_mode;
            }
        }

        if let Ok(raw) = env::var("SHELL_QUEST_USE_VIRTUAL_BUFFER") {
            if let Some(parsed) = parse_bool(&raw) {
                settings.use_virtual_buffer = parsed;
            }
        }

        if let Ok(raw) = env::var("SHELL_QUEST_VIRTUAL_SIZE") {
            if let Some((w, h)) = parse_virtual_size(&raw) {
                settings.virtual_width = w;
                settings.virtual_height = h;
            }
        }

        if let Ok(raw) = env::var("SHELL_QUEST_VIRTUAL_POLICY") {
            if let Some(policy) = parse_virtual_policy(&raw) {
                settings.virtual_policy = policy;
            }
        }

        if let Ok(raw) = env::var("SHELL_QUEST_RENDERER_MODE") {
            if let Some(mode) = parse_renderer_mode(&raw) {
                settings.renderer_mode_override = Some(mode);
            }
        }

        settings
    }
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_virtual_policy(raw: &str) -> Option<VirtualPolicy> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "strict" => Some(VirtualPolicy::Strict),
        "fit" => Some(VirtualPolicy::Fit),
        _ => None,
    }
}

fn parse_renderer_mode(raw: &str) -> Option<SceneRenderedMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "cell" => Some(SceneRenderedMode::Cell),
        "halfblock" | "half-block" => Some(SceneRenderedMode::HalfBlock),
        "quadblock" | "quad-block" => Some(SceneRenderedMode::QuadBlock),
        "braille" => Some(SceneRenderedMode::Braille),
        _ => None,
    }
}

fn parse_virtual_size(raw: &str) -> Option<(u16, u16)> {
    let mut parts = raw.trim().split('x');
    let w = parts.next()?.trim().parse::<u16>().ok()?;
    let h = parts.next()?.trim().parse::<u16>().ok()?;
    if parts.next().is_some() || w == 0 || h == 0 {
        return None;
    }
    Some((w, h))
}

#[cfg(test)]
mod tests {
    use super::{RuntimeSettings, VirtualPolicy};

    #[test]
    fn parses_runtime_settings_from_manifest_terminal_block() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "terminal:\n  use-virtual-buffer: true\n  virtual-size: \"320x200\"\n  virtual-policy: strict\n",
        )
        .expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert!(settings.use_virtual_buffer);
        assert_eq!(settings.virtual_width, 320);
        assert_eq!(settings.virtual_height, 200);
        assert_eq!(settings.virtual_policy, VirtualPolicy::Strict);
        assert_eq!(settings.renderer_mode_override, None);
    }

    #[test]
    fn keeps_defaults_when_block_absent() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>("name: test\n").expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert!(!settings.use_virtual_buffer);
        assert_eq!(settings.virtual_width, 320);
        assert_eq!(settings.virtual_height, 240);
        assert_eq!(settings.virtual_policy, VirtualPolicy::Fit);
        assert_eq!(settings.renderer_mode_override, None);
    }

    #[test]
    fn parses_renderer_mode_from_manifest_terminal_block() {
        let yaml =
            serde_yaml::from_str::<serde_yaml::Value>("terminal:\n  renderer-mode: braille\n")
                .expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert_eq!(
            settings.renderer_mode_override,
            Some(crate::scene::SceneRenderedMode::Braille)
        );
    }
}

pub mod access;

use serde_yaml::Value;
use std::env;

use engine_core::scene::SceneRenderedMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationPolicy {
    Strict,
    Fit,
    Stretch,
}

pub type VirtualPolicy = PresentationPolicy;

impl Default for PresentationPolicy {
    fn default() -> Self {
        Self::Fit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSize {
    Fixed { width: u16, height: u16 },
    MatchOutput,
    /// Fix the width; derive height from the live terminal's aspect ratio.
    /// Authored as `"640x~"` in YAML.
    FitWidth { width: u16 },
}

impl Default for RenderSize {
    fn default() -> Self {
        Self::Fixed {
            width: 320,
            height: 240,
        }
    }
}

impl RenderSize {
    pub fn resolve(self, output_width: u16, output_height: u16) -> (u16, u16) {
        match self {
            Self::Fixed { width, height } => (width, height),
            Self::MatchOutput => (output_width.max(1), output_height.max(1)),
            Self::FitWidth { width } => {
                let ow = output_width.max(1) as u32;
                let oh = output_height.max(1) as u32;
                let height = ((width as u32 * oh) / ow).clamp(1, u16::MAX as u32) as u16;
                (width, height)
            }
        }
    }

    pub fn matches_output(self) -> bool {
        matches!(self, Self::MatchOutput | Self::FitWidth { .. })
    }

    pub fn fixed(self) -> Option<(u16, u16)> {
        match self {
            Self::Fixed { width, height } => Some((width, height)),
            Self::MatchOutput | Self::FitWidth { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferLayout {
    pub render_width: u16,
    pub render_height: u16,
    pub output_width: u16,
    pub output_height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSettings {
    pub render_size: RenderSize,
    pub presentation_policy: PresentationPolicy,
    pub renderer_mode_override: Option<SceneRenderedMode>,
    /// Optional mod-level default text font used when sprite `font` is set to
    /// `default`. Supports both generic specs and named bitmap fonts.
    pub default_font: Option<String>,
    /// True when rendering to a pixel backend (SDL2), false for terminal.
    /// Used by the compositor to select backend-appropriate font modes.
    pub is_pixel_backend: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentationLayout {
    pub dst_x: u32,
    pub dst_y: u32,
    pub dst_width: u32,
    pub dst_height: u32,
    pub src_x: u32,
    pub src_y: u32,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            render_size: RenderSize::default(),
            presentation_policy: PresentationPolicy::Fit,
            renderer_mode_override: None,
            default_font: None,
            is_pixel_backend: false,
        }
    }
}

impl RuntimeSettings {
    pub fn from_manifest(manifest: &Value) -> Self {
        let mut settings = Self::default();

        if let Some(block) = manifest.get("terminal") {
            let _legacy_use_virtual_buffer = block
                .get("use_virtual_buffer")
                .or_else(|| block.get("use-virtual-buffer"))
                .and_then(Value::as_bool);

            let size = block
                .get("render_size")
                .or_else(|| block.get("render-size"))
                .or_else(|| block.get("virtual_size"))
                .or_else(|| block.get("virtual-size"))
                .and_then(Value::as_str)
                .and_then(parse_render_size);
            if let Some(size) = size {
                settings.render_size = size;
            }

            let policy = block
                .get("presentation_policy")
                .or_else(|| block.get("presentation-policy"))
                .or_else(|| block.get("virtual_policy"))
                .or_else(|| block.get("virtual-policy"))
                .and_then(Value::as_str)
                .and_then(parse_presentation_policy);
            if let Some(policy) = policy {
                settings.presentation_policy = policy;
            }

            let renderer_mode = block
                .get("renderer_mode")
                .or_else(|| block.get("renderer-mode"))
                .and_then(Value::as_str)
                .and_then(parse_renderer_mode);
            if renderer_mode.is_some() {
                settings.renderer_mode_override = renderer_mode;
            }

            let default_font = block
                .get("default_font")
                .or_else(|| block.get("default-font"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            if default_font.is_some() {
                settings.default_font = default_font;
            }
        }

        if let Some(size) = env::var("SHELL_QUEST_RENDER_SIZE")
            .ok()
            .as_deref()
            .and_then(parse_render_size)
        {
            settings.render_size = size;
        }

        if let Some(policy) = env::var("SHELL_QUEST_PRESENTATION_POLICY")
            .ok()
            .as_deref()
            .and_then(parse_presentation_policy)
        {
            settings.presentation_policy = policy;
        }

        if let Ok(raw) = env::var("SHELL_QUEST_RENDERER_MODE") {
            if let Some(mode) = parse_renderer_mode(&raw) {
                settings.renderer_mode_override = Some(mode);
            }
        }

        if let Some(default_font) = env::var("SHELL_QUEST_DEFAULT_FONT")
            .ok()
            .map(|raw| raw.trim().to_string())
            .filter(|raw| !raw.is_empty())
        {
            settings.default_font = Some(default_font);
        }

        settings
    }

    pub fn resolved_render_size(&self, output_width: u16, output_height: u16) -> (u16, u16) {
        self.render_size.resolve(output_width, output_height)
    }

    pub fn buffer_layout(&self, output_width: u16, output_height: u16) -> BufferLayout {
        let output_width = output_width.max(1);
        let output_height = output_height.max(1);
        let (render_width, render_height) = self.resolved_render_size(output_width, output_height);
        BufferLayout {
            render_width,
            render_height,
            output_width,
            output_height,
        }
    }

    pub fn render_size_matches_output(&self) -> bool {
        self.render_size.matches_output()
    }

    pub fn fixed_render_size(&self) -> Option<(u16, u16)> {
        self.render_size.fixed()
    }

}

pub fn compute_presentation_layout(
    container_width: u32,
    container_height: u32,
    content_width: u32,
    content_height: u32,
    policy: PresentationPolicy,
) -> PresentationLayout {
    let container_width = container_width.max(1);
    let container_height = container_height.max(1);
    let content_width = content_width.max(1);
    let content_height = content_height.max(1);

    match policy {
        PresentationPolicy::Stretch => PresentationLayout {
            dst_x: 0,
            dst_y: 0,
            dst_width: container_width,
            dst_height: container_height,
            src_x: 0,
            src_y: 0,
        },
        PresentationPolicy::Fit => {
            let (dst_width, dst_height) = fit_size(
                container_width,
                container_height,
                content_width,
                content_height,
            );
            PresentationLayout {
                dst_x: centered_offset(container_width, dst_width),
                dst_y: centered_offset(container_height, dst_height),
                dst_width,
                dst_height,
                src_x: 0,
                src_y: 0,
            }
        }
        PresentationPolicy::Strict => {
            let dst_width = container_width.min(content_width);
            let dst_height = container_height.min(content_height);
            PresentationLayout {
                dst_x: centered_offset(container_width, dst_width),
                dst_y: centered_offset(container_height, dst_height),
                dst_width,
                dst_height,
                src_x: centered_offset(content_width, dst_width),
                src_y: centered_offset(content_height, dst_height),
            }
        }
    }
}

fn fit_size(
    container_width: u32,
    container_height: u32,
    content_width: u32,
    content_height: u32,
) -> (u32, u32) {
    if container_width.saturating_mul(content_height)
        <= container_height.saturating_mul(content_width)
    {
        (
            container_width.max(1),
            (container_width.saturating_mul(content_height) / content_width.max(1)).max(1),
        )
    } else {
        (
            (container_height.saturating_mul(content_width) / content_height.max(1)).max(1),
            container_height.max(1),
        )
    }
}

fn centered_offset(container: u32, content: u32) -> u32 {
    container.saturating_sub(content) / 2
}

fn parse_presentation_policy(raw: &str) -> Option<PresentationPolicy> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "strict" => Some(PresentationPolicy::Strict),
        "fit" => Some(PresentationPolicy::Fit),
        "stretch" => Some(PresentationPolicy::Stretch),
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

pub fn parse_render_size(raw: &str) -> Option<RenderSize> {
    let normalized = raw.trim().to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "match-output"
            | "match_output"
            | "matchoutput"
            | "max-available"
            | "max_available"
            | "maxavailable"
    ) {
        return Some(RenderSize::MatchOutput);
    }
    let mut parts = normalized.split('x');
    let w_str = parts.next()?.trim();
    let h_str = parts.next()?.trim();
    if parts.next().is_some() {
        return None;
    }
    // "640x~" — fix width, adapt height to terminal aspect ratio
    if h_str == "~" {
        let w = w_str.parse::<u16>().ok()?;
        if w == 0 {
            return None;
        }
        return Some(RenderSize::FitWidth { width: w });
    }
    let w = w_str.parse::<u16>().ok()?;
    let h = h_str.parse::<u16>().ok()?;
    if w == 0 || h == 0 {
        return None;
    }
    Some(RenderSize::Fixed {
        width: w,
        height: h,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        compute_presentation_layout, BufferLayout, PresentationLayout, PresentationPolicy,
        RenderSize, RuntimeSettings,
    };

    #[test]
    fn parses_runtime_settings_from_manifest_terminal_block() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "terminal:\n  use-virtual-buffer: true\n  render-size: \"320x200\"\n  presentation-policy: strict\n",
        )
        .expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert_eq!(
            settings.render_size,
            RenderSize::Fixed {
                width: 320,
                height: 200
            }
        );
        assert_eq!(settings.presentation_policy, PresentationPolicy::Strict);
        assert_eq!(settings.renderer_mode_override, None);
        assert_eq!(settings.default_font, None);
    }

    #[test]
    fn parses_stretch_presentation_policy() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "terminal:\n  use-virtual-buffer: true\n  render-size: \"120x30\"\n  presentation-policy: stretch\n",
        )
        .expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert_eq!(settings.presentation_policy, PresentationPolicy::Stretch);
    }

    #[test]
    fn keeps_defaults_when_block_absent() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>("name: test\n").expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert_eq!(
            settings.render_size,
            RenderSize::Fixed {
                width: 320,
                height: 240
            }
        );
        assert_eq!(settings.presentation_policy, PresentationPolicy::Fit);
        assert_eq!(settings.renderer_mode_override, None);
        assert_eq!(settings.default_font, None);
    }

    #[test]
    fn parses_renderer_mode_from_manifest_terminal_block() {
        let yaml =
            serde_yaml::from_str::<serde_yaml::Value>("terminal:\n  renderer-mode: braille\n")
                .expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert_eq!(
            settings.renderer_mode_override,
            Some(engine_core::scene::SceneRenderedMode::Braille)
        );
    }

    #[test]
    fn parses_default_font_from_manifest_terminal_block() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "terminal:\n  default-font: DejaVuSans-BoldOblique\n",
        )
        .expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert_eq!(
            settings.default_font.as_deref(),
            Some("DejaVuSans-BoldOblique")
        );
    }

    #[test]
    fn parses_max_available_virtual_size() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "terminal:\n  use-virtual-buffer: true\n  render-size: match-output\n",
        )
        .expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert!(settings.render_size_matches_output());
        assert_eq!(settings.resolved_render_size(180, 52), (180, 52));
    }

    #[test]
    fn keeps_virtual_aliases_compatible() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "terminal:\n  virtual-size: max-available\n  virtual-policy: fit\n",
        )
        .expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert!(settings.render_size_matches_output());
        assert_eq!(settings.presentation_policy, PresentationPolicy::Fit);
    }

    #[test]
    fn computes_buffer_layout_from_render_and_output_sizes() {
        let settings = RuntimeSettings {
            render_size: RenderSize::Fixed {
                width: 120,
                height: 30,
            },
            presentation_policy: PresentationPolicy::Stretch,
            renderer_mode_override: None,
            default_font: None,
            is_pixel_backend: false,
        };

        assert_eq!(
            settings.buffer_layout(80, 24),
            BufferLayout {
                render_width: 120,
                render_height: 30,
                output_width: 80,
                output_height: 24,
            }
        );
    }

    #[test]
    fn presentation_layout_fit_preserves_aspect_ratio_for_letterboxing() {
        assert_eq!(
            compute_presentation_layout(960, 640, 960, 480, PresentationPolicy::Fit),
            PresentationLayout {
                dst_x: 0,
                dst_y: 80,
                dst_width: 960,
                dst_height: 480,
                src_x: 0,
                src_y: 0,
            }
        );
    }

    #[test]
    fn presentation_layout_fit_upscales_proportionally() {
        assert_eq!(
            compute_presentation_layout(210, 109, 180, 30, PresentationPolicy::Fit),
            PresentationLayout {
                dst_x: 0,
                dst_y: 37,
                dst_width: 210,
                dst_height: 35,
                src_x: 0,
                src_y: 0,
            }
        );
    }

    #[test]
    fn presentation_layout_strict_centers_and_crops_when_needed() {
        assert_eq!(
            compute_presentation_layout(800, 400, 960, 480, PresentationPolicy::Strict),
            PresentationLayout {
                dst_x: 0,
                dst_y: 0,
                dst_width: 800,
                dst_height: 400,
                src_x: 80,
                src_y: 40,
            }
        );
    }

    #[test]
    fn presentation_layout_stretch_fills_container() {
        assert_eq!(
            compute_presentation_layout(1200, 800, 960, 480, PresentationPolicy::Stretch),
            PresentationLayout {
                dst_x: 0,
                dst_y: 0,
                dst_width: 1200,
                dst_height: 800,
                src_x: 0,
                src_y: 0,
            }
        );
    }

    #[test]
    fn parses_fit_width_render_size() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "terminal:\n  render-size: \"640x~\"\n",
        )
        .expect("yaml parse");
        let settings = RuntimeSettings::from_manifest(&yaml);
        assert!(settings.render_size_matches_output());
        // 16:9-ish terminal (160 cols × 50 rows) → height = 640 * 50 / 160 = 200
        assert_eq!(settings.resolved_render_size(160, 50), (640, 200));
        // 4:3-ish terminal (160 cols × 120 rows) → height = 640 * 120 / 160 = 480
        assert_eq!(settings.resolved_render_size(160, 120), (640, 480));
        // Square terminal → height = width = 640
        assert_eq!(settings.resolved_render_size(100, 100), (640, 640));
    }
}


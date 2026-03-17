use crate::scene::SceneRenderedMode;
use crate::scene::SpriteSizePreset;

pub fn resolve_renderer_mode(
    scene_mode: SceneRenderedMode,
    force_renderer_mode: Option<SceneRenderedMode>,
) -> SceneRenderedMode {
    force_renderer_mode.unwrap_or(scene_mode)
}

pub fn resolve_font_spec(
    font: Option<&str>,
    force_font_mode: Option<&str>,
    scene_mode: SceneRenderedMode,
    force_renderer_mode: Option<SceneRenderedMode>,
) -> Option<String> {
    let base = font?.trim();
    if base.is_empty() {
        return None;
    }

    let force_font_mode = force_font_mode.map(str::trim).filter(|m| !m.is_empty());
    if base.starts_with("generic") {
        let explicit = force_font_mode
            .and_then(normalize_generic_mode)
            .map(str::to_string);
        let derived = if explicit.is_none() && scene_mode == SceneRenderedMode::Cell {
            force_renderer_mode
                .map(|mode| resolve_renderer_mode(scene_mode, Some(mode)))
                .and_then(map_renderer_to_generic_mode)
                .map(str::to_string)
        } else {
            None
        };
        return Some(apply_generic_mode_override(
            base,
            explicit.or(derived).as_deref(),
        ));
    }

    Some(match force_font_mode {
        Some(mode) => apply_named_font_mode(base, normalize_named_font_mode(mode)),
        None => base.to_string(),
    })
}

pub fn resolve_text_font_spec(
    font: Option<&str>,
    force_font_mode: Option<&str>,
    size: Option<SpriteSizePreset>,
    scene_mode: SceneRenderedMode,
    force_renderer_mode: Option<SceneRenderedMode>,
) -> Option<String> {
    let sized_font = match (font.map(str::trim).filter(|f| !f.is_empty()), size) {
        (Some(base), Some(size)) if base.starts_with("generic") => {
            Some(format!("generic:{}", size.generic_mode()))
        }
        (None, Some(size)) => Some(format!("generic:{}", size.generic_mode())),
        (Some(base), _) => Some(base.to_string()),
        (None, None) => None,
    };

    resolve_font_spec(
        sized_font.as_deref(),
        force_font_mode,
        scene_mode,
        force_renderer_mode,
    )
}

fn apply_generic_mode_override(base: &str, mode: Option<&str>) -> String {
    if let Some(mode) = mode {
        return format!("generic:{mode}");
    }
    base.to_string()
}

fn apply_named_font_mode(base: &str, mode: &str) -> String {
    let mut parts = base.split(':');
    let name = parts.next().unwrap_or(base).trim();
    format!("{name}:{mode}")
}

fn map_renderer_to_generic_mode(mode: SceneRenderedMode) -> Option<&'static str> {
    match mode {
        SceneRenderedMode::Cell => None,
        SceneRenderedMode::HalfBlock => Some("half"),
        SceneRenderedMode::QuadBlock => Some("quad"),
        SceneRenderedMode::Braille => Some("braille"),
    }
}

fn normalize_generic_mode(mode: &str) -> Option<&'static str> {
    match mode.to_ascii_lowercase().as_str() {
        "1" | "tiny" => Some("1"),
        "2" | "standard" => Some("2"),
        "3" | "large" => Some("3"),
        "half" | "half-block" | "halfblock" => Some("half"),
        "quad" | "quadrant" | "quadblock" => Some("quad"),
        "braille" | "br" => Some("braille"),
        _ => None,
    }
}

fn normalize_named_font_mode(mode: &str) -> &str {
    match mode.to_ascii_lowercase().as_str() {
        "cell" | "raster" => "raster",
        "terminal-pixels" | "terminal_pixels" | "terminalpixels" => "terminal-pixels",
        "ascii" => "ascii",
        _ => mode,
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_font_spec, resolve_renderer_mode, resolve_text_font_spec};
    use crate::scene::{SceneRenderedMode, SpriteSizePreset};

    #[test]
    fn sprite_force_mode_overrides_scene_mode() {
        let mode =
            resolve_renderer_mode(SceneRenderedMode::Cell, Some(SceneRenderedMode::HalfBlock));
        assert_eq!(mode, SceneRenderedMode::HalfBlock);
    }

    #[test]
    fn derives_generic_mode_from_forced_renderer_when_scene_is_cell() {
        let resolved = resolve_font_spec(
            Some("generic"),
            None,
            SceneRenderedMode::Cell,
            Some(SceneRenderedMode::QuadBlock),
        )
        .expect("font should resolve");
        assert_eq!(resolved, "generic:quad");
    }

    #[test]
    fn explicit_force_font_mode_has_priority() {
        let resolved = resolve_font_spec(
            Some("generic:2"),
            Some("braille"),
            SceneRenderedMode::Cell,
            Some(SceneRenderedMode::HalfBlock),
        )
        .expect("font should resolve");
        assert_eq!(resolved, "generic:braille");
    }

    #[test]
    fn applies_named_font_mode_override() {
        let resolved = resolve_font_spec(
            Some("Abril Fatface"),
            Some("ascii"),
            SceneRenderedMode::Cell,
            None,
        )
        .expect("font should resolve");
        assert_eq!(resolved, "Abril Fatface:ascii");
    }

    #[test]
    fn size_preset_creates_generic_text_font_when_font_missing() {
        let resolved = resolve_text_font_spec(
            None,
            None,
            Some(SpriteSizePreset::Large),
            SceneRenderedMode::Cell,
            None,
        )
        .expect("font should resolve");
        assert_eq!(resolved, "generic:3");
    }

    #[test]
    fn size_preset_overrides_generic_font_mode() {
        let resolved = resolve_text_font_spec(
            Some("generic:1"),
            None,
            Some(SpriteSizePreset::Medium),
            SceneRenderedMode::Cell,
            None,
        )
        .expect("font should resolve");
        assert_eq!(resolved, "generic:2");
    }

    #[test]
    fn size_preset_does_not_override_named_font() {
        let resolved = resolve_text_font_spec(
            Some("Abril Fatface"),
            None,
            Some(SpriteSizePreset::Small),
            SceneRenderedMode::Cell,
            None,
        )
        .expect("font should resolve");
        assert_eq!(resolved, "Abril Fatface");
    }
}

//! Font-spec resolution helpers used by the compositor.

use engine_core::scene::SpriteSizePreset;

/// Built-in engine fallback used when `font: "default"` is authored but the
/// mod does not provide a `default_font`.
pub const ENGINE_DEFAULT_FONT_SPEC: &str = "generic:2";
/// When `is_pixel_backend` is true (SDL2), named fonts without an explicit mode suffix
/// default to `raster` (shade-char bitmaps look best once SDL blends them).
pub fn resolve_font_spec(
    font: Option<&str>,
    force_font_mode: Option<&str>,
    is_pixel_backend: bool,
    default_font: Option<&str>,
) -> Option<String> {
    let authored = font?.trim();
    let base = if authored.eq_ignore_ascii_case("default") {
        default_font
            .map(str::trim)
            .filter(|font_name| !font_name.is_empty())
            .unwrap_or(ENGINE_DEFAULT_FONT_SPEC)
    } else {
        authored
    };
    if base.is_empty() {
        return None;
    }

    let force_font_mode = force_font_mode.map(str::trim).filter(|m| !m.is_empty());
    if base.starts_with("generic") {
        let explicit = force_font_mode
            .and_then(normalize_generic_mode)
            .map(str::to_string);
        return Some(apply_generic_mode_override(base, explicit.as_deref()));
    }

    Some(match force_font_mode {
        Some(mode) => apply_named_font_mode(base, normalize_named_font_mode(mode)),
        None => {
            // Named font without an explicit mode: default to `raster` on pixel backends
            // so shade-char glyphs render with proper alpha blending.
            if is_pixel_backend && !base.contains(':') {
                apply_named_font_mode(base, "raster")
            } else {
                base.to_string()
            }
        }
    })
}

/// Resolves the font spec for a text sprite, deriving a generic mode from `size` when appropriate.
pub fn resolve_text_font_spec(
    font: Option<&str>,
    force_font_mode: Option<&str>,
    size: Option<SpriteSizePreset>,
    is_pixel_backend: bool,
    default_font: Option<&str>,
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
        is_pixel_backend,
        default_font,
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

fn normalize_generic_mode(mode: &str) -> Option<&'static str> {
    let m = mode.to_ascii_lowercase();
    match m.as_str() {
        "1" | "tiny" => Some("1"),
        "2" | "standard" => Some("2"),
        "3" | "large" => Some("3"),
        _ => None,
    }
}

fn normalize_named_font_mode(mode: &str) -> &str {
    match mode.to_ascii_lowercase().as_str() {
        "raster" => "raster",
        "ascii" => "ascii",
        _ => mode,
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_font_spec, resolve_text_font_spec, ENGINE_DEFAULT_FONT_SPEC};
    use engine_core::scene::SpriteSizePreset;

    #[test]
    fn applies_named_font_mode_override() {
        let resolved = resolve_font_spec(
            Some("Abril Fatface"),
            Some("ascii"),
            false,
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
            false,
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
            false,
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
            false,
            None,
        )
        .expect("font should resolve");
        assert_eq!(resolved, "Abril Fatface");
    }

    #[test]
    fn pixel_backend_adds_raster_mode_to_bare_named_font() {
        let resolved = resolve_font_spec(
            Some("Abril Fatface"),
            None,
            true,
            None,
        )
        .expect("font should resolve");
        assert_eq!(resolved, "Abril Fatface:raster");
    }

    #[test]
    fn pixel_backend_does_not_override_explicit_mode() {
        let resolved = resolve_font_spec(
            Some("Abril Fatface:ascii"),
            None,
            true,
            None,
        )
        .expect("font should resolve");
        assert_eq!(resolved, "Abril Fatface:ascii");
    }

    #[test]
    fn default_font_alias_uses_mod_default_when_available() {
        let resolved = resolve_font_spec(
            Some("default"),
            None,
            false,
            Some("Abril Fatface"),
        )
        .expect("font should resolve");
        assert_eq!(resolved, "Abril Fatface");
    }

    #[test]
    fn default_font_alias_uses_engine_fallback_when_mod_default_missing() {
        let resolved = resolve_font_spec(
            Some("default"),
            None,
            false,
            None,
        )
        .expect("font should resolve");
        assert_eq!(resolved, ENGINE_DEFAULT_FONT_SPEC);
    }
}

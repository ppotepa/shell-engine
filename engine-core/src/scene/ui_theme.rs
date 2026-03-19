//! UI theme registry shared between authoring sugar and runtime systems.

/// Frame style used by semantic window decorations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowFrameStyle {
    Unicode,
    Ascii,
}

/// Default style set for `type: window` sugar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowThemeStyle {
    pub border_fg: &'static str,
    pub title_fg: &'static str,
    pub body_fg: &'static str,
    pub footer_fg: &'static str,
    pub frame_style: WindowFrameStyle,
}

/// Default style set for `type: scroll-list` sugar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollListThemeStyle {
    pub selected_fg: &'static str,
    pub alt_a_fg: &'static str,
    pub alt_b_fg: &'static str,
}

/// Resolved UI theme style bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiThemeStyle {
    pub id: &'static str,
    pub window: WindowThemeStyle,
    pub scroll_list: ScrollListThemeStyle,
}

/// Normalizes authored theme id into canonical lookup key.
pub fn normalize_theme_key(theme_id: Option<&str>) -> Option<String> {
    let trimmed = theme_id?.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_ascii_lowercase().replace('_', "-"))
}

/// Resolves a scene UI theme id into a concrete style preset.
pub fn resolve_ui_theme(theme_id: Option<&str>) -> Option<UiThemeStyle> {
    let key = normalize_theme_key(theme_id)?;
    match key.as_str() {
        "terminal" | "terminal-shell" | "shell" => Some(UiThemeStyle {
            id: "terminal",
            window: WindowThemeStyle {
                border_fg: "gray",
                title_fg: "white",
                body_fg: "silver",
                footer_fg: "gray",
                frame_style: WindowFrameStyle::Ascii,
            },
            scroll_list: ScrollListThemeStyle {
                selected_fg: "white",
                alt_a_fg: "silver",
                alt_b_fg: "gray",
            },
        }),
        "win98" | "windows98" | "windows-98" => Some(UiThemeStyle {
            id: "win98",
            window: WindowThemeStyle {
                border_fg: "silver",
                title_fg: "white",
                body_fg: "white",
                footer_fg: "silver",
                frame_style: WindowFrameStyle::Ascii,
            },
            scroll_list: ScrollListThemeStyle {
                selected_fg: "yellow",
                alt_a_fg: "white",
                alt_b_fg: "silver",
            },
        }),
        "xp" | "windowsxp" | "windows-xp" => Some(UiThemeStyle {
            id: "xp",
            window: WindowThemeStyle {
                border_fg: "silver",
                title_fg: "cyan",
                body_fg: "white",
                footer_fg: "gray",
                frame_style: WindowFrameStyle::Unicode,
            },
            scroll_list: ScrollListThemeStyle {
                selected_fg: "cyan",
                alt_a_fg: "white",
                alt_b_fg: "silver",
            },
        }),
        "jrpg" | "jrpg-dialog" | "jrpg-dialogue" => Some(UiThemeStyle {
            id: "jrpg",
            window: WindowThemeStyle {
                border_fg: "white",
                title_fg: "yellow",
                body_fg: "white",
                footer_fg: "silver",
                frame_style: WindowFrameStyle::Unicode,
            },
            scroll_list: ScrollListThemeStyle {
                selected_fg: "yellow",
                alt_a_fg: "white",
                alt_b_fg: "gray",
            },
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_theme_key, resolve_ui_theme, WindowFrameStyle};

    #[test]
    fn normalizes_theme_key() {
        assert_eq!(
            normalize_theme_key(Some(" Win_98 ")).as_deref(),
            Some("win-98")
        );
        assert_eq!(normalize_theme_key(Some("   ")), None);
        assert_eq!(normalize_theme_key(None), None);
    }

    #[test]
    fn resolves_terminal_theme_aliases() {
        let terminal = resolve_ui_theme(Some("terminal")).expect("terminal theme");
        let shell = resolve_ui_theme(Some("shell")).expect("shell alias");
        let terminal_shell = resolve_ui_theme(Some("terminal-shell")).expect("terminal-shell");
        assert_eq!(terminal.id, "terminal");
        assert_eq!(terminal, shell);
        assert_eq!(terminal, terminal_shell);
        assert_eq!(terminal.window.frame_style, WindowFrameStyle::Ascii);
    }

    #[test]
    fn resolves_windows_aliases() {
        let win98 = resolve_ui_theme(Some("windows_98")).expect("win98 alias");
        assert_eq!(win98.id, "win98");
        let xp = resolve_ui_theme(Some("windows-xp")).expect("xp alias");
        assert_eq!(xp.id, "xp");
        assert_eq!(xp.window.frame_style, WindowFrameStyle::Unicode);
    }

    #[test]
    fn returns_none_for_unknown_theme() {
        assert!(resolve_ui_theme(Some("neo-glass")).is_none());
    }
}

#![allow(dead_code)]

//! Icon glyph helpers with automatic fallback when Nerd Fonts are unavailable.
//!
//! The default output is ASCII-safe so icons still render on basic terminals.
//! Set `SHELL_ENGINE_ICON_THEME` to `emoji` or `nerd` to opt into richer glyphs.

use std::sync::OnceLock;

use crate::state::SidebarItem;

const ICON_THEME_VAR: &str = "SHELL_ENGINE_ICON_THEME";

/// Active icon theme determining which glyph set is used at runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconTheme {
    Ascii,
    Emoji,
    Nerd,
}

fn theme() -> IconTheme {
    static THEME: OnceLock<IconTheme> = OnceLock::new();
    *THEME.get_or_init(|| {
        std::env::var(ICON_THEME_VAR)
            .ok()
            .and_then(|value| {
                let normalized = value.trim().to_ascii_lowercase();
                match normalized.as_str() {
                    "nerd" | "nerdfont" | "nf" => Some(IconTheme::Nerd),
                    "emoji" => Some(IconTheme::Emoji),
                    "ascii" | "default" => Some(IconTheme::Ascii),
                    _ => None,
                }
            })
            .unwrap_or(IconTheme::Ascii)
    })
}

/// Returns the active [`IconTheme`] resolved from the environment variable at startup.
pub fn current_theme() -> IconTheme {
    theme()
}

fn pick(nerd: &'static str, emoji: &'static str, ascii: &'static str) -> &'static str {
    match theme() {
        IconTheme::Nerd => nerd,
        IconTheme::Emoji => emoji,
        IconTheme::Ascii => ascii,
    }
}

/// Returns the sidebar icon glyph for the given [`SidebarItem`] using the active theme.
pub fn sidebar_glyph(item: SidebarItem) -> &'static str {
    match item {
        SidebarItem::Scenes => pick("\u{f008}", "\u{1f3ac}", "[1]"),
        SidebarItem::Explorer => pick("\u{f115}", "\u{1f4c1}", "[2]"),
        SidebarItem::Search => pick("\u{f002}", "\u{1f50d}", "[3]"),
        SidebarItem::Cutscene => pick("\u{f03d}", "\u{1f4f7}", "[C]"),
    }
}

/// Returns a short human-readable label for the active icon theme.
pub fn theme_hint() -> &'static str {
    match theme() {
        IconTheme::Nerd => "Nerd Font",
        IconTheme::Emoji => "Emoji",
        IconTheme::Ascii => "ASCII",
    }
}

// Generic glyphs (still useful for other widgets; gated to avoid warnings).
pub mod nerd {
    pub const FILE: &str = "\u{f15b}";
    pub const FOLDER: &str = "\u{f07b}";
    pub const FOLDER_OPEN: &str = "\u{f07c}";
    pub const RUST: &str = "\u{e7a8}";
    pub const YAML: &str = "\u{f481}";
    pub const PNG: &str = "\u{f1c5}";
    pub const JSON: &str = "\u{e60b}";
    pub const CHECK: &str = "\u{f00c}";
    pub const CROSS: &str = "\u{f00d}";
    pub const WARN: &str = "\u{f071}";
    pub const INFO: &str = "\u{f05a}";
    pub const ARROW_RIGHT: &str = "\u{f054}";
    pub const ARROW_DOWN: &str = "\u{f078}";
    pub const ARROW_UP: &str = "\u{f077}";
    pub const PLAY: &str = "\u{f04b}";
    pub const PAUSE: &str = "\u{f04c}";
    pub const STOP: &str = "\u{f04d}";
}

pub mod emoji {
    pub const FILE: &str = "\u{1f4c4}";
    pub const FOLDER: &str = "\u{1f4c1}";
    pub const FOLDER_OPEN: &str = "\u{1f4c2}";
    pub const RUST: &str = "\u{1f980}";
    pub const YAML: &str = "\u{1f4dd}";
    pub const PNG: &str = "\u{1f5bc}\u{fe0f}";
    pub const JSON: &str = "\u{1f4cb}";
    pub const CHECK: &str = "\u{2714}\u{fe0f}";
    pub const CROSS: &str = "\u{2716}\u{fe0f}";
    pub const WARN: &str = "\u{26a0}\u{fe0f}";
    pub const INFO: &str = "\u{2139}\u{fe0f}";
    pub const ARROW_RIGHT: &str = "\u{27a1}\u{fe0f}";
    pub const ARROW_DOWN: &str = "\u{2b07}\u{fe0f}";
    pub const ARROW_UP: &str = "\u{2b06}\u{fe0f}";
    pub const PLAY: &str = "\u{25b6}\u{fe0f}";
    pub const PAUSE: &str = "\u{23f8}\u{fe0f}";
    pub const STOP: &str = "\u{23f9}\u{fe0f}";
}

pub mod ascii {
    pub const FILE: &str = "[fi]";
    pub const FOLDER: &str = "[fo]";
    pub const FOLDER_OPEN: &str = "[fO]";
    pub const RUST: &str = "[rs]";
    pub const YAML: &str = "[yml]";
    pub const PNG: &str = "[png]";
    pub const JSON: &str = "[json]";
    pub const CHECK: &str = "[ok]";
    pub const CROSS: &str = "[xx]";
    pub const WARN: &str = "[!!]";
    pub const INFO: &str = "[i]";
    pub const ARROW_RIGHT: &str = "->";
    pub const ARROW_DOWN: &str = "\\/";
    pub const ARROW_UP: &str = "/\\";
    pub const PLAY: &str = "[>]";
    pub const PAUSE: &str = "[||]";
    pub const STOP: &str = "[■]";
}

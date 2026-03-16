use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::fs;

// Catppuccin Mocha inspired theme
#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    #[serde(default = "default_base")]
    pub base: [u8; 3], // #1e1e2e
    #[serde(default = "default_mantle")]
    #[allow(dead_code)]
    pub mantle: [u8; 3], // #181825 (reserved for future use)
    #[serde(default = "default_surface0")]
    pub surface0: [u8; 3], // #313244
    #[serde(default = "default_text")]
    pub text: [u8; 3], // #cdd6f4
    #[serde(default = "default_subtext0")]
    pub subtext0: [u8; 3], // #a6adc8
    #[serde(default = "default_overlay0")]
    pub overlay0: [u8; 3], // #6c7086
    #[serde(default = "default_accent")]
    pub accent: [u8; 3], // #fab387 (peach/orange)
}

fn default_base() -> [u8; 3] {
    [30, 30, 46]
} // #1e1e2e
fn default_mantle() -> [u8; 3] {
    [24, 24, 37]
} // #181825
fn default_surface0() -> [u8; 3] {
    [49, 50, 68]
} // #313244
fn default_text() -> [u8; 3] {
    [205, 214, 244]
} // #cdd6f4
fn default_subtext0() -> [u8; 3] {
    [166, 173, 200]
} // #a6adc8
fn default_overlay0() -> [u8; 3] {
    [108, 112, 134]
} // #6c7086
fn default_accent() -> [u8; 3] {
    [250, 179, 135]
} // #fab387 (peach)

impl Default for Theme {
    fn default() -> Self {
        Self {
            base: default_base(),
            mantle: default_mantle(),
            surface0: default_surface0(),
            text: default_text(),
            subtext0: default_subtext0(),
            overlay0: default_overlay0(),
            accent: default_accent(),
        }
    }
}

impl Theme {
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let theme_path = config_dir.join("sq-editor").join("theme.toml");
            if let Ok(content) = fs::read_to_string(&theme_path) {
                if let Ok(theme) = toml::from_str(&content) {
                    return theme;
                }
            }
        }
        Self::default()
    }
}

// Singleton for loaded theme
static THEME: std::sync::OnceLock<Theme> = std::sync::OnceLock::new();

pub fn init_theme() {
    THEME.get_or_init(Theme::load);
}

fn theme() -> &'static Theme {
    THEME.get_or_init(Theme::load)
}

pub fn fg_normal() -> Style {
    let t = theme();
    Style::default().fg(Color::Rgb(t.subtext0[0], t.subtext0[1], t.subtext0[2]))
}

pub fn fg_active() -> Style {
    let t = theme();
    Style::default()
        .fg(Color::Rgb(t.text[0], t.text[1], t.text[2]))
        .add_modifier(Modifier::BOLD)
}

pub fn fg_disabled() -> Style {
    let t = theme();
    Style::default().fg(Color::Rgb(t.overlay0[0], t.overlay0[1], t.overlay0[2]))
}

pub fn accent() -> Style {
    let t = theme();
    Style::default().fg(Color::Rgb(t.accent[0], t.accent[1], t.accent[2]))
}

pub fn sidebar_active_entry() -> Style {
    let t = theme();
    Style::default()
        .bg(Color::Rgb(t.accent[0], t.accent[1], t.accent[2]))
        .fg(Color::Rgb(t.base[0], t.base[1], t.base[2]))
        .add_modifier(Modifier::BOLD)
}

pub fn pane_title(is_focused: bool) -> Style {
    if is_focused {
        fg_active()
    } else {
        fg_normal()
    }
}

pub fn pane_border(is_focused: bool) -> Style {
    if is_focused {
        accent()
    } else {
        fg_normal()
    }
}

pub fn pane_background(is_focused: bool) -> Style {
    let t = theme();
    if is_focused {
        Style::default()
            .bg(Color::Rgb(t.surface0[0], t.surface0[1], t.surface0[2]))
            .fg(Color::Rgb(t.text[0], t.text[1], t.text[2]))
    } else {
        Style::default()
            .bg(Color::Rgb(t.base[0], t.base[1], t.base[2]))
            .fg(Color::Rgb(t.subtext0[0], t.subtext0[1], t.subtext0[2]))
    }
}

pub fn preview_background() -> Style {
    Style::default().bg(Color::Rgb(0, 0, 0))
}

pub fn badge_transition() -> Style {
    // Blue — scene-level / transitions
    Style::default()
        .fg(Color::Rgb(137, 180, 250))
        .add_modifier(Modifier::BOLD)
}

pub fn badge_effect() -> Style {
    // Purple — object-level effects
    Style::default()
        .fg(Color::Rgb(203, 166, 247))
        .add_modifier(Modifier::BOLD)
}

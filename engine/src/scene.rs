use serde::Deserialize;

/// A terminal colour name as declared in YAML (e.g. "black", "white", "gray").
/// Maps to `ratatui::style::Color` during rendering.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TermColour {
    Black,
    White,
    Gray,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
}

impl From<&TermColour> for ratatui::style::Color {
    fn from(c: &TermColour) -> Self {
        match c {
            TermColour::Black   => ratatui::style::Color::Black,
            TermColour::White   => ratatui::style::Color::White,
            TermColour::Gray    => ratatui::style::Color::Gray,
            TermColour::Red     => ratatui::style::Color::Red,
            TermColour::Green   => ratatui::style::Color::Green,
            TermColour::Blue    => ratatui::style::Color::Blue,
            TermColour::Yellow  => ratatui::style::Color::Yellow,
            TermColour::Cyan    => ratatui::style::Color::Cyan,
            TermColour::Magenta => ratatui::style::Color::Magenta,
        }
    }
}

/// A single layer rendered on top of the scene buffer.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Layer {
    Text {
        content: String,
        #[serde(default)]
        x: u16,
        #[serde(default)]
        y: u16,
        fg_colour: Option<TermColour>,
        bg_colour: Option<TermColour>,
    },
}

/// A parsed scene loaded from a `.yml` file.
#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub cutscene: bool,
    #[serde(default)]
    pub skippable: bool,
    /// Fills the entire terminal buffer with this colour on scene load.
    pub bg_colour: Option<TermColour>,
    #[serde(default)]
    pub layers: Vec<Layer>,
    pub next: Option<String>,
}

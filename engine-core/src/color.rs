//! Engine-owned color abstraction — platform-agnostic RGB + named colors.
//!
//! Replaces crossterm::style::Color with an engine-internal type that can be:
//! - Rendered as terminal ANSI (via engine-render-terminal)
//! - Rendered as SDL2 pixels (via engine-render-sdl2)
//! - Captured to binary frames (via engine-capture)

/// Platform-agnostic color representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    // True RGB color
    Rgb { r: u8, g: u8, b: u8 },

    // Reset to default terminal color
    Reset,

    // Named colors (map to RGB internally)
    Black,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    White,
    Grey,
    DarkGrey,
    DarkRed,
    DarkGreen,
    DarkBlue,
    DarkYellow,
    DarkCyan,
    DarkMagenta,
}

impl Color {
    /// Construct an RGB color.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Rgb { r, g, b }
    }

    /// Black: RGB(0, 0, 0)
    pub const BLACK: Color = Color::Rgb { r: 0, g: 0, b: 0 };

    /// White: RGB(255, 255, 255)
    pub const WHITE: Color = Color::Rgb {
        r: 255,
        g: 255,
        b: 255,
    };

    /// Resolve named color or RGB to (r, g, b) tuple.
    pub fn to_rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Rgb { r, g, b } => (r, g, b),
            Color::Black => (0, 0, 0),
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Yellow => (255, 255, 0),
            Color::Cyan => (0, 255, 255),
            Color::Magenta => (255, 0, 255),
            Color::White => (255, 255, 255),
            Color::Grey => (128, 128, 128),
            Color::DarkGrey => (64, 64, 64),
            Color::DarkRed => (128, 0, 0),
            Color::DarkGreen => (0, 128, 0),
            Color::DarkBlue => (0, 0, 128),
            Color::DarkYellow => (128, 128, 0),
            Color::DarkCyan => (0, 128, 128),
            Color::DarkMagenta => (128, 0, 128),
            Color::Reset => (0, 0, 0), // Default to black for reset
        }
    }

    /// Check if this is the reset color.
    pub fn is_reset(self) -> bool {
        matches!(self, Color::Reset)
    }
}

impl Default for Color {
    /// Default: black RGB(0, 0, 0)
    fn default() -> Self {
        Color::BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_color_to_rgb() {
        assert_eq!(Color::rgb(100, 150, 200).to_rgb(), (100, 150, 200));
    }

    #[test]
    fn named_colors_to_rgb() {
        assert_eq!(Color::Black.to_rgb(), (0, 0, 0));
        assert_eq!(Color::White.to_rgb(), (255, 255, 255));
        assert_eq!(Color::Red.to_rgb(), (255, 0, 0));
    }

    #[test]
    fn default_is_black() {
        assert_eq!(Color::default(), Color::BLACK);
    }

    #[test]
    fn color_copy() {
        let c = Color::White;
        let c2 = c;
        assert_eq!(c, c2);
    }

    #[test]
    fn is_reset() {
        assert!(Color::Reset.is_reset());
        assert!(!Color::White.is_reset());
    }
}

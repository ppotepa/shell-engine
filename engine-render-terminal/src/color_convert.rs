//! Color conversion between engine-core and crossterm backends.

use crossterm::style::Color as CrosstermColor;
use engine_core::color::Color;

/// Convert engine-internal Color to crossterm Color.
pub fn to_crossterm(c: Color) -> CrosstermColor {
    match c {
        Color::Reset => CrosstermColor::Reset,
        Color::Black => CrosstermColor::Black,
        Color::DarkGrey => CrosstermColor::DarkGrey,
        Color::Red => CrosstermColor::Red,
        Color::DarkRed => CrosstermColor::DarkRed,
        Color::Green => CrosstermColor::Green,
        Color::DarkGreen => CrosstermColor::DarkGreen,
        Color::Yellow => CrosstermColor::Yellow,
        Color::DarkYellow => CrosstermColor::DarkYellow,
        Color::Blue => CrosstermColor::Blue,
        Color::DarkBlue => CrosstermColor::DarkBlue,
        Color::Magenta => CrosstermColor::Magenta,
        Color::DarkMagenta => CrosstermColor::DarkMagenta,
        Color::Cyan => CrosstermColor::Cyan,
        Color::DarkCyan => CrosstermColor::DarkCyan,
        Color::White => CrosstermColor::White,
        Color::Grey => CrosstermColor::Grey,
        Color::Rgb { r, g, b } => CrosstermColor::Rgb { r, g, b },
    }
}

/// Convert crossterm Color to engine-internal Color.
pub fn from_crossterm(c: CrosstermColor) -> Color {
    match c {
        CrosstermColor::Reset => Color::Reset,
        CrosstermColor::Black => Color::Black,
        CrosstermColor::DarkGrey => Color::DarkGrey,
        CrosstermColor::Red => Color::Red,
        CrosstermColor::DarkRed => Color::DarkRed,
        CrosstermColor::Green => Color::Green,
        CrosstermColor::DarkGreen => Color::DarkGreen,
        CrosstermColor::Yellow => Color::Yellow,
        CrosstermColor::DarkYellow => Color::DarkYellow,
        CrosstermColor::Blue => Color::Blue,
        CrosstermColor::DarkBlue => Color::DarkBlue,
        CrosstermColor::Magenta => Color::Magenta,
        CrosstermColor::DarkMagenta => Color::DarkMagenta,
        CrosstermColor::Cyan => Color::Cyan,
        CrosstermColor::DarkCyan => Color::DarkCyan,
        CrosstermColor::White => Color::White,
        CrosstermColor::Grey => Color::Grey,
        CrosstermColor::Rgb { r, g, b } => Color::Rgb { r, g, b },
        CrosstermColor::AnsiValue(_) => Color::Black, // Fallback: AnsiValue not in our enum
    }
}

/// Convert a slice of diff tuples from engine Color to crossterm Color.
pub fn diffs_to_crossterm(
    diffs: &[(u16, u16, char, Color, Color)],
) -> Vec<(u16, u16, char, CrosstermColor, CrosstermColor)> {
    diffs
        .iter()
        .map(|(x, y, ch, fg, bg)| (*x, *y, *ch, to_crossterm(*fg), to_crossterm(*bg)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_named_colors() {
        let colors = vec![
            Color::Black,
            Color::Red,
            Color::Green,
            Color::Blue,
            Color::White,
            Color::Reset,
        ];
        for c in colors {
            assert_eq!(from_crossterm(to_crossterm(c)), c);
        }
    }

    #[test]
    fn round_trip_rgb() {
        let c = Color::Rgb { r: 100, g: 150, b: 200 };
        assert_eq!(from_crossterm(to_crossterm(c)), c);
    }

    #[test]
    fn converts_diffs() {
        let diffs = vec![
            (0, 0, 'A', Color::Red, Color::Black),
            (1, 1, 'B', Color::Green, Color::Blue),
        ];
        let converted = diffs_to_crossterm(&diffs);
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].0, 0);
        assert_eq!(converted[0].1, 0);
        assert_eq!(converted[0].2, 'A');
    }
}

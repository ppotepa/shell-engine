//! Color conversion between engine-core and crossterm backends.
//! 
//! OPT-42: Color conversion caching - cache engine Color → crossterm Color conversions
//! to avoid per-frame RGB↔256 conversions.

use crossterm::style::Color as CrosstermColor;
use engine_core::color::Color;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static COLOR_CACHE: RefCell<HashMap<u32, CrosstermColor>> = RefCell::new(HashMap::new());
}

/// Color hashable as u32 for caching: Named colors 0-16, RGB packed as (r<<16|g<<8|b).
#[inline]
fn color_to_cache_key(c: Color) -> u32 {
    match c {
        Color::Reset => 0,
        Color::Black => 1,
        Color::DarkGrey => 2,
        Color::Red => 3,
        Color::DarkRed => 4,
        Color::Green => 5,
        Color::DarkGreen => 6,
        Color::Yellow => 7,
        Color::DarkYellow => 8,
        Color::Blue => 9,
        Color::DarkBlue => 10,
        Color::Magenta => 11,
        Color::DarkMagenta => 12,
        Color::Cyan => 13,
        Color::DarkCyan => 14,
        Color::White => 15,
        Color::Grey => 16,
        Color::Rgb { r, g, b } => (100 << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
    }
}

/// Convert engine-internal Color to crossterm Color with caching.
/// OPT-42: Cached conversions avoid per-frame RGB↔256 conversions on color-heavy scenes.
pub fn to_crossterm(c: Color) -> CrosstermColor {
    let key = color_to_cache_key(c);
    COLOR_CACHE.with(|cache| {
        let mut cache_ref = cache.borrow_mut();
        if let Some(&cached) = cache_ref.get(&key) {
            return cached;
        }
        let converted = match c {
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
        };
        cache_ref.insert(key, converted);
        converted
    })
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
        let c = Color::Rgb {
            r: 100,
            g: 150,
            b: 200,
        };
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

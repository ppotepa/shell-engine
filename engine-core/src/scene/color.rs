use crossterm::style::Color;
use serde::{Deserialize, Deserializer};

/// A colour value in YAML — accepts named colours or `#rrggbb` hex strings.
///
/// Examples: `black`, `white`, `silver`, `#C0C0C0`, `#ff8800`
#[derive(Debug, Clone, PartialEq)]
pub enum TermColour {
    Black,
    White,
    Gray,
    Silver,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    Rgb(u8, u8, u8),
}

impl<'de> Deserialize<'de> for TermColour {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s = String::deserialize(de)?;
        match s.to_lowercase().as_str() {
            "black" => return Ok(TermColour::Black),
            "white" => return Ok(TermColour::White),
            "gray" | "grey" => return Ok(TermColour::Gray),
            "silver" => return Ok(TermColour::Silver),
            "red" => return Ok(TermColour::Red),
            "green" => return Ok(TermColour::Green),
            "blue" => return Ok(TermColour::Blue),
            "yellow" => return Ok(TermColour::Yellow),
            "cyan" => return Ok(TermColour::Cyan),
            "magenta" => return Ok(TermColour::Magenta),
            _ => {}
        }
        if let Some(hex) = s.strip_prefix('#') {
            if hex.len() == 6 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                ) {
                    return Ok(TermColour::Rgb(r, g, b));
                }
            }
        }
        Err(serde::de::Error::custom(format!(
            "unknown colour: '{}'. Use a name (black/white/silver/gray/red/...) or #rrggbb hex",
            s
        )))
    }
}

impl From<&TermColour> for Color {
    fn from(c: &TermColour) -> Self {
        match c {
            TermColour::Black => Color::Rgb { r: 0, g: 0, b: 0 },
            TermColour::White => Color::Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
            TermColour::Gray => Color::Rgb {
                r: 128,
                g: 128,
                b: 128,
            },
            TermColour::Silver => Color::Rgb {
                r: 192,
                g: 192,
                b: 200,
            },
            TermColour::Red => Color::Red,
            TermColour::Green => Color::Green,
            TermColour::Blue => Color::Blue,
            TermColour::Yellow => Color::Yellow,
            TermColour::Cyan => Color::Cyan,
            TermColour::Magenta => Color::Magenta,
            TermColour::Rgb(r, g, b) => Color::Rgb {
                r: *r,
                g: *g,
                b: *b,
            },
        }
    }
}

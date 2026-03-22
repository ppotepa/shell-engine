use crossterm::style::Color;
use serde::{Deserialize, Deserializer};

/// A colour value in YAML.
///
/// Accepted formats:
/// - Named: `black`, `white`, `silver`, `gray`, `red`, `green`, `blue`,
///   `yellow`, `cyan`, `magenta`
/// - Hex: `#rrggbb` (e.g. `#C0C0C0`, `#ff8800`)
/// - CSS-style: `rgb(r,g,b)` or `rgba(r,g,b,a)` — alpha is accepted but
///   ignored (terminal has no alpha channel)
/// - Bare tuple: `r,g,b` or `r,g,b,a`
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

/// Parse a colour string. Shared between YAML deserialization and markup tags.
pub fn parse_colour_str(s: &str) -> Option<TermColour> {
    let trimmed = s.trim();
    let lower = trimmed.to_lowercase();

    // Named colours
    match lower.as_str() {
        "black" => return Some(TermColour::Black),
        "white" => return Some(TermColour::White),
        "gray" | "grey" => return Some(TermColour::Gray),
        "silver" => return Some(TermColour::Silver),
        "red" => return Some(TermColour::Red),
        "green" => return Some(TermColour::Green),
        "blue" => return Some(TermColour::Blue),
        "yellow" => return Some(TermColour::Yellow),
        "cyan" => return Some(TermColour::Cyan),
        "magenta" => return Some(TermColour::Magenta),
        _ => {}
    }

    // #rrggbb hex
    if let Some(hex) = trimmed.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Some(TermColour::Rgb(r, g, b));
            }
        }
    }

    // rgb(r,g,b) or rgba(r,g,b,a) — extract the inner part
    let inner = lower
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
        .or_else(|| lower.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')));

    if let Some(inner) = inner {
        return parse_csv_rgb(inner);
    }

    // Bare tuple: r,g,b or r,g,b,a
    if trimmed.contains(',') {
        return parse_csv_rgb(trimmed);
    }

    None
}

/// Parse comma-separated `r,g,b` or `r,g,b,a` (alpha accepted but ignored).
fn parse_csv_rgb(csv: &str) -> Option<TermColour> {
    let parts: Vec<&str> = csv.split(',').map(|p| p.trim()).collect();
    if parts.len() < 3 || parts.len() > 4 {
        return None;
    }
    let r = parts[0].parse::<u8>().ok()?;
    let g = parts[1].parse::<u8>().ok()?;
    let b = parts[2].parse::<u8>().ok()?;
    // parts[3] (alpha) is intentionally ignored — terminals have no alpha
    Some(TermColour::Rgb(r, g, b))
}

impl<'de> Deserialize<'de> for TermColour {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s = String::deserialize(de)?;
        parse_colour_str(&s).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "unknown colour: '{s}'. Use a name, #rrggbb hex, rgb(r,g,b), or r,g,b"
            ))
        })
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

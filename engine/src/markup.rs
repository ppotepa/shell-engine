use crate::scene::color::TermColour;

/// A run of text with an optional colour override.
/// `colour: None` means use the sprite's base fg_colour.
#[derive(Debug, Clone)]
pub struct Span {
    pub text: String,
    pub colour: Option<TermColour>,
}

/// Parse a string with `[colour]text[/]` markup into a list of Spans.
/// Supports:
///   [named]   — named colour (black/white/gray/silver/red/green/blue/yellow/cyan/magenta)
///   [#rrggbb] — hex colour
///   [/] or [/name] — close current colour span (revert to base fg)
///
/// Unclosed tags: the rest of the string uses the opened colour.
/// `[text without closing ']'` — the `[` is treated as a literal character.
/// Spans with empty text are omitted.
pub fn parse_spans(input: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut current_colour: Option<TermColour> = None;
    let mut current_text = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            let mut tag = String::new();
            let mut closed = false;
            for tc in chars.by_ref() {
                if tc == ']' {
                    closed = true;
                    break;
                }
                tag.push(tc);
            }
            if !closed {
                // Not a valid tag — treat '[' as literal text
                current_text.push('[');
                current_text.push_str(&tag);
                continue;
            }
            // Flush current text as a span
            if !current_text.is_empty() {
                spans.push(Span {
                    text: current_text.clone(),
                    colour: current_colour.clone(),
                });
                current_text.clear();
            }
            if tag.starts_with('/') {
                // Closing tag — revert to base colour
                current_colour = None;
            } else {
                // Opening tag — try to parse as colour
                current_colour = parse_colour_tag(&tag);
            }
        } else {
            current_text.push(ch);
        }
    }
    if !current_text.is_empty() {
        spans.push(Span {
            text: current_text,
            colour: current_colour,
        });
    }
    spans
}

/// Strip all `[tag]` markup from a string, returning only the visible text.
/// An unclosed `[` (no matching `]`) is treated as a literal character.
pub fn strip_markup(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            let mut tag = String::new();
            let mut closed = false;
            for tc in chars.by_ref() {
                if tc == ']' {
                    closed = true;
                    break;
                }
                tag.push(tc);
            }
            if !closed {
                result.push('[');
                result.push_str(&tag);
            }
            // Closed tag — silently skip
        } else {
            result.push(ch);
        }
    }
    result
}

fn parse_colour_tag(tag: &str) -> Option<TermColour> {
    match tag.to_lowercase().as_str() {
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
    if let Some(hex) = tag.strip_prefix('#') {
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
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spans_plain_text() {
        let spans = parse_spans("hello");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "hello");
        assert!(spans[0].colour.is_none());
    }

    #[test]
    fn parse_spans_named_colour() {
        let spans = parse_spans("[red]PRESS[/] ANY");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].text, "PRESS");
        assert!(matches!(spans[0].colour, Some(TermColour::Red)));
        assert_eq!(spans[1].text, " ANY");
        assert!(spans[1].colour.is_none());
    }

    #[test]
    fn parse_spans_hex_colour() {
        let spans = parse_spans("[#ff8800]text[/]");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "text");
        assert!(matches!(
            spans[0].colour,
            Some(TermColour::Rgb(0xff, 0x88, 0x00))
        ));
    }

    #[test]
    fn parse_spans_unclosed_bracket_is_literal() {
        let spans = parse_spans("[unclosed");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "[unclosed");
    }

    #[test]
    fn strip_markup_removes_tags() {
        assert_eq!(
            strip_markup("[red]PRESS[/] ANY [red]KEY[/]"),
            "PRESS ANY KEY"
        );
    }

    #[test]
    fn strip_markup_plain_passthrough() {
        assert_eq!(strip_markup("PRESS ANY KEY"), "PRESS ANY KEY");
    }
}

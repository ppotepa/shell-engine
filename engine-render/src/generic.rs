use engine_core::scene::sprite::TextTransform;

/// Engine-wide emergency fallback font spec.
///
/// Uses the standard 5x7 generic mode for readability when named font assets
/// cannot be resolved.
pub const ENGINE_FALLBACK_FONT_SPEC: &str = "generic:2";

/// Compact tiny-font metrics used by `generic:1`.
///
/// Uses a one-cell gap to keep tiny glyphs legible on SDL/terminal surfaces.
pub const GENERIC_TINY_GLYPH_WIDTH: u16 = 4;
pub const GENERIC_TINY_GLYPH_HEIGHT: u16 = 5;
pub const GENERIC_TINY_GLYPH_GAP: u16 = 1;

fn apply_transform(c: char, transform: &TextTransform) -> char {
    match transform {
        TextTransform::Uppercase => c.to_ascii_uppercase(),
        TextTransform::None => c,
    }
}

/// 5×7 bitmapped glyphs for the built-in "generic" pixel font.
/// Each entry: character → array of 7 rows, each row is a 5-bit mask (bit 4 = leftmost).
pub fn generic_glyph_rows(ch: char) -> Option<[u8; 7]> {
    match ch {
        'A' => Some([
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ]),
        'B' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ]),
        'C' => Some([
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ]),
        'D' => Some([
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ]),
        'E' => Some([
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ]),
        'F' => Some([
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ]),
        'G' => Some([
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ]),
        'H' => Some([
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ]),
        'I' => Some([
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ]),
        'J' => Some([
            0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110,
        ]),
        'K' => Some([
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ]),
        'L' => Some([
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ]),
        'M' => Some([
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ]),
        'N' => Some([
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ]),
        'O' => Some([
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ]),
        'P' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ]),
        'Q' => Some([
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ]),
        'R' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ]),
        'S' => Some([
            0b01110, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ]),
        'T' => Some([
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ]),
        'U' => Some([
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ]),
        'V' => Some([
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ]),
        'W' => Some([
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ]),
        'X' => Some([
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ]),
        'Y' => Some([
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ]),
        'Z' => Some([
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ]),
        '0' => Some([
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ]),
        '1' => Some([
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ]),
        '2' => Some([
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ]),
        '3' => Some([
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ]),
        '4' => Some([
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ]),
        '5' => Some([
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ]),
        '6' => Some([
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ]),
        '7' => Some([
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ]),
        '8' => Some([
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ]),
        '9' => Some([
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ]),
        '.' => Some([
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100,
        ]),
        ',' => Some([
            0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
        ]),
        ':' => Some([
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ]),
        ';' => Some([
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b01000,
        ]),
        '-' => Some([
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ]),
        '_' => Some([
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ]),
        '/' => Some([
            0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000,
        ]),
        '\\' => Some([
            0b10000, 0b01000, 0b00100, 0b00010, 0b00001, 0b00000, 0b00000,
        ]),
        '>' => Some([
            0b10000, 0b01000, 0b00100, 0b00010, 0b00100, 0b01000, 0b10000,
        ]),
        '<' => Some([
            0b00001, 0b00010, 0b00100, 0b01000, 0b00100, 0b00010, 0b00001,
        ]),
        '?' => Some([
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ]),
        '!' => Some([
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ]),
        '#' => Some([
            0b01010, 0b11111, 0b01010, 0b01010, 0b11111, 0b01010, 0b00000,
        ]),
        '$' => Some([
            0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100,
        ]),
        '(' => Some([
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ]),
        ')' => Some([
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ]),
        '[' => Some([
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ]),
        ']' => Some([
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ]),
        '\'' => Some([
            0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ]),
        '"' => Some([
            0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ]),
        '=' => Some([
            0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ]),
        '+' => Some([
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ]),
        '*' => Some([
            0b00000, 0b01010, 0b00100, 0b11111, 0b00100, 0b01010, 0b00000,
        ]),
        '@' => Some([
            0b01110, 0b10001, 0b10111, 0b10101, 0b10111, 0b10000, 0b01110,
        ]),
        '|' => Some([
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ]),
        ' ' => Some([
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ]),
        // ── Symbols ──────────────────────────────────────────────────────────
        // Classic 5×7 retro heart: top two bumps, filled body, tapering point.
        '♥' => Some([
            0b00000, 0b01010, 0b11111, 0b11111, 0b01110, 0b00100, 0b00000,
        ]),
        // ── Lowercase a–z (distinct designs, 5×7 px) ────────────────────────
        'a' => Some([
            0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b10011, 0b01101,
        ]),
        'b' => Some([
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b11001, 0b10110,
        ]),
        'c' => Some([
            0b00000, 0b01110, 0b10001, 0b10000, 0b10000, 0b10001, 0b01110,
        ]),
        'd' => Some([
            0b00001, 0b00001, 0b01101, 0b10011, 0b10001, 0b10011, 0b01101,
        ]),
        'e' => Some([
            0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b10001, 0b01110,
        ]),
        'f' => Some([
            0b00110, 0b01001, 0b01000, 0b11110, 0b01000, 0b01000, 0b01000,
        ]),
        'g' => Some([
            0b00000, 0b00000, 0b01110, 0b10001, 0b01111, 0b00001, 0b01110,
        ]),
        'h' => Some([
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001,
        ]),
        'i' => Some([
            0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110,
        ]),
        'j' => Some([
            0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100,
        ]),
        'k' => Some([
            0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010,
        ]),
        'l' => Some([
            0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ]),
        'm' => Some([
            0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10001, 0b10001,
        ]),
        'n' => Some([
            0b00000, 0b00000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001,
        ]),
        'o' => Some([
            0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110,
        ]),
        'p' => Some([
            0b00000, 0b00000, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000,
        ]),
        'q' => Some([
            0b00000, 0b00000, 0b01111, 0b10001, 0b01111, 0b00001, 0b00001,
        ]),
        'r' => Some([
            0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000,
        ]),
        's' => Some([
            0b00000, 0b00000, 0b01110, 0b10000, 0b01110, 0b00001, 0b11110,
        ]),
        't' => Some([
            0b00100, 0b00100, 0b01110, 0b00100, 0b00100, 0b00101, 0b00010,
        ]),
        'u' => Some([
            0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101,
        ]),
        'v' => Some([
            0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ]),
        'w' => Some([
            0b00000, 0b00000, 0b10001, 0b10101, 0b10101, 0b11011, 0b01010,
        ]),
        'x' => Some([
            0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001,
        ]),
        'y' => Some([
            0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110,
        ]),
        'z' => Some([
            0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111,
        ]),
        c if c.is_ascii_control() => None,
        _ => Some([
            0b01110, 0b00001, 0b00110, 0b00100, 0b00000, 0b00100, 0b00000,
        ]), // '?'
    }
}

/// Rasterize a string using the built-in generic pixel font into a layer buffer.
/// `scale` = cells per pixel (1 = 1:1, 2 = 2×2 cells per pixel).
/// Each ON pixel = solid cell glyph with transparent background.
/// OFF pixels = transparent (unchanged).
pub fn rasterize_generic(
    content: &str,
    scale: u16,
    fg_col: engine_core::color::Color,
    draw_x: u16,
    draw_y: u16,
    buffer: &mut engine_core::buffer::Buffer,
    transform: &TextTransform,
) {
    let scale = scale.max(1);
    let glyph_w = 5u16;
    let _glyph_h = 7u16;
    let gap = 1u16;

    let mut cursor_x = draw_x;
    for ch in content.chars().map(|c| apply_transform(c, transform)) {
        let rows = match generic_glyph_rows(ch) {
            Some(r) => r,
            None => continue,
        };
        if ch == ' ' {
            cursor_x += (glyph_w + gap) * scale;
            continue;
        }
        for (row_idx, &row_bits) in rows.iter().enumerate() {
            for col in 0..glyph_w {
                let bit = (row_bits >> (4 - col)) & 1;
                if bit == 1 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let cx = cursor_x + col * scale + sx;
                            let cy = draw_y + row_idx as u16 * scale + sy;
                            buffer.set(cx, cy, '█', fg_col, engine_core::color::Color::Reset);
                        }
                    }
                }
            }
        }
        cursor_x += (glyph_w + gap) * scale;
    }
}

/// Compute the pixel dimensions of a string rendered with the generic font.
/// Returns (width_cells, height_cells).
pub fn generic_dimensions(content: &str, scale: u16) -> (u16, u16) {
    let scale = scale.max(1);
    let glyph_w = 5u16;
    let glyph_h = 7u16;
    let gap = 1u16;

    let char_count = content
        .chars()
        .filter(|&c| generic_glyph_rows(c).is_some())
        .count() as u16;
    if char_count == 0 {
        return (1, glyph_h * scale);
    }
    let width = char_count * (glyph_w + gap) * scale;
    let height = glyph_h * scale;
    (width, height)
}

/// Compact 4×5 minimal bitmaps. Each row is a 4-bit mask (bit 3 = leftmost).
pub fn generic_glyph_rows_tiny(ch: char) -> Option<[u8; 5]> {
    match ch {
        // Hand-tuned 4x5 overrides for readability where block-OR downsample
        // collapses distinct 5x7 glyphs into ambiguous shapes.
        'A' => return Some([0b0110, 0b1001, 0b1111, 0b1001, 0b1001]),
        'R' => return Some([0b1110, 0b1001, 0b1110, 0b1010, 0b1001]),
        'a' => return Some([0b0000, 0b0110, 0b0001, 0b0111, 0b0111]),
        'r' => return Some([0b0000, 0b1110, 0b1001, 0b1000, 0b1000]),
        ' ' => return Some([0b0000, 0b0000, 0b0000, 0b0000, 0b0000]),
        '.' => return Some([0b0000, 0b0000, 0b0000, 0b0000, 0b0010]),
        ',' => return Some([0b0000, 0b0000, 0b0000, 0b0010, 0b0100]),
        ':' => return Some([0b0000, 0b0010, 0b0000, 0b0010, 0b0000]),
        ';' => return Some([0b0000, 0b0010, 0b0000, 0b0010, 0b0100]),
        '-' => return Some([0b0000, 0b0000, 0b1111, 0b0000, 0b0000]),
        '_' => return Some([0b0000, 0b0000, 0b0000, 0b0000, 0b1111]),
        '!' => return Some([0b0010, 0b0010, 0b0010, 0b0000, 0b0010]),
        '?' => return Some([0b1111, 0b0001, 0b0010, 0b0000, 0b0010]),
        '/' => return Some([0b0001, 0b0001, 0b0010, 0b0100, 0b0100]),
        '\\' => return Some([0b1000, 0b1000, 0b0100, 0b0010, 0b0010]),
        '>' => return Some([0b1000, 0b0100, 0b0010, 0b0100, 0b1000]),
        '<' => return Some([0b0001, 0b0010, 0b0100, 0b0010, 0b0001]),
        // Common TUI / box-drawing symbols used by debug overlays.
        '■' | '█' | '◼' => return Some([0b1111, 0b1111, 0b1111, 0b1111, 0b1111]),
        '▪' => return Some([0b0000, 0b0110, 0b0110, 0b0110, 0b0000]),
        '─' | '━' | '═' => return Some([0b0000, 0b0000, 0b1111, 0b0000, 0b0000]),
        '│' | '┃' | '║' => return Some([0b0010, 0b0010, 0b0010, 0b0010, 0b0010]),
        '┌' | '╭' => return Some([0b1111, 0b1000, 0b1000, 0b1000, 0b1000]),
        '┐' | '╮' => return Some([0b1111, 0b0001, 0b0001, 0b0001, 0b0001]),
        '└' | '╰' => return Some([0b1000, 0b1000, 0b1000, 0b1000, 0b1111]),
        '┘' | '╯' => return Some([0b0001, 0b0001, 0b0001, 0b0001, 0b1111]),
        '├' => return Some([0b1000, 0b1000, 0b1111, 0b1000, 0b1000]),
        '┤' => return Some([0b0001, 0b0001, 0b1111, 0b0001, 0b0001]),
        '┬' => return Some([0b1111, 0b0010, 0b0010, 0b0010, 0b0010]),
        '┴' => return Some([0b0010, 0b0010, 0b0010, 0b0010, 0b1111]),
        '┼' => return Some([0b0010, 0b0010, 0b1111, 0b0010, 0b0010]),
        '·' => return Some([0b0000, 0b0000, 0b0010, 0b0000, 0b0000]),
        '•' => return Some([0b0000, 0b0110, 0b0110, 0b0000, 0b0000]),
        '♥' => return Some([0b0101, 0b1111, 0b1111, 0b0110, 0b0010]),
        '…' => return Some([0b0000, 0b0000, 0b0000, 0b1010, 0b0000]),
        '→' => return Some([0b0010, 0b0001, 0b1111, 0b0001, 0b0010]),
        '←' => return Some([0b0100, 0b1000, 0b1111, 0b1000, 0b0100]),
        '↑' => return Some([0b0010, 0b0111, 0b0010, 0b0010, 0b0010]),
        '↓' => return Some([0b0010, 0b0010, 0b0010, 0b0111, 0b0010]),
        _ => {}
    }
    let rows_5x7 = generic_glyph_rows(ch)?;
    Some(shrink_5x7_to_4x5(rows_5x7))
}

fn shrink_5x7_to_4x5(rows_5x7: [u8; 7]) -> [u8; 5] {
    // Pixel-preserving downsample (block OR), no smoothing/anti-aliasing.
    // 5×7 -> 4×5 with fixed source blocks.
    let y_blocks = [(0usize, 1usize), (2, 2), (3, 4), (5, 5), (6, 6)];
    let x_blocks = [(0usize, 1usize), (2, 2), (3, 3), (4, 4)];
    let mut out = [0u8; 5];

    for (oy, &(y0, y1)) in y_blocks.iter().enumerate() {
        let mut mask = 0u8;
        for (ox, &(x0, x1)) in x_blocks.iter().enumerate() {
            let mut on = false;
            for row in &rows_5x7[y0..=y1] {
                for xx in x0..=x1 {
                    if ((row >> (4 - xx)) & 1) == 1 {
                        on = true;
                        break;
                    }
                }
                if on {
                    break;
                }
            }

            if on {
                mask |= 1 << (3 - ox);
            }
        }
        out[oy] = mask;
    }
    out
}

/// Rasterize a string using the compact 4×5 tiny generic pixel font into a
/// layer buffer.
/// Each ON pixel = solid cell glyph with transparent background.
/// OFF pixels = transparent (unchanged).
pub fn rasterize_generic_tiny(
    content: &str,
    fg_col: engine_core::color::Color,
    draw_x: u16,
    draw_y: u16,
    buffer: &mut engine_core::buffer::Buffer,
    transform: &TextTransform,
) {
    let glyph_w = GENERIC_TINY_GLYPH_WIDTH;
    let gap = GENERIC_TINY_GLYPH_GAP;

    let mut cursor_x = draw_x;
    for ch in content.chars().map(|c| apply_transform(c, transform)) {
        let rows = match generic_glyph_rows_tiny(ch) {
            Some(r) => r,
            None => continue,
        };
        if ch == ' ' {
            cursor_x += glyph_w + gap;
            continue;
        }
        for (row_idx, &row_bits) in rows.iter().enumerate() {
            for col in 0..glyph_w {
                let bit = (row_bits >> (glyph_w - 1 - col)) & 1;
                if bit == 1 {
                    let cx = cursor_x + col;
                    let cy = draw_y + row_idx as u16;
                    buffer.set(cx, cy, '█', fg_col, engine_core::color::Color::Reset);
                }
            }
        }
        cursor_x += glyph_w + gap;
    }
}

/// Compute the pixel dimensions of a string rendered with the compact 4×5 tiny
/// generic font.
/// Returns (width_cells, height_cells).
pub fn generic_dimensions_tiny(content: &str) -> (u16, u16) {
    let glyph_w = GENERIC_TINY_GLYPH_WIDTH;
    let glyph_h = GENERIC_TINY_GLYPH_HEIGHT;
    let gap = GENERIC_TINY_GLYPH_GAP;

    let char_count = content
        .chars()
        .filter(|&c| generic_glyph_rows_tiny(c).is_some())
        .count() as u16;

    if char_count == 0 {
        return (1, glyph_h);
    }
    let width = char_count * (glyph_w + gap);
    (width, glyph_h)
}

/// Width in cells for a text string rendered with the 5×7 generic font at the given scale.
pub fn span_width(text: &str, scale: u16) -> u16 {
    generic_dimensions(text, scale).0
}

/// Width in cells for a text string rendered with the compact 4×5 tiny generic
/// font.
pub fn span_width_tiny(text: &str) -> u16 {
    generic_dimensions_tiny(text).0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericMode {
    Tiny,
    Standard,
    Large,
}

impl GenericMode {
    pub fn from_font_name(font_name: &str) -> Self {
        let spec = font_name
            .strip_prefix("generic")
            .and_then(|s| s.strip_prefix(':'))
            .map(|s| s.to_ascii_lowercase());

        match spec.as_deref() {
            Some("1") | Some("tiny") => Self::Tiny,
            Some("3") | Some("large") => Self::Large,
            Some("2") | Some("standard") | None | Some("") => Self::Standard,
            _ => Self::Standard,
        }
    }
}

pub fn generic_dimensions_mode(content: &str, mode: GenericMode) -> (u16, u16) {
    let char_count = content
        .chars()
        .filter(|&c| generic_glyph_rows(c).is_some())
        .count() as u16;

    if char_count == 0 {
        return match mode {
            GenericMode::Tiny => (1, GENERIC_TINY_GLYPH_HEIGHT),
            GenericMode::Standard => (1, 7),
            GenericMode::Large => (1, 14),
        };
    }

    match mode {
        GenericMode::Tiny => (
            char_count * (GENERIC_TINY_GLYPH_WIDTH + GENERIC_TINY_GLYPH_GAP),
            GENERIC_TINY_GLYPH_HEIGHT,
        ),
        GenericMode::Standard => (char_count * 6, 7),
        GenericMode::Large => (char_count * 12, 14),
    }
}

pub fn rasterize_spans_mode(
    spans: &[(&str, engine_core::color::Color)],
    mode: GenericMode,
    draw_x: u16,
    draw_y: u16,
    buf: &mut engine_core::buffer::Buffer,
    transform: &TextTransform,
) {
    let mut x = draw_x;
    for (text, colour) in spans {
        let w = generic_dimensions_mode(text, mode).0;
        match mode {
            GenericMode::Tiny => rasterize_generic_tiny(text, *colour, x, draw_y, buf, transform),
            GenericMode::Standard => rasterize_generic(text, 1, *colour, x, draw_y, buf, transform),
            GenericMode::Large => rasterize_generic(text, 2, *colour, x, draw_y, buf, transform),
        }
        x += w;
    }
}

/// Rasterize a list of (text, colour) spans using the generic font at the
/// given preset.
/// preset 1 = compact 4×5 tiny, preset 3 = 5×7 ×2 scale, default = 5×7 ×1.
pub fn rasterize_spans(
    spans: &[(&str, engine_core::color::Color)],
    preset: u16,
    draw_x: u16,
    draw_y: u16,
    buf: &mut engine_core::buffer::Buffer,
    transform: &TextTransform,
) {
    let mode = match preset {
        1 => GenericMode::Tiny,
        3 => GenericMode::Large,
        _ => GenericMode::Standard,
    };
    rasterize_spans_mode(spans, mode, draw_x, draw_y, buf, transform);
}

#[cfg(test)]
mod tests {
    use super::{generic_dimensions_mode, generic_glyph_rows_tiny, GenericMode};

    #[test]
    fn parses_generic_mode_from_font_name() {
        assert_eq!(
            GenericMode::from_font_name("generic"),
            GenericMode::Standard
        );
        assert_eq!(GenericMode::from_font_name("generic:1"), GenericMode::Tiny);
        assert_eq!(
            GenericMode::from_font_name("generic:2"),
            GenericMode::Standard
        );
        assert_eq!(GenericMode::from_font_name("generic:3"), GenericMode::Large);
    }

    #[test]
    fn computes_dimensions_for_all_modes() {
        assert_eq!(generic_dimensions_mode("AB", GenericMode::Tiny), (10, 5));
        assert_eq!(
            generic_dimensions_mode("AB", GenericMode::Standard),
            (12, 7)
        );
        assert_eq!(generic_dimensions_mode("AB", GenericMode::Large), (24, 14));
    }

    #[test]
    fn tiny_glyphs_keep_a_and_r_distinct() {
        assert_ne!(generic_glyph_rows_tiny('A'), generic_glyph_rows_tiny('R'));
        assert_ne!(generic_glyph_rows_tiny('a'), generic_glyph_rows_tiny('r'));
    }

    #[test]
    fn tiny_glyph_shapes_for_a_and_r_are_stable() {
        assert_eq!(
            generic_glyph_rows_tiny('A'),
            Some([0b0110, 0b1001, 0b1111, 0b1001, 0b1001])
        );
        assert_eq!(
            generic_glyph_rows_tiny('R'),
            Some([0b1110, 0b1001, 0b1110, 0b1010, 0b1001])
        );
    }

    #[test]
    fn tiny_glyphs_include_common_tui_symbols() {
        assert_ne!(generic_glyph_rows_tiny('─'), generic_glyph_rows_tiny('?'));
        assert_ne!(generic_glyph_rows_tiny('│'), generic_glyph_rows_tiny('?'));
        assert_ne!(generic_glyph_rows_tiny('■'), generic_glyph_rows_tiny('?'));
        assert_eq!(
            generic_glyph_rows_tiny('┼'),
            Some([0b0010, 0b0010, 0b1111, 0b0010, 0b0010])
        );
    }

    #[test]
    fn text_transform_none_preserves_lowercase_glyph_lookup() {
        use super::{apply_transform, generic_glyph_rows, TextTransform};
        // With None transform, 'h' stays 'h' and has its own distinct glyph.
        let c = apply_transform('h', &TextTransform::None);
        assert_eq!(c, 'h');
        assert!(generic_glyph_rows('h').is_some());
        // Lowercase 'h' should have a DIFFERENT glyph than uppercase 'H'.
        assert_ne!(generic_glyph_rows('h'), generic_glyph_rows('H'));
    }

    #[test]
    fn text_transform_uppercase_converts_to_uppercase() {
        use super::{apply_transform, TextTransform};
        assert_eq!(apply_transform('h', &TextTransform::Uppercase), 'H');
        assert_eq!(apply_transform('z', &TextTransform::Uppercase), 'Z');
        assert_eq!(apply_transform('A', &TextTransform::Uppercase), 'A');
    }

    #[test]
    fn lowercase_g_aligns_with_lowercase_o_top_height() {
        use super::generic_glyph_rows;
        let g = generic_glyph_rows('g').expect("g glyph");
        let o = generic_glyph_rows('o').expect("o glyph");
        let g_top = g
            .iter()
            .position(|&row| row != 0)
            .expect("g has visible pixels");
        let o_top = o
            .iter()
            .position(|&row| row != 0)
            .expect("o has visible pixels");
        assert_eq!(g_top, o_top);
    }

    #[test]
    fn rasterize_generic_lowercase_differs_from_uppercase() {
        use super::{rasterize_generic, TextTransform};
        use engine_core::buffer::Buffer;
        use engine_core::color::Color;

        let mut buf_lower = Buffer::new(100, 10);
        buf_lower.fill(Color::Reset);
        rasterize_generic(
            "hello",
            1,
            Color::White,
            0,
            0,
            &mut buf_lower,
            &TextTransform::None,
        );

        let mut buf_upper = Buffer::new(100, 10);
        buf_upper.fill(Color::Reset);
        rasterize_generic(
            "HELLO",
            1,
            Color::White,
            0,
            0,
            &mut buf_upper,
            &TextTransform::None,
        );

        // Lowercase and uppercase should render differently now that we have distinct glyphs.
        let mut any_diff = false;
        for y in 0..7u16 {
            for x in 0..30u16 {
                if buf_lower.get(x, y).map(|c| c.symbol) != buf_upper.get(x, y).map(|c| c.symbol) {
                    any_diff = true;
                }
            }
        }
        assert!(
            any_diff,
            "lowercase and uppercase glyphs should look different"
        );
    }
}

/// 5×7 bitmapped glyphs for the built-in "generic" pixel font.
/// Each entry: character → array of 7 rows, each row is a 5-bit mask (bit 4 = leftmost).
pub fn generic_glyph_rows(ch: char) -> Option<[u8; 7]> {
    match ch {
        'S' => Some([0b01110, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110]),
        'H' => Some([0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
        'E' => Some([0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111]),
        'L' => Some([0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111]),
        'Q' => Some([0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101]),
        'U' => Some([0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
        'T' => Some([0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100]),
        'I' => Some([0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111]),
        'P' => Some([0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000]),
        'R' => Some([0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001]),
        'A' => Some([0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
        'N' => Some([0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001]),
        'Y' => Some([0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100]),
        'K' => Some([0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001]),
        ' ' => Some([0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000]),
        _   => None,
    }
}

/// Rasterize a string using the built-in generic pixel font into a layer buffer.
/// `scale` = cells per pixel (1 = 1:1, 2 = 2×2 cells per pixel).
/// Each ON pixel = cell with bg=fg_col, ch=' '.
/// OFF pixels = transparent (unchanged).
pub fn rasterize_generic(
    content: &str,
    scale: u16,
    fg_col: crossterm::style::Color,
    draw_x: u16,
    draw_y: u16,
    buffer: &mut crate::buffer::Buffer,
) {
    let scale = scale.max(1);
    let glyph_w = 5u16;
    let _glyph_h = 7u16;
    let gap = 1u16;

    let mut cursor_x = draw_x;
    for ch in content.chars().map(|c| c.to_ascii_uppercase()) {
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
                            buffer.set(cx, cy, ' ', fg_col, fg_col);
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

    let char_count = content.chars().map(|c| c.to_ascii_uppercase())
        .filter(|&c| generic_glyph_rows(c).is_some())
        .count() as u16;

    if char_count == 0 {
        return (1, glyph_h * scale);
    }
    let width = char_count * (glyph_w + gap) * scale;
    let height = glyph_h * scale;
    (width, height)
}

/// 3×5 minimal bitmaps. Each row is a 3-bit mask (bit 2 = leftmost pixel).
pub fn generic_glyph_rows_tiny(ch: char) -> Option<[u8; 5]> {
    match ch {
        'P' => Some([0b110, 0b101, 0b110, 0b100, 0b100]),
        'R' => Some([0b110, 0b101, 0b110, 0b101, 0b101]),
        'E' => Some([0b111, 0b100, 0b110, 0b100, 0b111]),
        'S' => Some([0b111, 0b100, 0b111, 0b001, 0b111]),
        ' ' => Some([0b000, 0b000, 0b000, 0b000, 0b000]),
        'A' => Some([0b010, 0b101, 0b111, 0b101, 0b101]),
        'N' => Some([0b101, 0b111, 0b101, 0b101, 0b101]),
        'Y' => Some([0b101, 0b101, 0b010, 0b010, 0b010]),
        'K' => Some([0b101, 0b110, 0b100, 0b110, 0b101]),
        'H' => Some([0b101, 0b101, 0b111, 0b101, 0b101]),
        'I' => Some([0b111, 0b010, 0b010, 0b010, 0b111]),
        'L' => Some([0b100, 0b100, 0b100, 0b100, 0b111]),
        'Q' => Some([0b010, 0b101, 0b101, 0b110, 0b011]),
        'T' => Some([0b111, 0b010, 0b010, 0b010, 0b010]),
        'U' => Some([0b101, 0b101, 0b101, 0b101, 0b111]),
        _   => None,
    }
}

/// Rasterize a string using the 3×5 tiny generic pixel font into a layer buffer.
/// Each ON pixel = cell with bg=fg_col, ch=' '. OFF pixels = transparent (unchanged).
pub fn rasterize_generic_tiny(
    content: &str,
    fg_col: crossterm::style::Color,
    draw_x: u16,
    draw_y: u16,
    buffer: &mut crate::buffer::Buffer,
) {
    let glyph_w = 3u16;
    let gap = 1u16;

    let mut cursor_x = draw_x;
    for ch in content.chars().map(|c| c.to_ascii_uppercase()) {
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
                let bit = (row_bits >> (2 - col)) & 1;
                if bit == 1 {
                    let cx = cursor_x + col;
                    let cy = draw_y + row_idx as u16;
                    buffer.set(cx, cy, ' ', fg_col, fg_col);
                }
            }
        }
        cursor_x += glyph_w + gap;
    }
}

/// Compute the pixel dimensions of a string rendered with the 3×5 tiny generic font.
/// Returns (width_cells, height_cells).
pub fn generic_dimensions_tiny(content: &str) -> (u16, u16) {
    let glyph_w = 3u16;
    let glyph_h = 5u16;
    let gap = 1u16;

    let char_count = content.chars().map(|c| c.to_ascii_uppercase())
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

/// Width in cells for a text string rendered with the 3×5 tiny generic font.
pub fn span_width_tiny(text: &str) -> u16 {
    generic_dimensions_tiny(text).0
}

/// Rasterize a list of (text, colour) spans using the generic font at the given preset.
/// preset 1 = 3×5 tiny, preset 3 = 5×7 ×2 scale, default = 5×7 ×1 scale.
pub fn rasterize_spans(
    spans: &[(String, crossterm::style::Color)],
    preset: u16,
    draw_x: u16,
    draw_y: u16,
    buf: &mut crate::buffer::Buffer,
) {
    let mut x = draw_x;
    for (text, colour) in spans {
        match preset {
            1 => {
                let w = span_width_tiny(text);
                rasterize_generic_tiny(text, *colour, x, draw_y, buf);
                x += w;
            }
            3 => {
                let w = span_width(text, 2);
                rasterize_generic(text, 2, *colour, x, draw_y, buf);
                x += w;
            }
            _ => {
                let w = span_width(text, 1);
                rasterize_generic(text, 1, *colour, x, draw_y, buf);
                x += w;
            }
        }
    }
}

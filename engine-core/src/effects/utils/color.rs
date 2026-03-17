use crossterm::style::Color;

/// Convert a crossterm Color to its (r, g, b) components.
pub fn colour_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::DarkGrey => (85, 85, 85),
        Color::Grey => (170, 170, 170),
        Color::White => (255, 255, 255),
        Color::Red => (255, 0, 0),
        Color::DarkRed => (128, 0, 0),
        Color::Green => (0, 255, 0),
        Color::DarkGreen => (0, 128, 0),
        Color::Blue => (0, 0, 255),
        Color::DarkBlue => (0, 0, 128),
        Color::Yellow => (255, 255, 0),
        Color::DarkYellow => (128, 128, 0),
        Color::Cyan => (0, 255, 255),
        Color::DarkCyan => (0, 128, 128),
        Color::Magenta => (255, 0, 255),
        Color::DarkMagenta => (128, 0, 128),
        Color::AnsiValue(_) | Color::Reset => (255, 255, 255),
    }
}

pub fn lerp_colour(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let (fr, fg, fb) = colour_to_rgb(from);
    let (tr, tg, tb) = colour_to_rgb(to);
    let rr = fr as f32 + (tr as f32 - fr as f32) * t;
    let rg = fg as f32 + (tg as f32 - fg as f32) * t;
    let rb = fb as f32 + (tb as f32 - fb as f32) * t;
    Color::Rgb {
        r: rr.round() as u8,
        g: rg.round() as u8,
        b: rb.round() as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_named_colour_to_rgb() {
        assert_eq!(colour_to_rgb(Color::Blue), (0, 0, 255));
    }

    #[test]
    fn lerp_colour_interpolates_midpoint() {
        let mid = lerp_colour(
            Color::Rgb { r: 0, g: 0, b: 0 },
            Color::Rgb {
                r: 100,
                g: 50,
                b: 0,
            },
            0.5,
        );
        assert_eq!(mid, Color::Rgb { r: 50, g: 25, b: 0 });
    }
}

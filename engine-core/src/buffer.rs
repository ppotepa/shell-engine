use crossterm::style::Color;

/// A single terminal cell — the atomic "pixel" of the engine.
/// Stores a Unicode character plus foreground and background colours.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub symbol: char,
    pub fg: Color,
    pub bg: Color,
}

/// True black constant — bypasses terminal theme palette.
pub const TRUE_BLACK: Color = Color::Rgb { r: 0, g: 0, b: 0 };

impl Cell {
    pub fn blank(bg: Color) -> Self {
        Self {
            symbol: ' ',
            fg: TRUE_BLACK,
            bg,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::blank(TRUE_BLACK)
    }
}

/// A changed cell ready to be written to the terminal.
pub struct CellDiff<'a> {
    pub x: u16,
    pub y: u16,
    pub cell: &'a Cell,
}

/// Double-buffered pixel bitmap for the terminal screen.
///
/// `back` is written to by the compositor each frame.
/// `front` mirrors what is currently visible on the terminal.
/// Only cells that differ between back and front are flushed on each render.
/// After flushing, `swap()` copies back → front.
#[derive(Debug, Clone)]
pub struct Buffer {
    pub width: u16,
    pub height: u16,
    /// Back buffer — written to by compositor each frame.
    back: Vec<Cell>,
    /// Front buffer — mirrors what the terminal currently shows.
    front: Vec<Cell>,
}

/// Optional off-screen fixed-resolution buffer used before presenting to terminal output.
#[derive(Debug, Clone)]
pub struct VirtualBuffer(pub Buffer);

impl VirtualBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        Self(Buffer::new(width, height))
    }
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        let back = vec![Cell::default(); size];
        // Initialise front with a sentinel that differs from every real cell,
        // so the very first frame always flushes all pixels.
        let front = vec![
            Cell {
                symbol: '\0',
                fg: Color::Reset,
                bg: Color::Reset
            };
            size
        ];
        Self {
            width,
            height,
            back,
            front,
        }
    }

    /// Fill the entire back buffer with blank cells of the given background colour.
    pub fn fill(&mut self, bg: Color) {
        for cell in &mut self.back {
            *cell = Cell::blank(bg);
        }
    }

    /// Write a single pixel to the back buffer.
    pub fn set(&mut self, x: u16, y: u16, symbol: char, fg: Color, bg: Color) {
        if x < self.width && y < self.height {
            self.back[y as usize * self.width as usize + x as usize] = Cell { symbol, fg, bg };
        }
    }

    /// Read a pixel from the back buffer.
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        if x < self.width && y < self.height {
            self.back.get(y as usize * self.width as usize + x as usize)
        } else {
            None
        }
    }

    /// Return cells that differ between back and front — the minimal render set.
    pub fn diff(&self) -> Vec<CellDiff<'_>> {
        let mut result = Vec::new();
        let mut idx = 0usize;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.back[idx] != self.front[idx] {
                    result.push(CellDiff { x, y, cell: &self.back[idx] });
                }
                idx += 1;
            }
        }
        result
    }

    /// Fill `out` with raw (pre-resolve) diff tuples, reusing the allocation across frames.
    pub fn diff_into(&self, out: &mut Vec<(u16, u16, char, Color, Color)>) {
        let mut idx = 0usize;
        for y in 0..self.height {
            for x in 0..self.width {
                let b = &self.back[idx];
                if *b != self.front[idx] {
                    out.push((x, y, b.symbol, b.fg, b.bg));
                }
                idx += 1;
            }
        }
    }

    /// Promote back buffer to front — call after every successful flush.
    /// Uses O(1) pointer swap instead of O(W×H) memcpy; the caller always
    /// calls `fill()` on the back buffer before the next frame anyway.
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }

    /// Restore the front buffer (last flushed frame) into the back buffer.
    ///
    /// Used by the last-good-frame fallback: when a script error is detected,
    /// calling this before the diff preserves the last visible frame on screen
    /// instead of showing a compositor-cleared blank.
    pub fn restore_front_to_back(&mut self) {
        self.back.clone_from(&self.front);
    }

    /// Force a full re-flush on the next render (e.g. after terminal resize).
    pub fn invalidate(&mut self) {
        for cell in &mut self.front {
            cell.symbol = '\0';
        }
    }

    /// Resize both buffers, preserving nothing (invalidates front for full redraw).
    pub fn resize(&mut self, width: u16, height: u16) {
        let size = width as usize * height as usize;
        self.width = width;
        self.height = height;
        self.back = vec![Cell::default(); size];
        self.front = vec![
            Cell {
                symbol: '\0',
                fg: Color::Reset,
                bg: Color::Reset
            };
            size
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_has_all_cells_as_diff() {
        let buf = Buffer::new(4, 2);
        // Front is all '\0', back is all ' ' → every cell should diff
        assert_eq!(buf.diff().len(), 8);
    }

    #[test]
    fn after_swap_diff_is_empty_on_unchanged_buffer() {
        let mut buf = Buffer::new(4, 2);
        buf.fill(Color::Black);
        buf.swap();
        buf.fill(Color::Black); // matches real game loop: fill() after swap()
        assert_eq!(buf.diff().len(), 0);
    }

    #[test]
    fn set_then_diff_returns_only_changed_cells() {
        let mut buf = Buffer::new(4, 2);
        buf.fill(Color::Black);
        buf.swap();
        buf.fill(Color::Black); // same content — matches real game loop
        buf.set(1, 0, 'X', Color::White, Color::Black); // one change
        assert_eq!(buf.diff().len(), 1);
        let d = &buf.diff()[0];
        assert_eq!(d.x, 1);
        assert_eq!(d.y, 0);
        assert_eq!(d.cell.symbol, 'X');
    }

    #[test]
    fn invalidate_forces_full_redraw() {
        let mut buf = Buffer::new(2, 2);
        buf.fill(Color::Black);
        buf.swap();
        buf.fill(Color::Black); // real game loop: fill after swap
        assert_eq!(buf.diff().len(), 0);
        buf.invalidate();
        assert_eq!(buf.diff().len(), 4);
    }

    #[test]
    fn resize_resets_both_buffers() {
        let mut buf = Buffer::new(2, 2);
        buf.swap();
        buf.resize(3, 3);
        assert_eq!(buf.width, 3);
        assert_eq!(buf.height, 3);
        assert_eq!(buf.diff().len(), 9);
    }
}

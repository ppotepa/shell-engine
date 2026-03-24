use crossterm::style::Color;

/// A single terminal cell — the atomic "pixel" of the engine.
/// Stores a Unicode character plus foreground and background colours.
#[derive(Debug, Clone, Copy, PartialEq)]
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
///
/// Dirty rect tracking avoids O(W*H) fill/diff scans by tracking modified regions.
#[derive(Debug, Clone)]
pub struct Buffer {
    pub width: u16,
    pub height: u16,
    /// Back buffer — written to by compositor each frame.
    back: Vec<Cell>,
    /// Front buffer — mirrors what the terminal currently shows.
    front: Vec<Cell>,
    /// Generation counter for lazy invalidation (avoids rewriting every front cell).
    generation: u64,
    /// Tracks which generation this frame's front buffer represents.
    front_generation: u64,
    /// Dirty region bounds: (min_x, max_x, min_y, max_y) — uint::MAX means no dirty region.
    dirty_x_min: u16,
    dirty_x_max: u16,
    dirty_y_min: u16,
    dirty_y_max: u16,
    /// Monotonic counter incremented on every mutation (set/fill/blit/resize).
    pub write_count: u64,
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
            generation: 1,
            front_generation: 0,
            dirty_x_min: u16::MAX,
            dirty_x_max: 0,
            dirty_y_min: u16::MAX,
            dirty_y_max: 0,
            write_count: 0,
        }
    }

    /// Fill the entire back buffer with blank cells of the given background colour.
    /// Uses generation-based lazy invalidation instead of rewriting every front cell.
    pub fn fill(&mut self, bg: Color) {
        self.back.fill(Cell::blank(bg));
        self.dirty_x_min = 0;
        self.dirty_x_max = self.width.saturating_sub(1);
        self.dirty_y_min = 0;
        self.dirty_y_max = self.height.saturating_sub(1);
        self.write_count += 1;
    }

    /// Write a single pixel to the back buffer, tracking dirty region.
    pub fn set(&mut self, x: u16, y: u16, symbol: char, fg: Color, bg: Color) {
        if x < self.width && y < self.height {
            let idx = y as usize * self.width as usize + x as usize;
            self.back[idx] = Cell { symbol, fg, bg };
            self.write_count += 1;
            if x < self.dirty_x_min { self.dirty_x_min = x; }
            if x > self.dirty_x_max { self.dirty_x_max = x; }
            if y < self.dirty_y_min { self.dirty_y_min = y; }
            if y > self.dirty_y_max { self.dirty_y_max = y; }
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

    /// Return cells that differ between back and front within dirty region — minimal render set.
    /// If generation changed (invalidate/resize), scans entire buffer.
    pub fn diff(&self) -> Vec<CellDiff<'_>> {
        let mut result = Vec::new();
        
        // If generation changed, front needs full redraw.
        if self.generation != self.front_generation {
            // Full buffer scan — front was invalidated.
            let mut idx = 0usize;
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.back[idx] != self.front[idx] {
                        result.push(CellDiff { x, y, cell: &self.back[idx] });
                    }
                    idx += 1;
                }
            }
            return result;
        }
        
        // Normal case: no generation change, scan dirty region only.
        if self.dirty_x_min > self.dirty_x_max || self.dirty_y_min > self.dirty_y_max {
            return result;
        }
        
        // Scan only the dirty region.
        for y in self.dirty_y_min..=self.dirty_y_max.min(self.height.saturating_sub(1)) {
            for x in self.dirty_x_min..=self.dirty_x_max.min(self.width.saturating_sub(1)) {
                let idx = y as usize * self.width as usize + x as usize;
                if self.back[idx] != self.front[idx] {
                    result.push(CellDiff { x, y, cell: &self.back[idx] });
                }
            }
        }
        result
    }

    /// Fill `out` with raw (pre-resolve) diff tuples within dirty region, reusing the allocation.
    /// If generation changed, scans entire buffer.
    pub fn diff_into(&self, out: &mut Vec<(u16, u16, char, Color, Color)>) {
        // If generation changed, front needs full redraw.
        if self.generation != self.front_generation {
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
            return;
        }
        
        // Normal case: no generation change, scan dirty region only.
        if self.dirty_x_min > self.dirty_x_max || self.dirty_y_min > self.dirty_y_max {
            return;
        }
        
        // Scan only the dirty region.
        for y in self.dirty_y_min..=self.dirty_y_max.min(self.height.saturating_sub(1)) {
            for x in self.dirty_x_min..=self.dirty_x_max.min(self.width.saturating_sub(1)) {
                let idx = y as usize * self.width as usize + x as usize;
                let b = &self.back[idx];
                if *b != self.front[idx] {
                    out.push((x, y, b.symbol, b.fg, b.bg));
                }
            }
        }
    }

    /// Promote back buffer to front — call after every successful flush.
    /// Uses O(1) pointer swap instead of O(W×H) memcpy; resets dirty tracking.
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
        self.front_generation = self.generation;
        // Reset dirty region for next frame.
        self.dirty_x_min = u16::MAX;
        self.dirty_x_max = 0;
        self.dirty_y_min = u16::MAX;
        self.dirty_y_max = 0;
    }

    /// Restore the front buffer (last flushed frame) into the back buffer.
    ///
    /// Used by the last-good-frame fallback: when a script error is detected,
    /// calling this before the diff preserves the last visible frame on screen
    /// instead of showing a compositor-cleared blank. Marks entire region as dirty.
    pub fn restore_front_to_back(&mut self) {
        self.back.clone_from(&self.front);
        // Mark entire buffer as dirty so restored frame gets flushed.
        self.dirty_x_min = 0;
        self.dirty_x_max = self.width.saturating_sub(1);
        self.dirty_y_min = 0;
        self.dirty_y_max = self.height.saturating_sub(1);
    }

    /// Force a full re-flush on the next render (e.g. after terminal resize).
    /// Bumps generation to trigger full diff scan without per-cell writes.
    pub fn invalidate(&mut self) {
        for cell in &mut self.front {
            cell.symbol = '\0';
        }
        self.dirty_x_min = 0;
        self.dirty_x_max = self.width.saturating_sub(1);
        self.dirty_y_min = 0;
        self.dirty_y_max = self.height.saturating_sub(1);
    }

    /// Reset dirty bounds and write_count to zero without clearing buffer contents.
    /// Call after a background fill to track only subsequent content writes.
    /// Used by compositor to make dirty-region and skip-static optimizations effective.
    pub fn reset_dirty(&mut self) {
        self.dirty_x_min = u16::MAX;
        self.dirty_x_max = 0;
        self.dirty_y_min = u16::MAX;
        self.dirty_y_max = 0;
        self.write_count = 0;
    }

    /// Compute a fast 64-bit hash of the back buffer contents for frame comparison.
    /// Uses XOR folding — cheap enough to run every frame, accurate enough to detect changes.
    pub fn back_hash(&self) -> u64 {
        let mut h: u64 = self.width as u64 | ((self.height as u64) << 16);
        for (i, cell) in self.back.iter().enumerate() {
            let i64 = i as u64;
            // Combine position with cell content via a fast mixing step.
            let v = (cell.symbol as u64)
                .wrapping_add(color_byte(cell.fg).wrapping_mul(257))
                .wrapping_add(color_byte(cell.bg).wrapping_mul(65537));
            h ^= v.wrapping_mul(0x9e3779b97f4a7c15u64).wrapping_add(i64);
            // Cheap rotation to spread bits.
            h = h.rotate_left(7);
        }
        h
    }

    /// #5 opt-comp-halfblock: return dirty bounds (x_min, x_max, y_min, y_max).
    /// Returns None if no dirty region exists.
    pub fn dirty_bounds(&self) -> Option<(u16, u16, u16, u16)> {
        if self.dirty_x_min > self.dirty_x_max || self.dirty_y_min > self.dirty_y_max {
            None
        } else {
            Some((self.dirty_x_min, self.dirty_x_max, self.dirty_y_min, self.dirty_y_max))
        }
    }

    /// Resize both buffers, preserving nothing (invalidates front for full redraw).
    pub fn resize(&mut self, width: u16, height: u16) {
        let size = width as usize * height as usize;
        self.width = width;
        self.height = height;
        self.back.resize(size, Cell::default());
        self.front.resize(size, Cell {
            symbol: '\0',
            fg: Color::Reset,
            bg: Color::Reset
        });
        // Increment generation to force full redraw.
        self.generation = self.generation.wrapping_add(1);
        self.dirty_x_min = 0;
        self.dirty_x_max = width.saturating_sub(1);
        self.dirty_y_min = 0;
        self.dirty_y_max = height.saturating_sub(1);
        self.write_count += 1;
    }

    /// Blit a rectangular region from source buffer to this buffer's back.
    /// Only copies non-transparent cells (space with Color::Reset background).
    /// Tracks dirty region once after all copies (instead of per-pixel updates).
    pub fn blit_from(
        &mut self,
        src: &Buffer,
        src_x: u16,
        src_y: u16,
        dst_x: u16,
        dst_y: u16,
        width: u16,
        height: u16,
    ) {
        let mut min_x = u16::MAX;
        let mut max_x = 0u16;
        let mut min_y = u16::MAX;
        let mut max_y = 0u16;
        let mut any_written = false;

        for y in 0..height {
            let src_row = src_y.wrapping_add(y);
            let dst_row = dst_y.wrapping_add(y);
            if dst_row >= self.height || src_row >= src.height {
                continue;
            }
            for x in 0..width {
                let src_col = src_x.wrapping_add(x);
                let dst_col = dst_x.wrapping_add(x);
                if dst_col >= self.width || src_col >= src.width {
                    continue;
                }
                if let Some(cell) = src.get(src_col, src_row) {
                    // Skip transparent cells (space with Reset background).
                    if cell.symbol == ' ' && cell.bg == Color::Reset {
                        continue;
                    }
                    let dest_idx = dst_row as usize * self.width as usize + dst_col as usize;
                    self.back[dest_idx] = *cell;
                    self.write_count += 1;
                    // Track dirty bounds regardless (pack still needs to reprocess this region).
                    min_x = min_x.min(dst_col);
                    max_x = max_x.max(dst_col);
                    min_y = min_y.min(dst_row);
                    max_y = max_y.max(dst_row);
                    any_written = true;
                }
            }
        }

        // Update dirty region once after all writes.
        if any_written {
            self.dirty_x_min = self.dirty_x_min.min(min_x);
            self.dirty_x_max = self.dirty_x_max.max(max_x);
            self.dirty_y_min = self.dirty_y_min.min(min_y);
            self.dirty_y_max = self.dirty_y_max.max(max_y);
        }
    }

    /// #7 opt-postfx-swap: copy only the back buffer from source (skip front).
    /// Used by postfx cache to avoid cloning the front buffer which is never needed.
    pub fn copy_back_from(&mut self, src: &Buffer) {
        debug_assert_eq!(self.width, src.width);
        debug_assert_eq!(self.height, src.height);
        self.back.copy_from_slice(&src.back);
        self.dirty_x_min = 0;
        self.dirty_x_max = self.width.saturating_sub(1);
        self.dirty_y_min = 0;
        self.dirty_y_max = self.height.saturating_sub(1);
        self.write_count += 1;
    }
}

/// Map a Color to a single byte for fast hashing.
fn color_byte(c: Color) -> u64 {
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
        Color::Rgb { r, g, b } => {
            (r as u64).wrapping_mul(65521)
                ^ (g as u64).wrapping_mul(257)
                ^ b as u64
        }
        Color::AnsiValue(v) => 200u64.wrapping_add(v as u64),
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

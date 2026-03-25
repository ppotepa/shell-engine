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
    /// Number of cells emitted by the most recent diff_into / diff_into_dirty call.
    pub last_diff_count: u32,
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
            last_diff_count: 0,
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

    /// Fill `out` with raw (pre-resolve) diff tuples, reusing the allocation.
    /// Always scans the full buffer — safe default. Use DirtyRegionDiff strategy
    /// behind --opt-diff for the narrowed scan optimization.
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

    /// Dirty-region variant of diff_into — scans only the tracked dirty bounds.
    /// Used by the DirtyRegionDiff strategy behind --opt-diff.
    /// ONLY safe when fill() is guaranteed to have run this frame with no reset_dirty() after it.
    pub fn diff_into_dirty(&self, out: &mut Vec<(u16, u16, char, Color, Color)>) {
        if self.dirty_x_min > self.dirty_x_max || self.dirty_y_min > self.dirty_y_max {
            return;
        }
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

    /// Expand dirty bounds to cover the given rectangle without writing any cells.
    /// Used to force dirty-region tracking for animated sprites whose frame area
    /// may shrink between frames (old pixels need to be covered by the diff).
    pub fn mark_dirty_region(&mut self, x: u16, y: u16, w: u16, h: u16) {
        if w == 0 || h == 0 { return; }
        let x_max = x.saturating_add(w.saturating_sub(1)).min(self.width.saturating_sub(1));
        let y_max = y.saturating_add(h.saturating_sub(1)).min(self.height.saturating_sub(1));
        if x < self.dirty_x_min { self.dirty_x_min = x; }
        if x_max > self.dirty_x_max { self.dirty_x_max = x_max; }
        if y < self.dirty_y_min { self.dirty_y_min = y; }
        if y_max > self.dirty_y_max { self.dirty_y_max = y_max; }
    }

    /// Total number of cells in the buffer.
    pub fn total_cells(&self) -> u32 { self.width as u32 * self.height as u32 }

    /// Number of cells inside the current dirty region (0 if no dirty region).
    pub fn dirty_cell_count(&self) -> u32 {
        match self.dirty_bounds() {
            Some((x0, x1, y0, y1)) => {
                (x1 - x0 + 1) as u32 * (y1 - y0 + 1) as u32
            }
            None => 0,
        }
    }

    /// Expand the dirty region to include another dirty bounds.
    /// Used when merging dirty regions from multiple passes (e.g., compositor + postfx layers).
    pub fn expand_dirty_bounds(&mut self, other: Option<(u16, u16, u16, u16)>) {
        if let Some((x_min, x_max, y_min, y_max)) = other {
            if x_min <= x_max {
                if x_min < self.dirty_x_min { self.dirty_x_min = x_min; }
                if x_max > self.dirty_x_max { self.dirty_x_max = x_max; }
            }
            if y_min <= y_max {
                if y_min < self.dirty_y_min { self.dirty_y_min = y_min; }
                if y_max > self.dirty_y_max { self.dirty_y_max = y_max; }
            }
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

    // ── existing diff tests ────────────────────────────────────────

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

    // ── viewport validity helpers ──────────────────────────────────

    /// Returns true when every cell in the back buffer is a "valid" cell:
    /// either fully transparent (space + Reset bg) or has a concrete RGB/named colour.
    /// Catches cells left in an impossible state (e.g. non-space glyph + Reset bg).
    fn assert_no_orphan_glyphs(buf: &Buffer) {
        for y in 0..buf.height {
            for x in 0..buf.width {
                let cell = buf.get(x, y).expect("in bounds");
                let is_transparent = cell.symbol == ' ' && cell.bg == Color::Reset;
                let is_concrete = cell.bg != Color::Reset;
                assert!(
                    is_transparent || is_concrete,
                    "Orphan glyph at ({x},{y}): symbol='{}' fg={:?} bg={:?} — \
                     non-space glyph with Reset bg would be invisible/glitchy",
                    cell.symbol, cell.fg, cell.bg,
                );
            }
        }
    }

    /// Returns true when every cell inside the given rectangle has bg != Reset
    /// (i.e. the region is fully opaque — no accidental holes).
    fn assert_region_opaque(buf: &Buffer, x0: u16, y0: u16, w: u16, h: u16) {
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                let cell = buf.get(x, y).expect("in bounds");
                assert!(
                    cell.bg != Color::Reset,
                    "Transparent hole at ({x},{y}) in expected-opaque region \
                     [{x0},{y0}..{},{})]: symbol='{}' bg={:?}",
                    x0 + w, y0 + h, cell.symbol, cell.bg,
                );
            }
        }
    }

    /// Asserts that the entire buffer contains only cells matching `bg_color` with
    /// space glyphs. Used to verify a clean fill with no stale data.
    fn assert_uniform_fill(buf: &Buffer, bg_color: Color) {
        for y in 0..buf.height {
            for x in 0..buf.width {
                let cell = buf.get(x, y).expect("in bounds");
                assert_eq!(
                    (cell.symbol, cell.bg),
                    (' ', bg_color),
                    "Cell ({x},{y}) expected uniform fill bg={bg_color:?}, \
                     got symbol='{}' bg={:?}",
                    cell.symbol, cell.bg,
                );
            }
        }
    }

    // ── viewport validation tests ──────────────────────────────────

    #[test]
    fn fill_produces_uniform_valid_buffer() {
        let mut buf = Buffer::new(10, 8);
        let black = Color::Rgb { r: 0, g: 0, b: 0 };
        buf.fill(black);
        assert_uniform_fill(&buf, black);
        assert_no_orphan_glyphs(&buf);
    }

    #[test]
    fn fill_with_reset_is_fully_transparent() {
        let mut buf = Buffer::new(6, 4);
        buf.fill(Color::Reset);
        for y in 0..buf.height {
            for x in 0..buf.width {
                let cell = buf.get(x, y).expect("in bounds");
                assert_eq!(cell.symbol, ' ');
                assert_eq!(cell.bg, Color::Reset);
            }
        }
    }

    #[test]
    fn blit_from_skips_transparent_cells() {
        let bg = Color::Rgb { r: 20, g: 20, b: 30 };
        let mut dst = Buffer::new(6, 4);
        dst.fill(bg);

        let mut src = Buffer::new(6, 4);
        src.fill(Color::Reset); // all transparent
        src.set(2, 1, 'A', Color::White, Color::Red);

        dst.blit_from(&src, 0, 0, 0, 0, 6, 4);

        // Only (2,1) should be overwritten; rest keeps dst fill.
        let cell_a = dst.get(2, 1).unwrap();
        assert_eq!(cell_a.symbol, 'A');
        assert_eq!(cell_a.bg, Color::Red);

        // Neighbouring cell untouched.
        let cell_n = dst.get(3, 1).unwrap();
        assert_eq!(cell_n.symbol, ' ');
        assert_eq!(cell_n.bg, bg);

        assert_no_orphan_glyphs(&dst);
    }

    #[test]
    fn blit_from_preserves_dst_outside_src_bounds() {
        let bg = Color::Rgb { r: 10, g: 10, b: 10 };
        let mut dst = Buffer::new(10, 8);
        dst.fill(bg);

        let mut src = Buffer::new(4, 3);
        src.fill(Color::Rgb { r: 255, g: 0, b: 0 });
        src.set(0, 0, 'X', Color::White, Color::Rgb { r: 255, g: 0, b: 0 });

        dst.blit_from(&src, 0, 0, 2, 2, 4, 3);

        // Blitted region is opaque.
        assert_region_opaque(&dst, 2, 2, 4, 3);
        // Margin region keeps original bg.
        assert_eq!(dst.get(0, 0).unwrap().bg, bg);
        assert_eq!(dst.get(9, 7).unwrap().bg, bg);
        assert_no_orphan_glyphs(&dst);
    }

    #[test]
    fn swap_then_fill_produces_zero_diff_on_same_content() {
        // Simulates the real game loop: fill → render → swap → fill(same) → diff = 0.
        let bg = Color::Rgb { r: 0, g: 0, b: 0 };
        let mut buf = Buffer::new(8, 6);
        buf.fill(bg);
        buf.swap();
        buf.fill(bg);
        assert_eq!(buf.diff().len(), 0, "same content after swap should produce zero diff");
    }

    #[test]
    fn swap_then_different_fill_produces_full_diff() {
        let mut buf = Buffer::new(4, 4);
        buf.fill(Color::Rgb { r: 0, g: 0, b: 0 });
        buf.swap();
        buf.fill(Color::Rgb { r: 255, g: 255, b: 255 });
        assert_eq!(buf.diff().len(), 16, "different fill should diff every cell");
    }

    #[test]
    fn dirty_region_tracks_writes_correctly() {
        let mut buf = Buffer::new(10, 10);
        buf.fill(Color::Black);
        buf.reset_dirty();
        assert_eq!(buf.dirty_cell_count(), 0);

        buf.set(3, 4, 'X', Color::White, Color::Black);
        buf.set(7, 2, 'Y', Color::White, Color::Black);
        let bounds = buf.dirty_bounds();
        assert!(bounds.is_some());
        let (xmin, xmax, ymin, ymax) = bounds.unwrap();
        assert_eq!((xmin, xmax), (3, 7));
        assert_eq!((ymin, ymax), (2, 4));
    }

    #[test]
    fn dirty_region_diff_matches_full_diff() {
        let bg = Color::Rgb { r: 10, g: 10, b: 20 };
        let mut buf = Buffer::new(20, 15);
        buf.fill(bg);
        buf.swap();
        buf.fill(bg);

        // Write a few cells.
        buf.set(5, 3, 'A', Color::White, Color::Red);
        buf.set(10, 7, 'B', Color::Green, Color::Blue);
        buf.set(15, 12, 'C', Color::Yellow, Color::Magenta);

        let full_diff = buf.diff();
        let mut dirty_diff = Vec::new();
        buf.diff_into_dirty(&mut dirty_diff);

        assert_eq!(full_diff.len(), dirty_diff.len(),
            "dirty-region diff must find same cells as full-scan diff");
    }

    #[test]
    fn write_count_increments_on_mutations() {
        let mut buf = Buffer::new(4, 4);
        let initial = buf.write_count;
        buf.fill(Color::Black);
        assert!(buf.write_count > initial, "fill should increment write_count");

        let after_fill = buf.write_count;
        buf.set(0, 0, 'Z', Color::White, Color::Black);
        assert!(buf.write_count > after_fill, "set should increment write_count");
    }

    #[test]
    fn total_cells_and_dirty_cells_consistent() {
        let mut buf = Buffer::new(20, 10);
        assert_eq!(buf.total_cells(), 200);

        buf.fill(Color::Black);
        assert_eq!(buf.dirty_cell_count(), 200, "fill dirties entire buffer");

        buf.reset_dirty();
        assert_eq!(buf.dirty_cell_count(), 0, "reset clears dirty count");

        buf.set(5, 5, 'X', Color::White, Color::Black);
        assert!(buf.dirty_cell_count() >= 1, "single set creates at least 1 dirty cell");
    }

    #[test]
    fn blit_transparent_src_leaves_dst_unchanged() {
        let bg = Color::Rgb { r: 50, g: 50, b: 50 };
        let mut dst = Buffer::new(8, 6);
        dst.fill(bg);

        let src = Buffer::new(8, 6); // default: space + TRUE_BLACK bg — NOT transparent!
        let transparent_src = {
            let mut s = Buffer::new(8, 6);
            s.fill(Color::Reset); // all transparent
            s
        };

        let hash_before = dst.back_hash();
        dst.blit_from(&transparent_src, 0, 0, 0, 0, 8, 6);
        let hash_after = dst.back_hash();
        assert_eq!(hash_before, hash_after,
            "blitting fully-transparent source should not change destination");

        // Confirm dst still uniform.
        assert_uniform_fill(&dst, bg);
    }

    #[test]
    fn true_black_is_not_transparent() {
        // Critical invariant: TRUE_BLACK (Rgb{0,0,0}) is NOT the same as Reset.
        // Cells with TRUE_BLACK bg are opaque and WILL be blitted.
        let mut dst = Buffer::new(4, 4);
        dst.fill(Color::Rgb { r: 100, g: 100, b: 100 });

        let mut src = Buffer::new(4, 4);
        src.fill(TRUE_BLACK); // opaque black

        dst.blit_from(&src, 0, 0, 0, 0, 4, 4);
        assert_uniform_fill(&dst, TRUE_BLACK);
    }

    #[test]
    fn diff_into_and_diff_into_dirty_agree_after_set() {
        let bg = Color::Rgb { r: 0, g: 0, b: 0 };
        let mut buf = Buffer::new(30, 20);
        buf.fill(bg);
        buf.swap();
        buf.fill(bg);
        buf.set(15, 10, 'Z', Color::White, Color::Red);

        let mut full = Vec::new();
        buf.diff_into(&mut full);

        let mut dirty = Vec::new();
        buf.diff_into_dirty(&mut dirty);

        assert_eq!(full.len(), dirty.len());
        assert_eq!(full.len(), 1);
        assert_eq!(full[0], dirty[0]);
    }

    #[test]
    fn back_hash_changes_on_mutation() {
        let mut buf = Buffer::new(8, 8);
        buf.fill(Color::Black);
        let h1 = buf.back_hash();
        buf.set(4, 4, 'Q', Color::White, Color::Red);
        let h2 = buf.back_hash();
        assert_ne!(h1, h2, "hash must change when buffer content changes");
    }

    #[test]
    fn back_hash_stable_for_same_content() {
        let mut buf = Buffer::new(8, 8);
        buf.fill(Color::Rgb { r: 42, g: 42, b: 42 });
        let h1 = buf.back_hash();
        let h2 = buf.back_hash();
        assert_eq!(h1, h2, "hash must be deterministic for same content");
    }

    // ── mark_dirty_region tests ──────────────────────────────────

    #[test]
    fn mark_dirty_region_expands_bounds() {
        let mut buf = Buffer::new(80, 40);
        buf.fill(Color::Reset);
        buf.reset_dirty();
        assert!(buf.dirty_bounds().is_none(), "no dirty region after reset");

        buf.mark_dirty_region(10, 5, 20, 10);
        let (xmin, xmax, ymin, ymax) = buf.dirty_bounds().unwrap();
        assert_eq!((xmin, xmax, ymin, ymax), (10, 29, 5, 14));
    }

    #[test]
    fn mark_dirty_region_merges_with_existing() {
        let mut buf = Buffer::new(80, 40);
        buf.fill(Color::Reset);
        buf.reset_dirty();

        buf.set(5, 5, 'X', Color::White, Color::Black);
        buf.mark_dirty_region(20, 20, 10, 10);
        let (xmin, xmax, ymin, ymax) = buf.dirty_bounds().unwrap();
        assert_eq!(xmin, 5);
        assert_eq!(ymin, 5);
        assert_eq!(xmax, 29);
        assert_eq!(ymax, 29);
    }

    #[test]
    fn mark_dirty_region_clamps_to_buffer_size() {
        let mut buf = Buffer::new(20, 10);
        buf.fill(Color::Reset);
        buf.reset_dirty();

        buf.mark_dirty_region(15, 5, 100, 100);
        let (_, xmax, _, ymax) = buf.dirty_bounds().unwrap();
        assert_eq!(xmax, 19);
        assert_eq!(ymax, 9);
    }

    #[test]
    fn mark_dirty_region_noop_on_zero_size() {
        let mut buf = Buffer::new(20, 10);
        buf.fill(Color::Reset);
        buf.reset_dirty();

        buf.mark_dirty_region(5, 5, 0, 0);
        assert!(buf.dirty_bounds().is_none());
    }
}

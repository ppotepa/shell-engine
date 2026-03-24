use crossterm::style::Color;

/// Controls how diff tuples are written to the terminal.
///
/// The default `AnsiBatchFlusher` batches consecutive same-colour cells into
/// a single MoveTo+SetColor+Print run, minimising I/O system calls.
/// Additional implementations (NaiveFlusher, SixelFlusher, etc.) can be
/// selected at startup without touching the renderer.
pub trait TerminalFlusher: Send + Sync {
    fn flush(
        &self,
        stdout: &mut std::io::BufWriter<std::io::Stdout>,
        diffs: &[(u16, u16, char, Color, Color)],
    );
}

/// One command per cell — no batching. Useful as a correctness reference or debug sink.
/// Always produces correct output regardless of diff ordering.
pub struct NaiveFlusher;

impl TerminalFlusher for NaiveFlusher {
    fn flush(
        &self,
        stdout: &mut std::io::BufWriter<std::io::Stdout>,
        diffs: &[(u16, u16, char, Color, Color)],
    ) {
        use crossterm::{cursor, queue, style};
        use std::io::Write;
        for &(x, y, ch, raw_fg, raw_bg) in diffs {
            let fg = crate::systems::renderer::resolve_color(raw_fg);
            let bg = crate::systems::renderer::resolve_color(raw_bg);
            let _ = queue!(
                stdout,
                cursor::MoveTo(x, y),
                style::SetForegroundColor(fg),
                style::SetBackgroundColor(bg),
                style::Print(ch)
            );
        }
        let _ = stdout.flush();
    }
}

/// The default high-performance ANSI batch flusher.
/// Consecutive cells on the same row sharing the same fg+bg are merged into a
/// single MoveTo+SetFg+SetBg+Print(run) command.
///
/// Delegates to `crate::systems::renderer::flush_batched` which owns the
/// thread-local `ANSI_BUF` / `RUN_BUF` scratch allocations.
pub struct AnsiBatchFlusher;

impl TerminalFlusher for AnsiBatchFlusher {
    fn flush(
        &self,
        stdout: &mut std::io::BufWriter<std::io::Stdout>,
        diffs: &[(u16, u16, char, Color, Color)],
    ) {
        crate::systems::renderer::flush_batched(stdout, diffs);
    }
}

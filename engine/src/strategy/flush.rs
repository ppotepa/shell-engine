use crossterm::style::Color;

/// Controls how diff tuples are written to the terminal.
///
/// The default `AnsiBatchFlusher` batches consecutive same-colour cells into
/// a single MoveTo+SetColor+Print run, minimising I/O system calls.
/// Additional implementations (NaiveFlusher, SixelFlusher, etc.) can be
/// selected at startup without touching the renderer.
pub trait TerminalFlusher: Send + Sync {
    /// Flush a slice of `(x, y, char, fg, bg)` diffs to the terminal.
    /// The diffs arrive in row-major order; implementations may rely on this.
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
        for &(x, y, ch, fg, bg) in diffs {
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
/// This is a marker type — the actual hot-path implementation lives in
/// `renderer::flush_batched` to keep the thread-locals collocated.
/// Systems that receive `Box<dyn TerminalFlusher>` call `flush_batched` when
/// they detect `AnsiBatchFlusher`.
pub struct AnsiBatchFlusher;

impl TerminalFlusher for AnsiBatchFlusher {
    fn flush(
        &self,
        stdout: &mut std::io::BufWriter<std::io::Stdout>,
        diffs: &[(u16, u16, char, Color, Color)],
    ) {
        // Delegates to the optimised renderer implementation.
        // Imported as a free function in the renderer where thread-locals live.
        // This fallback should never be called directly; it is here for completeness.
        NaiveFlusher.flush(stdout, diffs);
    }
}

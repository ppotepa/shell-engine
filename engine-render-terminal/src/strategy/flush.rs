use crossterm::style::Color;
use engine_pipeline::TerminalFlusher;

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
            let fg = crate::renderer::resolve_color(raw_fg);
            let bg = crate::renderer::resolve_color(raw_bg);
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

/// High-performance ANSI batch flusher.
pub struct AnsiBatchFlusher;

impl TerminalFlusher for AnsiBatchFlusher {
    fn flush(
        &self,
        stdout: &mut std::io::BufWriter<std::io::Stdout>,
        diffs: &[(u16, u16, char, Color, Color)],
    ) {
        crate::renderer::flush_batched(stdout, diffs);
    }
}

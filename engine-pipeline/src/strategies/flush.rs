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

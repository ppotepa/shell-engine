//! Trait for terminal cell-flush strategies.

use engine_core::color::Color;
use std::io::BufWriter;
use std::io::Stdout;

/// Flushes a set of dirty cell diffs to the terminal.
///
/// Implementations decide how to batch/order ANSI escape sequences.
pub trait TerminalFlusher: Send {
    fn flush(&self, stdout: &mut BufWriter<Stdout>, diffs: &[(u16, u16, char, Color, Color)]);
}

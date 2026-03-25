use crossterm::style::Color;

/// Immutable snapshot of diff ready for async display/flush.
/// Handoff from main thread to display thread (if async mode is active).
#[derive(Debug, Clone)]
pub struct DisplayFrame {
    /// Raw diff tuples: (x, y, char, fg, bg)
    pub diffs: Vec<(u16, u16, char, Color, Color)>,
    /// Frame identifier for coordination (can be ignored in simple mode)
    pub frame_id: u64,
}

/// Controls how diff snapshots are flushed to terminal output.
///
/// `SyncDisplaySink` (default) flushes inline on the main thread.
/// `AsyncDisplaySink` queues diffs to a background thread for batched flushing.
pub trait DisplaySink: Send + Sync {
    /// Queue a display frame for flushing (now or later).
    fn submit(&mut self, frame: DisplayFrame);
    /// Drain pending frames and shutdown (call on engine shutdown).
    fn drain(&mut self);
}

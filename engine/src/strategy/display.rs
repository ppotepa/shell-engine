use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;
use std::io;
use crate::pipeline::{DisplaySink, DisplayFrame};

/// Default: flushes synchronously on the main thread.
pub struct SyncDisplaySink;

impl DisplaySink for SyncDisplaySink {
    fn submit(&mut self, frame: DisplayFrame) {
        // Flush immediately via the main thread renderer
        if !frame.diffs.is_empty() {
            let mut stdout = io::BufWriter::new(io::stdout());
            crate::systems::renderer::flush_batched(&mut stdout, &frame.diffs);
        }
    }

    fn drain(&mut self) {
        // No-op: sync flusher has no pending state
    }
}

/// Experimental: queues diffs to a background thread via mpsc.
/// Main thread submits frames; display thread dequeues and flushes.
/// Allows main thread to start next frame's compositor while display thread I/O completes.
pub struct AsyncDisplaySink {
    tx: Option<Sender<DisplayFrame>>,
    _thread: Option<JoinHandle<()>>,
}

impl AsyncDisplaySink {
    /// Spawn the background display thread and return the sink.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<DisplayFrame>();
        let thread = std::thread::spawn(move || {
            let mut stdout = io::BufWriter::new(io::stdout());
            while let Ok(frame) = rx.recv() {
                if !frame.diffs.is_empty() {
                    crate::systems::renderer::flush_batched(&mut stdout, &frame.diffs);
                }
            }
            // Channel closed, thread exits
        });
        Self {
            tx: Some(tx),
            _thread: Some(thread),
        }
    }
}

impl DisplaySink for AsyncDisplaySink {
    fn submit(&mut self, frame: DisplayFrame) {
        // Queue to background thread (non-blocking, unless channel is full)
        if let Some(ref tx) = self.tx {
            let _ = tx.send(frame);
        }
    }

    fn drain(&mut self) {
        // Drop the sender to close the channel and signal EOF to the receiver.
        // This allows the display thread's recv() loop to exit.
        self.tx.take();
        // Wait for thread to finish draining queue and exit
        if let Some(thread) = self._thread.take() {
            let _ = thread.join();
        }
    }
}

impl Default for AsyncDisplaySink {
    fn default() -> Self {
        Self::new()
    }
}

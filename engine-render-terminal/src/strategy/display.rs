use engine_pipeline::{DisplayFrame, DisplaySink};
use std::io;
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;

/// Default: flushes synchronously on the main thread.
pub struct SyncDisplaySink;

impl DisplaySink for SyncDisplaySink {
    fn submit(&mut self, frame: DisplayFrame) {
        if !frame.diffs.is_empty() {
            let mut stdout = io::BufWriter::new(io::stdout());
            crate::renderer::flush_batched(&mut stdout, &frame.diffs);
        }
    }

    fn drain(&mut self) {}
}

/// Experimental: queues diffs to a background thread via mpsc.
/// Main thread submits frames; display thread dequeues and flushes.
pub struct AsyncDisplaySink {
    tx: Option<Sender<DisplayFrame>>,
    _thread: Option<JoinHandle<()>>,
}

impl AsyncDisplaySink {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<DisplayFrame>();
        let thread = std::thread::spawn(move || {
            let mut stdout = io::BufWriter::new(io::stdout());
            while let Ok(frame) = rx.recv() {
                if !frame.diffs.is_empty() {
                    crate::renderer::flush_batched(&mut stdout, &frame.diffs);
                }
            }
        });
        Self {
            tx: Some(tx),
            _thread: Some(thread),
        }
    }

    /// Drain all pending frames (wait for background thread to flush them),
    /// then reopen a fresh channel for continued use.
    /// Call before writing directly to stdout (e.g., overlays) to prevent
    /// interleaving with in-flight background flushes to the same fd.
    pub fn drain_and_reopen(&mut self) {
        self.tx.take(); // Close sender → background thread will finish and exit
        if let Some(thread) = self._thread.take() {
            let _ = thread.join(); // Wait for all pending frames to flush
        }
        // Reopen for subsequent frames
        let (tx, rx) = mpsc::channel::<DisplayFrame>();
        let thread = std::thread::spawn(move || {
            let mut stdout = io::BufWriter::new(io::stdout());
            while let Ok(frame) = rx.recv() {
                if !frame.diffs.is_empty() {
                    crate::renderer::flush_batched(&mut stdout, &frame.diffs);
                }
            }
        });
        self.tx = Some(tx);
        self._thread = Some(thread);
    }
}

impl DisplaySink for AsyncDisplaySink {
    fn submit(&mut self, frame: DisplayFrame) {
        if let Some(ref tx) = self.tx {
            let _ = tx.send(frame);
        }
    }

    fn drain(&mut self) {
        self.tx.take();
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

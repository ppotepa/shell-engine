//! Lock-free command channels for worker thread communication.
//!
//! Uses crossbeam for high-performance MPSC/MPMC channels.

use crossbeam_channel::{bounded, unbounded, Receiver, Sender, TryRecvError};

/// Sender end of a command channel.
pub struct CommandSender<T> {
    pub(crate) inner: Sender<T>,
}

impl<T> CommandSender<T> {
    /// Send a command (non-blocking for unbounded, may block for bounded).
    pub fn send(&self, cmd: T) -> Result<(), T> {
        self.inner.send(cmd).map_err(|e| e.0)
    }

    /// Try to send without blocking.
    pub fn try_send(&self, cmd: T) -> Result<(), T> {
        self.inner.try_send(cmd).map_err(|e| match e {
            crossbeam_channel::TrySendError::Full(v) => v,
            crossbeam_channel::TrySendError::Disconnected(v) => v,
        })
    }
}

impl<T> Clone for CommandSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Receiver end of a command channel.
pub struct CommandReceiver<T> {
    pub(crate) inner: Receiver<T>,
}

impl<T> CommandReceiver<T> {
    /// Blocking receive.
    pub fn recv(&self) -> Option<T> {
        self.inner.recv().ok()
    }

    /// Non-blocking receive.
    pub fn try_recv(&self) -> Option<T> {
        match self.inner.try_recv() {
            Ok(v) => Some(v),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }

    /// Drain all available messages without blocking.
    pub fn drain(&self) -> Vec<T> {
        let mut results = Vec::new();
        while let Some(v) = self.try_recv() {
            results.push(v);
        }
        results
    }

    /// Check if channel is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Number of messages in channel.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Create an unbounded command channel (never blocks on send).
pub fn command_channel<T>() -> (CommandSender<T>, CommandReceiver<T>) {
    let (tx, rx) = unbounded();
    (CommandSender { inner: tx }, CommandReceiver { inner: rx })
}

/// Create a bounded command channel with capacity.
pub fn bounded_channel<T>(capacity: usize) -> (CommandSender<T>, CommandReceiver<T>) {
    let (tx, rx) = bounded(capacity);
    (CommandSender { inner: tx }, CommandReceiver { inner: rx })
}

/// Physics command sent to worker thread.
#[derive(Clone, Debug)]
pub enum PhysicsCommand {
    /// Process physics for this frame.
    Step {
        dt_ms: u64,
        /// Packed physics data to process.
        items: Vec<PhysicsWorkItem>,
    },
    /// Shutdown the worker.
    Shutdown,
}

/// Single physics work item (all data needed for one entity).
#[derive(Clone, Debug)]
pub struct PhysicsWorkItem {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub heading: f32,
    pub vx: f32,
    pub vy: f32,
    pub ax: f32,
    pub ay: f32,
    pub drag: f32,
    pub max_speed: f32,
    pub gravity_scale: f32,
}

/// Result of physics computation.
#[derive(Clone, Debug)]
pub struct PhysicsResultItem {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_channel() {
        let (tx, rx) = command_channel::<i32>();
        
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();
        
        let drained = rx.drain();
        assert_eq!(drained, vec![1, 2, 3]);
        assert!(rx.is_empty());
    }
}

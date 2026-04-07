use crate::kernel::unit_of_work::ScheduledLine;

/// A pending output line with its due time.
pub struct PendingLine {
    pub due_at_ms: u64,
    pub text: String,
}

/// Stack-based application manager with delayed output drain.
pub struct AppStack {
    pending: Vec<PendingLine>,
}

impl Default for AppStack {
    fn default() -> Self {
        Self::new()
    }
}

impl AppStack {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    /// Enqueue scheduled lines from a command execution.
    pub fn enqueue(&mut self, _base_ms: u64, lines: Vec<ScheduledLine>) {
        for line in lines {
            self.pending.push(PendingLine {
                due_at_ms: line.due_ms,
                text: line.text,
            });
        }
        // Keep sorted by due time
        self.pending.sort_by_key(|p| p.due_at_ms);
    }

    /// Drain all lines that are due by `now_ms`. Returns them in order.
    pub fn drain_ready(&mut self, now_ms: u64) -> Vec<String> {
        let mut ready = Vec::new();
        let mut i = 0;
        while i < self.pending.len() {
            if self.pending[i].due_at_ms <= now_ms {
                ready.push(self.pending.remove(i).text);
            } else {
                i += 1;
            }
        }
        ready
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
}

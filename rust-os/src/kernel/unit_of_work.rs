use crate::session::UserSession;
use crate::state::QuestState;

/// Accumulated output with per-line delays.
pub struct ScheduledLine {
    pub due_ms: u64,
    pub text: String,
}

/// Per-command execution scope. Accumulates delayed output lines.
pub struct UnitOfWork<'a> {
    pub session: &'a mut UserSession,
    pub quest: &'a mut QuestState,
    pub base_time_ms: u64,
    pending_delay_ms: u64,
    scheduled: Vec<ScheduledLine>,
    pub exit_requested: bool,
}

impl<'a> UnitOfWork<'a> {
    pub fn new(session: &'a mut UserSession, quest: &'a mut QuestState, base_time_ms: u64) -> Self {
        Self {
            session,
            quest,
            base_time_ms,
            pending_delay_ms: 0,
            scheduled: Vec::new(),
            exit_requested: false,
        }
    }

    /// Schedule a line to appear after cumulative delay.
    pub fn schedule(&mut self, text: impl Into<String>, delay_ms: u64) {
        self.pending_delay_ms += delay_ms;
        self.scheduled.push(ScheduledLine {
            due_ms: self.base_time_ms + self.pending_delay_ms,
            text: text.into(),
        });
    }

    /// Immediate output (delay 0).
    pub fn print(&mut self, text: impl Into<String>) {
        self.schedule(text, 0);
    }

    /// Drain all scheduled lines (called by ApplicationStack).
    pub fn drain(&mut self) -> Vec<ScheduledLine> {
        std::mem::take(&mut self.scheduled)
    }

    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }
}

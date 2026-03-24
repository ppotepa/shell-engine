use crate::exec::MinixPipeline;
use crate::hosts::RemoteHostIndex;
use crate::kernel::unit_of_work::ScheduledLine;
use crate::kernel::Kernel;
use crate::session::UserSession;
use crate::state::QuestState;
use crate::style;

pub struct ShellApp {
    pipeline: MinixPipeline,
}

impl ShellApp {
    pub fn new() -> Self {
        Self {
            pipeline: MinixPipeline::new(),
        }
    }

    pub fn prompt(&self, session: &UserSession, exit_code: i32) -> String {
        let user = style::fg(style::PROMPT_USER, &session.user);
        let at = style::fg(style::DIM, "@");
        let host = style::fg(style::PROMPT_HOST, &session.hostname);
        let colon = style::fg(style::DIM, ":");
        let cwd = style::fg(style::PROMPT_PATH, &session.display_cwd());
        let code_color = if exit_code == 0 { style::BOOT_OK } else { style::ERROR };
        let code = style::fg(code_color, &format!("[{exit_code}]"));
        format!("{user}{at}{host}{colon}{cwd} {code}$ ")
    }

    /// Execute a line. Returns (exit_requested, scheduled_lines).
    pub fn handle_input(
        &self,
        input: &str,
        session: &mut UserSession,
        quest: &mut QuestState,
        kernel: &mut Kernel,
        host_index: &RemoteHostIndex,
    ) -> (bool, Vec<ScheduledLine>) {
        // Check for ping command to route through host registry
        let trimmed = input.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts.first() == Some(&"ping") && parts.len() >= 2 {
            let base_time = kernel.uptime_ms();
            let mut uow = crate::kernel::unit_of_work::UnitOfWork::new(session, quest, base_time);
            crate::commands::net::ping_with_registry(parts[1], host_index, &mut uow, kernel);
            let lines = uow.drain();
            return (false, lines);
        }

        self.pipeline.execute(input, session, quest, kernel)
    }
}

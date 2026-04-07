use crate::kernel::unit_of_work::{ScheduledLine, UnitOfWork};
use crate::kernel::Kernel;
use crate::session::UserSession;
use crate::state::QuestState;

pub struct MailApp {
    current_index: Option<usize>,
}

impl Default for MailApp {
    fn default() -> Self {
        Self::new()
    }
}

impl MailApp {
    pub fn new() -> Self {
        Self {
            current_index: None,
        }
    }

    pub fn prompt(&self) -> &str {
        "& "
    }

    pub fn on_enter(
        &mut self,
        session: &mut UserSession,
        quest: &mut QuestState,
        kernel: &mut Kernel,
    ) -> Vec<ScheduledLine> {
        let base = kernel.uptime_ms();
        let user = session.user.clone();
        let messages = kernel.mail.list();
        let msg_count = messages.len();
        let unread = messages.iter().filter(|m| !m.read).count();
        let msg_lines: Vec<String> = messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let flag = if m.read { " " } else { "N" };
                let date_short = if m.date.len() >= 16 {
                    &m.date[..16]
                } else {
                    &m.date
                };
                format!(
                    "{}{:3}  {}  {:<20} {}",
                    flag,
                    i + 1,
                    date_short,
                    m.from,
                    m.subject
                )
            })
            .collect();

        let mut uow = UnitOfWork::new(session, quest, base);
        if msg_count == 0 {
            uow.print("No mail for torvalds.");
        } else {
            uow.print("Mail version 2.12 6/28/83.  Type ? for help.".to_string());
            uow.print(format!(
                "\"/var/spool/mail/{user}\": {msg_count} messages {unread} new"
            ));
            for line in msg_lines {
                uow.print(line);
            }
        }
        uow.drain()
    }

    pub fn handle_input(
        &mut self,
        input: &str,
        session: &mut UserSession,
        quest: &mut QuestState,
        kernel: &mut Kernel,
    ) -> (bool, Vec<ScheduledLine>) {
        let base = kernel.uptime_ms();
        let mut uow = UnitOfWork::new(session, quest, base);
        let trimmed = input.trim();

        match trimmed {
            "q" | "x" | "quit" | "exit" => {
                uow.request_exit();
            }
            "?" | "h" | "help" => {
                uow.print("Mail commands:");
                uow.print("  p [n]   print message n (or current)");
                uow.print("  d [n]   delete message n");
                uow.print("  q       quit");
            }
            s if s.chars().all(|c| c.is_ascii_digit()) => {
                // Read message by number
                if let Ok(n) = s.parse::<usize>() {
                    let idx = n.saturating_sub(1);
                    if let Some(msg) = kernel.mail.list().get(idx) {
                        let text = format!(
                            "From {} {}\nSubject: {}\n\n{}",
                            msg.from, msg.date, msg.subject, msg.body
                        );
                        for line in text.lines() {
                            uow.print(line.to_string());
                        }
                        kernel.mail.mark_read(idx);
                        self.current_index = Some(idx);
                    } else {
                        uow.print(format!("No applicable message {n}"));
                    }
                }
            }
            "p" => {
                // Print current/next
                let idx = self.current_index.map(|i| i + 1).unwrap_or(0);
                if let Some(msg) = kernel.mail.list().get(idx) {
                    let text = format!(
                        "From {} {}\nSubject: {}\n\n{}",
                        msg.from, msg.date, msg.subject, msg.body
                    );
                    for line in text.lines() {
                        uow.print(line.to_string());
                    }
                    kernel.mail.mark_read(idx);
                    self.current_index = Some(idx);
                } else {
                    uow.print("At EOF.".to_string());
                }
            }
            "" => {
                // advance to next
                let idx = self.current_index.map(|i| i + 1).unwrap_or(0);
                if let Some(msg) = kernel.mail.list().get(idx) {
                    let text = format!(
                        "From {} {}\nSubject: {}\n\n{}",
                        msg.from, msg.date, msg.subject, msg.body
                    );
                    for line in text.lines() {
                        uow.print(line.to_string());
                    }
                    kernel.mail.mark_read(idx);
                    self.current_index = Some(idx);
                } else if self.current_index.is_none() {
                    // nothing to show
                } else {
                    uow.print("At EOF.".to_string());
                }
            }
            _ => {
                uow.print(format!("Unknown command: {trimmed}. Type ? for help."));
            }
        }

        let exit = uow.exit_requested;
        let lines = uow.drain();
        (exit, lines)
    }
}

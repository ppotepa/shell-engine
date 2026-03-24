use engine_io::{IoEvent, IoRequest};
use crate::app_stack::AppStack;
use crate::apps::{ShellApp, FtpApp, MailApp};
use crate::boot;
use crate::difficulty::{Difficulty, MachineSpec};
use crate::hosts::RemoteHostIndex;
use crate::kernel::Kernel;
use crate::protocol;
use crate::screen_buffer::ScreenBuffer;
use crate::session::UserSession;
use crate::state::{MachineState, QuestState, SessionMode};
use crate::style;
use crate::vfs::Vfs;

enum ActiveApp {
    Shell(ShellApp),
    Ftp(FtpApp),
    Mail(MailApp),
}

pub struct AppHost {
    kernel: Kernel,
    state: MachineState,
    session: Option<UserSession>,
    screen: ScreenBuffer,
    app_stack: AppStack,
    host_index: RemoteHostIndex,
    active_app: Option<ActiveApp>,
    mode: SessionMode,
    input_line: String,
    // Timed mail delivery tracking
    mail_delivered_ftp_hint: bool,
    mail_delivered_mailer_daemon: bool,
    mail_delivered_stroustrup: bool,
}

impl AppHost {
    pub fn new(spec: MachineSpec) -> Self {
        let vfs = Vfs::new();
        let kernel = Kernel::new(spec, vfs);
        let state = MachineState::new();
        let host_index = RemoteHostIndex::build();

        Self {
            kernel,
            state,
            session: None,
            screen: ScreenBuffer::new(120, 40),
            app_stack: AppStack::new(),
            host_index,
            active_app: None,
            mode: SessionMode::Booting,
            input_line: String::new(),
            mail_delivered_ftp_hint: false,
            mail_delivered_mailer_daemon: false,
            mail_delivered_stroustrup: false,
        }
    }

    /// Process one IoRequest. Returns zero or more IoEvents to send back to the engine.
    pub fn handle(&mut self, req: IoRequest) -> Vec<IoEvent> {
        match req {
            IoRequest::Hello { cols, rows, boot_scene, difficulty } => {
                self.handle_hello(cols, rows, boot_scene, difficulty)
            }
            IoRequest::Tick { dt_ms } => {
                self.handle_tick(dt_ms)
            }
            IoRequest::Resize { cols, rows } => {
                self.screen.set_viewport(cols as usize, rows as usize);
                vec![self.screen.send_frame()]
            }
            IoRequest::Submit { line } => {
                self.handle_submit(&line)
            }
            IoRequest::SetInput { text } => {
                self.input_line = text.clone();
                self.screen.set_input_line(&text);
                vec![self.screen.send_frame()]
            }
            IoRequest::Key { .. } => vec![],
        }
    }

    fn handle_hello(
        &mut self,
        cols: u16, rows: u16,
        boot_scene: bool,
        difficulty: Option<String>,
    ) -> Vec<IoEvent> {
        // Re-initialize with the given difficulty
        let diff = difficulty.as_deref()
            .map(Difficulty::from_label)
            .unwrap_or(Difficulty::ICanExitVim);
        let spec = MachineSpec::from_difficulty(diff);
        let vfs = Vfs::new();
        self.kernel = Kernel::new(spec, vfs);
        self.host_index = RemoteHostIndex::build();
        self.screen = ScreenBuffer::new(cols as usize, rows as usize);
        self.app_stack = AppStack::new();

        let mut events = Vec::new();

        if boot_scene {
            self.mode = SessionMode::Booting;
            let steps = boot::build_boot_steps(&self.kernel.spec);
            // Enqueue boot steps as scheduled lines
            for step in steps {
                self.app_stack.enqueue(0, vec![step]);
            }
        } else {
            self.mode = SessionMode::LoginUser;
            let motd = self.kernel.vfs.read_file("/etc/motd")
                .map(|s| s.to_string())
                .unwrap_or_default();
            for line in motd.lines() {
                self.screen.append(&[line.to_string()]);
            }
            self.screen.append(&["".to_string()]);
            self.screen.set_prompt("kruuna login: ");
            events.push(protocol::set_prompt("kruuna login: "));
        }

        events.push(self.screen.send_frame());
        events
    }

    fn handle_tick(&mut self, dt_ms: u64) -> Vec<IoEvent> {
        self.kernel.tick(dt_ms);
        let now = self.kernel.uptime_ms();

        // Drain delayed output
        let ready = self.app_stack.drain_ready(now);
        if !ready.is_empty() {
            let lines: Vec<String> = ready.clone();
            self.screen.append(&lines);
        }

        // After boot finishes (no more pending), switch to login
        if self.mode == SessionMode::Booting && !self.app_stack.has_pending() && now > 0 {
            // Check if boot is done (all lines delivered)
            self.mode = SessionMode::LoginUser;
            self.screen.set_prompt("kruuna login: ");
            let mut events = vec![protocol::set_prompt("kruuna login: ")];
            events.push(self.screen.send_frame());
            return events;
        }

        // Timed mail delivery
        self.check_timed_mail();

        if !ready.is_empty() {
            vec![self.screen.send_frame()]
        } else {
            vec![]
        }
    }

    fn handle_submit(&mut self, input: &str) -> Vec<IoEvent> {
        let mut events = Vec::new();

        match &self.mode {
            SessionMode::Booting => {}
            SessionMode::LoginUser => {
                self.state.pending_login_user = input.to_string();
                self.screen.commit_input_line();
                self.mode = SessionMode::LoginPassword;
                self.screen.set_prompt("Password: ");
                events.push(protocol::set_prompt("Password: "));
                events.push(protocol::set_masked(true));
                events.push(self.screen.send_frame());
            }
            SessionMode::LoginPassword => {
                self.screen.commit_input_line();
                let user = self.state.pending_login_user.clone();
                // Accept any password for the game
                self.enter_shell(&user, &mut events);
            }
            SessionMode::Shell => {
                self.screen.commit_input_line();
                events.push(protocol::set_masked(false));
                self.handle_shell_input(input, &mut events);
            }
        }

        events
    }

    fn enter_shell(&mut self, user: &str, events: &mut Vec<IoEvent>) {
        events.push(protocol::set_masked(false));
        let session = UserSession::new(user, "kruuna");
        self.session = Some(session);
        self.mode = SessionMode::Shell;

        // Clear screen on fresh login
        self.screen.clear();
        events.push(protocol::clear());

        // Show MOTD
        let motd = self.kernel.vfs.read_file("/etc/motd")
            .map(|s| s.to_string())
            .unwrap_or_default();
        let mut lines: Vec<String> = motd.lines().map(|s| s.to_string()).collect();
        lines.push("".to_string());

        // Show mail count
        let unread = self.kernel.mail.unread_count();
        if unread > 0 {
            lines.push(format!("You have {unread} new message{}.", if unread == 1 { "" } else { "s" }));
            lines.push("".to_string());
        }

        self.screen.append(&lines);
        self.active_app = Some(ActiveApp::Shell(ShellApp::new()));
        self.update_prompt(events);
        events.push(self.screen.send_frame());
    }

    fn handle_shell_input(&mut self, input: &str, events: &mut Vec<IoEvent>) {
        if self.session.is_none() { return; }

        // Dispatch to active sub-app if one is running
        match self.active_app.take() {
            Some(ActiveApp::Ftp(mut ftp)) => {
                let session = self.session.as_mut().unwrap();
                let quest = &mut self.state.quest;
                let (exit, lines) = ftp.handle_input(input, session, quest, &mut self.kernel);
                let now = self.kernel.uptime_ms();
                self.app_stack.enqueue(now, lines);
                let immediate = self.app_stack.drain_ready(now);
                if !immediate.is_empty() { self.screen.append(&immediate); }
                if exit {
                    self.active_app = Some(ActiveApp::Shell(ShellApp::new()));
                } else {
                    self.active_app = Some(ActiveApp::Ftp(ftp));
                }
                self.update_prompt(events);
                events.push(self.screen.send_frame());
                return;
            }
            Some(ActiveApp::Mail(mut mail)) => {
                let session = self.session.as_mut().unwrap();
                let quest = &mut self.state.quest;
                let (exit, lines) = mail.handle_input(input, session, quest, &mut self.kernel);
                let now = self.kernel.uptime_ms();
                self.app_stack.enqueue(now, lines);
                let immediate = self.app_stack.drain_ready(now);
                if !immediate.is_empty() { self.screen.append(&immediate); }
                if exit {
                    self.active_app = Some(ActiveApp::Shell(ShellApp::new()));
                } else {
                    self.active_app = Some(ActiveApp::Mail(mail));
                }
                self.update_prompt(events);
                events.push(self.screen.send_frame());
                return;
            }
            other => {
                self.active_app = other;
            }
        }

        // Check for app-switching commands
        let trimmed = input.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts.first().copied().unwrap_or("");

        match cmd {
            "ftp" if parts.len() >= 1 => {
                let mut ftp = FtpApp::new();
                // If host given, auto-open
                if let Some(host) = parts.get(1) {
                    let host = host.to_string();
                    let session = self.session.as_mut().unwrap();
                    let quest = &mut self.state.quest;
                    let (_, lines) = ftp.handle_input(&format!("open {host}"), session, quest, &mut self.kernel);
                    self.app_stack.enqueue(self.kernel.uptime_ms(), lines);
                }
                self.active_app = Some(ActiveApp::Ftp(ftp));
                self.update_prompt(events);
                events.push(self.screen.send_frame());
                return;
            }
            "mail" => {
                let mut mail_app = MailApp::new();
                let session = self.session.as_mut().unwrap();
                let quest = &mut self.state.quest;
                let enter_lines = mail_app.on_enter(session, quest, &mut self.kernel);
                let now = self.kernel.uptime_ms();
                self.app_stack.enqueue(now, enter_lines);
                let immediate = self.app_stack.drain_ready(now);
                if !immediate.is_empty() {
                    self.screen.append(&immediate);
                }
                self.active_app = Some(ActiveApp::Mail(mail_app));
                self.update_prompt(events);
                events.push(self.screen.send_frame());
                return;
            }
            _ => {}
        }

        // Normal shell execution
        let mut app = match self.active_app.take() {
            Some(ActiveApp::Shell(s)) => s,
            _ => ShellApp::new(),
        };

        let session = self.session.as_mut().unwrap();
        let quest = &mut self.state.quest;
        let (exit, lines) = app.handle_input(input, session, quest, &mut self.kernel, &self.host_index);
        let now = self.kernel.uptime_ms();
        self.app_stack.enqueue(now, lines);
        // Drain immediate (0-delay) outputs right away
        let immediate = self.app_stack.drain_ready(now);
        if !immediate.is_empty() {
            self.screen.append(&immediate);
        }

        if exit {
            // Shell exit = logout
            self.mode = SessionMode::LoginUser;
            self.session = None;
            self.active_app = None;
            self.screen.append(&["".to_string(), "logout".to_string(), "".to_string()]);
            self.screen.set_prompt("kruuna login: ");
            events.push(protocol::set_prompt("kruuna login: "));
        } else {
            self.active_app = Some(ActiveApp::Shell(app));
        }

        self.update_prompt(events);
        events.push(self.screen.send_frame());
    }

    fn update_prompt(&mut self, events: &mut Vec<IoEvent>) {
        let prompt = match &self.active_app {
            Some(ActiveApp::Shell(s)) => {
                self.session.as_ref().map(|session| {
                    s.prompt(session, session.last_exit_code)
                })
            }
            Some(ActiveApp::Ftp(f)) => Some(f.prompt()),
            Some(ActiveApp::Mail(m)) => Some(m.prompt().to_string()),
            None => None,
        };
        if let Some(p) = prompt {
            self.screen.set_prompt(&p);
            events.push(protocol::set_prompt(&p));
        }
    }

    fn check_timed_mail(&mut self) {
        let quest = &self.state.quest;
        let now = self.kernel.uptime_ms();

        // After first failed FTP upload, deliver ast hint
        if !self.mail_delivered_ftp_hint
            && quest.upload_attempted
            && !quest.upload_success
        {
            self.mail_delivered_ftp_hint = true;
            self.kernel.mail.deliver(
                "ast@cs.vu.nl",
                "torvalds@kruuna",
                "Re: file upload",
                "Linus,\n\nI noticed you tried to upload. Remember: binary mode.\nCompressed archives are binary. ASCII transfer corrupts them.\n\n— ast",
                "Mon, 16 Sep 1991 21:44:00 +0200",
            );
        }

        // After 5+ anomalies, deliver MAILER-DAEMON
        if !self.mail_delivered_mailer_daemon && quest.anomaly_count() >= 5 {
            self.mail_delivered_mailer_daemon = true;
            self.kernel.mail.deliver(
                "MAILER-DAEMON",
                "torvalds@kruuna",
                "Mail delivery failed",
                "The following message to <future@void.null> was undeliverable.\nReason: host unreachable (route exists in 2024 only)\n\nOriginal message headers:\nFrom: torvalds@kruuna\nTo: future@void.null\nSubject: kernel upload complete",
                "Mon, 16 Sep 1991 21:52:00 +0300",
            );
        }

        // After successful upload, deliver congratulations
        if !self.mail_delivered_stroustrup && quest.upload_success {
            self.mail_delivered_stroustrup = true;
            self.kernel.mail.deliver(
                "bs@research.att.com",
                "torvalds@kruuna",
                "Congratulations",
                "Mr. Torvalds,\n\nI heard you uploaded something interesting today.\nC++ might run on it someday.\n\n— B. Stroustrup",
                "Mon, 16 Sep 1991 22:10:00 -0500",
            );
        }
    }
}

use crate::kernel::unit_of_work::{ScheduledLine, UnitOfWork};
use crate::kernel::Kernel;
use crate::session::UserSession;
use crate::state::QuestState;
use crate::style;

#[derive(Debug, Clone, PartialEq)]
pub enum FtpState {
    Disconnected,
    Connected(String), // host name
}

pub struct FtpApp {
    pub state: FtpState,
    pub transfer_mode: String,
    pub remote_cwd: String,
    transfer_counter: u64,
}

impl FtpApp {
    pub fn new() -> Self {
        Self {
            state: FtpState::Disconnected,
            transfer_mode: "ascii".to_string(),
            remote_cwd: "/pub".to_string(),
            transfer_counter: 0,
        }
    }

    pub fn prompt(&self) -> String {
        match &self.state {
            FtpState::Disconnected => "ftp> ".to_string(),
            FtpState::Connected(host) => format!("ftp [{}]> ", style::fg(style::PROMPT_HOST, host)),
        }
    }

    /// Handle one FTP command. Returns (exit_requested, scheduled_lines).
    pub fn handle_input(
        &mut self,
        input: &str,
        session: &mut UserSession,
        quest: &mut QuestState,
        kernel: &mut Kernel,
    ) -> (bool, Vec<ScheduledLine>) {
        let base_time = kernel.uptime_ms();
        let mut uow = UnitOfWork::new(session, quest, base_time);

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts.first().copied().unwrap_or("");

        match cmd {
            "open" => {
                let host = parts.get(1).copied().unwrap_or("nic.funet.fi");
                self.handle_open(host, &mut uow, kernel);
            }
            "binary" | "bin" => {
                self.transfer_mode = "binary".to_string();
                uow.quest.ftp_transfer_mode = "binary".to_string();
                uow.print("200 Type set to I.".to_string());
            }
            "ascii" => {
                self.transfer_mode = "ascii".to_string();
                uow.quest.ftp_transfer_mode = "ascii".to_string();
                uow.print("200 Type set to A.".to_string());
            }
            "put" | "send" => {
                let file = parts.get(1).copied().unwrap_or("");
                self.handle_put(file, &mut uow, kernel);
            }
            "ls" | "dir" => {
                self.handle_ls(&mut uow, kernel);
            }
            "cd" => {
                let dir = parts.get(1).copied().unwrap_or("/pub");
                self.remote_cwd = dir.to_string();
                uow.print(format!("250 CWD command successful."));
            }
            "pwd" => {
                let cwd = self.remote_cwd.clone();
                uow.print(format!("257 \"{cwd}\" is current directory."));
            }
            "bye" | "quit" | "exit" => {
                if matches!(&self.state, FtpState::Connected(_)) {
                    kernel.modem.hangup(&mut uow);
                    uow.quest.ftp_connected = false;
                }
                uow.request_exit();
            }
            "help" | "?" => {
                uow.print("Commands: open binary ascii put ls cd pwd bye quit");
            }
            "" => {}
            _ => {
                uow.print(format!("?Invalid command"));
            }
        }

        let exit = uow.exit_requested;
        let lines = uow.drain();
        (exit, lines)
    }

    fn handle_open(&mut self, host: &str, uow: &mut UnitOfWork, kernel: &mut Kernel) {
        if matches!(&self.state, FtpState::Connected(_)) {
            uow.print("Already connected. Use close first.".to_string());
            return;
        }

        // Look up IP
        let ip = kernel
            .vfs
            .read_file("/etc/hosts")
            .and_then(|content| {
                let content = content.to_string();
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[1..].iter().any(|&n| n == host) {
                        return Some(parts[0].to_string());
                    }
                }
                None
            })
            .unwrap_or_else(|| "128.214.6.100".to_string());

        kernel.modem.dial(&ip, host, uow);
        uow.schedule("Name (torvalds):".to_string(), 200);
        uow.schedule("331 Password required.".to_string(), 300);
        uow.schedule("Password:".to_string(), 0);
        uow.schedule("230 User torvalds logged in.".to_string(), 300);
        uow.schedule(
            format!("Remote system type is UNIX. Using binary mode to transfer files."),
            0,
        );

        self.state = FtpState::Connected(host.to_string());
        uow.quest.ftp_connected = true;
        uow.quest.ftp_remote_host = Some(host.to_string());
    }

    fn handle_put(&mut self, file: &str, uow: &mut UnitOfWork, kernel: &mut Kernel) {
        if !matches!(&self.state, FtpState::Connected(_)) {
            uow.print("Not connected.".to_string());
            return;
        }
        if file.is_empty() {
            uow.print("usage: put local-file".to_string());
            return;
        }

        let local_path = uow.session.resolve_path(Some(file));
        let file_size = kernel
            .vfs
            .stat(&local_path)
            .map(|s| s.size)
            .unwrap_or(73091);

        if !kernel.vfs.exists(&local_path) {
            uow.print(format!("local: {file}: No such file or directory"));
            return;
        }

        uow.quest.upload_attempted = true;
        self.transfer_counter += 1;

        let baud = kernel.spec.modem_baud;
        let transfer_secs = (file_size as f64 / (baud as f64 / 8.0)) as u64;
        let transfer_ms = transfer_secs * 1000;
        let chunk_ms = 500u64;
        let chunks = (transfer_ms / chunk_ms).max(1);
        let bytes_per_chunk = file_size / chunks;

        uow.print(format!("local: {file}  remote: {file}"));
        uow.schedule(format!("200 PORT command successful."), 100);
        uow.schedule(
            format!("150 Opening BINARY connection for {file} ({file_size} bytes)."),
            200,
        );

        let noise = kernel.modem.noise_chance;
        let baud_str = baud.to_string();
        let mut sent = 0u64;
        for i in 0..chunks {
            sent += bytes_per_chunk;
            let pct = (sent * 100 / file_size).min(100);
            let status = format!("{sent:6} bytes ({pct:3}%) at {baud_str} baud");

            // Apply line noise deterministically
            let counter = self.transfer_counter * 100 + i;
            let roll = (counter * 6271 + 3571) % 1000;
            let threshold = (noise * 1000.0) as u64;
            let line = if roll < threshold {
                let nc = ["~", "#", "@", "^"][((counter / 3) % 4) as usize];
                format!("{status} {nc}")
            } else {
                status
            };

            uow.schedule(line, chunk_ms);
        }

        if self.transfer_mode == "binary" {
            uow.schedule(format!("226 Transfer complete."), chunk_ms);
            uow.schedule(
                format!(
                    "{file_size} bytes received in {transfer_secs:.1} seconds ({:.1} KBps)",
                    file_size as f64 / 1024.0 / transfer_secs.max(1) as f64
                ),
                0,
            );
            uow.quest.upload_success = true;
        } else {
            // ASCII mode corrupts compressed files
            uow.schedule("226 Transfer complete.".to_string(), chunk_ms);
            uow.schedule(
                format!("WARNING: ASCII mode may have corrupted binary file."),
                0,
            );
            uow.schedule(
                "Note: compressed archives MUST be transferred in binary mode.".to_string(),
                0,
            );
            uow.quest.upload_success = false;
        }
    }

    fn handle_ls(&self, uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        if !matches!(&self.state, FtpState::Connected(_)) {
            uow.print("Not connected.".to_string());
            return;
        }
        uow.schedule(format!("227 Entering Passive Mode."), 200);
        uow.schedule(
            format!("150 Opening ASCII mode data connection for file list."),
            100,
        );
        uow.schedule(format!("total 48"), 300);
        uow.schedule(format!("drwxr-xr-x  4 ftp   ftp   512 Sep 15 12:00 ."), 0);
        uow.schedule(format!("drwxr-xr-x  4 ftp   ftp   512 Sep 15 12:00 .."), 0);
        uow.schedule(
            format!("-rw-r--r--  1 ftp   ftp  4096 Sep 16 08:00 README"),
            0,
        );
        uow.schedule(format!("drwxr-xr-x  2 ftp   ftp   512 Sep 12 00:00 pub"), 0);

        if uow.quest.upload_success {
            uow.schedule(
                format!("-rw-r--r--  1 ftp   ftp 73091 Sep 17 21:30 linux-0.01.tar.Z"),
                0,
            );
        }
        uow.schedule(format!("226 Transfer complete."), 100);
    }
}

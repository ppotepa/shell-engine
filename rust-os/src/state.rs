use crate::difficulty::MachineSpec;

#[derive(Debug, Clone, PartialEq)]
pub enum SessionMode {
    Booting,
    LoginUser,
    LoginPassword,
    Shell,
}

#[derive(Debug, Clone)]
pub struct QuestState {
    pub ftp_transfer_mode: String,
    pub upload_attempted: bool,
    pub backup_made: bool,
    pub upload_success: bool,
    pub ftp_remote_host: Option<String>,
    pub ftp_connected: bool,
    pub anomalies_discovered: Vec<String>,
}

impl Default for QuestState {
    fn default() -> Self {
        Self {
            ftp_transfer_mode: "ascii".to_string(),
            upload_attempted: false,
            backup_made: false,
            upload_success: false,
            ftp_remote_host: None,
            ftp_connected: false,
            anomalies_discovered: Vec::new(),
        }
    }
}

impl QuestState {
    pub fn anomaly_count(&self) -> usize {
        self.anomalies_discovered.len()
    }

    pub fn note_anomaly(&mut self, host: &str) {
        if !self.anomalies_discovered.contains(&host.to_string()) {
            self.anomalies_discovered.push(host.to_string());
        }
    }

    /// Decay tier for progressive system degradation.
    /// 0 = normal, 1 = subtle, 2 = noticeable, 3 = severe
    pub fn decay_tier(&self) -> u8 {
        match self.anomaly_count() {
            0..=2 => 0,
            3..=5 => 1,
            6..=8 => 2,
            _ => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MailMessage {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub date: String,
    pub read: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessEntry {
    pub pid: u32,
    pub ppid: u32,
    pub uid: u32,
    pub user: String,
    pub name: String,
    pub state_ch: char,
    pub tty: String,
    pub sz_kb: u32,
    pub cpu_pct: f32,
}

#[derive(Debug, Clone)]
pub struct FileStat {
    pub permissions: String,
    pub links: u32,
    pub owner: String,
    pub group: String,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, Clone)]
pub struct MachineState {
    pub user_name: Option<String>,
    pub password: Option<String>,
    pub last_login: Option<String>,
    pub uptime_ms: u64,
    pub mode: SessionMode,
    pub pending_login_user: String,
    pub processes: Vec<ProcessEntry>,
    pub mail_messages: Vec<MailMessage>,
    pub unread_mail_count: usize,
    pub quest: QuestState,
}

impl MachineState {
    pub fn new() -> Self {
        Self {
            user_name: None,
            password: None,
            last_login: None,
            uptime_ms: 0,
            mode: SessionMode::Booting,
            pending_login_user: String::new(),
            processes: Vec::new(),
            mail_messages: Vec::new(),
            unread_mail_count: 0,
            quest: QuestState::default(),
        }
    }

    pub fn has_account(&self) -> bool {
        self.user_name
            .as_ref()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }
}

impl Default for MachineState {
    fn default() -> Self {
        Self::new()
    }
}

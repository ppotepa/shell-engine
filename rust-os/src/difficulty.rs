#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    MouseEnjoyer,
    ScriptKiddie,
    ICanExitVim,
    Dvorak,
    Su,
}

impl Difficulty {
    pub fn from_label(s: &str) -> Self {
        match s.to_uppercase().trim() {
            "MOUSE ENJOYER" => Self::MouseEnjoyer,
            "SCRIPT KIDDIE" => Self::ScriptKiddie,
            "I CAN EXIT VIM" => Self::ICanExitVim,
            "DVORAK" => Self::Dvorak,
            "SU" => Self::Su,
            _ => Self::ICanExitVim,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MachineSpec {
    pub difficulty: Difficulty,
    pub cpu_model: &'static str,
    pub cpu_mhz: u32,
    pub ram_kb: u32,
    pub disk_kb: u32,
    pub disk_free_kb: u32,
    pub modem_model: &'static str,
    pub modem_baud: u32,
    pub ftp_timeout_ms: u64,
    pub operation_speed_multiplier: f64,
    pub max_processes: usize,
    pub max_open_files: usize,
}

impl MachineSpec {
    pub fn from_difficulty(d: Difficulty) -> Self {
        match d {
            Difficulty::MouseEnjoyer => Self {
                difficulty: d,
                cpu_model: "Intel 486 DX2-66",
                cpu_mhz: 66,
                ram_kb: 8192,
                disk_kb: 122880,
                disk_free_kb: 68000,
                modem_model: "USRobotics Sportster 2400",
                modem_baud: 2400,
                ftp_timeout_ms: 15000,
                operation_speed_multiplier: 0.6,
                max_processes: 32,
                max_open_files: 64,
            },
            Difficulty::ScriptKiddie => Self {
                difficulty: d,
                cpu_model: "Intel 486 DX-33",
                cpu_mhz: 33,
                ram_kb: 4096,
                disk_kb: 81920,
                disk_free_kb: 40000,
                modem_model: "Hayes Smartmodem 1200",
                modem_baud: 1200,
                ftp_timeout_ms: 25000,
                operation_speed_multiplier: 0.8,
                max_processes: 24,
                max_open_files: 32,
            },
            Difficulty::ICanExitVim => Self {
                difficulty: d,
                cpu_model: "Intel 386 DX-33",
                cpu_mhz: 33,
                ram_kb: 4096,
                disk_kb: 40960,
                disk_free_kb: 22000,
                modem_model: "Hayes Smartmodem 1200",
                modem_baud: 1200,
                ftp_timeout_ms: 30000,
                operation_speed_multiplier: 1.0,
                max_processes: 16,
                max_open_files: 24,
            },
            Difficulty::Dvorak => Self {
                difficulty: d,
                cpu_model: "Intel 386 SX-16",
                cpu_mhz: 16,
                ram_kb: 2048,
                disk_kb: 20480,
                disk_free_kb: 10000,
                modem_model: "Hayes Smartmodem 600",
                modem_baud: 600,
                ftp_timeout_ms: 45000,
                operation_speed_multiplier: 1.4,
                max_processes: 12,
                max_open_files: 16,
            },
            Difficulty::Su => Self {
                difficulty: d,
                cpu_model: "Intel 386 SX-16",
                cpu_mhz: 16,
                ram_kb: 1024,
                disk_kb: 10240,
                disk_free_kb: 4000,
                modem_model: "Hayes Smartmodem 300",
                modem_baud: 300,
                ftp_timeout_ms: 60000,
                operation_speed_multiplier: 2.0,
                max_processes: 8,
                max_open_files: 12,
            },
        }
    }
}

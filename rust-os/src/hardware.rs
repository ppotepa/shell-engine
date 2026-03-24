use crate::difficulty::MachineSpec;

/// Derived hardware timing constants from MachineSpec.
/// All values scaled by operation_speed_multiplier (slower CPU = longer delays).
#[derive(Debug, Clone)]
pub struct HardwareProfile {
    // Disk (Seagate ST-157A era)
    pub disk_seek_ms: f64,
    pub disk_rotation_ms: f64,
    pub disk_access_ms: f64,
    pub disk_read_ms: f64,
    pub disk_write_ms: f64,
    pub disk_transfer_kbs: f64,
    pub disk_dir_entry_ms: f64,

    // Spindle state machine
    pub disk_spin_up_ms: f64,
    pub disk_coast_ms: f64,
    pub disk_idle_stop_ms: u64,
    pub disk_coast_threshold_ms: u64,

    // Network
    pub net_bandwidth_kbs: f64,
    pub net_base_ping_ms: f64,
    pub net_connect_ms: f64,
    pub net_send_per_kb_ms: f64,
    pub net_recv_per_kb_ms: f64,
    pub max_sockets: u8,

    // CPU
    pub fork_ms: f64,
    pub exec_ms: f64,
    pub context_switch_ms: f64,

    // Memory
    pub mem_alloc_ms: f64,

    // Buffer cache
    pub cache_hit_ratio: f64,
    pub cache_capacity: usize,

    // Modem
    pub modem_baud: u32,
    pub modem_dial_ms: f64,
    pub modem_handshake_ms: u64,
    pub modem_noise_chance: f64,
}

impl HardwareProfile {
    pub fn from_spec(spec: &MachineSpec) -> Self {
        let f = spec.operation_speed_multiplier;
        let baud = spec.modem_baud;
        Self {
            disk_seek_ms: 15.0 * f,
            disk_rotation_ms: 8.3 * f,
            disk_access_ms: (15.0 + 8.3) * f,
            disk_read_ms: 8.0 * f,
            disk_write_ms: 12.0 * f,
            disk_transfer_kbs: 750.0 / f,
            disk_dir_entry_ms: (15.0 + 8.3) / 4.0 * f,

            disk_spin_up_ms: 300.0 * f,
            disk_coast_ms: 80.0 * f,
            disk_idle_stop_ms: 30_000,
            disk_coast_threshold_ms: 2_000,

            net_bandwidth_kbs: baud as f64 / 8000.0,
            net_base_ping_ms: 120.0 * f,
            net_connect_ms: 200.0 * f,
            net_send_per_kb_ms: 50.0 * f,
            net_recv_per_kb_ms: 40.0 * f,
            max_sockets: 8,

            fork_ms: 80.0 * f,
            exec_ms: 60.0 * f,
            context_switch_ms: 1.0 * f,

            mem_alloc_ms: 2.0 * f,

            cache_hit_ratio: 0.7,
            cache_capacity: (spec.ram_kb / 64).max(8) as usize,

            modem_baud: baud,
            modem_dial_ms: 2000.0 * f,
            modem_handshake_ms: match baud {
                300 => 3500,
                600 => 2800,
                1200 => 1800,
                2400 => 800,
                _ => 1800,
            },
            modem_noise_chance: match baud {
                300 => 0.08,
                600 => 0.05,
                1200 => 0.03,
                2400 => 0.01,
                _ => 0.03,
            },
        }
    }
}

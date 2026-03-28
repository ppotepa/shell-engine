/// Epoch: September 17, 1991, 21:12:00 — historical Minix release date
pub const EPOCH_YEAR: u32 = 1991;
pub const EPOCH_MONTH: u32 = 9;
pub const EPOCH_DAY: u32 = 17;
pub const EPOCH_HOUR: u32 = 21;
pub const EPOCH_MINUTE: u32 = 12;

pub struct SimulatedClock {
    elapsed_ms: u64,
}

impl SimulatedClock {
    pub fn new() -> Self {
        Self { elapsed_ms: 0 }
    }

    pub fn advance(&mut self, dt_ms: u64) {
        self.elapsed_ms += dt_ms;
    }

    pub fn uptime_ms(&self) -> u64 {
        self.elapsed_ms
    }

    /// Simulated datetime as a formatted string.
    pub fn now_str(&self) -> String {
        let total_secs = self.elapsed_ms / 1000;
        let s = (EPOCH_HOUR as u64 * 3600 + EPOCH_MINUTE as u64 * 60 + total_secs) % 86400;
        let h = s / 3600;
        let m = (s % 3600) / 60;
        let sec = s % 60;
        let day_offset = (self.elapsed_ms / 1000) / 86400;
        let day = EPOCH_DAY as u64 + day_offset;
        let month_names = [
            "", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let month = EPOCH_MONTH as usize;
        format!(
            "{} {:.0} {:02}:{:02}:{:02} EET 1991",
            month_names[month], day, h, m, sec
        )
    }

    /// Short time string for prompts: HH:MM:SS
    pub fn time_str(&self) -> String {
        let total_secs = self.elapsed_ms / 1000;
        let s = (EPOCH_HOUR as u64 * 3600 + EPOCH_MINUTE as u64 * 60 + total_secs) % 86400;
        let h = s / 3600;
        let m = (s % 3600) / 60;
        let sec = s % 60;
        format!("{:02}:{:02}:{:02}", h, m, sec)
    }

    /// Uptime formatted as "X min" or "X:XX"
    pub fn uptime_str(&self) -> String {
        let total_secs = self.elapsed_ms / 1000;
        let m = total_secs / 60;
        if m < 60 {
            format!("{m} min")
        } else {
            format!("{}:{:02}", m / 60, m % 60)
        }
    }
}

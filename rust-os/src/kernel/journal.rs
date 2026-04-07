pub struct Journal {
    entries: Vec<String>,
}

impl Default for Journal {
    fn default() -> Self {
        Self::new()
    }
}

impl Journal {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, line: impl Into<String>) {
        self.entries.push(line.into());
    }

    pub fn all(&self) -> &[String] {
        &self.entries
    }

    /// Append anomaly event to kernel ring buffer / dmesg log.
    pub fn append_anomaly(&mut self, host: &str, clock_str: &str) {
        self.entries.push(format!(
            "{clock_str} kernel: eth0: unexpected packet from {host} (dropped)"
        ));
    }

    pub fn append_clock_drift(&mut self, clock_str: &str) {
        self.entries.push(format!(
            "{clock_str} kernel: clock: drift detected, resync failed"
        ));
    }

    pub fn append_inode_anomaly(&mut self, inode: u32, clock_str: &str) {
        self.entries.push(format!(
            "{clock_str} kernel: WARNING: inode {inode} accessed before creation"
        ));
    }
}

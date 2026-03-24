/// Simulated disk — thin wrapper; actual file storage is in the Vfs.
/// This layer handles latency calculation and inodes.
pub struct SimulatedDisk {
    pub total_reads: u64,
    pub total_writes: u64,
}

impl SimulatedDisk {
    pub fn new() -> Self {
        Self { total_reads: 0, total_writes: 0 }
    }

    pub fn record_read(&mut self) { self.total_reads += 1; }
    pub fn record_write(&mut self) { self.total_writes += 1; }
}

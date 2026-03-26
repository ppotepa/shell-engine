use crate::difficulty::MachineSpec;
use crate::hardware::HardwareProfile;
use std::collections::{HashMap, VecDeque};

// ── Spindle State Machine ──

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpindleState {
    Stopped,
    Coasting,
    Running,
}

pub struct DiskController {
    pub state: SpindleState,
    pub last_access_ms: u64,
    pub active_io_count: u32,
}

impl DiskController {
    pub fn new() -> Self {
        Self {
            state: SpindleState::Stopped,
            last_access_ms: 0,
            active_io_count: 0,
        }
    }

    pub fn acquire(&mut self, now_ms: u64, hw: &HardwareProfile) -> f64 {
        self.last_access_ms = now_ms;
        self.active_io_count += 1;

        let spin_up_cost = match self.state {
            SpindleState::Stopped => hw.disk_spin_up_ms,
            SpindleState::Coasting => hw.disk_coast_ms,
            SpindleState::Running => 0.0,
        };
        self.state = SpindleState::Running;

        if self.active_io_count > 1 {
            let busy_chance = (0.8f64).min((self.active_io_count - 1) as f64 * 0.15);
            let roll = ((self.active_io_count * 7919) % 100) as f64 / 100.0;
            if roll < busy_chance {
                return spin_up_cost + hw.disk_seek_ms * 0.7;
            }
        }
        spin_up_cost
    }

    pub fn release(&mut self) {
        self.active_io_count = self.active_io_count.saturating_sub(1);
    }

    pub fn update_spindle_state(&mut self, now_ms: u64, hw: &HardwareProfile) {
        if self.active_io_count > 0 {
            return;
        }
        let idle_ms = now_ms.saturating_sub(self.last_access_ms);
        self.state = if idle_ms > hw.disk_idle_stop_ms {
            SpindleState::Stopped
        } else if idle_ms > hw.disk_coast_threshold_ms {
            SpindleState::Coasting
        } else {
            SpindleState::Running
        };
    }

    pub fn latency_for_read(&mut self, now_ms: u64, hw: &HardwareProfile, size_bytes: u64) -> f64 {
        let spindle_cost = self.acquire(now_ms, hw);
        let transfer_ms = (size_bytes as f64 / 1024.0) / hw.disk_transfer_kbs * 1000.0;
        let result = spindle_cost + hw.disk_access_ms + transfer_ms;
        self.release();
        result
    }
}

// ── RAM Allocator ──

pub struct RamAllocator {
    pub total_kb: u32,
    pub used_kb: u32,
    pub high_water_mark: u32,
}

impl RamAllocator {
    pub fn new(spec: &MachineSpec) -> Self {
        Self {
            total_kb: spec.ram_kb,
            used_kb: 768,
            high_water_mark: 768,
        }
    }

    pub fn available_kb(&self) -> u32 {
        self.total_kb.saturating_sub(self.used_kb)
    }

    pub fn alloc(&mut self, kb: u32) -> bool {
        if self.available_kb() >= kb {
            self.used_kb += kb;
            if self.used_kb > self.high_water_mark {
                self.high_water_mark = self.used_kb;
            }
            true
        } else {
            false
        }
    }

    pub fn free(&mut self, kb: u32) {
        self.used_kb = self.used_kb.saturating_sub(kb);
    }
}

// ── File Descriptor Table ──

#[derive(Debug, Clone)]
pub struct FdEntry {
    pub path: String,
    pub mode: FdMode,
    pub offset: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FdMode {
    Read,
    Write,
    ReadWrite,
}

pub struct FdTable {
    next_fd: u32,
    pub max: usize,
    pub open: u32,
    pub entries: HashMap<u32, FdEntry>,
}

impl FdTable {
    pub fn new(spec: &MachineSpec) -> Self {
        let mut entries = HashMap::new();
        entries.insert(
            0,
            FdEntry {
                path: "/dev/stdin".into(),
                mode: FdMode::Read,
                offset: 0,
            },
        );
        entries.insert(
            1,
            FdEntry {
                path: "/dev/stdout".into(),
                mode: FdMode::Write,
                offset: 0,
            },
        );
        entries.insert(
            2,
            FdEntry {
                path: "/dev/stderr".into(),
                mode: FdMode::Write,
                offset: 0,
            },
        );
        Self {
            next_fd: 3,
            max: spec.max_open_files,
            open: 3,
            entries,
        }
    }

    pub fn alloc(&mut self) -> Option<u32> {
        if self.open as usize >= self.max {
            return None;
        }
        let fd = self.next_fd;
        self.next_fd += 1;
        self.open += 1;
        Some(fd)
    }

    pub fn open_file(&mut self, path: &str, mode: FdMode) -> Option<u32> {
        let fd = self.alloc()?;
        self.entries.insert(
            fd,
            FdEntry {
                path: path.to_string(),
                mode,
                offset: 0,
            },
        );
        Some(fd)
    }

    pub fn close_fd(&mut self, fd: u32) {
        if self.entries.remove(&fd).is_some() {
            self.open = self.open.saturating_sub(1);
        }
    }

    pub fn free(&mut self) {
        self.open = self.open.saturating_sub(1);
    }
}

// ── Buffer Cache (LRU) ──

struct CacheEntry {
    data: Vec<u8>,
}

pub struct BufferCache {
    capacity: usize,
    entries: HashMap<String, CacheEntry>,
    lru_order: VecDeque<String>,
    pub last_lookup_was_miss: bool,
}

impl BufferCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: HashMap::new(),
            lru_order: VecDeque::new(),
            last_lookup_was_miss: true,
        }
    }

    pub fn lookup(&mut self, path: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.entries.get(path) {
            self.last_lookup_was_miss = false;
            // Move to front of LRU
            self.lru_order.retain(|p| p != path);
            self.lru_order.push_front(path.to_string());
            Some(entry.data.clone())
        } else {
            self.last_lookup_was_miss = true;
            None
        }
    }

    pub fn insert(&mut self, path: &str, data: Vec<u8>) {
        if self.entries.contains_key(path) {
            self.entries.get_mut(path).unwrap().data = data;
            self.lru_order.retain(|p| p != path);
            self.lru_order.push_front(path.to_string());
            return;
        }
        // Evict LRU if at capacity
        while self.entries.len() >= self.capacity {
            if let Some(evicted) = self.lru_order.pop_back() {
                self.entries.remove(&evicted);
            } else {
                break;
            }
        }
        self.entries.insert(path.to_string(), CacheEntry { data });
        self.lru_order.push_front(path.to_string());
    }

    pub fn invalidate(&mut self, path: &str) {
        self.entries.remove(path);
        self.lru_order.retain(|p| p != path);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

// ── CPU Scheduler (Round Robin) ──

pub struct CpuScheduler {
    run_queue: VecDeque<u32>,
    pub time_slice_ms: u64,
}

impl CpuScheduler {
    pub fn new() -> Self {
        Self {
            run_queue: VecDeque::new(),
            time_slice_ms: 100,
        }
    }

    pub fn add(&mut self, pid: u32) {
        if !self.run_queue.contains(&pid) {
            self.run_queue.push_back(pid);
        }
    }

    pub fn remove(&mut self, pid: u32) {
        self.run_queue.retain(|&p| p != pid);
    }

    pub fn schedule(&mut self) -> Option<u32> {
        if let Some(pid) = self.run_queue.pop_front() {
            self.run_queue.push_back(pid);
            Some(pid)
        } else {
            None
        }
    }

    pub fn run_queue_len(&self) -> usize {
        self.run_queue.len()
    }
}

// ── Network Controller ──

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SocketState {
    SynSent,
    Established,
    Closing,
    Closed,
}

#[derive(Debug, Clone)]
pub struct SocketEntry {
    pub remote_host: String,
    pub remote_port: u16,
    pub state: SocketState,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
}

pub struct NetworkController {
    sockets: HashMap<u8, SocketEntry>,
    next_socket_id: u8,
    pub max_sockets: u8,
    packet_queues: HashMap<u8, VecDeque<Vec<u8>>>,
}

impl NetworkController {
    pub fn new(max_sockets: u8) -> Self {
        Self {
            sockets: HashMap::new(),
            next_socket_id: 1,
            max_sockets,
            packet_queues: HashMap::new(),
        }
    }

    pub fn connect(&mut self, host: &str, port: u16) -> Option<u8> {
        if self.sockets.len() >= self.max_sockets as usize {
            return None;
        }
        let id = self.next_socket_id;
        self.next_socket_id = self.next_socket_id.wrapping_add(1);
        if self.next_socket_id == 0 {
            self.next_socket_id = 1;
        }
        self.sockets.insert(
            id,
            SocketEntry {
                remote_host: host.to_string(),
                remote_port: port,
                state: SocketState::Established,
                bytes_sent: 0,
                bytes_recv: 0,
            },
        );
        self.packet_queues.insert(id, VecDeque::new());
        Some(id)
    }

    pub fn send(&mut self, socket_id: u8, data: &[u8]) -> bool {
        if let Some(sock) = self.sockets.get_mut(&socket_id) {
            if sock.state == SocketState::Established {
                sock.bytes_sent += data.len() as u64;
                return true;
            }
        }
        false
    }

    pub fn recv(&mut self, socket_id: u8) -> Option<Vec<u8>> {
        if let Some(queue) = self.packet_queues.get_mut(&socket_id) {
            let pkt = queue.pop_front();
            if let (Some(ref data), Some(sock)) = (&pkt, self.sockets.get_mut(&socket_id)) {
                sock.bytes_recv += data.len() as u64;
            }
            pkt
        } else {
            None
        }
    }

    pub fn enqueue_recv(&mut self, socket_id: u8, data: Vec<u8>) {
        if let Some(queue) = self.packet_queues.get_mut(&socket_id) {
            queue.push_back(data);
        }
    }

    pub fn close(&mut self, socket_id: u8) {
        if let Some(sock) = self.sockets.get_mut(&socket_id) {
            sock.state = SocketState::Closed;
        }
        self.sockets.remove(&socket_id);
        self.packet_queues.remove(&socket_id);
    }

    pub fn active_count(&self) -> usize {
        self.sockets
            .values()
            .filter(|s| s.state == SocketState::Established)
            .count()
    }

    pub fn list_sockets(&self) -> Vec<(u8, &SocketEntry)> {
        self.sockets.iter().map(|(&id, e)| (id, e)).collect()
    }
}

// ── Composite Resource State ──

pub struct ResourceState {
    pub ram: RamAllocator,
    pub disk_ctrl: DiskController,
    pub fd_table: FdTable,
    pub cache: BufferCache,
    pub cpu: CpuScheduler,
    pub net: NetworkController,
    pub disk_free_kb: u32,
    pub disk_total_kb: u32,
}

impl ResourceState {
    pub fn new(spec: &MachineSpec, hw: &HardwareProfile) -> Self {
        Self {
            ram: RamAllocator::new(spec),
            disk_ctrl: DiskController::new(),
            fd_table: FdTable::new(spec),
            cache: BufferCache::new(hw.cache_capacity),
            cpu: CpuScheduler::new(),
            net: NetworkController::new(hw.max_sockets),
            disk_free_kb: spec.disk_free_kb,
            disk_total_kb: spec.disk_kb,
        }
    }
}

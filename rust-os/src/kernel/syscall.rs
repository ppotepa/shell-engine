use crate::hardware::HardwareProfile;
use super::resources::ResourceState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallKind {
    // Disk
    DiskRead,
    DiskWrite,
    DiskAppend,
    DiskStat,
    DiskUnlink,
    DiskMkdir,
    DiskListDir,
    // Network
    NetConnect,
    NetSend,
    NetRecv,
    NetClose,
    NetResolve,
    // Process
    ProcessFork,
    ProcessExec,
    ProcessExit,
    ProcessKill,
    ProcessList,
    // Memory
    MemAlloc,
    MemFree,
    // Clock
    ClockRead,
    // Mail
    MailRead,
    MailDeliver,
    // Journal
    JournalAppend,
}

impl SyscallKind {
    pub fn is_disk(&self) -> bool {
        matches!(self,
            Self::DiskRead | Self::DiskWrite | Self::DiskAppend |
            Self::DiskStat | Self::DiskUnlink | Self::DiskMkdir | Self::DiskListDir
        )
    }

    pub fn is_net(&self) -> bool {
        matches!(self,
            Self::NetConnect | Self::NetSend | Self::NetRecv |
            Self::NetClose | Self::NetResolve
        )
    }
}

pub struct MinixSyscallGate;

impl MinixSyscallGate {
    /// Check whether the kernel has enough resources to satisfy this syscall.
    pub fn can_satisfy(kind: SyscallKind, res: &ResourceState) -> bool {
        match kind {
            // Disk ops need an FD slot (except stat/list which are transient)
            SyscallKind::DiskRead | SyscallKind::DiskWrite | SyscallKind::DiskAppend => {
                (res.fd_table.open as usize) < res.fd_table.max
            }
            SyscallKind::DiskStat | SyscallKind::DiskUnlink |
            SyscallKind::DiskMkdir | SyscallKind::DiskListDir => true,

            // Net ops need a socket slot (except close/resolve)
            SyscallKind::NetConnect => res.net.active_count() < res.net.max_sockets as usize,
            SyscallKind::NetSend | SyscallKind::NetRecv => true,
            SyscallKind::NetClose | SyscallKind::NetResolve => true,

            // Process ops
            SyscallKind::ProcessFork => res.ram.available_kb() >= 64,
            SyscallKind::ProcessExec | SyscallKind::ProcessExit |
            SyscallKind::ProcessKill | SyscallKind::ProcessList => true,

            // Memory
            SyscallKind::MemAlloc => res.ram.available_kb() > 0,
            SyscallKind::MemFree => true,

            // Always available
            SyscallKind::ClockRead | SyscallKind::MailRead |
            SyscallKind::MailDeliver | SyscallKind::JournalAppend => true,
        }
    }

    /// Reserve resources before executing the syscall.
    pub fn debit(kind: SyscallKind, res: &mut ResourceState) {
        match kind {
            SyscallKind::DiskRead | SyscallKind::DiskWrite | SyscallKind::DiskAppend => {
                res.fd_table.alloc();
                res.disk_ctrl.active_io_count += 1;
            }
            SyscallKind::DiskStat | SyscallKind::DiskUnlink |
            SyscallKind::DiskMkdir | SyscallKind::DiskListDir => {
                res.disk_ctrl.active_io_count += 1;
            }
            SyscallKind::ProcessFork => {
                res.ram.alloc(64);
            }
            _ => {}
        }
    }

    /// Release resources after executing the syscall.
    pub fn credit(kind: SyscallKind, res: &mut ResourceState) {
        match kind {
            SyscallKind::DiskRead | SyscallKind::DiskWrite | SyscallKind::DiskAppend => {
                res.fd_table.free();
                res.disk_ctrl.active_io_count = res.disk_ctrl.active_io_count.saturating_sub(1);
            }
            SyscallKind::DiskStat | SyscallKind::DiskUnlink |
            SyscallKind::DiskMkdir | SyscallKind::DiskListDir => {
                res.disk_ctrl.active_io_count = res.disk_ctrl.active_io_count.saturating_sub(1);
            }
            SyscallKind::ProcessExit => {
                res.ram.free(64);
            }
            _ => {}
        }
    }

    /// Compute latency in ms for this syscall, given hardware profile and current resource state.
    pub fn latency_for(kind: SyscallKind, hw: &HardwareProfile, res: &ResourceState) -> u64 {
        let ms = match kind {
            SyscallKind::DiskRead => {
                let cache_miss = res.cache.last_lookup_was_miss;
                if cache_miss {
                    hw.disk_seek_ms + hw.disk_read_ms
                } else {
                    1.0 // cache hit — near-instant
                }
            }
            SyscallKind::DiskWrite | SyscallKind::DiskAppend => {
                hw.disk_seek_ms + hw.disk_write_ms
            }
            SyscallKind::DiskStat => hw.disk_seek_ms * 0.5,
            SyscallKind::DiskUnlink => hw.disk_seek_ms + hw.disk_write_ms * 0.5,
            SyscallKind::DiskMkdir => hw.disk_seek_ms + hw.disk_write_ms,
            SyscallKind::DiskListDir => hw.disk_seek_ms + hw.disk_read_ms * 0.5,

            SyscallKind::NetConnect => hw.net_connect_ms,
            SyscallKind::NetSend => hw.net_send_per_kb_ms,
            SyscallKind::NetRecv => hw.net_recv_per_kb_ms,
            SyscallKind::NetClose => 1.0,
            SyscallKind::NetResolve => hw.net_connect_ms * 0.5,

            SyscallKind::ProcessFork => hw.fork_ms,
            SyscallKind::ProcessExec => hw.exec_ms,
            SyscallKind::ProcessExit => 1.0,
            SyscallKind::ProcessKill => 1.0,
            SyscallKind::ProcessList => 2.0,

            SyscallKind::MemAlloc => hw.mem_alloc_ms,
            SyscallKind::MemFree => 1.0,

            SyscallKind::ClockRead => 1.0,

            SyscallKind::MailRead => hw.disk_seek_ms + hw.disk_read_ms,
            SyscallKind::MailDeliver => hw.disk_seek_ms + hw.disk_write_ms,

            SyscallKind::JournalAppend => hw.disk_write_ms * 0.5,
        };

        // Contention penalty for disk ops: longer queue = slower
        let contention = if kind.is_disk() && res.disk_ctrl.active_io_count > 1 {
            (res.disk_ctrl.active_io_count as f64 - 1.0) * 0.15 * hw.disk_seek_ms
        } else {
            0.0
        };

        // CPU scheduler penalty: more processes in run queue = slower context switches
        let sched_penalty = if matches!(kind, SyscallKind::ProcessFork | SyscallKind::ProcessExec) {
            res.cpu.run_queue_len() as f64 * hw.context_switch_ms
        } else {
            0.0
        };

        (ms + contention + sched_penalty) as u64
    }
}

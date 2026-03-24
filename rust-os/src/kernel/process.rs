use crate::difficulty::MachineSpec;
use crate::state::ProcessEntry;

pub struct SimulatedProcessTable {
    pub processes: Vec<ProcessEntry>,
    pub next_pid: u32,
    pub max_processes: usize,
}

impl SimulatedProcessTable {
    pub fn new(spec: &MachineSpec) -> Self {
        Self {
            processes: Vec::new(),
            next_pid: 1,
            max_processes: spec.max_processes,
        }
    }

    pub fn boot_system_processes(&mut self) {
        let sys = vec![
            (0, 0, 0, "kernel", "root", 'S', "?",  0),
            (1, 0, 0, "init",   "root", 'S', "?",  16),
            (2, 1, 0, "mm",     "root", 'S', "?",  48),
            (3, 1, 0, "fs",     "root", 'S', "?",  64),
            (4, 1, 0, "update", "root", 'S', "?",  12),
            (5, 1, 0, "cron",   "root", 'S', "?",  8),
            (6, 1, 0, "getty",  "root", 'S', "tty0", 4),
            (7, 6, 1000, "sh",  "torvalds", 'S', "tty0", 20),
        ];
        for (pid, ppid, uid, name, user, state_ch, tty, sz) in sys {
            self.processes.push(ProcessEntry {
                pid,
                ppid,
                uid,
                user: user.to_string(),
                name: name.to_string(),
                state_ch,
                tty: tty.to_string(),
                sz_kb: sz,
                cpu_pct: 0.0,
            });
        }
        self.next_pid = 8;
    }

    pub fn fork(&mut self, name: &str, sz_kb: u32, user: &str, tty: &str) -> Option<u32> {
        if self.processes.len() >= self.max_processes {
            return None;
        }
        let pid = self.next_pid;
        self.next_pid += 1;
        let ppid = 7; // sh is parent
        let uid = if user == "root" { 0 } else { 1000 };
        self.processes.push(ProcessEntry {
            pid,
            ppid,
            uid,
            user: user.to_string(),
            name: name.to_string(),
            state_ch: 'R',
            tty: tty.to_string(),
            sz_kb,
            cpu_pct: 0.5,
        });
        Some(pid)
    }

    pub fn exit_pid(&mut self, pid: u32) {
        self.processes.retain(|p| p.pid != pid);
    }

    pub fn kill(&mut self, pid: u32) -> bool {
        // Cannot kill PID 0-5 (system)
        if pid <= 5 { return false; }
        let before = self.processes.len();
        self.processes.retain(|p| p.pid != pid);
        self.processes.len() < before
    }

    pub fn list(&self) -> &[ProcessEntry] {
        &self.processes
    }

    pub fn get(&self, pid: u32) -> Option<&ProcessEntry> {
        self.processes.iter().find(|p| p.pid == pid)
    }

    /// Add the phantom (null) process when decay tier >= 2
    pub fn inject_phantom_process(&mut self) {
        if self.processes.iter().any(|p| p.name == "(null)") {
            return;
        }
        self.processes.push(ProcessEntry {
            pid: 31337,
            ppid: 0,
            uid: 0,
            user: "(null)".to_string(),
            name: "(null)".to_string(),
            state_ch: 'D',
            tty: "?".to_string(),
            sz_kb: 0,
            cpu_pct: 0.0,
        });
    }

    pub fn remove_phantom_process(&mut self) {
        self.processes.retain(|p| p.name != "(null)");
    }
}

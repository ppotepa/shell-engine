use std::collections::HashMap;

pub struct SimulatedNetwork {
    pub active_connections: HashMap<u32, String>, // fd -> host
    pub next_fd: u32,
}

impl Default for SimulatedNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulatedNetwork {
    pub fn new() -> Self {
        Self {
            active_connections: HashMap::new(),
            next_fd: 10,
        }
    }

    pub fn connect(&mut self, host: &str) -> u32 {
        let fd = self.next_fd;
        self.next_fd += 1;
        self.active_connections.insert(fd, host.to_string());
        fd
    }

    pub fn close(&mut self, fd: u32) {
        self.active_connections.remove(&fd);
    }

    pub fn is_connected(&self) -> bool {
        !self.active_connections.is_empty()
    }
}

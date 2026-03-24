use crate::hardware::HardwareProfile;
use crate::kernel::unit_of_work::UnitOfWork;

static PHONE_BOOK: &[(&str, &str)] = &[
    ("128.214.6.100",  "90-4574100"),
    ("130.37.24.3",    "020-6464411"),
    ("128.214.3.1",    "90-4713220"),
    ("192.48.96.9",    "1-800-388-4434"),
    ("18.71.0.38",     "1-617-253-2751"),
    ("26.0.0.73",      "1-415-859-2777"),
];

pub struct SimulatedModem {
    pub connected: bool,
    pub connected_to: Option<String>,
    pub baud: u32,
    handshake_ms: u64,
    pub noise_chance: f64,
}

impl SimulatedModem {
    pub fn new(hw: &HardwareProfile) -> Self {
        Self {
            connected: false,
            connected_to: None,
            baud: hw.modem_baud,
            handshake_ms: hw.modem_handshake_ms,
            noise_chance: hw.modem_noise_chance,
        }
    }

    /// Simulate modem dial sequence, scheduling output lines.
    pub fn dial(&mut self, host_ip: &str, host_name: &str, uow: &mut UnitOfWork) {
        let number = self.lookup_number(host_ip);
        let baud = self.baud;

        uow.schedule("ATH0", 0);
        uow.schedule("OK", 200);
        uow.schedule(format!("ATDT {number}"), 100);
        uow.schedule("DIALING...", 300);
        uow.schedule("RINGING", 800);

        // Handshake varies with baud
        let connect_delay = self.handshake_ms;
        uow.schedule(format!("CONNECT {baud}"), connect_delay);

        // Line noise on slow connections
        if self.baud <= 600 {
            uow.schedule("~~~#@~~ (line noise)", 200);
            uow.schedule(format!("CONNECT {baud} (stabilized)"), 400);
        }

        uow.schedule(format!("Connected to {host_name}."), 200);
        uow.schedule(format!("220 {host_name} FTP server ready."), 300);

        self.connected = true;
        self.connected_to = Some(host_name.to_string());
    }

    pub fn hangup(&mut self, uow: &mut UnitOfWork) {
        uow.schedule("ATH", 0);
        uow.schedule("NO CARRIER", 300);
        self.connected = false;
        self.connected_to = None;
    }

    fn lookup_number(&self, ip: &str) -> String {
        for (book_ip, number) in PHONE_BOOK {
            if *book_ip == ip {
                return number.to_string();
            }
        }
        // Synthesize from IP octets
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            format!("{}-{}{}", parts[0], parts[1], parts[2])
        } else {
            ip.to_string()
        }
    }

    /// Inject line noise into a transfer status string.
    pub fn apply_noise(&self, line: &str, counter: u64) -> Option<String> {
        // Use counter as deterministic "random"
        let roll = (counter * 6271 + 3571) % 1000;
        let threshold = (self.noise_chance * 1000.0) as u64;
        if roll < threshold {
            // Corrupt one char
            let pos = (counter * 17 + 5) as usize % line.len().max(1);
            let mut corrupted = line.to_string();
            let noise_chars = ["~", "#", "@", "^", "\x07"];
            let nc = noise_chars[(counter % 5) as usize];
            corrupted.insert_str(pos, nc);
            Some(corrupted)
        } else {
            None
        }
    }
}

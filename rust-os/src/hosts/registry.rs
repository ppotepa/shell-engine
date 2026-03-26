use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum HostKind {
    Real,
    EasterEgg,
    Loopback,
}

#[derive(Debug, Clone)]
pub struct RemoteHost {
    pub hostname: String,
    pub ip: String,
    pub aliases: Vec<String>,
    pub base_ping_ms: u64,
    pub kind: HostKind,
    /// For easter eggs: lines to output per ping reply (variable RTTs).
    /// Each entry: (delay_ms, text)
    pub ping_lines: Vec<(u64, String)>,
    /// Anomaly tag for quest tracking
    pub anomaly_tag: Option<String>,
}

impl RemoteHost {
    fn real(hostname: &str, ip: &str, ping_ms: u64, aliases: &[&str]) -> Self {
        Self {
            hostname: hostname.to_string(),
            ip: ip.to_string(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            base_ping_ms: ping_ms,
            kind: HostKind::Real,
            ping_lines: Vec::new(),
            anomaly_tag: None,
        }
    }

    fn loopback(hostname: &str, ip: &str, aliases: &[&str]) -> Self {
        Self {
            hostname: hostname.to_string(),
            ip: ip.to_string(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            base_ping_ms: 0,
            kind: HostKind::Loopback,
            ping_lines: Vec::new(),
            anomaly_tag: None,
        }
    }

    fn egg(hostname: &str, ip: &str, tag: &str, lines: Vec<(u64, &str)>) -> Self {
        Self {
            hostname: hostname.to_string(),
            ip: ip.to_string(),
            aliases: Vec::new(),
            base_ping_ms: 0,
            kind: HostKind::EasterEgg,
            ping_lines: lines.into_iter().map(|(d, s)| (d, s.to_string())).collect(),
            anomaly_tag: Some(tag.to_string()),
        }
    }
}

pub struct RemoteHostIndex {
    by_hostname: HashMap<String, RemoteHost>,
}

impl RemoteHostIndex {
    pub fn build() -> Self {
        let mut idx = Self {
            by_hostname: HashMap::new(),
        };
        idx.register_real_hosts();
        idx.register_easter_eggs();
        idx
    }

    fn add(&mut self, host: RemoteHost) {
        let hostname = host.hostname.clone();
        for alias in &host.aliases {
            self.by_hostname.insert(alias.clone(), host.clone());
        }
        self.by_hostname.insert(hostname, host);
    }

    pub fn resolve(&self, name: &str) -> Option<&RemoteHost> {
        self.by_hostname.get(&name.to_lowercase())
    }

    pub fn is_known(&self, name: &str) -> bool {
        self.by_hostname.contains_key(&name.to_lowercase())
    }

    fn register_real_hosts(&mut self) {
        // Loopback
        self.add(RemoteHost::loopback(
            "localhost",
            "127.0.0.1",
            &["kruuna", "kruuna.helsinki.fi"],
        ));

        // Finnish academic (FUNET) — short ping from Helsinki
        self.add(RemoteHost::real(
            "nic.funet.fi",
            "128.214.6.100",
            12,
            &["ftp.funet.fi"],
        ));
        self.add(RemoteHost::real("helsinki.fi", "128.214.3.1", 8, &[]));
        self.add(RemoteHost::real("tut.fi", "130.188.8.1", 18, &[]));
        self.add(RemoteHost::real("oulu.fi", "130.231.1.1", 24, &[]));
        self.add(RemoteHost::real("utu.fi", "130.232.1.1", 20, &[]));
        self.add(RemoteHost::real("jyu.fi", "130.234.1.1", 22, &[]));

        // European academic
        self.add(RemoteHost::real("cs.vu.nl", "130.37.24.3", 55, &[]));
        self.add(RemoteHost::real("ethz.ch", "129.132.1.1", 62, &[]));
        self.add(RemoteHost::real("doc.ic.ac.uk", "155.198.1.1", 88, &[]));
        self.add(RemoteHost::real("info.cern.ch", "128.141.201.74", 65, &[]));

        // North American
        self.add(RemoteHost::real("prep.ai.mit.edu", "18.71.0.38", 220, &[]));
        self.add(RemoteHost::real(
            "wuarchive.wustl.edu",
            "128.252.135.4",
            240,
            &[],
        ));
        self.add(RemoteHost::real("ftp.uu.net", "192.48.96.9", 230, &[]));
        self.add(RemoteHost::real("sun.com", "192.9.9.1", 245, &[]));
        self.add(RemoteHost::real(
            "research.att.com",
            "135.104.1.1",
            235,
            &[],
        ));
        self.add(RemoteHost::real("cs.cmu.edu", "128.2.1.1", 225, &[]));
        self.add(RemoteHost::real("cs.berkeley.edu", "128.32.1.1", 250, &[]));
        self.add(RemoteHost::real("cs.stanford.edu", "36.1.0.1", 255, &[]));

        // Pacific
        self.add(RemoteHost::real("munnari.oz.au", "128.250.1.21", 380, &[]));
    }

    fn register_easter_eggs(&mut self) {
        // --- Modern services (future-dated, temporal anomalies) ---
        self.add(RemoteHost::egg(
            "google.com",
            "66.102.7.99",
            "google",
            vec![
                (0, "PING google.com (66.102.7.99): 56 data bytes"),
                (
                    180,
                    "64 bytes from 66.102.7.99: icmp_seq=0 ttl=52 time=181 ms",
                ),
                (600, "Request timeout for icmp_seq 1"),
                (600, "Request timeout for icmp_seq 2"),
                (0, "--- google.com ping statistics ---"),
                (0, "3 packets transmitted, 1 received, 66% packet loss"),
                (0, "round-trip min/avg/max = 181/181/181 ms"),
                (
                    0,
                    "note: route fragment timestamp: 04 Sep 1998 — 7-year discrepancy",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "github.com",
            "192.30.255.112",
            "github",
            vec![
                (0, "PING github.com (192.30.255.112): 56 data bytes"),
                (
                    240,
                    "64 bytes from 192.30.255.112: icmp_seq=0 ttl=47 time=241 ms",
                ),
                (
                    240,
                    "64 bytes from 192.30.255.112: icmp_seq=1 ttl=47 time=238 ms",
                ),
                (
                    240,
                    "64 bytes from 192.30.255.112: icmp_seq=2 ttl=47 time=243 ms",
                ),
                (0, "--- github.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "round-trip min/avg/max = 238/240/243 ms"),
                (
                    0,
                    "WARNING: route fragment timestamp: 10 Apr 2008 — 17-year discrepancy",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "wikipedia.org",
            "91.198.174.192",
            "wikipedia",
            vec![
                (0, "PING wikipedia.org (91.198.174.192): 56 data bytes"),
                (1200, "Request timeout for icmp_seq 0"),
                (1200, "Request timeout for icmp_seq 1"),
                (1200, "Request timeout for icmp_seq 2"),
                (0, "--- wikipedia.org ping statistics ---"),
                (0, "3 packets transmitted, 0 received, 100% packet loss"),
                (
                    0,
                    "note: route fragment from AS 14907 received — host may exist in 2001",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "kernel.org",
            "149.20.4.69",
            "kernel_org",
            vec![
                (0, "PING kernel.org (149.20.4.69): 56 data bytes"),
                (
                    210,
                    "64 bytes from 149.20.4.69: icmp_seq=0 ttl=53 time=211 ms",
                ),
                (
                    210,
                    "64 bytes from 149.20.4.69: icmp_seq=1 ttl=53 time=208 ms",
                ),
                (
                    210,
                    "64 bytes from 149.20.4.69: icmp_seq=2 ttl=53 time=214 ms",
                ),
                (0, "--- kernel.org ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "ICMP payload banner: \"The Linux Kernel Archives\""),
                (
                    0,
                    "note: domain registered 1997 — this host should not exist yet",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "archive.org",
            "207.241.224.2",
            "archive_org",
            vec![
                (0, "PING archive.org (207.241.224.2): 56 data bytes"),
                (
                    280,
                    "64 bytes from 207.241.224.2: icmp_seq=0 ttl=48 time=279 ms",
                ),
                (
                    280,
                    "64 bytes from 207.241.224.2: icmp_seq=1 ttl=48 time=282 ms",
                ),
                (
                    280,
                    "64 bytes from 207.241.224.2: icmp_seq=2 ttl=48 time=277 ms",
                ),
                (0, "--- archive.org ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (
                    0,
                    "ICMP payload: \"Wayback Machine — saving the web since 1996\"",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "facebook.com",
            "31.13.66.35",
            "facebook",
            vec![
                (0, "PING facebook.com (31.13.66.35): 56 data bytes"),
                (
                    260,
                    "64 bytes from 31.13.66.35: icmp_seq=0 ttl=51 time=261 ms",
                ),
                (
                    260,
                    "64 bytes from 31.13.66.35: icmp_seq=1 ttl=51 time=258 ms",
                ),
                (
                    260,
                    "64 bytes from 31.13.66.35: icmp_seq=2 ttl=51 time=263 ms",
                ),
                (0, "--- facebook.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (
                    0,
                    "ICMP payload: \"thefacebook.com — a social utility that connects you\"",
                ),
                (
                    0,
                    "port 443 responded: SSL not yet standardized on this machine",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "amazon.com",
            "205.251.242.103",
            "amazon",
            vec![
                (0, "PING amazon.com (205.251.242.103): 56 data bytes"),
                (
                    240,
                    "64 bytes from 205.251.242.103: icmp_seq=0 ttl=49 time=241 ms",
                ),
                (
                    240,
                    "64 bytes from 205.251.242.103: icmp_seq=1 ttl=49 time=239 ms",
                ),
                (
                    240,
                    "64 bytes from 205.251.242.103: icmp_seq=2 ttl=49 time=243 ms",
                ),
                (0, "--- amazon.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "ICMP payload: \"Earth's Biggest Bookstore\""),
            ],
        ));

        self.add(RemoteHost::egg(
            "youtube.com",
            "74.125.224.72",
            "youtube",
            vec![
                (0, "PING youtube.com (74.125.224.72): 56 data bytes"),
                (
                    260,
                    "64 bytes from 74.125.224.72: icmp_seq=0 ttl=52 time=261 ms",
                ),
                (
                    260,
                    "64 bytes from 74.125.224.72: icmp_seq=1 ttl=52 time=258 ms",
                ),
                (
                    260,
                    "64 bytes from 74.125.224.72: icmp_seq=2 ttl=52 time=263 ms",
                ),
                (0, "--- youtube.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "ICMP payload: \"Broadcast Yourself\""),
                (
                    0,
                    "WARNING: payload contained 18,802,501 bytes of streaming video data",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "twitter.com",
            "104.244.42.1",
            "twitter",
            vec![
                (0, "PING twitter.com (104.244.42.1): 56 data bytes"),
                (
                    220,
                    "64 bytes from 104.244.42.1: icmp_seq=0 ttl=50 time=221 ms",
                ),
                (
                    220,
                    "64 bytes from 104.244.42.1: icmp_seq=1 ttl=50 time=219 ms",
                ),
                (
                    220,
                    "64 bytes from 104.244.42.1: icmp_seq=2 ttl=50 time=223 ms",
                ),
                (0, "--- twitter.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "note: ICMP payload truncated at exactly 140 bytes"),
            ],
        ));

        self.add(RemoteHost::egg(
            "stackoverflow.com",
            "151.101.193.69",
            "stackoverflow",
            vec![
                (0, "PING stackoverflow.com (151.101.193.69): 56 data bytes"),
                (
                    230,
                    "64 bytes from 151.101.193.69: icmp_seq=0 ttl=51 time=231 ms",
                ),
                (
                    230,
                    "64 bytes from 151.101.193.69: icmp_seq=1 ttl=51 time=228 ms",
                ),
                (
                    230,
                    "64 bytes from 151.101.193.69: icmp_seq=2 ttl=51 time=233 ms",
                ),
                (0, "--- stackoverflow.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "ICMP payload: \"How do I exit vim?\""),
            ],
        ));

        self.add(RemoteHost::egg(
            "tor.org",
            "86.59.21.38",
            "tor_org",
            vec![
                (0, "PING tor.org (86.59.21.38): 56 data bytes"),
                (0, "0 bytes from ???: icmp_seq=0 ttl=0 time=0 ms"),
                (0, "0 bytes from ???: icmp_seq=1 ttl=0 time=0 ms"),
                (0, "0 bytes from ???: icmp_seq=2 ttl=0 time=0 ms"),
                (0, "--- tor.org ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (
                    0,
                    "note: response entirely anonymised — 0 of 17 hops visible",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "microsoft.com",
            "134.170.185.46",
            "microsoft",
            vec![
                (0, "PING microsoft.com (134.170.185.46): 56 data bytes"),
                (
                    280,
                    "64 bytes from 134.170.185.46: icmp_seq=0 ttl=46 time=281 ms",
                ),
                (
                    280,
                    "64 bytes from 134.170.185.46: icmp_seq=1 ttl=46 time=278 ms",
                ),
                (
                    280,
                    "64 bytes from 134.170.185.46: icmp_seq=2 ttl=46 time=283 ms",
                ),
                (0, "--- microsoft.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "ICMP payload: 412 critical updates pending"),
                (0, "note: remote host requires reboot to apply"),
            ],
        ));

        self.add(RemoteHost::egg(
            "apple.com",
            "17.178.96.59",
            "apple",
            vec![
                (0, "PING apple.com (17.178.96.59): 56 data bytes"),
                (1200, "Request timeout for icmp_seq 0"),
                (1200, "Request timeout for icmp_seq 1"),
                (1200, "Request timeout for icmp_seq 2"),
                (0, "--- apple.com ping statistics ---"),
                (0, "3 packets transmitted, 0 received, 100% packet loss"),
                (0, "note: ICMP only works on Apple hardware"),
                (0, "note: requires iTunes 1.0 or later"),
            ],
        ));

        self.add(RemoteHost::egg(
            "stripe.com",
            "54.187.216.72",
            "stripe",
            vec![
                (0, "PING stripe.com (54.187.216.72): 56 data bytes"),
                (
                    220,
                    "64 bytes from 54.187.216.72: icmp_seq=0 ttl=50 time=221 ms",
                ),
                (
                    220,
                    "64 bytes from 54.187.216.72: icmp_seq=1 ttl=50 time=219 ms",
                ),
                (
                    220,
                    "64 bytes from 54.187.216.72: icmp_seq=2 ttl=50 time=223 ms",
                ),
                (0, "--- stripe.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "note: ICMP payload charged $0.01 to account on file"),
                (0, "note: this ping cannot be refunded"),
            ],
        ));

        self.add(RemoteHost::egg(
            "aws.amazon.com",
            "205.251.242.103",
            "aws",
            vec![
                (0, "PING aws.amazon.com (205.251.242.103): 56 data bytes"),
                (
                    230,
                    "64 bytes from 205.251.242.103: icmp_seq=0 ttl=51 time=231 ms",
                ),
                (
                    230,
                    "64 bytes from 205.251.242.103: icmp_seq=1 ttl=51 time=228 ms",
                ),
                (
                    230,
                    "64 bytes from 205.251.242.103: icmp_seq=2 ttl=51 time=233 ms",
                ),
                (0, "--- aws.amazon.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "billing: $0.00001 per millisecond"),
                (
                    0,
                    "session bill: $0.00285 — invoice will be sent to your address",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "cloudflare.com",
            "104.16.0.1",
            "cloudflare",
            vec![
                (0, "PING cloudflare.com (104.16.0.1): 56 data bytes"),
                (5, "64 bytes from 104.16.0.1: icmp_seq=0 ttl=60 time=5 ms"),
                (5, "64 bytes from 104.16.0.1: icmp_seq=1 ttl=60 time=4 ms"),
                (5, "64 bytes from 104.16.0.1: icmp_seq=2 ttl=60 time=6 ms"),
                (0, "--- cloudflare.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (
                    0,
                    "note: served from anycast — origin unknown, 73 datacenters involved",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "bing.com",
            "204.79.197.200",
            "bing",
            vec![
                (0, "PING bing.com (204.79.197.200): 56 data bytes"),
                (
                    290,
                    "64 bytes from 204.79.197.200: icmp_seq=0 ttl=45 time=291 ms",
                ),
                (
                    290,
                    "64 bytes from 204.79.197.200: icmp_seq=1 ttl=45 time=288 ms",
                ),
                (
                    290,
                    "64 bytes from 204.79.197.200: icmp_seq=2 ttl=45 time=293 ms",
                ),
                (0, "--- bing.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "Did you mean: google.com?"),
            ],
        ));

        self.add(RemoteHost::egg(
            "duckduckgo.com",
            "54.225.61.12",
            "duckduckgo",
            vec![
                (0, "PING duckduckgo.com (54.225.61.12): 56 data bytes"),
                (
                    210,
                    "64 bytes from 54.225.61.12: icmp_seq=0 ttl=52 time=211 ms",
                ),
                (
                    210,
                    "64 bytes from 54.225.61.12: icmp_seq=1 ttl=52 time=208 ms",
                ),
                (
                    210,
                    "64 bytes from 54.225.61.12: icmp_seq=2 ttl=52 time=213 ms",
                ),
                (0, "--- duckduckgo.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "note: We didn't log your ping."),
                (0, "note: ICMP payload not tracked, stored, or sold."),
            ],
        ));

        self.add(RemoteHost::egg(
            "notion.so",
            "23.227.38.65",
            "notion",
            vec![
                (0, "PING notion.so (23.227.38.65): 56 data bytes"),
                (
                    147,
                    "64 bytes from 23.227.38.65: icmp_seq=0 ttl=53 time=147 ms",
                ),
                (
                    1847,
                    "64 bytes from 23.227.38.65: icmp_seq=1 ttl=53 time=1847 ms",
                ),
                (
                    148,
                    "64 bytes from 23.227.38.65: icmp_seq=2 ttl=53 time=148 ms",
                ),
                (0, "--- notion.so ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "note: loading spinner still visible on icmp_seq=1"),
            ],
        ));

        self.add(RemoteHost::egg(
            "gitlab.com",
            "52.167.219.168",
            "gitlab",
            vec![
                (0, "PING gitlab.com (52.167.219.168): 56 data bytes"),
                (
                    230,
                    "64 bytes from 52.167.219.168: icmp_seq=0 ttl=50 time=231 ms",
                ),
                (
                    230,
                    "64 bytes from 52.167.219.168: icmp_seq=1 ttl=50 time=228 ms",
                ),
                (
                    230,
                    "64 bytes from 52.167.219.168: icmp_seq=2 ttl=50 time=233 ms",
                ),
                (0, "--- gitlab.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "note: fork detected — 4 conflicting versions in transit"),
                (0, "note: merge conflict in icmp_seq=1"),
            ],
        ));

        self.add(RemoteHost::egg(
            "paypal.com",
            "66.211.169.3",
            "paypal",
            vec![
                (0, "PING paypal.com (66.211.169.3): 56 data bytes"),
                (
                    260,
                    "64 bytes from 66.211.169.3: icmp_seq=0 ttl=49 time=261 ms",
                ),
                (1200, "Request timeout for icmp_seq 1"),
                (
                    260,
                    "64 bytes from 66.211.169.3: icmp_seq=2 ttl=49 time=263 ms",
                ),
                (0, "--- paypal.com ping statistics ---"),
                (0, "3 packets transmitted, 2 received, 33% packet loss"),
                (
                    0,
                    "note: account frozen — please verify identity to continue",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "y2k.com",
            "209.67.202.208",
            "y2k",
            vec![
                (0, "PING y2k.com (209.67.202.208): 56 data bytes"),
                (
                    320,
                    "64 bytes from 209.67.202.208: icmp_seq=0 ttl=44 time=00:00:00.001 Jan 1 2000",
                ),
                (
                    320,
                    "64 bytes from 209.67.202.208: icmp_seq=1 ttl=44 time=00:00:00.001 Jan 1 2000",
                ),
                (
                    320,
                    "64 bytes from 209.67.202.208: icmp_seq=2 ttl=44 time=00:00:00.001 Jan 1 2000",
                ),
                (0, "--- y2k.com ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "note: all timestamps resolve to Jan 1 2000 00:00:00"),
            ],
        ));

        self.add(RemoteHost::egg(
            "ethereum.org",
            "104.18.14.101",
            "ethereum",
            vec![
                (0, "PING ethereum.org (104.18.14.101): 56 data bytes"),
                (
                    200,
                    "64 bytes from 104.18.14.101: icmp_seq=0 ttl=54 time=201 ms",
                ),
                (
                    200,
                    "64 bytes from 104.18.14.101: icmp_seq=1 ttl=54 time=198 ms",
                ),
                (
                    200,
                    "64 bytes from 104.18.14.101: icmp_seq=2 ttl=54 time=203 ms",
                ),
                (0, "--- ethereum.org ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "note: gas fee to complete ping: 0.0012 ETH"),
            ],
        ));

        self.add(RemoteHost::egg("slack.com", "54.192.1.1", "slack",
            vec![
                (0,   "PING slack.com (54.192.1.1): 56 data bytes"),
                (210, "64 bytes from 54.192.1.1: icmp_seq=0 ttl=52 time=211 ms"),
                (210, "64 bytes from 54.192.1.1: icmp_seq=1 ttl=52 time=208 ms"),
                (210, "64 bytes from 54.192.1.1: icmp_seq=2 ttl=52 time=213 ms"),
                (0,   "--- slack.com ping statistics ---"),
                (0,   "3 packets transmitted, 3 received, 0% packet loss"),
                (0,   "ICMP payload (translated from German): \"also ich bin vielleicht kein netter mensch\""),
            ],
        ));

        // --- Deep anomaly hosts ---

        self.add(RemoteHost::egg(
            "linus.torvalds.name",
            "127.0.0.1",
            "linus_name",
            vec![
                (0, "PING linus.torvalds.name (127.0.0.1): 56 data bytes"),
                (1, "64 bytes from 127.0.0.1: icmp_seq=0 ttl=64 time=0.1 ms"),
                (1, "64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=0.1 ms"),
                (1, "64 bytes from 127.0.0.1: icmp_seq=2 ttl=64 time=0.1 ms"),
                (0, "--- linus.torvalds.name ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "note: reverse DNS resolves to: you"),
            ],
        ));

        self.add(RemoteHost::egg(
            "crawlspace.net",
            "192.168.0.0",
            "crawlspace",
            vec![
                (0, "PING crawlspace.net (192.168.0.0): 56 data bytes"),
                (
                    340,
                    "64 bytes from 192.168.0.0: icmp_seq=0 ttl=32 time=341 ms timestamp=1987-03-12",
                ),
                (
                    340,
                    "64 bytes from 192.168.0.0: icmp_seq=1 ttl=32 time=338 ms timestamp=1987-03-12",
                ),
                (
                    340,
                    "64 bytes from 192.168.0.0: icmp_seq=2 ttl=32 time=343 ms timestamp=1987-03-12",
                ),
                (0, "--- crawlspace.net ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "WARNING: all timestamps 4 years before system clock"),
            ],
        ));

        self.add(RemoteHost::egg(
            "void.null",
            "0.0.0.0",
            "void_null",
            vec![
                (0, "PING void.null (0.0.0.0): 56 data bytes"),
                (50, "64 bytes from 0.0.0.0: icmp_seq=0 ttl=0 time=-4.2 ms"),
                (50, "64 bytes from 0.0.0.0: icmp_seq=1 ttl=0 time=-4.1 ms"),
                (50, "64 bytes from 0.0.0.0: icmp_seq=2 ttl=0 time=-4.3 ms"),
                (0, "--- void.null ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (
                    0,
                    "WARNING: negative round-trip time — replies arrived before request",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "ghost-hop",
            "255.255.255.0",
            "ghost_hop",
            vec![
                (0, "PING ghost-hop (255.255.255.0): 56 data bytes"),
                (
                    0,
                    "64 bytes from 255.255.255.0: icmp_seq=0 ttl=128 time=-0.04 ms",
                ),
                (
                    250,
                    "64 bytes from 255.255.255.0: icmp_seq=1 ttl=128 time=250 ms",
                ),
                (
                    250,
                    "64 bytes from 255.255.255.0: icmp_seq=2 ttl=128 time=251 ms",
                ),
                (0, "--- ghost-hop ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (
                    0,
                    "WARNING: icmp_seq=0 reply at -0.04ms — before packet was sent",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "mirror.null",
            "10.0.0.1",
            "mirror_null",
            vec![
                (0, "PING mirror.null (10.0.0.1): 56 data bytes"),
                (200, "64 bytes from 10.0.0.1: icmp_seq=0 ttl=64 time=201 ms"),
                (
                    0,
                    "64 bytes from 10.0.0.1: icmp_seq=0 ttl=64 time=201 ms DUP!",
                ),
                (200, "64 bytes from 10.0.0.1: icmp_seq=1 ttl=64 time=198 ms"),
                (
                    0,
                    "64 bytes from 10.0.0.1: icmp_seq=1 ttl=64 time=198 ms DUP!",
                ),
                (200, "64 bytes from 10.0.0.1: icmp_seq=2 ttl=64 time=203 ms"),
                (
                    0,
                    "64 bytes from 10.0.0.1: icmp_seq=2 ttl=64 time=203 ms DUP!",
                ),
                (0, "--- mirror.null ping statistics ---"),
                (0, "3 packets transmitted, 6 received, 0% packet loss"),
                (0, "note: all replies duplicated — reason unknown"),
            ],
        ));

        self.add(RemoteHost::egg(
            "echo.archive",
            "172.16.0.1",
            "echo_archive",
            vec![
                (0, "PING echo.archive (172.16.0.1): 56 data bytes"),
                (
                    280,
                    "64 bytes from 172.16.0.1: icmp_seq=0 ttl=48 time=281 ms [corruption: byte 23]",
                ),
                (
                    280,
                    "64 bytes from 172.16.0.1: icmp_seq=1 ttl=48 time=278 ms [corruption: byte 7]",
                ),
                (
                    280,
                    "64 bytes from 172.16.0.1: icmp_seq=2 ttl=48 time=283 ms [corruption: byte 31]",
                ),
                (0, "--- echo.archive ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (
                    0,
                    "note: data corruption detected on every reply — source unknown",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "limbo.route",
            "192.0.2.1",
            "limbo_route",
            vec![
                (0, "PING limbo.route (192.0.2.1): 56 data bytes"),
                (
                    300,
                    "64 bytes from 192.0.2.1: icmp_seq=1 ttl=44 time=301 ms",
                ),
                (900, "Request timeout for icmp_seq 2"),
                (
                    300,
                    "64 bytes from 192.0.2.1: icmp_seq=3 ttl=44 time=302 ms",
                ),
                (900, "Request timeout for icmp_seq 4"),
                (
                    300,
                    "64 bytes from 192.0.2.1: icmp_seq=7 ttl=44 time=298 ms",
                ),
                (0, "--- limbo.route ping statistics ---"),
                (0, "note: sequence skips — missing icmp_seq 2, 4, 5, 6"),
            ],
        ));

        self.add(RemoteHost::egg(
            "night-switch",
            "10.10.10.10",
            "night_switch",
            vec![
                (0, "PING night-switch (10.10.10.10): 56 data bytes"),
                (
                    300,
                    "64 bytes from 10.10.10.10: icmp_seq=3 ttl=62 time=301 ms",
                ),
                (
                    300,
                    "64 bytes from 10.10.10.10: icmp_seq=2 ttl=62 time=298 ms",
                ),
                (
                    300,
                    "64 bytes from 10.10.10.10: icmp_seq=1 ttl=62 time=303 ms",
                ),
                (
                    300,
                    "64 bytes from 10.10.10.10: icmp_seq=0 ttl=62 time=299 ms",
                ),
                (0, "--- night-switch ping statistics ---"),
                (0, "3 packets transmitted, 4 received, 0% packet loss"),
                (
                    0,
                    "note: sequence numbers descending — host time is running backwards",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "cold.tape",
            "10.9.8.7",
            "cold_tape",
            vec![
                (0, "PING cold.tape (10.9.8.7): 56 data bytes"),
                (800, "64 bytes from 10.9.8.7: icmp_seq=0 ttl=32 time=801 ms"),
                (800, "64 bytes from 10.9.8.7: icmp_seq=1 ttl=32 time=798 ms"),
                (800, "64 bytes from 10.9.8.7: icmp_seq=2 ttl=32 time=803 ms"),
                (0, "--- cold.tape ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "ICMP payload (readable): \"DO N\""),
            ],
        ));

        self.add(RemoteHost::egg(
            "unknown-peer",
            "169.254.0.1",
            "unknown_peer",
            vec![
                (0, "PING unknown-peer (169.254.0.1): 56 data bytes"),
                (
                    400,
                    "64 bytes from 169.254.0.1: icmp_seq=0 ttl=40 time=401 ms",
                ),
                (0, "169.254.0.1: who is there"),
                (
                    400,
                    "64 bytes from 169.254.0.1: icmp_seq=1 ttl=40 time=398 ms",
                ),
                (0, "169.254.0.1: who is there"),
                (
                    400,
                    "64 bytes from 169.254.0.1: icmp_seq=2 ttl=40 time=403 ms",
                ),
                (0, "169.254.0.1: who is there"),
                (0, "--- unknown-peer ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
            ],
        ));

        self.add(RemoteHost::egg(
            "hollow.link",
            "0.0.0.0",
            "hollow_link",
            vec![
                (0, "PING hollow.link (0.0.0.0): 56 data bytes"),
                (200, "From 0.0.0.0: Destination Net Unreachable"),
                (200, "From 0.0.0.0: Destination Net Unreachable"),
                (200, "From 0.0.0.0: Destination Net Unreachable"),
                (0, "--- hollow.link ping statistics ---"),
                (
                    0,
                    "3 packets transmitted, 0 received, +3 errors, 100% packet loss",
                ),
                (
                    0,
                    "note: destination 0.0.0.0 — the null route responds with its own absence",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "dusk-gw",
            "198.51.100.1",
            "dusk_gw",
            vec![
                (0, "PING dusk-gw (198.51.100.1): 56 data bytes"),
                (1200, "Request timeout for icmp_seq 0"),
                (1200, "Request timeout for icmp_seq 1"),
                (1200, "Request timeout for icmp_seq 2"),
                (0, "--- dusk-gw ping statistics ---"),
                (0, "3 packets transmitted, 0 received, 100% packet loss"),
                (
                    2800,
                    "64 bytes from 198.51.100.1: icmp_seq=0 ttl=52 time=5421 ms",
                ),
                (
                    0,
                    "note: packet arrived 5421ms after timeout — impossible routing delay",
                ),
            ],
        ));

        self.add(RemoteHost::egg(
            "unknown.global",
            "255.255.255.255",
            "unknown_global",
            vec![
                (0, "PING unknown.global (255.255.255.255): 56 data bytes"),
                (1200, "Request timeout for icmp_seq 0"),
                (1200, "Request timeout for icmp_seq 1"),
                (1200, "Request timeout for icmp_seq 2"),
                (0, "--- unknown.global ping statistics ---"),
                (0, "3 packets transmitted, 0 received, 100% packet loss"),
                (0, "nslookup: NXDOMAIN"),
                (0, "note: this host should not exist"),
            ],
        ));

        self.add(RemoteHost::egg(
            "void.gateway",
            "0.0.0.1",
            "void_gateway",
            vec![
                (0, "PING void.gateway (0.0.0.1): 56 data bytes"),
                (
                    120,
                    "64 bytes from 0.0.0.1: icmp_seq=0 ttl=0 time=121 ms date=1987-01-01",
                ),
                (
                    120,
                    "64 bytes from 0.0.0.1: icmp_seq=1 ttl=0 time=118 ms date=1987-01-01",
                ),
                (
                    120,
                    "64 bytes from 0.0.0.1: icmp_seq=2 ttl=0 time=123 ms date=1987-01-01",
                ),
                (0, "--- void.gateway ping statistics ---"),
                (0, "3 packets transmitted, 3 received, 0% packet loss"),
                (0, "WARNING: remote timestamp 4 years before system epoch"),
            ],
        ));
    }
}

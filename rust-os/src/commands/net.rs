use crate::commands::Command;
use crate::kernel::unit_of_work::UnitOfWork;
use crate::kernel::Kernel;

pub struct PingCmd;
impl Command for PingCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let host = match args.get(1) {
            Some(h) => *h,
            None => {
                uow.print("usage: ping host");
                return;
            }
        };

        // Check /etc/hosts first, then registry
        let resolved = kernel.vfs.read_file("/etc/hosts").and_then(|content| {
            let content = content.to_string();
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1..].contains(&host) {
                    return Some(parts[0].to_string());
                }
            }
            None
        });

        if let Some(remote) = kernel
            .network
            .active_connections
            .values()
            .next()
            .cloned()
            .or_else(|| None)
        {
            // placeholder
            let _ = remote;
        }

        // For now use a simplified ping implementation
        let ip = resolved.unwrap_or_else(|| format!("{}.0.1", host.len()));

        // Check if this is a known easter egg (by hostname pattern matching)
        let is_loopback = host == "localhost" || host == "kruuna" || host == "127.0.0.1";

        if is_loopback {
            uow.print(format!("PING {host} ({ip}): 56 data bytes"));
            for i in 0..4 {
                uow.schedule(
                    format!("64 bytes from {ip}: icmp_seq={i} ttl=255 time=0.1 ms"),
                    10,
                );
            }
            uow.schedule(format!("--- {host} ping statistics ---"), 100);
            uow.schedule(
                "4 packets transmitted, 4 received, 0% packet loss".to_string(),
                0,
            );
            uow.schedule("round-trip min/avg/max = 0.1/0.1/0.1 ms".to_string(), 0);
        } else {
            // Generic real host response
            let base_ms = 180u64;
            uow.print(format!("PING {host} ({ip}): 56 data bytes"));
            for i in 0..4 {
                let t = base_ms + (i * 3) as u64;
                uow.schedule(
                    format!("64 bytes from {ip}: icmp_seq={i} ttl=52 time={t} ms"),
                    250,
                );
            }
            uow.schedule(format!("--- {host} ping statistics ---"), 150);
            uow.schedule(
                "4 packets transmitted, 4 received, 0% packet loss".to_string(),
                0,
            );
            uow.schedule(
                format!(
                    "round-trip min/avg/max = {base_ms}/{}/{} ms",
                    base_ms + 4,
                    base_ms + 8
                ),
                0,
            );
        }
    }
}

pub struct PingWithRegistry<'a> {
    pub index: &'a crate::hosts::RemoteHostIndex,
}

/// Standalone ping function used by AppHost when registry is available.
pub fn ping_with_registry(
    host: &str,
    index: &crate::hosts::RemoteHostIndex,
    uow: &mut UnitOfWork,
    kernel: &mut Kernel,
) {
    use crate::hosts::HostKind;

    let remote = index.resolve(host);

    match remote {
        None => {
            uow.print(format!("ping: unknown host {host}"));
        }
        Some(r) if r.kind == HostKind::Loopback => {
            let ip = r.ip.clone();
            uow.print(format!("PING {host} ({ip}): 56 data bytes"));
            for i in 0..4 {
                uow.schedule(
                    format!("64 bytes from {ip}: icmp_seq={i} ttl=255 time=0.1 ms"),
                    5,
                );
            }
            uow.schedule(format!("--- {host} ping statistics ---"), 50);
            uow.schedule(
                "4 packets transmitted, 4 received, 0% packet loss".to_string(),
                0,
            );
            uow.schedule("round-trip min/avg/max = 0.1/0.1/0.1 ms".to_string(), 0);
        }
        Some(r) if r.kind == HostKind::EasterEgg => {
            // Record anomaly
            let tag = r.anomaly_tag.clone().unwrap_or_else(|| host.to_string());
            uow.quest.note_anomaly(&tag);

            // Decay tier journal entries
            let clock_str = kernel.clock.time_str();
            kernel.journal.append_anomaly(&r.ip, &clock_str);
            if uow.quest.anomaly_count() >= 5 {
                kernel.journal.append_clock_drift(&clock_str);
            }
            if uow.quest.anomaly_count() >= 7 {
                let inode = (kernel.uptime_ms() % 9000 + 1000) as u32;
                kernel.journal.append_inode_anomaly(inode, &clock_str);
            }

            // Output the easter egg ping lines
            for (delay, text) in &r.ping_lines {
                uow.schedule(text.clone(), *delay);
            }

            // Write to /usr/adm/net.trace
            let trace_entry = format!("{}  {}  {} [ANOMALY]", kernel.clock.time_str(), host, r.ip);
            let existing = kernel
                .vfs
                .read_file("/usr/adm/net.trace")
                .map(|s| s.to_string())
                .unwrap_or_default();
            kernel.vfs.write_file(
                "/usr/adm/net.trace",
                &format!("{existing}{trace_entry}\n"),
                "root",
            );
        }
        Some(r) => {
            let ip = r.ip.clone();
            let base = r.base_ping_ms;
            uow.print(format!("PING {host} ({ip}): 56 data bytes"));
            for i in 0..4 {
                let t = base + (i * 2) as u64;
                uow.schedule(
                    format!("64 bytes from {ip}: icmp_seq={i} ttl=52 time={t} ms"),
                    base + 50,
                );
            }
            uow.schedule(format!("--- {host} ping statistics ---"), 150);
            uow.schedule(
                "4 packets transmitted, 4 received, 0% packet loss".to_string(),
                0,
            );
            uow.schedule(
                format!(
                    "round-trip min/avg/max = {base}/{}/{} ms",
                    base + 3,
                    base + 7
                ),
                0,
            );
        }
    }
}

pub struct NslookupCmd;
impl Command for NslookupCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let host = match args.get(1) {
            Some(h) => *h,
            None => {
                uow.print("usage: nslookup host");
                return;
            }
        };

        // Check /etc/hosts
        let found = kernel.vfs.read_file("/etc/hosts").and_then(|content| {
            let content = content.to_string();
            for line in content.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1..].iter().any(|&n| n == host) {
                    return Some((parts[0].to_string(), parts[1].to_string()));
                }
            }
            None
        });

        uow.print(format!("Server:  localhost"));
        uow.print(format!("Address: 127.0.0.1"));
        uow.print("".to_string());

        if let Some((ip, canonical)) = found {
            uow.schedule(format!("Name:    {canonical}"), 80);
            uow.schedule(format!("Address: {ip}"), 0);
        } else {
            uow.schedule(format!("*** localhost can't find {host}: NXDOMAIN"), 80);
        }
    }
}

pub struct NetstatCmd;
impl Command for NetstatCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        let anomaly_count = uow.quest.anomaly_count();
        let ftp_connected = uow.quest.ftp_connected;

        uow.print("Active Internet connections (servers and established)");
        uow.print("Proto  Local Address          Foreign Address        State");
        uow.print("tcp    0.0.0.0:21             0.0.0.0:*              LISTEN");
        uow.print("tcp    0.0.0.0:23             0.0.0.0:*              LISTEN");

        if ftp_connected {
            let remote = uow
                .quest
                .ftp_remote_host
                .clone()
                .unwrap_or_else(|| "nic.funet.fi".to_string());
            uow.print(format!(
                "tcp    kruuna:1024             {remote}:21            ESTABLISHED"
            ));
        }

        // Anomaly port appears at high decay
        if anomaly_count >= 3 {
            uow.print("tcp    0.0.0.0:???             0.0.0.0:*              LISTEN");
        }

        uow.print("".to_string());
        uow.print("Active UNIX domain sockets (only servers)");
        uow.print("Proto  RefCnt  Type        State       I-Node  Path");
        uow.print("unix   2       STREAM      CONNECTED   142     /tmp/.X11-unix/X0");
    }
}

pub struct IfconfigCmd;
impl Command for IfconfigCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        uow.print("eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500");
        uow.print("      inet 128.214.220.32  netmask 255.255.255.0  broadcast 128.214.220.255");
        uow.print("      ether 00:60:97:1a:2b:3c  txqueuelen 100");
        uow.print("".to_string());
        uow.print("lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536");
        uow.print("      inet 127.0.0.1  netmask 255.0.0.0");
    }
}

pub struct FtpCmd;
impl Command for FtpCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        // FTP is handled as an Application; this command just hints at it
        let host = args.get(1).copied().unwrap_or("nic.funet.fi");
        uow.print(format!("Connected to {host}."));
        uow.print(format!("220 {host} FTP server (Version 6.4) ready."));
        uow.print("Name (torvalds): ".to_string());
        uow.print("Use the ftp application for full FTP session.".to_string());
    }
}

pub struct FingerCmd;
impl Command for FingerCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let anomaly_count = uow.quest.anomaly_count();
        let upload_success = uow.quest.upload_success;

        if let Some(user) = args.get(1) {
            // finger user
            let path = format!("/usr/{user}/.plan");
            let plan = kernel
                .vfs
                .read_file(&path)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "No plan.".to_string());
            uow.print(format!("Login: {user}"));
            uow.print(format!("Plan:\n{plan}"));
            return;
        }

        uow.print("Login     Name                 Tty      Idle  Login  Time");
        uow.print(format!(
            "torvalds  Linus Torvalds       tty0        Sep 17 21:12"
        ));
        uow.print(format!(
            "ast       A.S. Tanenbaum       tty1     14  Sep 17 21:12"
        ));

        if anomaly_count >= 2 && !upload_success {
            // (null) session visible — hides after upload success
            uow.print(format!(
                "(null)    ???                  tty2      0  Sep 17 21:12"
            ));
        }

        // After tier 3, .plan for (null)
        if anomaly_count >= 9 {
            uow.print("".to_string());
            uow.print("  (null) .plan:".to_string());
            uow.print("  I was here before you".to_string());
        }
    }
}

pub struct TelnetCmd;
impl Command for TelnetCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        let host = args.get(1).copied().unwrap_or("localhost");
        let port = args.get(2).and_then(|p| p.parse::<u16>().ok());
        let anomaly_count = uow.quest.anomaly_count();

        match (host, port) {
            ("localhost", _) | ("kruuna", _) => {
                uow.print(format!("Trying 127.0.0.1..."));
                uow.schedule("Connected to localhost.".to_string(), 200);
                uow.schedule("Escape character is '^]'.".to_string(), 0);
                uow.schedule("".to_string(), 0);
                uow.schedule("kruuna login: ".to_string(), 300);
                uow.schedule("Connection closed by foreign host.".to_string(), 500);
            }
            ("info.cern.ch", Some(80)) | ("info.cern.ch", None) => {
                uow.print("Trying 128.141.201.74...");
                uow.schedule("Connected to info.cern.ch.".to_string(), 300);
                uow.schedule("Escape character is '^]'.".to_string(), 0);
                uow.schedule("".to_string(), 0);
                uow.schedule("<TITLE>The World Wide Web project</TITLE>".to_string(), 400);
                uow.schedule("<NEXTID N=\"55\">".to_string(), 0);
                uow.schedule("<H1>World Wide Web</H1>".to_string(), 0);
                uow.schedule("The WorldWideWeb (W3) is a wide-area hypermedia information retrieval initiative".to_string(), 0);
                uow.schedule(
                    "aiming to give universal access to a large universe of documents.".to_string(),
                    0,
                );
                uow.schedule("".to_string(), 0);
                uow.schedule("Connection closed by foreign host.".to_string(), 800);
            }
            ("void.null", _) if anomaly_count >= 9 => {
                uow.print("Trying 0.0.0.0...");
                uow.schedule("Connected.".to_string(), 500);
                let msg = "can you hear me";
                let mut delay = 600u64;
                for c in msg.chars() {
                    uow.schedule(c.to_string(), delay);
                    delay = 200;
                }
                uow.schedule("".to_string(), 800);
                uow.schedule("Connection closed by foreign host.".to_string(), 400);
            }
            _ => {
                let h = host.to_string();
                uow.print(format!("Trying {h}..."));
                uow.schedule(
                    format!("telnet: connect to address {h}: Connection refused"),
                    800,
                );
            }
        }
    }
}

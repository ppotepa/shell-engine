use crate::commands::Command;
use crate::kernel::unit_of_work::UnitOfWork;
use crate::kernel::Kernel;

pub struct DfCmd;
impl Command for DfCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let decay = uow.quest.decay_tier();
        uow.print("Filesystem    1K-blocks     Used    Avail Use%  Mounted on");
        for m in kernel.mounts.get_mounts() {
            let used = if decay >= 1 {
                // slight flicker at decay 1+
                let jitter = (kernel.uptime_ms() % 3) as u32;
                m.used_kb + jitter
            } else {
                m.used_kb
            };
            let avail = m.total_kb.saturating_sub(used);
            let pct = (used as f32 / m.total_kb as f32 * 100.0) as u32;
            uow.print(format!(
                "{:<14} {:9} {:8} {:8}  {:2}%  {}",
                m.device, m.total_kb, used, avail, pct, m.mount_point
            ));
        }
    }
}

pub struct MountCmd;
impl Command for MountCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        for m in kernel.mounts.get_mounts() {
            uow.print(format!(
                "{} on {} type {} ({})",
                m.device, m.mount_point, m.fs_type, m.options
            ));
        }
    }
}

pub struct DateCmd;
impl Command for DateCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let decay = uow.quest.decay_tier();
        let date = kernel.clock.now_str();
        if decay >= 1 && kernel.uptime_ms().is_multiple_of(15) {
            // Brief wrong year
            uow.print("Mon Sep 17 21:12:00 EET 1977".to_string());
            uow.schedule(date, 100);
        } else {
            uow.print(date);
        }
    }
}

pub struct UnameCmd;
impl Command for UnameCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let all = args.contains(&"-a");
        let sys = args.contains(&"-s") || all;
        let node = args.contains(&"-n") || all;
        let rel = args.contains(&"-r") || all;
        let ver = args.contains(&"-v") || all;
        let mach = args.contains(&"-m") || all;

        if all {
            uow.print(format!(
                "MINIX kruuna.helsinki.fi 1.1 #1 {} {}",
                kernel.clock.now_str(),
                "i386"
            ));
        } else if sys || node || rel || ver || mach {
            let mut parts = Vec::new();
            if sys {
                parts.push("MINIX");
            }
            if node {
                parts.push("kruuna.helsinki.fi");
            }
            if rel {
                parts.push("1.1");
            }
            if ver {
                parts.push("#1");
            }
            if mach {
                parts.push("i386");
            }
            uow.print(parts.join(" "));
        } else {
            uow.print("MINIX");
        }
    }
}

pub struct HostnameCmd;
impl Command for HostnameCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let hostname = kernel
            .vfs
            .read_file("/etc/hostname")
            .unwrap_or("kruuna")
            .trim()
            .to_string();
        let decay = uow.quest.decay_tier();
        if decay >= 2 && kernel.uptime_ms().is_multiple_of(13) {
            uow.print(format!("{hostname}?"));
        } else {
            uow.print(hostname);
        }
    }
}

pub struct WhoamiCmd;
impl Command for WhoamiCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        uow.print(uow.session.user.clone());
    }
}

pub struct WhoCmd;
impl Command for WhoCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let anomaly_count = uow.quest.anomaly_count();
        let time = kernel.clock.time_str();
        uow.print(format!("torvalds  tty0  Sep 17 {time}"));
        uow.print("ast       tty1  Sep 17 21:12".to_string());
        if anomaly_count >= 2 {
            uow.print("(null)    tty2  Sep 17 21:12".to_string());
        }
    }
}

pub struct WCmd;
impl Command for WCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let anomaly_count = uow.quest.anomaly_count();
        let time = kernel.clock.time_str();
        let uptime = kernel.clock.uptime_str();
        let load = match uow.quest.decay_tier() {
            0 => "0.04",
            1 => "0.12",
            2 => "0.31",
            _ => "0.87",
        };
        uow.print(format!(
            " {time} up {uptime},  {} users,  load average: {load}, {load}, {load}",
            if anomaly_count >= 2 { 3 } else { 2 }
        ));
        uow.print(format!(
            "{:<9}{:<9}{:<14}{:<9}{:<7}{}",
            "USER", "TTY", "FROM", "LOGIN@", "IDLE", "WHAT"
        ));
        uow.print(format!(
            "{:<9}{:<9}{:<14}{:<9}{:<7}{}",
            "torvalds", "tty0", ":0", "21:12", "0:00", "sh"
        ));
        uow.print(format!(
            "{:<9}{:<9}{:<14}{:<9}{:<7}{}",
            "ast", "tty1", "cs.vu.nl", "21:12", "14:22", "vi /usr/src/minix/kernel/proc.c"
        ));
        if anomaly_count >= 2 {
            uow.print(format!(
                "{:<9}{:<9}{:<14}{:<9}{:<7}{}",
                "(null)", "tty2", "???", "21:12", "0:00", "-"
            ));
        }
    }
}

pub struct IdCmd;
impl Command for IdCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let user = &uow.session.user;
        let (uid, gid) = if user == "root" {
            (0u32, 0u32)
        } else {
            (1000, 10)
        };
        let groups = kernel.users.groups_for_user(user).join(", ");
        uow.print(format!(
            "uid={uid}({user}) gid={gid}(staff) groups={gid}({groups})"
        ));
    }
}

pub struct SyncCmd;
impl Command for SyncCmd {
    fn execute(&self, _args: &[&str], _uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        // silent success
    }
}

pub struct DmesgCmd;
impl Command for DmesgCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        // Static kernel boot messages
        let spec = &kernel.spec;
        let ram = spec.ram_kb;
        let avail = ram - 512;
        uow.print("Sep 17 21:12:00 kernel: MINIX 1.1 (i386)".to_string());
        uow.print(format!(
            "Sep 17 21:12:00 kernel: memory: {ram}K total, {avail}K available"
        ));
        uow.print(format!(
            "Sep 17 21:12:01 hd1: Seagate ST-157A, {}K",
            spec.disk_kb
        ));
        uow.print("Sep 17 21:12:01 eth0: NE2000 compatible at 0x300".to_string());
        uow.print(format!("Sep 17 21:12:01 rs232: {} baud", spec.modem_baud));
        uow.print("Sep 17 21:12:02 tty0: getty started".to_string());

        // Journal anomaly entries
        for entry in kernel.journal.all() {
            uow.print(entry.clone());
        }
    }
}

pub struct LastCmd;
impl Command for LastCmd {
    fn execute(&self, _args: &[&str], uow: &mut UnitOfWork, _kernel: &mut Kernel) {
        let anomaly_count = uow.quest.anomaly_count();
        uow.print(format!(
            "{:<10}{:<8}{:<20}{}",
            "torvalds", "tty0", ":0", "Mon Sep 17 21:12   still logged in"
        ));
        uow.print(format!(
            "{:<10}{:<8}{:<20}{}",
            "ast", "tty1", "cs.vu.nl", "Mon Sep 17 21:12   still logged in"
        ));
        uow.print(format!(
            "{:<10}{:<8}{:<20}{}",
            "reboot", "~", "system boot", "Mon Sep 17 21:12"
        ));
        if anomaly_count >= 3 {
            uow.print(format!(
                "{:<10}{:<8}{:<20}{}",
                "(null)", "tty2", "0.0.0.0", "Mon Sep 17 21:11   still logged in"
            ));
            uow.print("note: (null) login predates system boot by 1 second".to_string());
        }
        uow.print("".to_string());
        uow.print("wtmp begins Mon Sep 17 21:12".to_string());
    }
}

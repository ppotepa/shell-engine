use crate::commands::Command;
use crate::kernel::unit_of_work::UnitOfWork;
use crate::kernel::Kernel;

pub struct PsCmd;
impl Command for PsCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let long = args.contains(&"-aux") || args.contains(&"-ef") || args.contains(&"-a");
        let decay = uow.quest.decay_tier();

        // Inject phantom at tier 2+
        if decay >= 2 {
            kernel.process.inject_phantom_process();
        } else {
            kernel.process.remove_phantom_process();
        }

        if long {
            uow.print("  PID  PPID  UID STAT    SZ  TTY  TIME COMMAND");
            for p in kernel.process.list() {
                uow.print(format!(
                    "{:5} {:5} {:4}    {} {:5}  {:4} 0:00 {}",
                    p.pid, p.ppid, p.uid, p.state_ch, p.sz_kb, p.tty, p.name
                ));
            }
        } else {
            uow.print("  PID STAT    SZ  TTY  TIME COMMAND");
            for p in kernel.process.list() {
                if p.user == uow.session.user || p.pid <= 1 {
                    uow.print(format!(
                        "{:5}    {} {:5}  {:4} 0:00 {}",
                        p.pid, p.state_ch, p.sz_kb, p.tty, p.name
                    ));
                }
            }
        }
    }
}

pub struct KillCmd;
impl Command for KillCmd {
    fn execute(&self, args: &[&str], uow: &mut UnitOfWork, kernel: &mut Kernel) {
        let pids: Vec<u32> = args
            .iter()
            .skip(1)
            .filter(|a| !a.starts_with('-'))
            .filter_map(|a| a.parse().ok())
            .collect();

        if pids.is_empty() {
            uow.print("usage: kill [-signal] pid...");
            return;
        }

        for pid in pids {
            if pid == 31337 {
                uow.print(format!("kill: ({pid}) - Operation not permitted"));
                continue;
            }
            if pid <= 5 {
                uow.print(format!("kill: ({pid}) - Operation not permitted"));
                continue;
            }
            if kernel.process.kill(pid) {
                // silent success
            } else {
                uow.print(format!("kill: ({pid}) - No such process"));
            }
        }
    }
}

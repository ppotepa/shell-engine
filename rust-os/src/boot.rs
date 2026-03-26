use crate::difficulty::MachineSpec;
use crate::kernel::unit_of_work::ScheduledLine;

/// Generate boot sequence steps scaled by CPU speed.
pub fn build_boot_steps(spec: &MachineSpec) -> Vec<ScheduledLine> {
    let f = spec.operation_speed_multiplier;
    let mhz = spec.cpu_mhz;
    let ram = spec.ram_kb;
    let disk = spec.disk_kb;
    let baud = spec.modem_baud;

    let mut steps = Vec::new();
    let mut t = 0u64;

    macro_rules! step {
        ($delay:expr, $text:expr) => {{
            t += ($delay as f64 * f) as u64;
            steps.push(ScheduledLine {
                due_ms: t,
                text: $text.to_string(),
            });
        }};
    }

    // Boot monitor
    step!(0, "MINIX boot");
    step!(120, format!("Loading kernel... ({mhz} MHz, {ram}K RAM)"));
    step!(300, format!("MINIX 1.1 (i386)"));
    step!(
        200,
        format!("Copyright (c) 1987,1989,1991 Prentice-Hall, Inc.")
    );
    step!(
        400,
        format!(
            "memory: {ram}K total  (kernel 512K, available {}K)",
            ram - 512
        )
    );

    // Task startup
    step!(350, "starting tasks: clock");
    step!(80, "starting tasks: clock, memory");
    step!(
        80,
        format!("starting tasks: clock, memory, winchester ({disk}K)")
    );
    step!(
        100,
        format!("starting tasks: clock, memory, winchester, tty, rs232 ({baud} baud)")
    );

    // Filesystem
    step!(350, "mounting /dev/hd1 on /");
    step!(200, "mounting /dev/hd2 on /usr");
    step!(500, "file system check: clean");

    // Init and services
    step!(200, "init: starting /etc/rc");
    step!(100, "update: daemon started (pid 4)");
    if ram >= 2048 {
        step!(80, "cron: daemon started (pid 5)");
    }

    // Getty
    step!(180, "getty on tty0");
    step!(100, "");
    step!(0, format!("MINIX 1.1  kruuna.helsinki.fi"));
    step!(0, "");

    steps
}

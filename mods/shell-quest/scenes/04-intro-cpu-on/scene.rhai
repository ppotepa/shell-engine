fn clear_bios_lines(commands) {
    commands.push(#{ op: "set-text", target: "bios-line-1", text: " " });
    commands.push(#{ op: "set-text", target: "bios-line-2", text: " " });
    commands.push(#{ op: "set-text", target: "bios-line-3", text: " " });
    commands.push(#{ op: "set-text", target: "bios-line-4", text: " " });
    commands.push(#{ op: "set-text", target: "bios-line-5", text: " " });
    commands.push(#{ op: "set-text", target: "bios-line-6", text: " " });
    commands.push(#{ op: "set-text", target: "bios-line-7", text: " " });
    commands.push(#{ op: "set-text", target: "bios-line-8", text: " " });
    commands.push(#{ op: "set-text", target: "bios-line-9", text: " " });
    commands.push(#{ op: "set-text", target: "bios-line-10", text: " " });
    commands.push(#{ op: "set-text", target: "bios-line-11", text: " " });
    commands.push(#{ op: "set-text", target: "bios-footer-1", text: " " });
    commands.push(#{ op: "set-text", target: "bios-footer-2", text: " " });
}

let commands = [];
let t = scene_elapsed_ms;

if t >= 1800 && t < 18800 {
    clear_bios_lines(commands);
}

if t >= 2200 && t < 5600 {
    commands.push(#{ op: "set-text", target: "bios-line-1", text: "Reading boot sector..." });
    commands.push(#{ op: "set-text", target: "bios-line-2", text: "Boot sector loaded" });
    commands.push(#{ op: "set-text", target: "bios-line-3", text: "MINIX boot block" });
    commands.push(#{ op: "set-text", target: "bios-line-4", text: "Loading secondary boot..." });
    commands.push(#{ op: "set-text", target: "bios-line-5", text: "Secondary boot loaded" });
    commands.push(#{ op: "set-text", target: "bios-line-6", text: "MINIX boot monitor 1.3" });
    commands.push(#{ op: "set-text", target: "bios-line-7", text: "Root device: hd1a" });
    commands.push(#{ op: "set-text", target: "bios-line-8", text: "RAM disk: disabled" });
    commands.push(#{ op: "set-text", target: "bios-line-9", text: "Press ESC for monitor" });
    commands.push(#{ op: "set-text", target: "bios-line-10", text: "Booting MINIX..." });
    commands.push(#{ op: "set-text", target: "bios-footer-1", text: "[gray]Page 1/4: Boot loader[/]" });
} else if t >= 5600 && t < 9000 {
    commands.push(#{ op: "set-text", target: "bios-line-1", text: "286/386 protected mode enabled" });
    commands.push(#{ op: "set-text", target: "bios-line-2", text: "Kernel size: 109 KB" });
    commands.push(#{ op: "set-text", target: "bios-line-3", text: "Initializing interrupt vectors" });
    commands.push(#{ op: "set-text", target: "bios-line-4", text: "Initializing trap handlers" });
    commands.push(#{ op: "set-text", target: "bios-line-5", text: "Initializing memory manager" });
    commands.push(#{ op: "set-text", target: "bios-line-6", text: "Memory available to system: 3712 KB" });
    commands.push(#{ op: "set-text", target: "bios-line-7", text: "Buffer cache: 128 KB" });
    commands.push(#{ op: "set-text", target: "bios-line-8", text: "Detecting clock..." });
    commands.push(#{ op: "set-text", target: "bios-line-9", text: "[#ff5555]WARNING[/] CMOS time appears invalid" });
    commands.push(#{ op: "set-text", target: "bios-line-10", text: "Initializing tty drivers" });
    commands.push(#{ op: "set-text", target: "bios-line-11", text: "tty0 initialized, tty1 initialized" });
    commands.push(#{ op: "set-text", target: "bios-footer-1", text: "[gray]Page 2/4: Kernel init[/]" });
} else if t >= 9000 && t < 12400 {
    commands.push(#{ op: "set-text", target: "bios-line-1", text: "Initializing floppy driver (fd0 present)" });
    commands.push(#{ op: "set-text", target: "bios-line-2", text: "Initializing hard disk driver" });
    commands.push(#{ op: "set-text", target: "bios-line-3", text: "hd0: 977 cylinders, 5 heads, 17 sectors" });
    commands.push(#{ op: "set-text", target: "bios-line-4", text: "Partition table read successfully" });
    commands.push(#{ op: "set-text", target: "bios-line-5", text: "Root file system on /dev/hd1a" });
    commands.push(#{ op: "set-text", target: "bios-line-6", text: "Mounting root file system..." });
    commands.push(#{ op: "set-text", target: "bios-line-7", text: "/dev/hd1a mounted read-write [#55ff55]SUCCESS[/]" });
    commands.push(#{ op: "set-text", target: "bios-line-8", text: "Checking file system state..." });
    commands.push(#{ op: "set-text", target: "bios-line-9", text: "[#ff5555]WARNING[/] free block count incorrect" });
    commands.push(#{ op: "set-text", target: "bios-line-10", text: "Rebuilding free block map..." });
    commands.push(#{ op: "set-text", target: "bios-line-11", text: "File system check complete [#55ff55]SUCCESS[/]" });
    commands.push(#{ op: "set-text", target: "bios-footer-1", text: "[gray]Page 3/4: File system[/]" });
} else if t >= 12400 && t < 18800 {
    commands.push(#{ op: "set-text", target: "bios-line-1", text: "Mounting /usr from /dev/hd1d" });
    commands.push(#{ op: "set-text", target: "bios-line-2", text: "/dev/hd1d mounted read-only [#55ff55]SUCCESS[/]" });
    commands.push(#{ op: "set-text", target: "bios-line-3", text: "Starting init process" });
    commands.push(#{ op: "set-text", target: "bios-line-4", text: "Executing /etc/rc" });
    commands.push(#{ op: "set-text", target: "bios-line-5", text: "Setting system name: minix" });
    commands.push(#{ op: "set-text", target: "bios-line-6", text: "Starting update daemon / clock task sync" });
    commands.push(#{ op: "set-text", target: "bios-line-7", text: "Starting printer daemon / login processes" });
    commands.push(#{ op: "set-text", target: "bios-line-8", text: "IRQ 06  FLOPPY DISK  [#ff5555]FAIL[/]" });
    commands.push(#{ op: "set-text", target: "bios-line-9", text: "Minix 1.3  Copyright 1987, Prentice-Hall" });
    commands.push(#{ op: "set-text", target: "bios-line-10", text: "Console ready [#55ff55]SUCCESS[/]" });
    commands.push(#{ op: "set-text", target: "bios-line-11", text: "login:" });
    commands.push(#{ op: "set-text", target: "bios-footer-1", text: "[gray]Page 4/4: Init and login[/]" });
    commands.push(#{ op: "set-text", target: "bios-footer-2", text: "[#808080]Press ESC for monitor[/]" });
}

commands

let t = time.scene_elapsed_ms;
let text = " ";

// 1) BIOS + TUX screen
if t >= 1800 && t < 6200 {
    text = `TUX Modular BIOS v1.90, 1990 edition
Copyright (C) 1990, TUX Software, Inc.
* TUX AWARDS BIOS Extension v1.0A

`;

    if t >= 2500 {
        text += `CPU: 386QX compatible @ 20MHz
`;
    }

    if t >= 3000 && t < 4300 {
        let step = ((t - 3000) * 16) / 1300;
        let s = if step > 16 { 16 } else { step };
        text += `Checking RAM: ${s * 128}KB
`;
    } else if t >= 4300 && t < 5000 {
        let blink = ((t - 4300) / 175) % 4;
        if blink == 0 || blink == 2 {
            text += `[white]Checking RAM: 2048KB OK[/]
`;
        } else {
            text += `Checking RAM: 2048KB OK
`;
        }
    } else if t >= 5000 {
        text += `[white]Checking RAM: 2048KB OK[/]
`;
    }

    if t >= 4800 {
        text += `CD-ROM drive A:        .......... not found
`;
    }
    if t >= 5200 {
        text += `Floppy drive A: 1.44MB .......... not found
`;
    }
    if t >= 5600 {
        text += `Hard drive 0: SETAGE ST225 20MB
`;
    }

// 2) Separate device screen (1.5s)
} else if t >= 6200 && t < 7700 {
    text = `+------------------------------------------------------------------+
| TUX HyperBIOS device listing                     19/09/90 04:12 |
+------------------------------------------------------------------+

 CPU              : 386SX compatible @ 20MHz
 Extended memory  : 2048KB
 Cache size       : 64KB
 Display type     : VGA
 Serial port(s)   : 3F8 2F8
 Parallel port(s) : 378
 DRAM type        : FPM
 Data integrity   : ECC

 Diskette drive A : 1.44MB 3.5in
 Pri. master disk : SETAGE ST225 20MB
 Sec. master disk : CD-ROM (none)

 PCI/ISA device listing...
 00:00.0  Host bridge            IRQ 14/15
 00:02.0  VGA compatible         IRQ 11
 00:03.0  IDE controller         IRQ 14
 00:04.0  Floppy controller      IRQ 06`;

// 3) Blank gap (0.5s)
} else if t >= 7700 && t < 8200 {
    text = " ";

// 4) Fast boot sequence (8s), no clear — scroll up
} else if t >= 8200 && t < 16200 {
    // Freeze last visible frame for 0.5s (15700..16200)
    let p = if t < 15700 { t - 8200 } else { 7500 };
    let lines = [];

    if p >= 0    { lines.push("Starting from hard disk"); }
    if p >= 1000 { lines.push("Reading boot sector..."); }
    if p >= 1200 { lines.push("Boot sector loaded"); }
    if p >= 1450 { lines.push("MINIX boot block"); }
    if p >= 1700 { lines.push("Loading secondary boot..."); }
    if p >= 2000 { lines.push("Secondary boot loaded"); }
    if p >= 2250 { lines.push("MINIX boot monitor 1.3"); }
    if p >= 2500 { lines.push("Root device: hd1a"); }
    if p >= 2700 { lines.push("RAM disk: disabled"); }
    if p >= 3000 { lines.push("Booting MINIX..."); }
    if p >= 3250 { lines.push("286/386 protected mode enabled"); }
    if p >= 3450 { lines.push("Kernel size: [white]109 KB[/]"); }
    if p >= 3700 { lines.push("Initializing kernel subsystems"); }
    if p >= 3750 { lines.push("... interrupt vectors: OK"); }
    if p >= 3800 { lines.push("... trap handlers: OK"); }
    if p >= 3850 { lines.push("... memory manager: OK"); }
    if p >= 3850 { lines.push(""); }
    if p >= 4000 { lines.push("[white]... tty drivers: OK[/]"); }
    if p >= 4050 { lines.push("[white]... tty0 initialized[/]"); }
    if p >= 4150 { lines.push("[white]... tty1 initialized[/]"); }
    if p >= 4150 { lines.push(""); }
    if p >= 4250 { lines.push("... block devices: OK"); }
    if p >= 5200 { lines.push("Mounting root file system... OK"); }
    if p >= 5600 { lines.push("[yellow]Warning: CMOS time appears invalid[/]"); }
    if p >= 6200 { lines.push("Starting init process"); }
    if p >= 6400 { lines.push("... /etc/rc"); }
    if p >= 6600 { lines.push("... update daemon"); }
    if p >= 6900 { lines.push("... clock task sync"); }
    if p >= 6900 { lines.push(""); }
    if p >= 7200 { lines.push("... starting login processes"); }

    let max_lines = 14;
    let total = 0;
    for _line in lines {
        total += 1;
    }
    let start = if total > max_lines { total - max_lines } else { 0 };
    text = "";
    let idx = 0;
    for line in lines {
        if idx >= start {
            text += `${line}
`;
        }
        idx += 1;
    }
} else if t >= 16200 && t < 16700 {
    // Short black gap before intro-login scene.
    text = " ";
}

scene.set("bios-block", "text.content", text);

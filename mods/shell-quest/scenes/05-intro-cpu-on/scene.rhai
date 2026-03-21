let t = time.scene_elapsed_ms;
let text = " ";

if t >= 300 && t < 5800 {
    text = `TUX Modular BIOS v1.90, 1990 edition
Copyright (C) 1990, TUX Software, Inc.
* TUX AWARDS BIOS Extension v1.0A

`;

    if t >= 800 {
        text += `[white]CPU: 386QX compatible @ 20MHz[/]
`;
    }

    if t >= 1400 && t < 3200 {
        let step = ((t - 1400) * 16) / 1800;
        let s = if step > 16 { 16 } else { step };
        text += `Checking RAM: ${s * 128}KB
`;
    } else if t >= 3200 {
        text += `[white]Checking RAM: 2048KB OK[/]
`;
    }

    if t >= 3800 {
        text += `CD-ROM drive A:        .......... not found
`;
    }
    if t >= 4200 {
        text += `Floppy drive A: 1.44MB .......... not found
`;
    }
    if t >= 4600 {
        text += `Hard drive 0: SETAGE ST225 20MB
`;
    }
} else if t >= 5800 && t < 6000 {
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
}

scene.set("bios-block", "text.content", text);

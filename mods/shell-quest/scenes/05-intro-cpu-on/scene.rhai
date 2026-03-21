let t = time.scene_elapsed_ms;
let text = " ";

if t >= 3000 && t < 7400 {
    text = `TUX Modular BIOS v1.90, 1990 edition
Copyright (C) 1990, TUX Software, Inc.
* TUX AWARDS BIOS Extension v1.0A

`;

    if t >= 3700 {
        text += `CPU: 386QX compatible @ 20MHz
`;
    }

    if t >= 4200 && t < 5500 {
        let step = ((t - 4200) * 16) / 1300;
        let s = if step > 16 { 16 } else { step };
        text += `Checking RAM: ${s * 128}KB
`;
    } else if t >= 5500 {
        text += `[white]Checking RAM: 2048KB OK[/]
`;
    }

    if t >= 6000 {
        text += `CD-ROM drive A:        .......... not found
`;
    }
    if t >= 6400 {
        text += `Floppy drive A: 1.44MB .......... not found
`;
    }
    if t >= 6800 {
        text += `Hard drive 0: SETAGE ST225 20MB
`;
    }
} else if t >= 7400 && t < 10000 {
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

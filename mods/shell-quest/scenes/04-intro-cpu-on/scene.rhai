fn pad4(v) {
    if v < 10 { return `000${v}`; }
    if v < 100 { return `00${v}`; }
    if v < 1000 { return `0${v}`; }
    `${v}`
}

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

if t >= 1800 && t < 7600 {
    clear_bios_lines(commands);
}

if t >= 2200 && t < 3200 {
    commands.push(#{ op: "set-text", target: "bios-line-1", text: "Award Modular BIOS v4.50PG, An Energy Star Ally" });
    commands.push(#{ op: "set-text", target: "bios-line-2", text: "Copyright (C) 1984-95, Tux Software, Inc." });
    commands.push(#{ op: "set-text", target: "bios-line-4", text: "486API0, Rev 1.0" });
    commands.push(#{ op: "set-text", target: "bios-line-5", text: "80486DX2 CPU at 66MHz" });
} else if t >= 3200 && t < 5000 {
    let start = 3200;
    let end = 5000;
    let total_k = 16384;
    let elapsed = t - start;
    let duration = end - start;
    let raw_k = if duration > 0 {
        (elapsed * total_k) / duration
    } else {
        total_k
    };
    let quantized_k = (raw_k / 64) * 64;
    let value_k = if quantized_k > total_k { total_k } else { quantized_k };

    commands.push(#{ op: "set-text", target: "bios-line-1", text: "TUX Modular BIOS v4.50PG, An Energy Star Ally" });
    commands.push(#{ op: "set-text", target: "bios-line-2", text: "Copyright (C) 1984-95, Tux Inc." });
    commands.push(#{ op: "set-text", target: "bios-line-4", text: "486API0, Rev 1.0" });
    commands.push(#{ op: "set-text", target: "bios-line-5", text: "80486DX2 CPU at 66MHz" });
    commands.push(#{ op: "set-text", target: "bios-line-6", text: `Memory Test : ${pad4(value_k)}K` });
} else if t >= 5000 && t < 6200 {
    commands.push(#{ op: "set-text", target: "bios-line-1", text: "TUX Modular BIOS v4.50PG, An Energy Star Ally" });
    commands.push(#{ op: "set-text", target: "bios-line-2", text: "Copyright (C) 1984-95, Tux Inc." });
    commands.push(#{ op: "set-text", target: "bios-line-4", text: "486API0, Rev 1.0" });
    commands.push(#{ op: "set-text", target: "bios-line-5", text: "80486DX2 CPU at 66MHz" });
    commands.push(#{ op: "set-text", target: "bios-line-6", text: "Memory Test : 16384K OK" });
    commands.push(#{ op: "set-text", target: "bios-line-8", text: "Award Plug and Play BIOS Extension v1.0A" });
    commands.push(#{ op: "set-text", target: "bios-line-9", text: "Copyright (C) 1995, Award Software, Inc." });
    commands.push(#{ op: "set-text", target: "bios-line-10", text: "Detecting HDD Primary Master   ... None" });
} else if t >= 6200 && t < 7600 {
    commands.push(#{ op: "set-text", target: "bios-line-1", text: "TUX Modular BIOS v4.50PG, An Energy Star Ally" });
    commands.push(#{ op: "set-text", target: "bios-line-2", text: "Copyright (C) 1984-95, Tux Inc." });
    commands.push(#{ op: "set-text", target: "bios-line-4", text: "486API0, Rev 1.0" });
    commands.push(#{ op: "set-text", target: "bios-line-5", text: "80486DX2 CPU at 66MHz" });
    commands.push(#{ op: "set-text", target: "bios-line-6", text: "Memory Test : 16384K OK" });
    commands.push(#{ op: "set-text", target: "bios-line-8", text: "Award Plug and Play BIOS Extension v1.0A" });
    commands.push(#{ op: "set-text", target: "bios-line-9", text: "Copyright (C) 1995, Award Software, Inc." });
    commands.push(#{ op: "set-text", target: "bios-line-10", text: "Detecting HDD Primary Master   ... None" });
    commands.push(#{ op: "set-text", target: "bios-line-11", text: "Keyboard error or no keyboard present" });
    commands.push(#{ op: "set-text", target: "bios-footer-1", text: "Press F1 to continue, DEL to enter SETUP" });
    commands.push(#{ op: "set-text", target: "bios-footer-2", text: "08/16/95-TUX-486DX2-66-00" });
}

commands

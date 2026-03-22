using CognitosOs.Core;

namespace CognitosOs.Boot;

internal sealed class MinixBootSequence : IBootSequence
{
    // Kernel footprint in KB — fixed for Minix 1.1
    private const int MinixKb = 109;

    public IReadOnlyList<BootStep> BuildBootSteps(IOperatingSystem os)
    {
        var spec = os.Spec;

        // Speed factor: slower CPU = longer delays. Baseline is 33 MHz.
        var f = 33.0 / Math.Max(spec.CpuMhz, 1);

        var ramKb = spec.RamKb;
        var availKb = ramKb - MinixKb;

        var memoryLine =
            $"Memory size = {Style.Fg(Style.Bright, $"{ramKb}K")}" +
            $"     {Style.Fg(Style.BootKeyword, "MINIX")} = {MinixKb}K" +
            $"     RAM disk = 0K" +
            $"     Available = {Style.Fg(Style.Bright, $"{availKb}K")}";

        var steps = new List<BootStep>();

        // ── Phase 1: Boot monitor ──────────────────────────────────────────────
        // On hard disk boot the monitor prints a single line then boots immediately.
        steps.Add(S($"={Style.Fg(Style.Bright, "MINIX")} boot", 120, f));
        steps.Add(S(string.Empty, 80, f));

        // ── Phase 2: Kernel banner + memory line ──────────────────────────────
        // These two lines are the very first things the Minix kernel prints.
        steps.Add(S($"{Style.Fg(Style.Bright, "MINIX 1.1")}  Copyright 1987, Prentice-Hall, Inc.", 300, f));
        steps.Add(S(memoryLine, 400, f));
        steps.Add(S(string.Empty, 200, f));

        // ── Phase 3: Kernel task startup ──────────────────────────────────────
        // Minix starts numbered tasks in order. On a PC-AT the order is:
        //   CLOCK, MEM, FLOPPY, WINCHESTER, TTY, ETHERNET, PRINTER
        // We skip floppy (no removable media) and printer (not present).
        steps.Add(S($"{Style.Fg(Style.BootKeyword, "clock")} task", 120, f));
        steps.Add(S($"{Style.Fg(Style.BootKeyword, "memory")} task", 80, f));
        steps.Add(S($"{Style.Fg(Style.BootKeyword, "winchester")} task", 600, f));   // HDD seek takes time

        // TTY task — initialises consoles
        steps.Add(S($"{Style.Fg(Style.BootKeyword, "tty")} task", 80, f));

        // Ethernet only if NIC speed implies a real card is present
        if (spec.NicSpeedKbps > 0)
            steps.Add(S($"{Style.Fg(Style.BootKeyword, "ethernet")} task", 160, f));

        steps.Add(S(string.Empty, 100, f));

        // ── Phase 4: File system mount ────────────────────────────────────────
        // Minix 1.1 on hard disk uses hd1 (root) and hd2 (/usr).
        // The FS prints one line per mounted device.
        steps.Add(S($"root file system on /dev/hd1  {Style.Fg(Style.BootOk, "OK")}", 350, f));
        steps.Add(S($"/usr file system on /dev/hd2  {Style.Fg(Style.BootOk, "OK")}", 500, f));
        steps.Add(S(string.Empty, 120, f));

        // ── Phase 5: init + /etc/rc ───────────────────────────────────────────
        // init reads /etc/rc and starts background daemons.
        steps.Add(S($"{Style.Fg(Style.BootKeyword, "Init")}: Starting system.", 200, f));
        steps.Add(S(string.Empty, 80, f));
        steps.Add(S("/etc/rc", 180, f));
        steps.Add(S(string.Empty, 280, f));

        // update daemon (syncs file system every 30s — always present in Minix)
        steps.Add(S("update", 150, f));

        // cron only on better-specced machines (enough RAM)
        if (spec.RamKb >= 2048)
            steps.Add(S("cron", 130, f));

        steps.Add(S(string.Empty, 200, f));

        // ── Phase 6: getty ────────────────────────────────────────────────────
        // getty on the console — next thing printed would be the login prompt,
        // but AppHost clears the screen and shows it separately.
        steps.Add(S("/etc/getty tty0 &", 180, f));
        steps.Add(S(string.Empty, 100, f));

        return steps;
    }

    private static BootStep S(string text, double baseMs, double factor)
        => new(text, (ulong)Math.Max(30, baseMs * factor));
}

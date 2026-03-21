using CognitosOs.Core;

namespace CognitosOs.Boot;

internal sealed class MinixBootSequence : IBootSequence
{
    public IReadOnlyList<BootStep> BuildBootSteps(IOperatingSystem os)
    {
        var spec = os.Spec;
        var cpuLine = $"{spec.CpuModel} @ {spec.CpuMhz} MHz";
        var ramLine = $"{spec.RamKb} KB OK";
        var diskLine = $"hd0: {spec.DiskKb} KB";

        return new[]
        {
            new BootStep("Starting from hard disk", 800),
            new BootStep("Reading boot sector...", 350),
            new BootStep("Boot sector loaded", 200),
            new BootStep("MINIX boot block", 150),
            new BootStep("Loading secondary boot...", 550),
            new BootStep("Secondary boot loaded", 250),
            new BootStep("MINIX boot monitor 1.3", 150),
            new BootStep($"CPU: [white]{cpuLine}[/]", 180),
            new BootStep($"Memory: [white]{ramLine}[/]", 200),
            new BootStep($"Disk: [white]{diskLine}[/]", 150),
            new BootStep($"NIC: [white]{spec.NicModel}[/]", 120),
            new BootStep("Root device: hd1a", 120),
            new BootStep(spec.RamKb >= 4096 ? "RAM disk: disabled" : "[yellow]RAM disk: limited[/]", 120),
            new BootStep("Booting MINIX...", 450),
            new BootStep(spec.CpuMhz >= 33 ? "286/386 protected mode enabled" : "real mode only", 200),
            new BootStep($"Kernel size: [white]109 KB[/]", 180),
            new BootStep("Initializing kernel subsystems", 100),
            new BootStep("... interrupt vectors: OK", 80),
            new BootStep("... trap handlers: OK", 80),
            new BootStep("... memory manager: OK", 250),
            new BootStep(string.Empty, 100),
            new BootStep("[white]... tty drivers: OK[/]", 120),
            new BootStep("[white]... tty0 initialized[/]", 180),
            new BootStep("[white]... tty1 initialized[/]", 180),
            new BootStep(string.Empty, 150),
            new BootStep("... block devices: OK", 1200),
            new BootStep("Mounting root file system... OK", 650),
            new BootStep("[yellow]Warning: CMOS time appears invalid[/]", 800),
            new BootStep("Starting init process", 350),
            new BootStep("... /etc/rc", 280),
            new BootStep("... update daemon", 420),
            new BootStep("... clock task sync", 380),
            new BootStep(string.Empty, 350),
            new BootStep("... starting login processes", 450),
            new BootStep(string.Empty, 200),
        };
    }
}

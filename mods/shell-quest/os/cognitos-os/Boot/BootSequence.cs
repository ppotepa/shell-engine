using CognitosOs.Core;

namespace CognitosOs.Boot;

internal sealed class MinixBootSequence : IBootSequence
{
    public IReadOnlyList<BootStep> BuildBootSteps(IOperatingSystem os)
    {
        return new[]
        {
            new BootStep("Starting from hard disk", 1000),
            new BootStep("Reading boot sector...", 200),
            new BootStep("Boot sector loaded", 250),
            new BootStep("MINIX boot block", 250),
            new BootStep("Loading secondary boot...", 300),
            new BootStep("Secondary boot loaded", 250),
            new BootStep("MINIX boot monitor 1.3", 250),
            new BootStep("Root device: hd1a", 200),
            new BootStep("RAM disk: disabled", 300),
            new BootStep("Booting MINIX...", 250),
            new BootStep("286/386 protected mode enabled", 200),
            new BootStep("Kernel size: [white]109 KB[/]", 250),
            new BootStep("Initializing kernel subsystems", 50),
            new BootStep("... interrupt vectors: OK", 50),
            new BootStep("... trap handlers: OK", 50),
            new BootStep("... memory manager: OK", 150),
            new BootStep(string.Empty, 100),
            new BootStep("[white]... tty drivers: OK[/]", 50),
            new BootStep("[white]... tty0 initialized[/]", 100),
            new BootStep("[white]... tty1 initialized[/]", 150),
            new BootStep(string.Empty, 100),
            new BootStep("... block devices: OK", 950),
            new BootStep("Mounting root file system... OK", 400),
            new BootStep("[yellow]Warning: CMOS time appears invalid[/]", 600),
            new BootStep("Starting init process", 200),
            new BootStep("... /etc/rc", 200),
            new BootStep("... update daemon", 300),
            new BootStep("... clock task sync", 300),
            new BootStep(string.Empty, 300),
            new BootStep("... starting login processes", 300),
            new BootStep(string.Empty, 200),
        };
    }
}

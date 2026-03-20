using CognitosOs.Core;

namespace CognitosOs.Boot;

internal sealed class MinixBootSequence : IBootSequence
{
    public IReadOnlyList<BootStep> BuildBootSteps(IOperatingSystem os)
    {
        return new[]
        {
            new BootStep("booting minix 1.3...", 220),
            new BootStep("mounting root filesystem... ok", 180),
            new BootStep("starting tty0... ok", 140),
            new BootStep("starting netd... ok", 180),
            new BootStep("starting maild... ok", 180),
            new BootStep(string.Empty, 80),
            new BootStep(Style.Fg(Style.Info, "system ready."), 120),
            new BootStep(string.Empty, 80),
        };
    }
}

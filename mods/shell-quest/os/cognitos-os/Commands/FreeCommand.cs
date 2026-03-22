using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class FreeCommand : ICommand
{
    public string Name => "free";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var spec = ctx.Os.Spec;
        var (_, memPct) = ctx.Os.UsageSnapshot();
        var used = (int)(spec.RamKb * memPct / 100.0);
        var free = spec.RamKb - used;
        var swap = spec.RamKb < 2048 ? spec.RamKb / 2 : 0; // swap only on low RAM

        return new CommandResult(new[]
        {
            "             total       used       free",
            $"Mem:     {spec.RamKb,9}  {used,9}  {free,9}",
            $"Swap:    {swap,9}  {0,9}  {swap,9}",
        });
    }
}

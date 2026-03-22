using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class MountCommand : ICommand
{
    public string Name => "mount";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var spec = ctx.Os.Spec;
        return new CommandResult(new[]
        {
            $"/dev/hd1 on / type minix (rw) [{spec.DiskKb}K]",
            $"/dev/hd2 on /usr type minix (rw) [{spec.DiskKb / 2}K]",
        });
    }
}

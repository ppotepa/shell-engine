using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class DfCommand : ICommand
{
    public string Name => "df";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var spec = ctx.Os.Spec;
        var usedRoot = spec.DiskKb - spec.DiskFreeKb;
        var pctRoot = (int)((double)usedRoot / spec.DiskKb * 100);
        var usrTotal = spec.DiskKb / 2;
        var usrUsed = (int)(usrTotal * 0.89);
        var usrFree = usrTotal - usrUsed;
        var pctUsr = (int)((double)usrUsed / usrTotal * 100);

        return new CommandResult(new[]
        {
            "Filesystem   1K-blocks   Used   Avail   Use%   Mounted on",
            $"/dev/hd1     {spec.DiskKb,9}  {usedRoot,5}   {spec.DiskFreeKb,5}   {pctRoot,3}%   /",
            $"/dev/hd2     {usrTotal,9}  {usrUsed,5}   {usrFree,5}   {pctUsr,3}%   /usr",
        });
    }
}

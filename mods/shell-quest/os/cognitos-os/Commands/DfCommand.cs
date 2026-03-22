using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class DfCommand : IKernelCommand
{
    public string Name => "df";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var res = uow.Resources;
        var usedRoot = res.DiskUsedKb;
        var pctRoot = res.DiskTotalKb > 0 ? (int)((double)usedRoot / res.DiskTotalKb * 100) : 0;
        var usrTotal = res.DiskTotalKb / 2;
        var usrUsed = (int)(usrTotal * 0.89);
        var usrFree = usrTotal - usrUsed;
        var pctUsr = usrTotal > 0 ? (int)((double)usrUsed / usrTotal * 100) : 0;

        uow.Out.WriteLine("Filesystem   1K-blocks   Used   Avail   Use%   Mounted on");
        uow.Out.WriteLine($"/dev/hd1     {res.DiskTotalKb,9}  {usedRoot,5}   {res.DiskFreeKb,5}   {pctRoot,3}%   /");
        uow.Out.WriteLine($"/dev/hd2     {usrTotal,9}  {usrUsed,5}   {usrFree,5}   {pctUsr,3}%   /usr");
        return 0;
    }
}

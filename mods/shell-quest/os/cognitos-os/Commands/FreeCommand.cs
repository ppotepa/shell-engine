using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("free", OsTag = "minix")]
internal sealed class FreeCommand : IKernelCommand
{
    public string Name => "free";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var res = uow.Resources;
        var swap = res.TotalRamKb < 2048 ? res.TotalRamKb / 2 : 0;

        uow.Out.WriteLine("             total       used       free");
        uow.Out.WriteLine($"Mem:     {res.TotalRamKb,9}  {res.TotalRamKb - res.FreeRamKb,9}  {res.FreeRamKb,9}");
        uow.Out.WriteLine($"Swap:    {swap,9}  {0,9}  {swap,9}");
        return 0;
    }
}

using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("netstat", OsTag = "minix")]
internal sealed class NetstatCommand : IKernelCommand
{
    public string Name => "netstat";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        uow.Out.WriteLine("Proto  Local Address       Foreign Address     State");
        uow.Out.WriteLine("tcp    0.0.0.0:21          *:*                 LISTEN");
        uow.Out.WriteLine("tcp    0.0.0.0:23          *:*                 LISTEN");
        uow.Out.WriteLine("tcp    0.0.0.0:80          *:*                 LISTEN");

        if (uow.Quest.FtpConnected)
            uow.Out.WriteLine("tcp    130.234.48.5:1024   128.214.6.100:21    ESTABLISHED");

        var anomalyCount = uow.Quest.AnomaliesDiscovered?.Count ?? 0;
        if (anomalyCount >= 3)
            uow.Out.WriteLine("???    0.0.0.0:??          *:*                 UNKNOWN");

        uow.Out.WriteLine("");
        uow.Out.WriteLine($"interface: {uow.Spec.NicModel} at 0x300, UP");
        return 0;
    }
}

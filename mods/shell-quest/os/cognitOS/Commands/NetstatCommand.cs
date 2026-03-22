using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("netstat", OsTag = "minix")]
internal sealed class NetstatCommand : IKernelCommand
{
    public string Name => "netstat";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        uow.Out.WriteLine("Proto  Local Address       Foreign Address     State");

        // Listening services — derived from /etc/services in VFS
        uow.Out.WriteLine("tcp    0.0.0.0:21          *:*                 LISTEN");
        uow.Out.WriteLine("tcp    0.0.0.0:23          *:*                 LISTEN");

        // Active FTP connection — real quest state
        if (uow.Quest.FtpConnected)
            uow.Out.WriteLine($"tcp    130.234.48.5:1024   {uow.Quest.FtpRemoteHost ?? "?"}:21  ESTABLISHED");

        // Anomaly port — only if anomalies discovered
        var anomalyCount = uow.Quest.AnomaliesDiscovered?.Count ?? 0;
        if (anomalyCount >= 3)
            uow.Out.WriteLine("???    0.0.0.0:??          *:*                 UNKNOWN");

        uow.Out.WriteLine("");
        uow.Out.WriteLine($"interface: sl0  {uow.Spec.ModemModel}  {uow.Spec.ModemBaud} baud  UP");
        return 0;
    }
}

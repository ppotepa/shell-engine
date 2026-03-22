using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class NetstatCommand : ICommand
{
    public string Name => "netstat";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var spec = ctx.Os.Spec;
        var lines = new List<string>
        {
            "Proto  Local Address       Foreign Address     State",
            "tcp    0.0.0.0:21          *:*                 LISTEN",
            "tcp    0.0.0.0:23          *:*                 LISTEN",
            "tcp    0.0.0.0:80          *:*                 LISTEN",
        };

        // After FTP connection, show it
        if (ctx.Os.State.Quest.FtpConnected)
            lines.Add("tcp    130.234.48.5:1024   128.214.6.100:21    ESTABLISHED");

        // After all 3 anomalies: mysterious entry
        var anomalyCount = ctx.Os.State.Quest.AnomaliesDiscovered?.Count ?? 0;
        if (anomalyCount >= 3)
            lines.Add("???    0.0.0.0:??          *:*                 UNKNOWN");

        lines.Add("");
        lines.Add($"interface: {spec.NicModel} at 0x300, UP");

        return new CommandResult(lines);
    }
}

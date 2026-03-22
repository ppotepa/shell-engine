using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class IfconfigCommand : ICommand
{
    public string Name => "ifconfig";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var spec = ctx.Os.Spec;
        return new CommandResult(new[]
        {
            $"eth0: flags=UP,BROADCAST,RUNNING",
            $"        inet 130.234.48.5  netmask 255.255.255.0  broadcast 130.234.48.255",
            $"        ether 00:00:c0:a8:48:05  ({spec.NicModel})",
            $"        speed {spec.NicSpeedKbps} Kbps",
            "",
            "lo:   flags=UP,LOOPBACK,RUNNING",
            "        inet 127.0.0.1  netmask 255.0.0.0",
        });
    }
}

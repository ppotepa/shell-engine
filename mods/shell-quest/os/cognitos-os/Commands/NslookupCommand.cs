using CognitosOs.Core;
using CognitosOs.Network;

namespace CognitosOs.Commands;

internal sealed class NslookupCommand : ICommand
{
    private readonly NetworkRegistry _network;

    public NslookupCommand(NetworkRegistry network) => _network = network;

    public string Name => "nslookup";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 1)
            return new CommandResult(new[] { "usage: nslookup <host>" }, 1);

        var host = ctx.Argv[0];
        var server = _network.Resolve(host);

        if (server is null)
            return new CommandResult(new[]
            {
                "Server:  128.214.1.1",
                "Address: 128.214.1.1#53",
                "",
                $"** server can't find {host}: NXDOMAIN",
            }, 1);

        if (server is AnomalyServer)
            return new CommandResult(new[]
            {
                "Server:  128.214.1.1",
                "Address: 128.214.1.1#53",
                "",
                $"** server can't find {host}: SERVFAIL",
                $"** (partial response received from unallocated AS)",
            }, 1);

        return new CommandResult(new[]
        {
            "Server:  128.214.1.1",
            "Address: 128.214.1.1#53",
            "",
            $"Name:    {server.Hostname}",
            $"Address: {server.IpAddress}",
        });
    }
}

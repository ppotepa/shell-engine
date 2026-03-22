using CognitosOs.Core;
using CognitosOs.Kernel;
using CognitosOs.Network;

namespace CognitosOs.Commands;

internal sealed class NslookupCommand : IKernelCommand
{
    private readonly NetworkRegistry _network;

    public NslookupCommand(NetworkRegistry network) => _network = network;

    public string Name => "nslookup";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("usage: nslookup <host>");
            return 1;
        }

        var host = argv[1];
        var server = _network.Resolve(host);

        if (server is null)
        {
            uow.Out.WriteLine("Server:  128.214.1.1");
            uow.Out.WriteLine("Address: 128.214.1.1#53");
            uow.Out.WriteLine("");
            uow.Out.WriteLine($"** server can't find {host}: NXDOMAIN");
            return 1;
        }

        if (server is AnomalyServer)
        {
            uow.Out.WriteLine("Server:  128.214.1.1");
            uow.Out.WriteLine("Address: 128.214.1.1#53");
            uow.Out.WriteLine("");
            uow.Out.WriteLine($"** server can't find {host}: SERVFAIL");
            uow.Out.WriteLine("** (partial response received from unallocated AS)");
            return 1;
        }

        uow.Out.WriteLine("Server:  128.214.1.1");
        uow.Out.WriteLine("Address: 128.214.1.1#53");
        uow.Out.WriteLine("");
        uow.Out.WriteLine($"Name:    {server.Hostname}");
        uow.Out.WriteLine($"Address: {server.IpAddress}");
        return 0;
    }
}

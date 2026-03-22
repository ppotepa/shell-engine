using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class IfconfigCommand : IKernelCommand
{
    public string Name => "ifconfig";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var spec = uow.Spec;
        uow.Out.WriteLine("eth0: flags=UP,BROADCAST,RUNNING");
        uow.Out.WriteLine($"        inet 130.234.48.5  netmask 255.255.255.0  broadcast 130.234.48.255");
        uow.Out.WriteLine($"        ether 00:00:c0:a8:48:05  ({spec.NicModel})");
        uow.Out.WriteLine($"        speed {spec.NicSpeedKbps} Kbps");
        uow.Out.WriteLine("");
        uow.Out.WriteLine("lo:   flags=UP,LOOPBACK,RUNNING");
        uow.Out.WriteLine("        inet 127.0.0.1  netmask 255.0.0.0");
        return 0;
    }
}

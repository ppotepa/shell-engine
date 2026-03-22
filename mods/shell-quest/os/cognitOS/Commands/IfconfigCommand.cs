using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("ifconfig", OsTag = "minix")]
internal sealed class IfconfigCommand : IKernelCommand
{
    public string Name => "ifconfig";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var spec = uow.Spec;
        uow.Out.WriteLine("sl0:  flags=UP,POINTOPOINT,RUNNING");
        uow.Out.WriteLine($"        inet 130.234.48.5  --> 128.214.6.1");
        uow.Out.WriteLine($"        {spec.ModemModel}  {spec.ModemBaud} baud");
        uow.Out.WriteLine("");
        uow.Out.WriteLine("lo:   flags=UP,LOOPBACK,RUNNING");
        uow.Out.WriteLine("        inet 127.0.0.1  netmask 255.0.0.0");
        return 0;
    }
}

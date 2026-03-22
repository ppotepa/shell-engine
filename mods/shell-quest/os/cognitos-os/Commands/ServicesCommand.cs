using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("services", OsTag = "minix")]
internal sealed class ServicesCommand : IKernelCommand
{
    public string Name => "services";
    public IReadOnlyList<string> Aliases => new[] { "service" };

    public int Run(IUnitOfWork uow, string[] argv)
    {
        // Show known daemon processes as services
        var procs = uow.Process.List()
            .Where(p => p.User == "root" && p.Tty == "?")
            .OrderBy(p => p.Name);

        uow.Out.WriteLine("name     status  pid");
        foreach (var p in procs)
            uow.Out.WriteLine($"{p.Name,-8} active  {p.Pid}");

        return 0;
    }
}

using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("env", OsTag = "minix")]
internal sealed class EnvCommand : IKernelCommand
{
    public string Name => "env";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        uow.Out.WriteLine($"USER={uow.Session.User}");
        uow.Out.WriteLine($"HOME={uow.Session.Home}");
        uow.Out.WriteLine("SHELL=/bin/sh");
        uow.Out.WriteLine("PATH=/bin:/usr/bin");
        uow.Out.WriteLine("TERM=minix");
        uow.Out.WriteLine($"HOSTNAME={uow.Session.Hostname}");
        uow.Out.WriteLine($"PWD={uow.Session.Cwd}");
        uow.Out.WriteLine($"LOGNAME={uow.Session.User}");
        return 0;
    }
}

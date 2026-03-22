using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("kill", OsTag = "minix")]
internal sealed class KillCommand : IKernelCommand
{
    public string Name => "kill";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("usage: kill <pid>");
            return 1;
        }

        if (!int.TryParse(argv[1], out var pid))
        {
            uow.Err.WriteLine($"kill: {argv[1]}: arguments must be process IDs");
            return 1;
        }

        var process = uow.Process.Get(pid);
        if (process is null)
        {
            uow.Err.WriteLine($"kill: ({pid}) - No such process");
            return 1;
        }

        if (process.User != uow.Session.User)
        {
            uow.Err.WriteLine($"kill: ({pid}) - Not owner");
            return 1;
        }

        uow.Err.WriteLine($"kill: ({pid}) - Operation not permitted");
        return 1;
    }
}

using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("rm", OsTag = "minix")]
internal sealed class RmCommand : IKernelCommand
{
    public string Name => "rm";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("usage: rm file ...");
            return 1;
        }

        var code = 0;
        foreach (var arg in argv.Skip(1))
        {
            var path = uow.Session.ResolvePath(arg);
            if (!uow.Disk.Exists(path))
            {
                uow.Err.WriteLine($"rm: {arg}: No such file or directory");
                code = 1;
                continue;
            }
            uow.Disk.Unlink(path);
        }
        return code;
    }
}

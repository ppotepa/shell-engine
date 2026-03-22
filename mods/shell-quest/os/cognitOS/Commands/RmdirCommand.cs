using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("rmdir", OsTag = "minix")]
internal sealed class RmdirCommand : IKernelCommand
{
    public string Name => "rmdir";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("usage: rmdir dir ...");
            return 1;
        }

        var code = 0;
        foreach (var arg in argv.Skip(1))
        {
            var path = uow.Session.ResolvePath(arg);
            if (!uow.Disk.Exists(path))
            {
                uow.Err.WriteLine($"rmdir: {arg}: No such file or directory");
                code = 1;
                continue;
            }
            var entries = uow.Disk.RawReadDir(path);
            if (entries is { Count: > 0 })
            {
                uow.Err.WriteLine($"rmdir: {arg}: Directory not empty");
                code = 1;
                continue;
            }
            uow.Disk.Unlink(path);
        }
        return code;
    }
}

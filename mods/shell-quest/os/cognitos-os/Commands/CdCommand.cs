using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("cd", OsTag = "minix")]
internal sealed class CdCommand : IKernelCommand
{
    public string Name => "cd";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var target = argv.Length > 1 ? argv[1] : "~";
        var resolved = uow.Session.ResolvePath(target);

        try
        {
            uow.Disk.ReadDir(resolved);
        }
        catch (DirectoryNotFoundException)
        {
            uow.Out.WriteLine($"cd: {target}: No such file or directory");
            return 1;
        }

        uow.Session.SetCwd(resolved);
        return 0;
    }
}

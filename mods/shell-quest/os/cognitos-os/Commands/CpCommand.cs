using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("cp", OsTag = "minix")]
internal sealed class CpCommand : IKernelCommand
{
    public string Name => "cp";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 3)
        {
            uow.Err.WriteLine("usage: cp <source> <dest>");
            return 1;
        }

        var srcPath = uow.Session.ResolvePath(argv[1]);
        var dstPath = uow.Session.ResolvePath(argv[2]);

        try
        {
            var content = uow.Disk.ReadFile(srcPath);
            uow.Disk.WriteFile(dstPath, content);
            uow.Quest.BackupMade = true;
            return 0;
        }
        catch (FileNotFoundException)
        {
            uow.Err.WriteLine($"cp: {argv[1]}: No such file or directory");
            return 1;
        }
        catch (IOException ex)
        {
            uow.Err.WriteLine($"cp: {argv[2]}: {ex.Message}");
            return 1;
        }
    }
}

using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("mv", OsTag = "minix")]
internal sealed class MvCommand : IKernelCommand
{
    public string Name => "mv";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 3)
        {
            uow.Err.WriteLine("usage: mv source dest");
            return 1;
        }

        var src = uow.Session.ResolvePath(argv[1]);
        var dst = uow.Session.ResolvePath(argv[2]);

        try
        {
            var content = uow.Disk.ReadFile(src);
            uow.Disk.WriteFile(dst, content);
            uow.Disk.Unlink(src);
            return 0;
        }
        catch (FileNotFoundException)
        {
            uow.Err.WriteLine($"mv: {argv[1]}: No such file or directory");
            return 1;
        }
        catch (IOException ex)
        {
            uow.Err.WriteLine($"mv: {ex.Message}");
            return 1;
        }
    }
}

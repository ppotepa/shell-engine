using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("mkdir", OsTag = "minix")]
internal sealed class MkdirCommand : IKernelCommand
{
    public string Name => "mkdir";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("usage: mkdir dir ...");
            return 1;
        }

        var code = 0;
        foreach (var arg in argv.Skip(1))
        {
            var path = uow.Session.ResolvePath(arg);
            try
            {
                uow.Disk.Mkdir(path);
            }
            catch (IOException ex)
            {
                uow.Err.WriteLine($"mkdir: {arg}: {ex.Message}");
                code = 1;
            }
        }
        return code;
    }
}

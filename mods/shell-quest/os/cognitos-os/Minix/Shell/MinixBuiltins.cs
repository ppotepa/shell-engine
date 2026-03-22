namespace CognitosOs.Minix.Shell;

using CognitosOs.Core;
using CognitosOs.Framework.Execution;
using CognitosOs.Kernel;

internal sealed class MinixBuiltins : IShellBuiltins
{
    public bool TryHandle(IUnitOfWork uow, string[] argv, out ApplicationResult result)
    {
        result = ApplicationResult.Continue;
        if (argv.Length == 0)
            return false;

        switch (argv[0])
        {
            case "cd":
                result = HandleCd(uow, argv);
                return true;
            case "pwd":
                uow.Out.WriteLine(uow.Session.Cwd);
                uow.Session.LastExitCode = 0;
                return true;
            case "clear":
                uow.Session.LastExitCode = 901;
                return true;
            case "exit":
                uow.Out.WriteLine("logout");
                uow.Session.LastExitCode = 0;
                result = ApplicationResult.Exit;
                return true;
            default:
                return false;
        }
    }

    private static ApplicationResult HandleCd(IUnitOfWork uow, string[] argv)
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
            uow.Session.LastExitCode = 1;
            return ApplicationResult.Continue;
        }

        uow.Session.SetCwd(resolved);
        uow.Session.LastExitCode = 0;
        return ApplicationResult.Continue;
    }
}

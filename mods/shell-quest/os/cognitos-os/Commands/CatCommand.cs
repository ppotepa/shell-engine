using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("cat", OsTag = "universal")]
internal sealed class CatCommand : IKernelCommand
{
    public string Name => "cat";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Out.WriteLine(Style.Fg(Style.Warn, "usage: cat <file>"));
            return 2;
        }

        var path = uow.Session.ResolvePath(argv[1]);

        string content;
        try
        {
            content = uow.Disk.ReadFile(path);
        }
        catch (FileNotFoundException)
        {
            if (uow.Disk.Exists(path))
            {
                uow.Out.WriteLine(Style.Fg(Style.Error, $"cat: {argv[1]}: is a directory"));
                return 1;
            }
            uow.Out.WriteLine(Style.Fg(Style.Error, $"cat: {argv[1]}: no such file or directory"));
            return 1;
        }

        // TODO: mail read marking needs refactoring for index-based IMailSpool
        foreach (var line in content.Replace("\r\n", "\n").Split('\n'))
            uow.Out.WriteLine(line);
        return 0;
    }
}

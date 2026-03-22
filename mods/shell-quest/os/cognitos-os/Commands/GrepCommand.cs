using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("grep", OsTag = "universal")]
internal sealed class GrepCommand : IKernelCommand
{
    public string Name => "grep";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 3)
        {
            uow.Err.WriteLine("usage: grep <pattern> <file>");
            return 1;
        }

        var pattern = argv[1];
        var path = uow.Session.ResolvePath(argv[2]);

        try
        {
            var content = uow.Disk.ReadFile(path);
            var matches = content.Replace("\r\n", "\n").Split('\n')
                .Where(line => line.Contains(pattern, StringComparison.OrdinalIgnoreCase))
                .ToArray();

            if (matches.Length == 0)
                return 1;

            foreach (var m in matches)
                uow.Out.WriteLine(m);
            return 0;
        }
        catch (FileNotFoundException)
        {
            uow.Err.WriteLine($"grep: {argv[2]}: No such file or directory");
            return 2;
        }
    }
}

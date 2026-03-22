using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class HeadTailCommand : IKernelCommand
{
    private readonly bool _isHead;
    public string Name { get; }
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public HeadTailCommand(bool isHead)
    {
        _isHead = isHead;
        Name = isHead ? "head" : "tail";
    }

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine($"usage: {Name} <file>");
            return 1;
        }

        int count = 10;
        var fileArg = argv[1];

        if (argv.Length >= 3 && argv[1].StartsWith('-') && int.TryParse(argv[1][1..], out var n))
        {
            count = n;
            fileArg = argv[2];
        }

        var path = uow.Session.ResolvePath(fileArg);

        try
        {
            var content = uow.Disk.ReadFile(path);
            var allLines = content.Replace("\r\n", "\n").Split('\n');
            var result = _isHead ? allLines.Take(count) : allLines.TakeLast(count);

            foreach (var line in result)
                uow.Out.WriteLine(line);
            return 0;
        }
        catch (FileNotFoundException)
        {
            uow.Err.WriteLine($"{Name}: {fileArg}: No such file or directory");
            return 1;
        }
    }
}

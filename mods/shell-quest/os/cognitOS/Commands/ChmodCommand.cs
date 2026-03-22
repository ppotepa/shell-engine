using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("chmod", OsTag = "minix")]
internal sealed class ChmodCommand : IKernelCommand
{
    public string Name => "chmod";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 3)
        {
            uow.Err.WriteLine("usage: chmod mode file ...");
            return 1;
        }

        var modeArg = argv[1];
        bool isSymbolic = modeArg.IndexOfAny(new[] { '+', '-', '=' }) >= 0;

        string? modeStr = isSymbolic ? null : ParseOctalMode(modeArg);
        if (!isSymbolic && modeStr is null)
        {
            uow.Err.WriteLine($"chmod: invalid mode: {modeArg}");
            return 1;
        }

        int exitCode = 0;
        foreach (var file in argv.Skip(2))
        {
            var resolved = uow.Session.ResolvePath(file);
            if (!uow.Disk.Exists(resolved))
            {
                uow.Err.WriteLine($"chmod: {file}: No such file or directory");
                exitCode = 1;
                continue;
            }
            // Apply octal mode; symbolic modes accepted but not applied to inode string
            if (modeStr is not null)
                uow.Disk.Chmod(resolved, modeStr);
        }
        return exitCode;
    }

    /// <summary>
    /// Parse a 3-digit octal mode into the 9 permission chars (no leading type char).
    /// Returns null for invalid input.
    /// </summary>
    private static string? ParseOctalMode(string raw)
    {
        if (raw.Length != 3 || !raw.All(c => c >= '0' && c <= '7'))
            return null;

        int owner = raw[0] - '0';
        int grp   = raw[1] - '0';
        int other = raw[2] - '0';

        return "-"
            + ((owner & 4) != 0 ? 'r' : '-') + ((owner & 2) != 0 ? 'w' : '-') + ((owner & 1) != 0 ? 'x' : '-')
            + ((grp   & 4) != 0 ? 'r' : '-') + ((grp   & 2) != 0 ? 'w' : '-') + ((grp   & 1) != 0 ? 'x' : '-')
            + ((other & 4) != 0 ? 'r' : '-') + ((other & 2) != 0 ? 'w' : '-') + ((other & 1) != 0 ? 'x' : '-');
    }
}

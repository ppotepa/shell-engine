using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class LsCommand : IKernelCommand
{
    public string Name => "ls";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        bool showAll = false, longFmt = false, onePer = false;
        string? target = null;

        foreach (var arg in argv.Skip(1))
        {
            if (arg.StartsWith('-') && arg.Length > 1)
            {
                foreach (var c in arg[1..])
                {
                    switch (c)
                    {
                        case 'a': showAll = true; break;
                        case 'l': longFmt = true; break;
                        case '1': onePer = true; break;
                        default:
                            uow.Err.WriteLine($"ls: illegal option -- {c}");
                            uow.Err.WriteLine("Try: man ls");
                            return 1;
                    }
                }
            }
            else
            {
                target = arg;
            }
        }

        var absolute = target != null
            ? uow.Session.ResolvePath(target)
            : uow.Session.Cwd;

        IReadOnlyList<string> entries;
        try
        {
            entries = uow.Disk.ReadDir(absolute);
        }
        catch (DirectoryNotFoundException)
        {
            uow.Err.WriteLine(Style.Fg(Style.Error, $"ls: {target ?? "."}: No such file or directory"));
            return 2;
        }

        var filtered = entries.AsEnumerable();
        if (!showAll)
            filtered = filtered.Where(e => !SegmentName(e).StartsWith('.'));

        var result = filtered.ToArray();
        if (result.Length == 0)
            return 0;

        if (longFmt)
        {
            FormatLong(result, absolute, uow);
            return 0;
        }

        if (onePer)
        {
            foreach (var e in result)
                uow.Out.WriteLine(e);
            return 0;
        }

        uow.Out.WriteLine(string.Join("  ", result));
        return 0;
    }

    private static void FormatLong(string[] entries, string dirPath, IUnitOfWork uow)
    {
        foreach (var entry in entries)
        {
            var name = entry.TrimEnd('/');
            var fullPath = dirPath.Length > 0 ? $"{dirPath}/{name}" : name;
            try
            {
                var stat = uow.Disk.Stat(fullPath);
                var date = stat.Modified.ToString("MMM dd HH:mm");
                uow.Out.WriteLine(string.Format("{0} {1,2} {2,-8} {3,-6} {4,6} {5} {6}",
                    stat.Permissions, stat.Links, stat.Owner, stat.Group,
                    stat.Size, date, entry));
            }
            catch { /* skip entries we can't stat */ }
        }
    }

    private static string SegmentName(string path)
    {
        var trimmed = path.TrimEnd('/');
        var slash = trimmed.LastIndexOf('/');
        return slash < 0 ? trimmed : trimmed[(slash + 1)..];
    }
}

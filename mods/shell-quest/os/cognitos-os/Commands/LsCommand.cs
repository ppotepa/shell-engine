using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class LsCommand : ICommand
{
    public string Name => "ls";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        bool showAll = false, longFmt = false, onePer = false;
        string? target = null;

        foreach (var arg in ctx.Argv)
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
                            return new CommandResult(new[]
                            {
                                $"ls: illegal option -- {c}",
                                "Try: man ls"
                            }, 1);
                    }
                }
            }
            else
            {
                target = arg;
            }
        }

        var absolute = target != null
            ? ctx.Session.ResolvePath(target)
            : ctx.Session.Cwd;

        var vfsPath = ctx.Os.FileSystem.ToVfsPath(absolute);

        if (!ctx.Os.FileSystem.DirectoryExists(vfsPath))
            return new CommandResult(new[]
            {
                Style.Fg(Style.Error, $"ls: {target ?? "."}: No such file or directory")
            }, 2);

        var entries = ctx.Os.FileSystem.Ls(vfsPath).ToArray();

        // Filter dotfiles unless -a
        if (!showAll)
            entries = entries.Where(e => !SegmentName(e).StartsWith('.')).ToArray();

        if (entries.Length == 0)
            return new CommandResult(Array.Empty<string>());

        if (longFmt)
            return FormatLong(entries, vfsPath, ctx);

        if (onePer)
            return new CommandResult(entries);

        // Default: space-separated (multi-column approximation)
        return new CommandResult(new[] { string.Join("  ", entries) });
    }

    private static CommandResult FormatLong(string[] entries, string dirPath, CommandContext ctx)
    {
        var lines = new List<string>();

        foreach (var entry in entries)
        {
            var name = entry.TrimEnd('/');
            var fullPath = dirPath.Length > 0 ? $"{dirPath}/{name}" : name;
            var stat = ctx.Os.FileSystem.GetStat(fullPath);
            if (stat == null) continue;

            var date = stat.Modified.ToString("MMM dd HH:mm");
            lines.Add(string.Format("{0} {1,2} {2,-8} {3,-6} {4,6} {5} {6}",
                stat.Permissions, stat.Links, stat.Owner, stat.Group,
                stat.Size, date, entry));
        }

        return new CommandResult(lines);
    }

    private static string SegmentName(string path)
    {
        var trimmed = path.TrimEnd('/');
        var slash = trimmed.LastIndexOf('/');
        return slash < 0 ? trimmed : trimmed[(slash + 1)..];
    }
}

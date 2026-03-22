using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class UnameCommand : ICommand
{
    public string Name => "uname";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        // Parse single-char flags: -s, -n, -r, -v, -m, -p, -a
        var flags = new HashSet<char>();
        foreach (var arg in ctx.Argv)
        {
            if (arg.StartsWith('-') && arg.Length > 1)
            {
                foreach (var c in arg[1..])
                    flags.Add(c);
            }
        }

        if (flags.Contains('a'))
        {
            flags.UnionWith(new[] { 's', 'n', 'r', 'v', 'm' });
        }

        if (flags.Count == 0)
            return new CommandResult(new[] { "MINIX" });

        var parts = new List<string>();
        if (flags.Contains('s')) parts.Add("MINIX");
        if (flags.Contains('n')) parts.Add("kruuna");
        if (flags.Contains('r')) parts.Add("1.1");
        if (flags.Contains('v')) parts.Add("#1 Sep 17 1991");
        if (flags.Contains('m')) parts.Add("i386");
        if (flags.Contains('p')) parts.Add(ctx.Os.Spec.CpuModel);

        return new CommandResult(new[] { string.Join(" ", parts) });
    }
}

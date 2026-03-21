using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class CdCommand : ICommand
{
    public string Name => "cd";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        var target = ctx.Argv.Count > 0 ? ctx.Argv[0] : "~";

        if (target is "~" or "/home/linus")
        {
            ctx.Os.State.Cwd = "~";
            return new CommandResult(Array.Empty<string>());
        }

        if (target == "..")
        {
            if (ctx.Os.State.Cwd is "~" or "/")
            {
                return new CommandResult(Array.Empty<string>());
            }
            var parts = ctx.Os.State.Cwd.Split('/');
            ctx.Os.State.Cwd = parts.Length <= 2 ? "~" : string.Join("/", parts[..^1]);
            return new CommandResult(Array.Empty<string>());
        }

        if (target == ".")
        {
            return new CommandResult(Array.Empty<string>());
        }

        // Resolve relative path from cwd
        var resolved = target.StartsWith('/') ? target : $"{ctx.Os.State.Cwd}/{target}";
        resolved = resolved.Replace("~/", "").TrimStart('/');

        // Check if directory exists in the virtual filesystem
        var listing = ctx.Os.FileSystem.Ls(resolved);
        if (listing.Any())
        {
            ctx.Os.State.Cwd = target.StartsWith('/') ? target : $"~/{resolved}".TrimEnd('/');
            return new CommandResult(Array.Empty<string>());
        }

        return new CommandResult(new[] { $"cd: {target}: No such file or directory" }, 1);
    }
}

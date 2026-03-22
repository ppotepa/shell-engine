using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class UnameCommand : ICommand
{
    public string Name => "uname";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Any(a => a is "-a" or "--all"))
            return new CommandResult(new[] { $"MINIX 1.1 kruuna {ctx.Os.Spec.CpuModel.Split(' ').Last()} i386 Sep 17 1991" });

        if (ctx.Argv.Any(a => a is "-r" or "--release"))
            return new CommandResult(new[] { "1.1" });

        if (ctx.Argv.Any(a => a is "-n" or "--nodename"))
            return new CommandResult(new[] { "kruuna" });

        return new CommandResult(new[] { "MINIX" });
    }
}

using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class IdCommand : ICommand
{
    public string Name => "id";
    public IReadOnlyList<string> Aliases => new[] { "groups" };

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.CommandName == "groups")
            return new CommandResult(new[] { "staff operator" });

        return new CommandResult(new[] { $"uid=101({ctx.Session.User}) gid=10(staff) groups=10(staff),5(operator)" });
    }
}

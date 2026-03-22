using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class HelpCommand : ICommand
{
    public string Name => "help";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        return new CommandResult(new[]
        {
            "Type a command. For help: man <command>",
            "",
            "  ls [dir]       list directory       cat [file]    display file",
            "  cd [dir]       change directory     pwd           working directory",
            "  cp src dst     copy file            ps [-alx]     process status",
            "  who            logged-in users      whoami        current user",
            "  uname [-a]     system name          date          date and time",
            "  man <topic>    manual page          ftp [host]    file transfer",
            "  clear          clear screen",
        }, ExitCode: 0);
    }
}

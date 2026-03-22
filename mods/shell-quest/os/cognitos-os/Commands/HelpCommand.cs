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
            "commands:",
            "  ls [dir]       list directory          cat <file>     display file",
            "  cd <dir>       change directory         pwd            print working dir",
            "  cp <src> <dst> copy file                grep <p> <f>   search in file",
            "  head <file>    first lines              tail <file>    last lines",
            "  wc <file>      word count               file <path>    file type",
            "  echo <text>    print text               man <topic>    manual page",
            "  ftp [host]     FTP client               ping <host>    network ping",
            "  nslookup <h>   DNS lookup               netstat        connections",
            "  ifconfig       interface config          df             disk free",
            "  free           memory stats              mount          filesystems",
            "  ps             processes                 top            system monitor",
            "  services       running services          kill <pid>     signal process",
            "  dmesg          kernel messages           uptime         system uptime",
            "  date           current date              uname [-a]     system info",
            "  who            logged-in users            whoami         current user",
            "  hostname       machine name               id             user identity",
            "  finger <user>  user info                  env            environment",
            "  history        command history            fortune        random quote",
            "  sync           flush disk                  clear          clear screen",
            "  help           this message",
        }, ExitCode: 0);
    }
}

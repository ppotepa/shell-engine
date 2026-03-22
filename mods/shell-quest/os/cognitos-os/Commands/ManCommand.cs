using CognitosOs.Core;

namespace CognitosOs.Commands;

internal sealed class ManCommand : ICommand
{
    public string Name => "man";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public CommandResult Execute(CommandContext ctx)
    {
        if (ctx.Argv.Count < 1)
            return new CommandResult(new[] { "What manual page do you want?" }, 1);

        var topic = ctx.Argv.Last().ToLowerInvariant();
        if (Pages.TryGetValue(topic, out var page))
            return new CommandResult(page.Split('\n'));

        return new CommandResult(new[] { $"No manual entry for {topic}." }, 1);
    }

    private static readonly Dictionary<string, string> Pages = new(StringComparer.OrdinalIgnoreCase)
    {
        ["ftp"] = @"FTP(1)                  MINIX Programmer's Manual                 FTP(1)

NAME
    ftp - file transfer program

SYNOPSIS
    ftp [host]

DESCRIPTION
    ftp is the user interface to the Internet standard File
    Transfer Protocol.

COMMANDS
    open host       connect to remote host
    close           close connection
    bye             exit ftp
    ls              list remote directory
    cd dir          change remote directory
    put file        send file to remote
    get file        receive file from remote
    binary          set binary transfer mode
    ascii           set ascii transfer mode
    status          show current status
    help            show command list

TRANSFER MODES
    ascii       Text mode. Line endings are converted.
                Suitable for plain text files only.

    binary      Image mode. No conversion performed.
                REQUIRED for compressed, executable, or
                archive files (.Z, .tar, .gz, .a, .out).

                WARNING: transferring binary files in ascii
                mode WILL corrupt the data.

SEE ALSO
    ftpd(8), netstat(1)

MINIX 1.1                  Sep 1991                              FTP(1)",

        ["ls"] = @"LS(1)                   MINIX Programmer's Manual                  LS(1)

NAME
    ls - list directory contents

SYNOPSIS
    ls [-la] [directory]

DESCRIPTION
    List information about files in the current directory or
    the specified directory.

OPTIONS
    -a      include hidden files (starting with .)
    -l      long listing format

SEE ALSO
    cat(1), cd(1)

MINIX 1.1                  Sep 1991                               LS(1)",

        ["cat"] = @"CAT(1)                  MINIX Programmer's Manual                 CAT(1)

NAME
    cat - concatenate and print files

SYNOPSIS
    cat <file>

DESCRIPTION
    Reads the contents of the specified file and writes them
    to standard output.

SEE ALSO
    more(1), head(1), tail(1)

MINIX 1.1                  Sep 1991                              CAT(1)",

        ["cp"] = @"CP(1)                   MINIX Programmer's Manual                  CP(1)

NAME
    cp - copy files

SYNOPSIS
    cp <source> <destination>

DESCRIPTION
    Copy source file to destination. Fails if insufficient
    disk space is available.

SEE ALSO
    mv(1), rm(1)

MINIX 1.1                  Sep 1991                               CP(1)",

        ["ping"] = @"PING(8)                 MINIX Programmer's Manual                PING(8)

NAME
    ping - send ICMP echo request to network host

SYNOPSIS
    ping <host>

DESCRIPTION
    Sends 3 ICMP ECHO_REQUEST packets to the specified host
    and reports round-trip times.

SEE ALSO
    netstat(1), ifconfig(8)

MINIX 1.1                  Sep 1991                             PING(8)",

        ["chmod"] = @"CHMOD(1)                MINIX Programmer's Manual               CHMOD(1)

NAME
    chmod - change file mode bits

SYNOPSIS
    chmod <mode> <file>

DESCRIPTION
    Change the access permissions of the named file.
    Only the file owner or superuser may change permissions.

SEE ALSO
    chown(8), ls(1)

MINIX 1.1                  Sep 1991                            CHMOD(1)",

        ["hier"] = @"HIER(7)                 MINIX Programmer's Manual                HIER(7)

NAME
    hier - description of the filesystem hierarchy

DESCRIPTION
    /           root directory
    /bin        essential user command binaries
    /dev        device special files
    /etc        system configuration files
    /home       user home directories
    /tmp        temporary files
    /usr        secondary hierarchy
    /usr/ast    professor Tanenbaum's home directory
    /usr/bin    non-essential command binaries
    /usr/lib    libraries
    /usr/man    manual pages
    /usr/src    source code
    /var        variable data files
    /var/log    log files
    /var/spool  spool directories (mail, cron)

MINIX 1.1                  Sep 1991                             HIER(7)",
    };
}

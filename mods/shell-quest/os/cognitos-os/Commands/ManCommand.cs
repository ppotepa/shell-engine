using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("man", OsTag = "minix")]
internal sealed class ManCommand : IKernelCommand
{
    public string Name => "man";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 2)
        {
            uow.Err.WriteLine("What manual page do you want?");
            return 1;
        }

        var topic = argv.Last().ToLowerInvariant();
        if (Pages.TryGetValue(topic, out var page))
        {
            foreach (var line in page.Split('\n'))
                uow.Out.WriteLine(line);
            return 0;
        }

        uow.Err.WriteLine($"No manual entry for {topic}.");
        return 1;
    }

    private static readonly Dictionary<string, string> Pages = new(StringComparer.OrdinalIgnoreCase)
    {
        ["sh"] = @"SH(1)                   MINIX Programmer's Manual                  SH(1)

NAME
    sh - shell (command interpreter)

SYNOPSIS
    sh [-eixv] [file]

DESCRIPTION
    Sh is the standard command interpreter.  It reads commands from
    the terminal or from a file.

BUILTINS
    cd [dir]    change working directory (default: $HOME)
    pwd         print working directory
    exit [n]    exit the shell with status n
    export      set environment variables
    set         set shell options
    read        read a line from standard input
    umask       set file creation mask

STARTUP FILES
    /etc/profile        system-wide profile
    $HOME/.profile      per-user profile

SEE ALSO
    csh(1), login(1), profile(5)

MINIX 1.1                  Sep 1991                               SH(1)",

        ["ls"] = @"LS(1)                   MINIX Programmer's Manual                  LS(1)

NAME
    ls - list the contents of a directory

SYNOPSIS
    ls [-1adglrstu] [name ...]

DESCRIPTION
    For each file argument, list it.  For each directory argument,
    list its contents.  When no argument is given, the current
    directory is listed.

OPTIONS
    -1      one entry per line
    -a      all entries (including . files)
    -l      long listing

SEE ALSO
    cat(1), stat(1)

MINIX 1.1                  Sep 1991                               LS(1)",

        ["cat"] = @"CAT(1)                  MINIX Programmer's Manual                 CAT(1)

NAME
    cat - concatenate and print files

SYNOPSIS
    cat [-u] [file ...]

DESCRIPTION
    Cat reads each file in sequence and writes it on the standard
    output.  If no file is given, or if - is given, it reads from
    the standard input.

OPTIONS
    -u      unbuffered output

SEE ALSO
    cp(1), more(1)

MINIX 1.1                  Sep 1991                              CAT(1)",

        ["cp"] = @"CP(1)                   MINIX Programmer's Manual                  CP(1)

NAME
    cp - copy files

SYNOPSIS
    cp file1 file2
    cp file ... directory

DESCRIPTION
    In the first form, file1 is copied to file2.  In the second
    form, each file is copied into the directory.

SEE ALSO
    mv(1), rm(1), ln(1)

MINIX 1.1                  Sep 1991                               CP(1)",

        ["ps"] = @"PS(1)                   MINIX Programmer's Manual                  PS(1)

NAME
    ps - process status

SYNOPSIS
    ps [-alx]

DESCRIPTION
    Ps prints information about active processes.  Without options,
    only the caller's processes with controlling terminals are
    shown.

OPTIONS
    -a      all processes with terminals
    -l      long listing (F S UID PID PPID PGRP SZ TTY TIME CMD)
    -x      include processes without controlling terminals

SEE ALSO
    kill(1)

MINIX 1.1                  Sep 1991                               PS(1)",

        ["who"] = @"WHO(1)                  MINIX Programmer's Manual                 WHO(1)

NAME
    who - show who is logged on

SYNOPSIS
    who [file]

DESCRIPTION
    Who reads /etc/utmp and displays the name, terminal, and login
    time for each user currently logged in.

FILES
    /etc/utmp

SEE ALSO
    whoami(1), finger(1)

MINIX 1.1                  Sep 1991                              WHO(1)",

        ["whoami"] = @"WHOAMI(1)               MINIX Programmer's Manual              WHOAMI(1)

NAME
    whoami - print effective user name

SYNOPSIS
    whoami

DESCRIPTION
    Whoami prints the user name associated with the current
    effective user id.

SEE ALSO
    who(1)

MINIX 1.1                  Sep 1991                           WHOAMI(1)",

        ["uname"] = @"UNAME(1)                MINIX Programmer's Manual               UNAME(1)

NAME
    uname - print system information

SYNOPSIS
    uname [-snrvmpa]

DESCRIPTION
    Uname prints information about the machine and operating
    system it is running on.

OPTIONS
    -s      system name (default)
    -n      node name (network name of this machine)
    -r      release
    -v      version
    -m      machine hardware name
    -p      processor type
    -a      all of the above

SEE ALSO
    hostname(7)

MINIX 1.1                  Sep 1991                            UNAME(1)",

        ["date"] = @"DATE(1)                 MINIX Programmer's Manual                DATE(1)

NAME
    date - print or set the date

SYNOPSIS
    date [-qsu] [[MMDDYY]hhmm[ss]] [+format]

DESCRIPTION
    Without arguments, date prints the current date and time.
    With a + argument, the output format can be specified using
    conversion characters preceded by %.

FORMAT CHARACTERS
    %c      locale date and time
    %T      time as HH:MM:SS
    %D      date as MM/DD/YY

SEE ALSO
    time(2)

MINIX 1.1                  Sep 1991                             DATE(1)",

        ["mail"] = @"MAIL(1)                 MINIX Programmer's Manual                MAIL(1)

NAME
    mail - send and receive mail

SYNOPSIS
    mail [-s subject] [user ...]

DESCRIPTION
    Mail with no arguments reads mail from the mailbox.  Each
    message is displayed and the user is prompted for a
    disposition command.

    With arguments, mail sends a message to the named users.

COMMANDS (reading mode)
    <newline>   display next message
    d           delete current message
    p           re-display current message
    q           quit, saving undeleted messages
    x           exit, do not modify mailbox
    s [file]    save message to file

    This is an extremely simple electronic mail program.

SEE ALSO
    write(1)

MINIX 1.1                  Sep 1991                             MAIL(1)",

        ["ftp"] = @"FTP(1)                  MINIX Programmer's Manual                 FTP(1)

NAME
    ftp - file transfer program

SYNOPSIS
    ftp [host]

DESCRIPTION
    Ftp is the user interface to the Internet standard File
    Transfer Protocol.

COMMANDS
    open host       connect to remote host
    close           close connection
    bye             exit ftp
    user            send user credentials
    ls              list remote directory
    cd dir          change remote directory
    pwd             print remote directory
    put file        send file to remote
    get file        receive file from remote
    binary          set binary transfer mode
    ascii           set ascii transfer mode (default)
    status          show current status
    help            show command list

TRANSFER MODES
    ascii       Text mode.  Line endings are converted.
                Suitable for plain text files only.
    binary      Image mode.  No conversion is performed.
                REQUIRED for compressed or archive files.

    WARNING: transferring binary files in ascii mode will
    corrupt the data silently.

SEE ALSO
    ftpd(8)

MINIX 1.1                  Sep 1991                              FTP(1)",

        ["man"] = @"MAN(1)                  MINIX Programmer's Manual                 MAN(1)

NAME
    man - display manual pages

SYNOPSIS
    man [-s section] title ...

DESCRIPTION
    Man formats and displays the on-line manual pages.  Pages
    are found in /usr/man.

    Sections are numbered 1-9:
        1   User commands
        2   System calls
        3   Library functions
        4   Special files
        5   File formats
        6   Games
        7   Miscellaneous
        8   System administration
        9   Kernel internals

SEE ALSO
    whatis(1), apropos(1)

MINIX 1.1                  Sep 1991                              MAN(1)",

        ["clear"] = @"CLEAR(1)                MINIX Programmer's Manual               CLEAR(1)

NAME
    clear - clear the terminal screen

SYNOPSIS
    clear

DESCRIPTION
    Clear clears your screen if this is possible.

MINIX 1.1                  Sep 1991                            CLEAR(1)",

        ["finger"] = @"FINGER(1)               MINIX Programmer's Manual              FINGER(1)

NAME
    finger - user information lookup program

SYNOPSIS
    finger [options] [name ...]

DESCRIPTION
    Finger displays information about local users.  When given
    a login name, it shows the user's real name, home directory,
    shell, login time, and the contents of .plan and .project
    files in their home directory.

FILES
    ~/.plan         user plan file
    ~/.project      user project file

SEE ALSO
    who(1), w(1)

MINIX 1.1                  Sep 1991                           FINGER(1)",

        ["ed"] = @"ED(1)                   MINIX Programmer's Manual                  ED(1)

NAME
    ed - line-oriented text editor

SYNOPSIS
    ed [file]

DESCRIPTION
    Ed is the standard text editor.  If a file argument is given,
    ed reads the file into a buffer and reports the number of
    characters read.

COMMANDS
    a           append text after current line (end with .)
    d           delete current line
    i           insert text before current line (end with .)
    p           print current line
    q           quit
    r file      read file into buffer
    s/old/new/  substitute
    w [file]    write buffer to file

SEE ALSO
    vi(1), sed(1)

MINIX 1.1                  Sep 1991                               ED(1)",

        ["write"] = @"WRITE(1)                MINIX Programmer's Manual               WRITE(1)

NAME
    write - send a message to another user

SYNOPSIS
    write user [tty]

DESCRIPTION
    Write copies lines from your terminal to the named user's
    terminal.  End with CTRL-D.

SEE ALSO
    mesg(1), mail(1)

MINIX 1.1                  Sep 1991                            WRITE(1)",

        ["mesg"] = @"MESG(1)                 MINIX Programmer's Manual                MESG(1)

NAME
    mesg - permit or deny messages

SYNOPSIS
    mesg [n] [y]

DESCRIPTION
    Mesg with argument n forbids messages via write.  With
    argument y (or no argument) messages are permitted.

SEE ALSO
    write(1)

MINIX 1.1                  Sep 1991                             MESG(1)",

        ["tar"] = @"TAR(1)                  MINIX Programmer's Manual                 TAR(1)

NAME
    tar - tape archiver

SYNOPSIS
    tar [cxtv][f] tarfile [file ...]

DESCRIPTION
    Tar saves and restores files on an archive.

KEY FUNCTIONS
    c       create a new archive
    x       extract from archive
    t       list contents of archive

OPTIONS
    v       verbose (list files processed)
    f       next argument is archive filename

SEE ALSO
    compress(1), ar(1)

MINIX 1.1                  Sep 1991                              TAR(1)",

        ["compress"] = @"COMPRESS(1)             MINIX Programmer's Manual            COMPRESS(1)

NAME
    compress, uncompress, zcat - compress and expand data

SYNOPSIS
    compress [-cdfv] [file ...]
    uncompress [file ...]
    zcat [file ...]

DESCRIPTION
    Compress reduces the size of the named files using adaptive
    Lempel-Ziv coding.  Each file is replaced by one with the
    extension .Z.

    Uncompress restores compressed files.  Zcat writes the
    uncompressed data to standard output.

SEE ALSO
    tar(1)

MINIX 1.1                  Sep 1991                         COMPRESS(1)",

        ["ping"] = @"PING(8)                 MINIX Programmer's Manual                PING(8)

NAME
    ping - send ICMP echo requests to network host

SYNOPSIS
    ping host

DESCRIPTION
    Ping sends ICMP ECHO_REQUEST packets to the specified host
    and reports round-trip times.

SEE ALSO
    ftp(1)

MINIX 1.1                  Sep 1991                             PING(8)",

        ["hier"] = @"HIER(7)                 MINIX Programmer's Manual                HIER(7)

NAME
    hier - description of the filesystem hierarchy

DESCRIPTION
    A sketch of the filesystem hierarchy.

    /               root directory
    /bin            essential user command binaries
    /dev            device special files
    /etc            system configuration files
    /tmp            temporary files
    /usr            secondary hierarchy
    /usr/ast        Tanenbaum's home directory
    /usr/bin        non-essential command binaries
    /usr/lib        libraries
    /usr/linus      Torvalds' home directory
    /usr/man        manual pages
    /usr/src        source code
    /var            variable data files
    /var/log        log files
    /var/spool      spool directories (mail, cron)

SEE ALSO
    ls(1), find(1)

MINIX 1.1                  Sep 1991                             HIER(7)",

        ["passwd"] = @"PASSWD(5)               MINIX Programmer's Manual              PASSWD(5)

NAME
    passwd - password file

DESCRIPTION
    /etc/passwd contains one line for each user account, with
    seven colon-separated fields:

        name:password:uid:gid:gecos:dir:shell

    The password field contains an encrypted password or x if
    shadow passwords are used.

FILES
    /etc/passwd

SEE ALSO
    login(1), su(1)

MINIX 1.1                  Sep 1991                           PASSWD(5)",

        ["kill"] = @"KILL(1)                 MINIX Programmer's Manual                KILL(1)

NAME
    kill - send a signal to a process

SYNOPSIS
    kill [-signal] pid ...

DESCRIPTION
    Kill sends the specified signal (default SIGTERM) to the
    specified processes.

SIGNALS
    1   HUP     hangup
    2   INT     interrupt
    9   KILL    kill (cannot be caught)
    15  TERM    software termination (default)

SEE ALSO
    ps(1), signal(2)

MINIX 1.1                  Sep 1991                             KILL(1)",

        ["df"] = @"DF(1)                   MINIX Programmer's Manual                  DF(1)

NAME
    df - disk free

SYNOPSIS
    df [filesystem ...]

DESCRIPTION
    Df prints the amount of disk space available on the
    specified filesystem, or on all mounted filesystems.

SEE ALSO
    du(1), mount(8)

MINIX 1.1                  Sep 1991                               DF(1)",
    };
}

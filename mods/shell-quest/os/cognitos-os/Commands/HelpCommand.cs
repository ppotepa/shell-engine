using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("help", OsTag = "minix")]
internal sealed class HelpCommand : IKernelCommand
{
    public string Name => "help";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        uow.Out.WriteLine("Type a command. For help: man <command>");
        uow.Out.WriteLine("");
        uow.Out.WriteLine("  ls [dir]       list directory       cat [file]    display file");
        uow.Out.WriteLine("  cd [dir]       change directory     pwd           working directory");
        uow.Out.WriteLine("  cp src dst     copy file            ps [-alx]     process status");
        uow.Out.WriteLine("  who            logged-in users      whoami        current user");
        uow.Out.WriteLine("  uname [-a]     system name          date          date and time");
        uow.Out.WriteLine("  man <topic>    manual page          ftp [host]    file transfer");
        uow.Out.WriteLine("  clear          clear screen");
        return 0;
    }
}

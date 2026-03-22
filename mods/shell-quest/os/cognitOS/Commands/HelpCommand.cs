using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("help", OsTag = "minix")]
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
        uow.Out.WriteLine("  cp src dst     copy file            mv src dst    move/rename file");
        uow.Out.WriteLine("  rm file        remove file          mkdir dir     make directory");
        uow.Out.WriteLine("  ps [-alx]      process status       who           logged-in users");
        uow.Out.WriteLine("  date           date and time        uname [-a]    system name");
        uow.Out.WriteLine("  grep pat file  search file          man <topic>   manual page");
        uow.Out.WriteLine("  ftp [host]     file transfer        finger [user] user info");
        uow.Out.WriteLine("  clear          clear screen");
        return 0;
    }
}

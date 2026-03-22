using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

[CognitosOs.Framework.Ioc.Command("ftp", OsTag = "minix")]
internal sealed class FtpCommand : IKernelCommand
{
    public string Name => "ftp";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length > 1)
            uow.Quest.FtpRemoteHost = argv[1];
        return 900; // signal shell to launch FTP app
    }
}

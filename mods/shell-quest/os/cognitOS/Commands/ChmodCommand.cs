using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("chmod", OsTag = "minix")]
internal sealed class ChmodCommand : IKernelCommand
{
    public string Name => "chmod";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv.Length < 3)
        {
            uow.Err.WriteLine("usage: chmod mode file ...");
            return 1;
        }
        // In MINIX 1.1, only root can chmod most system files.
        // For the player's own files, we accept it silently (VFS doesn't enforce perms yet).
        return 0;
    }
}

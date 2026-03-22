using CognitOS.Core;
using CognitOS.Kernel;

namespace CognitOS.Commands;

[CognitOS.Framework.Ioc.Command("su", OsTag = "minix")]
internal sealed class SuCommand : IKernelCommand
{
    public string Name => "su";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        uow.Out.WriteLine("Password: ");
        uow.Out.WriteLine("su: incorrect password");
        return 1;
    }
}

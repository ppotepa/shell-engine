using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class IdCommand : IKernelCommand
{
    public string Name => "id";
    public IReadOnlyList<string> Aliases => new[] { "groups" };

    public int Run(IUnitOfWork uow, string[] argv)
    {
        if (argv[0] == "groups")
        {
            uow.Out.WriteLine("staff operator");
            return 0;
        }

        uow.Out.WriteLine($"uid=101({uow.Session.User}) gid=10(staff) groups=10(staff),5(operator)");
        return 0;
    }
}

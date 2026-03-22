using CognitosOs.Core;
using CognitosOs.Kernel;

namespace CognitosOs.Commands;

internal sealed class EchoCommand : IKernelCommand
{
    public string Name => "echo";
    public IReadOnlyList<string> Aliases => Array.Empty<string>();

    public int Run(IUnitOfWork uow, string[] argv)
    {
        var text = string.Join(" ", argv);

        text = text.Replace("$USER", uow.Session.User);
        text = text.Replace("$HOME", uow.Session.Home);
        text = text.Replace("$SHELL", "/bin/sh");
        text = text.Replace("$HOSTNAME", uow.Session.Hostname);
        text = text.Replace("$PWD", uow.Session.Cwd);
        text = text.Replace("$?", uow.Session.LastExitCode.ToString());

        uow.Out.WriteLine(text);
        return 0;
    }
}

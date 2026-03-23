using CognitOS.Kernel;

namespace CognitOS.Network;

/// <summary>
/// A remote host that produces anomalous ping output.
/// PingCommand checks <c>host is IEasterEgg</c> and calls Execute() instead of
/// running the normal ping loop. Execute() is responsible for all visible output.
/// </summary>
internal interface IEasterEgg : IRemoteHost
{
    void Execute(IUnitOfWork uow);
}

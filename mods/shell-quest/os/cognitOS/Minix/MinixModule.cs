namespace CognitOS.Minix;

using CognitOS.Core;
using CognitOS.Framework.Ioc;
using FrameworkKernel = CognitOS.Framework.Kernel.IKernel;
using MinixKernel = CognitOS.Kernel.Kernel;
using CognitOS.Network;
using CognitOS.State;

internal sealed class MinixModule : IOperatingSystemModule
{
    private readonly MachineSpec _spec;
    private readonly IMutableFileSystem _vfs;
    private readonly NetworkRegistry _network;

    public string Name => "MINIX 1.1";

    public MinixModule(MachineSpec spec, IMutableFileSystem vfs, NetworkRegistry network)
    {
        _spec = spec;
        _vfs = vfs;
        _network = network;
    }

    public void Register(ServiceContainer container)
    {
        // Kernel is per-OS-stage singleton — one instance, composed internally
        container.RegisterInstance<FrameworkKernel>(new MinixKernel(_spec, _vfs, _network));
    }
}

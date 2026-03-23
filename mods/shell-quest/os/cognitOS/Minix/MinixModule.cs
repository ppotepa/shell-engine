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
    private readonly RemoteHostIndex _hostIndex;

    public string Name => "MINIX 1.1";

    public MinixModule(MachineSpec spec, IMutableFileSystem vfs, RemoteHostIndex hostIndex)
    {
        _spec = spec;
        _vfs = vfs;
        _hostIndex = hostIndex;
    }

    public void Register(ServiceContainer container)
    {
        container.RegisterInstance<FrameworkKernel>(new MinixKernel(_spec, _vfs, _hostIndex));
    }
}

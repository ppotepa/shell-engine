namespace CognitOS.Kernel;

/// <summary>
/// Kernel-internal IUnitOfWork. Extends Framework.Kernel.IUnitOfWork so that
/// IKernelCommand signatures remain stable while the framework interface is canonical.
/// </summary>
internal interface IUnitOfWork : CognitOS.Framework.Kernel.IUnitOfWork { }

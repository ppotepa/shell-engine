namespace CognitOS.Kernel;

/// <summary>
/// Kernel-internal IUnitOfWork. Extends Framework.Kernel.IUnitOfWork so that
/// IKernelCommand signatures remain stable while the framework interface is canonical.
/// </summary>
internal interface IUnitOfWork : CognitOS.Framework.Kernel.IUnitOfWork
{
    /// <summary>
    /// Consume all scheduled delayed outputs collected during this command scope.
    /// Returns (delayMs, line) pairs where delayMs is cumulative from command start.
    /// Called by ApplicationStack after each command to feed the drain queue.
    /// </summary>
    IReadOnlyList<(ulong DelayMs, string Line)> DrainScheduledOutputs();
}

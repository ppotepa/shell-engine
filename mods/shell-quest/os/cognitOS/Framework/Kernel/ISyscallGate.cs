namespace CognitOS.Framework.Kernel;

/// <summary>
/// Single choke point for all simulated OS operations.
/// Enforces: resource check → resource debit → latency injection → execute → resource credit.
/// All subsystems call through here instead of managing delays themselves.
/// </summary>
internal interface ISyscallGate
{
    /// <summary>
    /// Dispatch a syscall. Checks resources, injects latency, executes the action,
    /// then adjusts resource accounting. Returns the result.
    /// Throws <see cref="SyscallException"/> on resource failure.
    /// </summary>
    SyscallResult Dispatch(SyscallRequest request, Action execute);

    /// <summary>Dispatch a syscall that returns a value.</summary>
    SyscallResult Dispatch<T>(SyscallRequest request, Func<T> execute, out T result);
}

namespace CognitosOs.Framework.Kernel;

internal readonly struct SyscallRequest
{
    public SyscallKind Kind { get; init; }
    /// <summary>Size of data involved (bytes). Used for latency calculation.</summary>
    public long SizeBytes { get; init; }
    /// <summary>PID of the calling process. 0 = kernel.</summary>
    public int Pid { get; init; }

    public static SyscallRequest For(SyscallKind kind, long sizeBytes = 0, int pid = 0)
        => new() { Kind = kind, SizeBytes = sizeBytes, Pid = pid };
}

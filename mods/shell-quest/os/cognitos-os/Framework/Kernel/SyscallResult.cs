namespace CognitosOs.Framework.Kernel;

internal readonly struct SyscallResult
{
    public bool Success { get; init; }
    public string? ErrorCode { get; init; }   // "ENOSPC", "ENOMEM", "EMFILE", "ETIMEDOUT" etc.
    public int ElapsedMs { get; init; }

    public static SyscallResult Ok(int elapsedMs = 0)
        => new() { Success = true, ElapsedMs = elapsedMs };

    public static SyscallResult Fail(string errorCode)
        => new() { Success = false, ErrorCode = errorCode };

    public void ThrowIfFailed()
    {
        if (!Success)
            throw new SyscallException(ErrorCode!);
    }
}

namespace CognitOS.Framework.Kernel;

internal sealed class SyscallException : Exception
{
    public string ErrorCode { get; }

    public SyscallException(string errorCode)
        : base($"Syscall failed: {errorCode}") => ErrorCode = errorCode;
}

namespace CognitOS.Kernel.Process;

using CognitOS.State;

/// <summary>
/// Process table management. Fork/exec/exit with RAM allocation and CPU accounting.
/// </summary>
internal interface IProcessTable
{
    /// <summary>Fork a new process. Allocates RAM, increments CPU runnable count.</summary>
    /// <returns>Assigned PID.</returns>
    /// <exception cref="InvalidOperationException">Not enough memory or process table full.</exception>
    int Fork(string name, int sizeKb, string user, string tty);

    /// <summary>Simulate exec — load binary from disk (delay).</summary>
    void Exec(int pid, string binaryPath);

    /// <summary>Process exit. Frees RAM, decrements CPU count.</summary>
    void Exit(int pid);

    /// <summary>Send signal to process.</summary>
    void Kill(int pid, int signal);

    /// <summary>Snapshot of all processes.</summary>
    IReadOnlyList<ProcessEntry> List();

    /// <summary>Get the next available PID.</summary>
    int NextPid();

    /// <summary>Get process by PID.</summary>
    ProcessEntry? Get(int pid);
}

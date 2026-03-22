namespace CognitosOs.Kernel.Resources;

/// <summary>
/// Global file descriptor table. Tracks open FDs across all processes.
/// Returns EMFILE when <see cref="MaxFds"/> is reached.
/// </summary>
internal sealed class FdTable
{
    private readonly HashSet<int> _open = new();
    private int _nextFd = 3; // 0=stdin, 1=stdout, 2=stderr are pre-allocated

    public int MaxFds { get; }
    public int OpenCount => _open.Count;

    public FdTable(int maxFds)
    {
        MaxFds = maxFds;
        // Pre-open stdin/stdout/stderr
        _open.Add(0);
        _open.Add(1);
        _open.Add(2);
    }

    /// <summary>
    /// Allocate a new file descriptor. Throws if limit reached.
    /// </summary>
    /// <exception cref="InvalidOperationException">EMFILE: too many open files</exception>
    public int Alloc()
    {
        if (_open.Count >= MaxFds)
            throw new InvalidOperationException("Too many open files");

        int fd = _nextFd++;
        _open.Add(fd);
        return fd;
    }

    /// <summary>Close a file descriptor.</summary>
    public void Close(int fd)
    {
        _open.Remove(fd);
    }

    /// <summary>Check if an FD can be allocated without throwing.</summary>
    public bool CanAlloc() => _open.Count < MaxFds;
}

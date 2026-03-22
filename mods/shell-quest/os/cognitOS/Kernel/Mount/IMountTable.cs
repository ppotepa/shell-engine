namespace CognitOS.Kernel.Mount;

/// <summary>
/// A single mounted filesystem entry.
/// </summary>
internal sealed class MountEntry
{
    public string Device { get; init; } = "";
    public string MountPoint { get; init; } = "";
    public string FsType { get; init; } = "minix";
    public string Options { get; init; } = "rw";
    public int SizeKb { get; init; }
}

/// <summary>
/// The kernel mount table — tracks what is mounted where.
/// Commands (mount, df) query this instead of hardcoding output.
/// </summary>
internal interface IMountTable
{
    IReadOnlyList<MountEntry> GetMounts();
    void AddMount(MountEntry entry);
}

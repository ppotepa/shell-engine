namespace CognitOS.Network;

/// <summary>
/// Marks a class as a remote host. The scanner uses <see cref="Hostname"/> and
/// <see cref="Aliases"/> as dictionary keys; adding a new host requires only
/// this attribute — no other file needs touching.
/// </summary>
[AttributeUsage(AttributeTargets.Class, AllowMultiple = false, Inherited = false)]
internal sealed class RemoteHostAttribute(string hostname) : Attribute
{
    public string Hostname { get; } = hostname;
    public string[] Aliases { get; init; } = [];
}

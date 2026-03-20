namespace CognitosOs.State;

internal interface IVirtualFileSystem
{
    IEnumerable<string> Ls(string? path);
    bool TryCat(string target, out string content);
}

internal sealed class VirtualFileSystem : IVirtualFileSystem
{
    private readonly Dictionary<string, string> _files = new(StringComparer.Ordinal);

    public VirtualFileSystem()
    {
        _files["mail/welcome.txt"] =
            "From: Operator <op@kruuna>\n" +
            "Subject: Welcome\n" +
            "\n" +
            "you made it in. good.\n" +
            "read the notes when you get a chance.\n";

        _files["notes/starter.txt"] =
            "- type ls to look around\n" +
            "- type cat mail/welcome.txt to read your mail\n" +
            "- try top to inspect machine status\n";
    }

    public IEnumerable<string> Ls(string? path)
    {
        if (string.IsNullOrWhiteSpace(path) || path is "." or "~" or "/" or "./")
            return new[] { "mail/", "notes/" };

        var p = path.Trim().TrimEnd('/');
        if (p == "mail") return new[] { "welcome.txt" };
        if (p == "notes") return new[] { "starter.txt" };
        return Array.Empty<string>();
    }

    public bool TryCat(string target, out string content)
    {
        target = target.Trim().TrimStart('/');
        return _files.TryGetValue(target, out content!);
    }
}

namespace CognitOS.Core;

/// <summary>
/// Holds mutable user-facing state for a single login session:
/// current working directory, last exit code, and canonical path resolution.
/// Created once on login and passed through all commands and applications.
/// </summary>
internal sealed class UserSession
{
    public string User { get; }
    public string Hostname { get; }
    public string Home { get; }
    public string Cwd { get; private set; }
    public int LastExitCode { get; set; }

    /// <param name="home">
    /// Home directory — should come from /etc/passwd via IUserDatabase.
    /// Falls back to /usr/{user} if not supplied.
    /// </param>
    public UserSession(string user, string hostname, string? home = null)
    {
        User = user;
        Hostname = hostname;
        Home = !string.IsNullOrWhiteSpace(home) ? home : $"/usr/{user}";
        Cwd = Home;
    }

    /// <summary>
    /// Resolves a user-supplied path string to a canonical absolute path.
    /// Handles: ~, ~/, .., ., relative, and absolute forms.
    /// </summary>
    public string ResolvePath(string? input)
    {
        if (string.IsNullOrWhiteSpace(input) || input is "." or "./")
            return Cwd;

        if (input is "~")
            return Home;

        if (input.StartsWith("~/"))
            return Normalize(Home + "/" + input[2..]);

        if (input.StartsWith('/'))
            return Normalize(input);

        return Normalize(Cwd + "/" + input);
    }

    /// <summary>Sets the current working directory to a canonical absolute path.</summary>
    public void SetCwd(string absolutePath)
    {
        Cwd = Normalize(absolutePath);
        if (string.IsNullOrEmpty(Cwd))
            Cwd = "/";
    }

    /// <summary>
    /// Returns the cwd formatted for display: /usr/torvalds → ~, /usr/torvalds/foo → ~/foo.
    /// </summary>
    public string DisplayCwd()
    {
        if (Cwd == Home) return "~";
        if (Cwd.StartsWith(Home + "/")) return "~" + Cwd[Home.Length..];
        return Cwd;
    }

    /// <summary>
    /// Collapses . and .. segments, removes double slashes.
    /// Always returns an absolute path.
    /// </summary>
    private static string Normalize(string path)
    {
        var segments = path.Split('/', StringSplitOptions.RemoveEmptyEntries);
        var stack = new Stack<string>();
        foreach (var seg in segments)
        {
            if (seg == ".") continue;
            if (seg == "..")
            {
                if (stack.Count > 0) stack.Pop();
            }
            else
            {
                stack.Push(seg);
            }
        }

        var result = "/" + string.Join("/", stack.Reverse());
        return result == "/" ? "/" : result.TrimEnd('/');
    }
}

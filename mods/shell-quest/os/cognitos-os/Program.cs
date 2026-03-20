// See https://aka.ms/new-console-template for more information
using System.Text.Json;
using System.Text.Json.Serialization;

static class Proto
{
    private static readonly JsonSerializerOptions JsonOpts = new()
    {
        PropertyNamingPolicy = null,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        WriteIndented = false,
    };

    public static void Send(object ev)
    {
        var json = JsonSerializer.Serialize(ev, JsonOpts);
        Console.Out.WriteLine(json);
        Console.Out.Flush();
    }

    public static string? GetTypeTag(JsonElement root)
    {
        if (root.TryGetProperty("type", out var t) && t.ValueKind == JsonValueKind.String)
            return t.GetString();
        return null;
    }

    public static string? GetString(JsonElement root, string name)
    {
        if (root.TryGetProperty(name, out var p) && p.ValueKind == JsonValueKind.String)
            return p.GetString();
        return null;
    }
}

sealed class VirtualFs
{
    private readonly Dictionary<string, string> _files = new(StringComparer.Ordinal);

    public VirtualFs()
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
            "- more coming soon\n";
    }

    public IEnumerable<string> Ls(string? path)
    {
        if (string.IsNullOrWhiteSpace(path) || path == "." || path == "~" || path == "/" || path == "./")
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

enum Mode
{
    LoginUser,
    LoginPass,
    Shell,
}

internal static class Program
{
    public static void Main()
    {
        var fs = new VirtualFs();
        var mode = Mode.LoginUser;
        var user = "";

        Proto.Send(new { type = "set-prompt-prefix", text = "kruuna login: " });
        Proto.Send(new { type = "set-prompt-masked", masked = false });
        Proto.Send(new
        {
            type = "out",
            lines = new[] { "minix 1.3  copyright 1987, prentice-hall", "console ready", "" }
        });

        string? line;
        while ((line = Console.ReadLine()) != null)
        {
            line = line.TrimEnd('\r', '\n');
            if (string.IsNullOrWhiteSpace(line))
                continue;

            try
            {
                using var doc = JsonDocument.Parse(line);
                var root = doc.RootElement;
                var type = Proto.GetTypeTag(root);

                if (type == "hello" || type == "resize" || type == "tick" || type == "key")
                    continue;

                if (type != "submit")
                    continue;

                var submitted = (Proto.GetString(root, "line") ?? "").Trim();

                if (mode == Mode.LoginUser)
                {
                    user = submitted;
                    Proto.Send(new { type = "out", lines = new[] { $"kruuna login: {user}" } });
                    mode = Mode.LoginPass;
                    Proto.Send(new { type = "set-prompt-prefix", text = "password: " });
                    Proto.Send(new { type = "set-prompt-masked", masked = true });
                    continue;
                }

                if (mode == Mode.LoginPass)
                {
                    var passOk = user == "linus" && submitted == "tux";
                    Proto.Send(new { type = "out", lines = new[] { "" } });

                    if (!passOk)
                    {
                        Proto.Send(new { type = "out", lines = new[] { "login incorrect", "" } });
                        user = "";
                        mode = Mode.LoginUser;
                        Proto.Send(new { type = "set-prompt-prefix", text = "kruuna login: " });
                        Proto.Send(new { type = "set-prompt-masked", masked = false });
                        continue;
                    }

                    mode = Mode.Shell;
                    Proto.Send(new { type = "clear" });
                    Proto.Send(new
                    {
                        type = "out",
                        lines = new[]
                        {
                            "last login: tue sep 17 21:12",
                            "you have 1 new message.",
                            "",
                            "type ls to look around.",
                            "type cat <file> to read notes.",
                            "",
                        }
                    });

                    Proto.Send(new { type = "set-prompt-prefix", text = $"{user}@kruuna:~$ " });
                    Proto.Send(new { type = "set-prompt-masked", masked = false });
                    continue;
                }

                if (submitted == "clear")
                {
                    Proto.Send(new { type = "clear" });
                    continue;
                }

                var parts = submitted.Split(' ',
                    StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
                var cmd = parts.Length > 0 ? parts[0] : "";

                if (cmd == "help")
                {
                    Proto.Send(new { type = "out", lines = new[] { "commands: ls  cat <file>  clear" } });
                    continue;
                }

                if (cmd == "ls")
                {
                    var arg = parts.Length > 1 ? parts[1] : null;
                    var entries = fs.Ls(arg).ToArray();
                    if (entries.Length == 0)
                        Proto.Send(new { type = "out", lines = new[] { "ls: no such file or directory" } });
                    else
                        Proto.Send(new { type = "out", lines = new[] { string.Join("  ", entries) } });
                    continue;
                }

                if (cmd == "cat")
                {
                    if (parts.Length < 2)
                    {
                        Proto.Send(new { type = "out", lines = new[] { "usage: cat <file>" } });
                        continue;
                    }

                    var target = parts[1];
                    if (target == "mail" || target == "notes")
                    {
                        Proto.Send(new { type = "out", lines = new[] { $"cat: {target}: is a directory" } });
                        continue;
                    }

                    if (fs.TryCat(target, out var content))
                    {
                        var outLines = content.Replace("\r\n", "\n").Split('\n');
                        Proto.Send(new { type = "out", lines = outLines });
                    }
                    else
                    {
                        Proto.Send(new { type = "out", lines = new[] { $"cat: {target}: no such file or directory" } });
                    }

                    continue;
                }

                Proto.Send(new { type = "out", lines = new[] { $"{cmd}: command not found" } });
            }
            catch (Exception ex)
            {
                Proto.Send(new { type = "out", lines = new[] { $"[cognitos-os] parse error: {ex.Message}" } });
            }
        }
    }
}


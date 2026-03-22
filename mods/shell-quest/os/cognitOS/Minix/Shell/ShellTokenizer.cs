using CognitOS.Core;

namespace CognitOS.Minix.Shell;

/// <summary>
/// Tokenizes a Bourne shell command line.
/// Handles: double-quoted strings, $VAR expansion, ; separator, | pipe, > >> redirects.
/// Does NOT handle: backticks, single quotes, glob expansion, heredocs.
/// </summary>
internal static class ShellTokenizer
{
    /// <summary>
    /// Split a raw input line into a list of simple commands separated by ';'.
    /// Each simple command may contain a pipe '|' and/or output redirect '>' / '>>'.
    /// </summary>
    public static IReadOnlyList<SimpleCommand> Parse(string raw, UserSession session)
    {
        // Expand variables before splitting so $VAR inside quotes works
        raw = ExpandVars(raw, session);

        var result = new List<SimpleCommand>();
        foreach (var segment in SplitSemicolon(raw))
        {
            var trimmed = segment.Trim();
            if (string.IsNullOrEmpty(trimmed)) continue;
            result.Add(ParseSimple(trimmed));
        }
        return result;
    }

    // ── Expand $VAR ──────────────────────────────────────────────────────────

    private static string ExpandVars(string raw, UserSession session)
    {
        if (!raw.Contains('$')) return raw;

        var sb = new System.Text.StringBuilder(raw.Length);
        int i = 0;
        while (i < raw.Length)
        {
            if (raw[i] != '$')
            {
                sb.Append(raw[i++]);
                continue;
            }

            i++; // skip $
            if (i >= raw.Length) { sb.Append('$'); break; }

            if (raw[i] == '?')
            {
                sb.Append(session.LastExitCode);
                i++;
                continue;
            }

            // Read variable name
            var start = i;
            while (i < raw.Length && (char.IsLetterOrDigit(raw[i]) || raw[i] == '_'))
                i++;
            var name = raw[start..i];

            sb.Append(name switch
            {
                "HOME"     => session.Home,
                "USER"     => session.User,
                "LOGNAME"  => session.User,
                "HOSTNAME" => session.Hostname,
                "HOST"     => session.Hostname,
                "PWD"      => session.Cwd,
                "SHELL"    => "/bin/sh",
                "TERM"     => "minix",
                "PATH"     => "/bin:/usr/bin",
                _          => "$" + name,   // leave unknown vars as-is
            });
        }
        return sb.ToString();
    }

    // ── Split by ';' (respecting quotes) ────────────────────────────────────

    private static IEnumerable<string> SplitSemicolon(string raw)
    {
        var segments = new List<string>();
        int start = 0, i = 0;
        bool inQuote = false;

        while (i < raw.Length)
        {
            var ch = raw[i];
            if (ch == '"') { inQuote = !inQuote; i++; continue; }
            if (!inQuote && ch == ';')
            {
                segments.Add(raw[start..i]);
                start = i + 1;
            }
            i++;
        }
        segments.Add(raw[start..]);
        return segments;
    }

    // ── Parse a single command (may have pipe) ───────────────────────────────

    private static SimpleCommand ParseSimple(string raw)
    {
        // Split on first '|' (not inside quotes) into left | right
        var pipeIdx = IndexOfUnquoted(raw, '|');
        if (pipeIdx >= 0)
        {
            var left  = ParseRedirectAndTokens(raw[..pipeIdx].Trim());
            var right = ParseSimple(raw[(pipeIdx + 1)..].Trim());
            return new SimpleCommand(left.Tokens, left.Redirect, left.RedirectAppend, right);
        }

        var cmd = ParseRedirectAndTokens(raw);
        return new SimpleCommand(cmd.Tokens, cmd.Redirect, cmd.RedirectAppend, null);
    }

    private static (string[] Tokens, string? Redirect, bool RedirectAppend) ParseRedirectAndTokens(string raw)
    {
        // Scan for >> or > (not inside quotes)
        string? redirect = null;
        bool append = false;
        int redirPos = -1;

        for (int i = 0; i < raw.Length; i++)
        {
            if (raw[i] == '"') { while (i < raw.Length && raw[++i] != '"') { } continue; }
            if (raw[i] == '>' && i + 1 < raw.Length && raw[i + 1] == '>')
            { redirPos = i; append = true; break; }
            if (raw[i] == '>')
            { redirPos = i; append = false; break; }
        }

        string cmdPart = raw;
        if (redirPos >= 0)
        {
            cmdPart  = raw[..redirPos].Trim();
            redirect = raw[(redirPos + (append ? 2 : 1))..].Trim();
            if (redirect.Length == 0) redirect = null;
        }

        return (Tokenize(cmdPart), redirect, append);
    }

    // ── Tokenizer: split on spaces respecting double quotes ──────────────────

    public static string[] Tokenize(string raw)
    {
        var tokens = new List<string>();
        var cur = new System.Text.StringBuilder();
        bool inQuote = false;

        foreach (var ch in raw)
        {
            if (ch == '"') { inQuote = !inQuote; continue; }
            if (!inQuote && ch == ' ')
            {
                if (cur.Length > 0) { tokens.Add(cur.ToString()); cur.Clear(); }
                continue;
            }
            cur.Append(ch);
        }
        if (cur.Length > 0) tokens.Add(cur.ToString());
        return tokens.ToArray();
    }

    private static int IndexOfUnquoted(string s, char target)
    {
        bool inQuote = false;
        for (int i = 0; i < s.Length; i++)
        {
            if (s[i] == '"') { inQuote = !inQuote; continue; }
            if (!inQuote && s[i] == target) return i;
        }
        return -1;
    }
}

/// <summary>
/// A parsed simple command: tokens, optional output redirect, optional pipe target.
/// </summary>
internal sealed class SimpleCommand
{
    public string[] Tokens { get; }
    public string? RedirectFile { get; }
    public bool RedirectAppend { get; }
    public SimpleCommand? PipeTo { get; }

    public SimpleCommand(string[] tokens, string? redirectFile, bool redirectAppend, SimpleCommand? pipeTo)
    {
        Tokens = tokens;
        RedirectFile = redirectFile;
        RedirectAppend = redirectAppend;
        PipeTo = pipeTo;
    }

    public bool HasRedirect => RedirectFile is not null;
    public bool HasPipe => PipeTo is not null;
}

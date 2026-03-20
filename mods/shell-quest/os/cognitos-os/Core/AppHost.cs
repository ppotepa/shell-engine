using CognitosOs.State;

namespace CognitosOs.Core;

internal sealed class AppHost
{
    private readonly IOperatingSystem _os;
    private readonly IMachineStart _store;
    private readonly Queue<BootStep> _bootQueue = new();
    private ulong _bootCountdownMs;
    private bool _bootFinished;

    public AppHost(IOperatingSystem os, IMachineStart store)
    {
        _os = os;
        _store = store;
    }

    public void EmitBoot(IBootSequence boot)
    {
        _bootQueue.Clear();
        foreach (var step in boot.BuildBootSteps(_os))
        {
            _bootQueue.Enqueue(step);
        }
        _bootCountdownMs = 0;
        _bootFinished = false;
        _os.State.Mode = SessionMode.Booting;
        Protocol.Send(new { type = "set-prompt-prefix", text = "" });
        Protocol.Send(new { type = "set-prompt-masked", masked = false });
    }

    public void HandleTick(ulong dtMs)
    {
        _os.Tick(dtMs);
        DriveBoot(dtMs);
    }

    public void HandleSubmit(string raw)
    {
        if (_os.State.Mode == SessionMode.Booting)
        {
            return;
        }
        var submitted = raw.Trim();
        if (string.IsNullOrWhiteSpace(submitted))
        {
            return;
        }

        switch (_os.State.Mode)
        {
            case SessionMode.LoginUser:
                HandleLoginUser(submitted);
                break;
            case SessionMode.LoginPassword:
                HandleLoginPassword(submitted);
                break;
            case SessionMode.Shell:
                HandleShell(submitted);
                break;
        }
    }

    private void HandleLoginUser(string user)
    {
        if (!_os.State.HasAccount && !user.Equals("linus", StringComparison.Ordinal))
        {
            Protocol.Send(new { type = "out", lines = new[] { Style.Fg(Style.Warn, "first boot: login as linus"), "" } });
            return;
        }

        _os.State.PendingLoginUser = user;
        _os.State.Mode = SessionMode.LoginPassword;
        ApplyPrompt();
    }

    private void HandleLoginPassword(string password)
    {
        if (!_os.State.HasAccount)
        {
            if (password.Length > 5)
            {
                Protocol.Send(new
                {
                    type = "out",
                    lines = new[] { Style.Fg(Style.Error, "password is too long (max 5)"), "" }
                });
                return;
            }

            _os.State.UserName = "linus";
            _os.State.Password = password;
            _os.State.LastLogin = _os.SimulatedNow();
            _store.Persist(_os.State);
            EnterShell(firstLogin: true);
            return;
        }

        var passOk = _os.State.PendingLoginUser == _os.State.UserName && password == _os.State.Password;
        if (!passOk)
        {
            Protocol.Send(new { type = "out", lines = new[] { Style.Fg(Style.Error, "login incorrect"), "" } });
            _os.State.PendingLoginUser = "";
            _os.State.Mode = SessionMode.LoginUser;
            ApplyPrompt();
            return;
        }

        EnterShell(firstLogin: false);
    }

    private void EnterShell(bool firstLogin)
    {
        var now = _os.SimulatedNow();
        var last = _os.State.LastLogin ?? now;
        _os.State.LastLogin = now;
        _os.State.Mode = SessionMode.Shell;
        _os.State.Cwd = "~";
        _store.Persist(_os.State);

        Protocol.Send(new { type = "clear" });
        Protocol.Send(new
        {
            type = "out",
            lines = firstLogin
                ? new[]
                {
                    Style.Fg(Style.Info, "account created."),
                    $"last login: {now:ddd MMM dd HH:mm}",
                    "you have 1 new message.",
                    "type ls to look around.",
                    "type cat <file> to read notes.",
                    ""
                }
                : new[]
                {
                    $"last login: {last:ddd MMM dd HH:mm}",
                    "you have 1 new message.",
                    "type ls to look around.",
                    "type cat <file> to read notes.",
                    ""
                }
        });

        ApplyPrompt();
    }

    private void HandleShell(string submitted)
    {
        var parts = submitted.Split(' ', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        var cmd = parts.Length > 0 ? parts[0] : string.Empty;
        var args = parts.Skip(1).ToArray();

        if (!_os.CommandIndex.TryGetValue(cmd, out var command))
        {
            Protocol.Send(new { type = "out", lines = new[] { Style.Fg(Style.Error, $"{cmd}: command not found"), "" } });
            return;
        }

        var result = command.Execute(new CommandContext(_os, _os.State.UserName ?? "linus", _os.State.Cwd), args);

        if (result.ClearScreen)
        {
            Protocol.Send(new { type = "clear" });
            return;
        }

        var lines = result.Lines.Count == 0 ? new[] { "" } : result.Lines.Concat(new[] { "" }).ToArray();
        Protocol.Send(new { type = "out", lines });
    }

    public void ApplyPrompt()
    {
        if (_os.State.Mode == SessionMode.Booting)
        {
            Protocol.Send(new { type = "set-prompt-prefix", text = "" });
            Protocol.Send(new { type = "set-prompt-masked", masked = false });
            return;
        }
        switch (_os.State.Mode)
        {
            case SessionMode.LoginUser:
                Protocol.Send(new
                {
                    type = "set-prompt-prefix",
                    text = $"{Style.Fg(Style.PromptHost, "kruuna")} login: "
                });
                Protocol.Send(new { type = "set-prompt-masked", masked = false });
                break;
            case SessionMode.LoginPassword:
                Protocol.Send(new { type = "set-prompt-prefix", text = "password: " });
                Protocol.Send(new { type = "set-prompt-masked", masked = true });
                break;
            case SessionMode.Shell:
                var user = _os.State.UserName ?? "linus";
                var cwd = _os.State.Cwd;
                Protocol.Send(new
                {
                    type = "set-prompt-prefix",
                    text = $"{Style.Fg(Style.PromptUser, user)}@{Style.Fg(Style.PromptHost, "kruuna")}:{Style.Fg(Style.PromptPath, cwd)}$ "
                });
                Protocol.Send(new { type = "set-prompt-masked", masked = false });
                break;
        }
    }

    private void DriveBoot(ulong dtMs)
    {
        if (_bootFinished || _os.State.Mode != SessionMode.Booting)
        {
            return;
        }

        if (_bootCountdownMs > dtMs)
        {
            _bootCountdownMs -= dtMs;
            return;
        }
        _bootCountdownMs = 0;

        while (_bootQueue.Count > 0 && _bootCountdownMs == 0)
        {
            var next = _bootQueue.Dequeue();
            Protocol.Send(new { type = "out", lines = new[] { next.Text } });
            _bootCountdownMs = next.DelayMs;
        }

        if (_bootQueue.Count == 0)
        {
            _bootFinished = true;
            _os.State.Mode = SessionMode.LoginUser;
            ApplyPrompt();
        }
    }
}

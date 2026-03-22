using CognitosOs.Applications;
using CognitosOs.Commands;
using CognitosOs.Framework.Kernel;
using CognitosOs.Network;
using CognitosOs.State;

namespace CognitosOs.Core;

internal sealed class AppHost
{
    private readonly IKernel _kernel;
    private readonly MachineState _machineState;
    private readonly IMachineStart _store;
    private readonly ScreenBuffer _screen;
    private readonly EasterEggRegistry _eggs;
    private readonly HistoryCommand _historyCmd;
    private readonly IReadOnlyDictionary<string, IKernelCommand> _commandIndex;
    private readonly Action? _reloadVfs;
    private readonly Queue<BootStep> _bootQueue = new();
    private ulong _bootCountdownMs;
    private ulong _bootPostDelayMs;
    private bool _bootFinished;

    // Active after login
    private UserSession? _session;
    private ApplicationStack? _appStack;

    public AppHost(
        IKernel kernel,
        MachineState machineState,
        IMachineStart store,
        EasterEggRegistry eggs,
        HistoryCommand historyCmd,
        IReadOnlyDictionary<string, IKernelCommand> commandIndex,
        Action? reloadVfs = null)
    {
        _kernel = kernel;
        _machineState = machineState;
        _store = store;
        _eggs = eggs;
        _historyCmd = historyCmd;
        _commandIndex = commandIndex;
        _reloadVfs = reloadVfs;
        _screen = new ScreenBuffer();
    }

    public void EmitBoot(IBootSequence boot)
    {
        _bootQueue.Clear();
        foreach (var step in boot.BuildBootSteps(_kernel))
            _bootQueue.Enqueue(step);

        _bootCountdownMs = 0;
        _bootPostDelayMs = 0;
        _bootFinished = false;
        _machineState.Mode = SessionMode.Booting;
        _screen.ClearViewport();
        _screen.SetPrompt("");
        _screen.SetInputLine("");
        Protocol.Send(new { type = "set-prompt-prefix", text = "" });
        Protocol.Send(new { type = "set-prompt-masked", masked = false });
    }

    public void StartAtLogin()
    {
        _bootQueue.Clear();
        _bootCountdownMs = 0;
        _bootFinished = true;
        _machineState.Mode = SessionMode.LoginUser;
        _screen.ClearViewport();
        var brightInfo = Style.BrightenHex(Style.Info, 1.15);
        _screen.Append("Minix 1.3  Copyright 1987, Prentice-Hall", Style.Fg(brightInfo, "Console ready"), "");
        ApplyPrompt();
    }

    public void HandleTick(ulong dtMs)
    {
        _kernel.Tick(dtMs);
        DriveBoot(dtMs);
        DriveBootPostDelay(dtMs);
    }

    public void HandleResize(int cols, int rows)
        => _screen.SetViewport(cols, rows);

    public void HandleInputChange(string text)
    {
        var input = text ?? string.Empty;
        if (_machineState.Mode == SessionMode.LoginPassword)
        {
            _screen.SetInputLine(new string('*', input.Length));
            return;
        }
        _screen.SetInputLine(input);
    }

    public void HandleSubmit(string raw)
    {
        if (_machineState.Mode == SessionMode.Booting) return;

        var submitted = raw.Trim();
        if (string.IsNullOrWhiteSpace(submitted)) return;

        _screen.CommitInputLine();

        switch (_machineState.Mode)
        {
            case SessionMode.LoginUser:
                HandleLoginUser(submitted);
                break;
            case SessionMode.LoginPassword:
                HandleLoginPassword(submitted);
                break;
            case SessionMode.Shell:
                _appStack!.HandleInput(submitted, _session!);
                ApplyPrompt();
                break;
        }
    }

    private void HandleLoginUser(string user)
    {
        if (!_machineState.HasAccount && !user.Equals("linus", StringComparison.Ordinal))
        {
            _screen.Append(Style.Fg(Style.Warn, "first boot: login as linus"), "");
            return;
        }

        _machineState.PendingLoginUser = user;
        _machineState.Mode = SessionMode.LoginPassword;
        ApplyPrompt();
    }

    private void HandleLoginPassword(string password)
    {
        if (!_machineState.HasAccount)
        {
            if (password.Length > 5)
            {
                _screen.Append(Style.Fg(Style.Error, "password is too long (max 5)"), "");
                return;
            }

            _machineState.UserName = "linus";
            _machineState.Password = password;
            _machineState.LastLogin = _kernel.Clock.Now();
            _store.Persist(_machineState);
            _reloadVfs?.Invoke();

            EnterShell(firstLogin: true);
            return;
        }

        var passOk = _machineState.PendingLoginUser == _machineState.UserName
                     && password == _machineState.Password;
        if (!passOk)
        {
            _screen.Append(Style.Fg(Style.Error, "login incorrect"), "");
            _machineState.PendingLoginUser = "";
            _machineState.Mode = SessionMode.LoginUser;
            ApplyPrompt();
            return;
        }

        EnterShell(firstLogin: false);
    }

    private void EnterShell(bool firstLogin)
    {
        var now = _kernel.Clock.Now();
        var last = _machineState.LastLogin ?? now;
        _machineState.LastLogin = now;
        _machineState.Mode = SessionMode.Shell;
        _store.Persist(_machineState);
        _reloadVfs?.Invoke();

        _session = new UserSession(_machineState.UserName ?? "linus", "kruuna");
        _appStack = new ApplicationStack(_screen);
        _appStack.Push(
            new ShellApplication(_kernel, _machineState, _commandIndex, _screen, _appStack, _eggs, _historyCmd),
            _session);

        var bi = Style.BrightenHex(Style.Info, 1.15);
        _screen.ClearViewport();
        var mailCount = _kernel.Mail.UnreadCount();
        _screen.Append(firstLogin
            ? new[]
            {
                Style.Fg(bi, "account created."),
                Style.Fg(bi, $"last login: {now:ddd MMM dd HH:mm}"),
                Style.Fg(bi, $"you have {mailCount} new message{(mailCount == 1 ? "" : "s")}."),
                "type ls to look around.",
                "type cat <file> to read notes.",
                ""
            }
            : new[]
            {
                Style.Fg(bi, $"last login: {last:ddd MMM dd HH:mm}"),
                Style.Fg(bi, $"you have {mailCount} new message{(mailCount == 1 ? "" : "s")}."),
                "type ls to look around.",
                "type cat <file> to read notes.",
                ""
            });

        ApplyPrompt();
    }

    public void ApplyPrompt()
    {
        switch (_machineState.Mode)
        {
            case SessionMode.Booting:
                Protocol.Send(new { type = "set-prompt-prefix", text = "" });
                Protocol.Send(new { type = "set-prompt-masked", masked = false });
                break;

            case SessionMode.LoginUser:
                _screen.SetPrompt("kruuna login: ");
                Protocol.Send(new
                {
                    type = "set-prompt-prefix",
                    text = $"{Style.Fg(Style.PromptHost, "kruuna")} login: "
                });
                Protocol.Send(new { type = "set-prompt-masked", masked = false });
                break;

            case SessionMode.LoginPassword:
                _screen.SetPrompt("password: ");
                Protocol.Send(new { type = "set-prompt-prefix", text = "password: " });
                Protocol.Send(new { type = "set-prompt-masked", masked = true });
                break;

            case SessionMode.Shell when _appStack is not null && _session is not null:
                var promptText = _appStack.CurrentPrompt(_session);
                _screen.SetPrompt(promptText);
                Protocol.Send(new { type = "set-prompt-prefix", text = promptText });
                Protocol.Send(new { type = "set-prompt-masked", masked = false });
                break;
        }
    }

    private void DriveBoot(ulong dtMs)
    {
        if (_bootFinished || _machineState.Mode != SessionMode.Booting) return;

        if (_bootCountdownMs > dtMs)
        {
            _bootCountdownMs -= dtMs;
            return;
        }
        _bootCountdownMs = 0;

        while (_bootQueue.Count > 0 && _bootCountdownMs == 0)
        {
            var next = _bootQueue.Dequeue();
            _screen.Append(next.Text);
            _bootCountdownMs = next.DelayMs;
        }

        if (_bootQueue.Count == 0)
        {
            _bootFinished = true;
            _bootPostDelayMs = 500;
        }
    }

    private void DriveBootPostDelay(ulong dtMs)
    {
        if (_bootPostDelayMs == 0) return;

        if (dtMs >= _bootPostDelayMs)
        {
            _bootPostDelayMs = 0;
            _machineState.Mode = SessionMode.LoginUser;
            _screen.ClearViewport();
            var brightInfo = Style.BrightenHex(Style.Info, 1.15);
            _screen.Append("Minix 1.3  Copyright 1987, Prentice-Hall", Style.Fg(brightInfo, "Console ready"), "");
            ApplyPrompt();
        }
        else
        {
            _bootPostDelayMs -= dtMs;
        }
    }
}

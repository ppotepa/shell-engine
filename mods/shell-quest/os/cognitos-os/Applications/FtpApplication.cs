using CognitosOs.Core;
using CognitosOs.State;

namespace CognitosOs.Applications;

/// <summary>
/// Simulated FTP client application. Pushed onto the application stack when
/// the user runs the ftp command. Popped when the user types bye/quit/exit.
/// Transfer mode defaults to ASCII (historically accurate — the prologue bug).
/// </summary>
internal sealed class FtpApplication : IApplication
{
    private readonly IOperatingSystem _os;
    private readonly ScreenBuffer _screen;

    private bool _connected;
    private string _remoteHost = "";
    private string _transferMode = "ascii";
    private string _remoteCwd = "/pub/OS/Linux";

    private static readonly Dictionary<string, string> DnsTable =
        new(StringComparer.OrdinalIgnoreCase)
        {
            ["nic.funet.fi"] = "128.214.6.100",
            ["ftp.funet.fi"] = "128.214.6.100",
        };

    public FtpApplication(IOperatingSystem os, ScreenBuffer screen)
    {
        _os = os;
        _screen = screen;
    }

    public string PromptPrefix(UserSession session)
        => _connected ? $"ftp {_remoteHost}> " : "ftp> ";

    public void OnEnter(UserSession session)
    {
        var pendingHost = _os.State.Quest.FtpRemoteHost;
        if (!string.IsNullOrWhiteSpace(pendingHost))
        {
            _os.State.Quest.FtpRemoteHost = null;
            HandleOpen(pendingHost);
        }

        Protocol.Send(new { type = "set-prompt-prefix", text = PromptPrefix(session) });
        Protocol.Send(new { type = "set-prompt-masked", masked = false });
    }

    public void OnExit(UserSession session)
    {
        _screen.Append("");
    }

    public ApplicationResult HandleInput(string input, UserSession session)
    {
        var parts = input.Trim().Split(' ', StringSplitOptions.RemoveEmptyEntries);
        if (parts.Length == 0)
        {
            Protocol.Send(new { type = "set-prompt-prefix", text = PromptPrefix(session) });
            return ApplicationResult.Continue;
        }

        var cmd = parts[0].ToLowerInvariant();
        var args = parts.Length > 1 ? parts[1..] : Array.Empty<string>();

        switch (cmd)
        {
            case "open":
                HandleOpen(args.Length > 0 ? args[0] : "");
                break;
            case "close":
                HandleClose();
                break;
            case "binary" or "bin":
                HandleBinary();
                break;
            case "ascii":
                HandleAscii();
                break;
            case "type" when args.Length > 0 && args[0].Equals("i", StringComparison.OrdinalIgnoreCase):
                HandleBinary();
                break;
            case "type" when args.Length > 0 && args[0].Equals("a", StringComparison.OrdinalIgnoreCase):
                HandleAscii();
                break;
            case "put":
                HandlePut(args.Length > 0 ? args[0] : "", session);
                break;
            case "ls" or "dir":
                HandleLs();
                break;
            case "pwd":
                HandlePwd();
                break;
            case "cd":
                HandleCd(args.Length > 0 ? args[0] : "");
                break;
            case "status":
                HandleStatus();
                break;
            case "help" or "?":
                HandleHelp();
                break;
            case "bye" or "quit" or "exit":
                HandleClose();
                _screen.Append("221 Goodbye.");
                return ApplicationResult.Exit;
            default:
                _screen.Append($"?Invalid command: {cmd}");
                break;
        }

        Protocol.Send(new { type = "set-prompt-prefix", text = PromptPrefix(session) });
        return ApplicationResult.Continue;
    }

    private void HandleOpen(string host)
    {
        if (_connected) { _screen.Append("Already connected. Use close first."); return; }
        if (string.IsNullOrWhiteSpace(host)) { _screen.Append("(to) "); return; }
        if (!DnsTable.TryGetValue(host, out var ip))
        {
            _screen.Append($"ftp: {host}: Name or service not known");
            return;
        }

        _remoteHost = host;
        _screen.Append($"Connected to {host} ({ip}).");
        _screen.Append("220 nic.funet.fi FTP server ready.");
        _screen.Append($"Name ({host}:anonymous): anonymous");
        _screen.Append("331 Guest login ok, send strIdent as password.");
        _screen.Append("230 Guest login ok, access restrictions apply.");
        _screen.Append("Remote system type is UNIX.");
        _screen.Append($"Using {_transferMode} mode to transfer files.");
        _connected = true;
        _os.State.Quest.FtpConnected = true;
    }

    private void HandleClose()
    {
        if (!_connected) { _screen.Append("Not connected."); return; }
        _screen.Append($"221 Goodbye from {_remoteHost}.");
        _connected = false;
        _remoteHost = "";
        _os.State.Quest.FtpConnected = false;
    }

    private void HandleBinary()
    {
        _transferMode = "binary";
        _os.State.Quest.FtpTransferMode = "binary";
        _screen.Append("200 Type set to I (binary).");
    }

    private void HandleAscii()
    {
        _transferMode = "ascii";
        _os.State.Quest.FtpTransferMode = "ascii";
        _screen.Append("200 Type set to A (ascii).");
    }

    private void HandlePut(string fileName, UserSession session)
    {
        if (!_connected) { _screen.Append("Not connected."); return; }
        if (string.IsNullOrWhiteSpace(fileName)) { _screen.Append("(local-file) "); return; }

        var absolute = session.ResolvePath(fileName);
        var vfsPath = _os.FileSystem.ToVfsPath(absolute);
        if (!_os.FileSystem.TryCat(vfsPath, out var content))
        {
            _screen.Append($"local: {fileName}: No such file or directory");
            return;
        }

        var sizeBytes = content.Length;
        _screen.Append("200 PORT command successful.");
        _screen.Append($"150 Opening {_transferMode.ToUpperInvariant()} mode data connection for {fileName}.");
        _os.State.Quest.UploadAttempted = true;

        var transferTimeMs = (sizeBytes * 8) / Math.Max(_os.Spec.NicSpeedKbps, 1);
        _screen.Append("226 Transfer complete.");
        _screen.Append($"{sizeBytes} bytes sent in {transferTimeMs / 1000.0:F1} seconds.");

        if (_transferMode == "ascii")
        {
            _os.State.Quest.UploadSuccess = false;
            _screen.Append("");
            _screen.Append(Style.Fg(Style.Warn,
                $"remote: warning: {fileName} - uncompress failed, archive may be damaged"));
            _screen.Append(Style.Fg(Style.Warn,
                "remote: hint: check transfer mode (ascii vs binary)"));
        }
        else
        {
            _os.State.Quest.UploadSuccess = true;
            _screen.Append("");
            _screen.Append(Style.Fg(Style.Info,
                $"remote: {fileName} received OK, archive integrity verified."));
        }
    }

    private void HandleLs()
    {
        if (!_connected) { _screen.Append("Not connected."); return; }
        _screen.Append("200 PORT command successful.");
        _screen.Append("150 Opening ASCII mode data connection for /bin/ls.");

        if (_os.State.Quest.UploadSuccess)
        {
            _screen.Append("total 234");
            _screen.Append("drwxr-xr-x  2 ftp  ftp  512 Sep 17 21:12 .");
            _screen.Append("-rw-r--r--  1 ftp  ftp  73091 Sep 17 21:12 linux-0.01.tar.Z");
        }
        else if (_os.State.Quest.UploadAttempted)
        {
            _screen.Append("total 234");
            _screen.Append("drwxr-xr-x  2 ftp  ftp  512 Sep 17 21:12 .");
            _screen.Append("-rw-r--r--  1 ftp  ftp  73091 Sep 17 21:12 linux-0.01.tar.Z  [CORRUPT]");
        }
        else
        {
            _screen.Append("total 0");
            _screen.Append("drwxr-xr-x  2 ftp  ftp  512 Sep 17 21:00 .");
        }

        _screen.Append("226 Transfer complete.");
    }

    private void HandlePwd()
    {
        if (!_connected) { _screen.Append("Not connected."); return; }
        _screen.Append($"257 \"{_remoteCwd}\" is current directory.");
    }

    private void HandleCd(string dir)
    {
        if (!_connected) { _screen.Append("Not connected."); return; }
        if (string.IsNullOrWhiteSpace(dir)) { _screen.Append("(remote-directory) "); return; }

        if (dir is "/pub/OS/Linux" or "/pub/OS" or "/pub" or "/")
        {
            _remoteCwd = dir;
            _screen.Append("250 CWD command successful.");
        }
        else if (dir == "..")
        {
            _remoteCwd = "/pub/OS";
            _screen.Append("250 CWD command successful.");
        }
        else
        {
            _screen.Append($"550 {dir}: No such file or directory.");
        }
    }

    private void HandleStatus()
    {
        _screen.Append($"Connected to: {(_connected ? _remoteHost : "(not connected)")}");
        _screen.Append($"Transfer mode: {_transferMode}");
        _screen.Append($"Remote cwd: {_remoteCwd}");
        _screen.Append($"NIC: {_os.Spec.NicModel} ({_os.Spec.NicSpeedKbps} Kbps)");
    }

    private void HandleHelp()
    {
        _screen.Append("Commands: open <host>  close  binary  ascii  put <file>");
        _screen.Append("          ls  cd <dir>  pwd  status  help  bye");
    }
}

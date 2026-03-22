using CognitOS.Core;
using CognitOS.State;

namespace CognitOS.Applications;

/// <summary>
/// Simulated FTP client application. Pushed onto the application stack when
/// the user runs the ftp command. Popped when the user types bye/quit/exit.
/// Transfer mode defaults to ASCII (historically accurate — the prologue bug).
/// </summary>
internal sealed class FtpApplication : IKernelApplication
{
    private readonly MachineState _machineState;

    private bool _connected;
    private string _remoteHost = "";
    private string _transferMode = "ascii";
    private string _remoteCwd = "/pub/OS/Linux";

    public FtpApplication(MachineState machineState)
    {
        _machineState = machineState;
    }

    public string PromptPrefix(UserSession session)
        => _connected ? $"ftp {_remoteHost}> " : "ftp> ";

    public void OnEnter(CognitOS.Kernel.IUnitOfWork uow)
    {
        var pendingHost = _machineState.Quest.FtpRemoteHost;
        if (!string.IsNullOrWhiteSpace(pendingHost))
        {
            _machineState.Quest.FtpRemoteHost = null;
            HandleOpen(uow, pendingHost);
        }
    }

    public void OnExit(CognitOS.Kernel.IUnitOfWork uow) { }

    public ApplicationResult HandleInput(CognitOS.Kernel.IUnitOfWork uow, string input)
    {
        var parts = input.Trim().Split(' ', StringSplitOptions.RemoveEmptyEntries);
        if (parts.Length == 0)
            return ApplicationResult.Continue;

        var cmd = parts[0].ToLowerInvariant();
        var args = parts.Length > 1 ? parts[1..] : Array.Empty<string>();

        switch (cmd)
        {
            case "open":
                HandleOpen(uow, args.Length > 0 ? args[0] : "");
                break;
            case "close":
                HandleClose(uow);
                break;
            case "binary" or "bin":
                HandleBinary(uow);
                break;
            case "ascii":
                HandleAscii(uow);
                break;
            case "type" when args.Length > 0 && args[0].Equals("i", StringComparison.OrdinalIgnoreCase):
                HandleBinary(uow);
                break;
            case "type" when args.Length > 0 && args[0].Equals("a", StringComparison.OrdinalIgnoreCase):
                HandleAscii(uow);
                break;
            case "put":
                HandlePut(uow, args.Length > 0 ? args[0] : "");
                break;
            case "ls" or "dir":
                HandleLs(uow);
                break;
            case "pwd":
                HandlePwd(uow);
                break;
            case "cd":
                HandleCd(uow, args.Length > 0 ? args[0] : "");
                break;
            case "status":
                HandleStatus(uow);
                break;
            case "help" or "?":
                HandleHelp(uow);
                break;
            case "bye" or "quit" or "exit":
                HandleClose(uow);
                uow.Out.WriteLine("221 Goodbye.");
                return ApplicationResult.Exit;
            default:
                uow.Out.WriteLine($"?Invalid command: {cmd}");
                break;
        }

        return ApplicationResult.Continue;
    }

    private void HandleOpen(CognitOS.Kernel.IUnitOfWork uow, string host)
    {
        if (_connected)
        {
            uow.Out.WriteLine("Already connected. Use close first.");
            return;
        }
        if (string.IsNullOrWhiteSpace(host))
        {
            uow.Out.WriteLine("(to) ");
            return;
        }
        var ip = uow.Net.Resolve(host);
        if (ip is null)
        {
            uow.Out.WriteLine($"ftp: {host}: Name or service not known");
            return;
        }

        // Modem dial sequence before the FTP handshake
        if (!uow.Modem.Dial(ip, uow.Out))
        {
            uow.Out.WriteLine($"ftp: {host}: Connection timed out");
            return;
        }

        uow.Out.WriteLine();
        _remoteHost = host;
        uow.Out.WriteLine($"Connected to {host} ({ip}).");
        uow.Out.WriteLine($"220 {host} FTP server ready.");
        uow.Out.WriteLine($"Name ({host}:anonymous): anonymous");
        uow.Out.WriteLine("331 Guest login ok, send ident as password.");
        uow.Out.WriteLine("230 Guest login ok, access restrictions apply.");
        uow.Out.WriteLine("Remote system type is UNIX.");
        uow.Out.WriteLine($"Using {_transferMode} mode to transfer files.");
        _connected = true;
        _machineState.Quest.FtpConnected = true;
    }

    private void HandleClose(CognitOS.Kernel.IUnitOfWork uow)
    {
        if (!_connected)
        {
            uow.Out.WriteLine("Not connected.");
            return;
        }
        uow.Out.WriteLine($"221 Goodbye from {_remoteHost}.");
        _connected = false;
        _remoteHost = "";
        _machineState.Quest.FtpConnected = false;
        uow.Modem.Hangup();
    }

    private void HandleBinary(CognitOS.Kernel.IUnitOfWork uow)
    {
        _transferMode = "binary";
        _machineState.Quest.FtpTransferMode = "binary";
        uow.Out.WriteLine("200 Type set to I (binary).");
    }

    private void HandleAscii(CognitOS.Kernel.IUnitOfWork uow)
    {
        _transferMode = "ascii";
        _machineState.Quest.FtpTransferMode = "ascii";
        uow.Out.WriteLine("200 Type set to A (ascii).");
    }

    private void HandlePut(CognitOS.Kernel.IUnitOfWork uow, string fileName)
    {
        if (!_connected)
        {
            uow.Out.WriteLine("Not connected.");
            return;
        }
        if (string.IsNullOrWhiteSpace(fileName))
        {
            uow.Out.WriteLine("(local-file) ");
            return;
        }

        var absolute = uow.Session.ResolvePath(fileName);
        var content = uow.Disk.RawRead(absolute);
        if (content == null)
        {
            uow.Out.WriteLine($"local: {fileName}: No such file or directory");
            return;
        }

        var sizeBytes = content.Length;
        uow.Out.WriteLine("200 PORT command successful.");
        uow.Out.WriteLine($"150 Opening {_transferMode.ToUpperInvariant()} mode data connection for {fileName}.");
        _machineState.Quest.UploadAttempted = true;

        var transferTimeMs = (long)(sizeBytes * 8) / Math.Max(uow.Spec.ModemBaud / 1000, 1);
        uow.Out.WriteLine("226 Transfer complete.");
        uow.Out.WriteLine($"{sizeBytes} bytes sent in {transferTimeMs / 1000.0:F1} seconds.");

        if (_transferMode == "ascii")
        {
            _machineState.Quest.UploadSuccess = false;
            uow.Out.WriteLine();
            uow.Out.WriteLine(Style.Fg(Style.Warn,
                $"remote: warning: {fileName} - uncompress failed, archive may be damaged"));
            uow.Out.WriteLine(Style.Fg(Style.Warn,
                "remote: hint: check transfer mode (ascii vs binary)"));
        }
        else
        {
            _machineState.Quest.UploadSuccess = true;
            uow.Out.WriteLine();
            uow.Out.WriteLine(Style.Fg(Style.Info,
                $"remote: {fileName} received OK, archive integrity verified."));
        }
    }

    private void HandleLs(CognitOS.Kernel.IUnitOfWork uow)
    {
        if (!_connected)
        {
            uow.Out.WriteLine("Not connected.");
            return;
        }
        uow.Out.WriteLine("200 PORT command successful.");
        uow.Out.WriteLine("150 Opening ASCII mode data connection for /bin/ls.");

        if (_machineState.Quest.UploadSuccess)
        {
            uow.Out.WriteLine("total 234");
            uow.Out.WriteLine("drwxr-xr-x  2 ftp  ftp  512 Sep 17 21:12 .");
            uow.Out.WriteLine("-rw-r--r--  1 ftp  ftp  73091 Sep 17 21:12 linux-0.01.tar.Z");
        }
        else if (_machineState.Quest.UploadAttempted)
        {
            uow.Out.WriteLine("total 234");
            uow.Out.WriteLine("drwxr-xr-x  2 ftp  ftp  512 Sep 17 21:12 .");
            uow.Out.WriteLine("-rw-r--r--  1 ftp  ftp  73091 Sep 17 21:12 linux-0.01.tar.Z  [CORRUPT]");
        }
        else
        {
            uow.Out.WriteLine("total 0");
            uow.Out.WriteLine("drwxr-xr-x  2 ftp  ftp  512 Sep 17 21:00 .");
        }

        uow.Out.WriteLine("226 Transfer complete.");
    }

    private void HandlePwd(CognitOS.Kernel.IUnitOfWork uow)
    {
        if (!_connected)
        {
            uow.Out.WriteLine("Not connected.");
            return;
        }
        uow.Out.WriteLine($"257 \"{_remoteCwd}\" is current directory.");
    }

    private void HandleCd(CognitOS.Kernel.IUnitOfWork uow, string dir)
    {
        if (!_connected)
        {
            uow.Out.WriteLine("Not connected.");
            return;
        }
        if (string.IsNullOrWhiteSpace(dir))
        {
            uow.Out.WriteLine("(remote-directory) ");
            return;
        }

        if (dir is "/pub/OS/Linux" or "/pub/OS" or "/pub" or "/")
        {
            _remoteCwd = dir;
            uow.Out.WriteLine("250 CWD command successful.");
        }
        else if (dir == "..")
        {
            _remoteCwd = "/pub/OS";
            uow.Out.WriteLine("250 CWD command successful.");
        }
        else
        {
            uow.Out.WriteLine($"550 {dir}: No such file or directory.");
        }
    }

    private void HandleStatus(CognitOS.Kernel.IUnitOfWork uow)
    {
        uow.Out.WriteLine($"Connected to: {(_connected ? _remoteHost : "(not connected)")}");
        uow.Out.WriteLine($"Transfer mode: {_transferMode}");
        uow.Out.WriteLine($"Remote cwd: {_remoteCwd}");
        uow.Out.WriteLine($"Modem: {uow.Spec.ModemModel} ({uow.Spec.ModemBaud} baud)");
    }

    private void HandleHelp(CognitOS.Kernel.IUnitOfWork uow)
    {
        uow.Out.WriteLine("Commands: open <host>  close  binary  ascii  put <file>");
        uow.Out.WriteLine("          ls  cd <dir>  pwd  status  help  bye");
    }
}

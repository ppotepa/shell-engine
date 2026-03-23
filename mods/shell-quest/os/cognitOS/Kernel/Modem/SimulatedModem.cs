namespace CognitOS.Kernel.Modem;

using CognitOS.Framework.Kernel;
using CognitOS.Kernel.Hardware;

/// <summary>
/// Simulated Hayes AT modem. Writes a realistic dial sequence to the caller's
/// UoW (replacing sequential BlockFor calls with scheduled output).
///
/// Phone book: maps known IP addresses to dialup numbers.
/// In September 1991, nic.funet.fi was reachable by modem from Helsinki.
/// </summary>
internal sealed class SimulatedModem : IModem
{
    private readonly HardwareProfile _hw;
    private bool _connected;

    // Dialup phone numbers for known hosts (Finland / Nordic FUNET, 1991)
    private static readonly Dictionary<string, string> PhoneBook =
        new(StringComparer.Ordinal)
        {
            ["128.214.6.100"] = "90-4574100",   // nic.funet.fi (CSC Helsinki)
            ["130.37.24.3"]   = "020-6464411",   // cs.vu.nl (Vrije Universiteit)
            ["128.214.1.1"]   = "90-4574001",    // helsinki.fi
        };

    public bool IsConnected => _connected;

    public SimulatedModem(HardwareProfile hw)
    {
        _hw = hw;
    }

    public bool Dial(IUnitOfWork uow, string host)
    {
        if (_connected)
            return true;

        // Look up the phone number for this host
        if (!PhoneBook.TryGetValue(host, out var number))
        {
            // Unknown host — synthesize a plausible number from the IP for realism
            number = FallbackNumber(host);
        }

        var baudLabel = BaudLabel(_hw.Spec.ModemBaud);

        // Hayes AT command sequence with realistic delays ────────────────────────
        uow.ScheduleOutput("ATH0", 0);
        uow.ScheduleOutput("OK", 200);
        uow.ScheduleOutput($"ATDT {number}", 100);
        uow.ScheduleOutput("DIALING...", 300);

        // Dialing tones
        double dialingMs = 800 + _hw.Spec.ModemBaud / 4.0;
        uow.ScheduleOutput("RINGING", (ulong)dialingMs);

        // Check if the host is actually reachable (phone book is the authority for modem dial)
        bool reachable = PhoneBook.ContainsKey(host);
        if (!reachable)
        {
            uow.ScheduleOutput("NO CARRIER", 1200);
            uow.ScheduleOutput("", 100);
            return false;
        }

        // Handshake noise — duration scales with baud rate
        double handshakeMs = 3500.0 - (_hw.Spec.ModemBaud / 2400.0) * 400.0;
        handshakeMs = Math.Max(800, handshakeMs);

        uow.ScheduleOutput("CONNECT " + baudLabel, (ulong)handshakeMs);

        _connected = true;
        return true;
    }

    public void Hangup()
    {
        _connected = false;
    }

    private static string BaudLabel(int baud) => baud switch
    {
        >= 2400 => "2400",
        >= 1200 => "1200",
        _       => "300",
    };

    private static string FallbackNumber(string ip)
    {
        // Deterministic fake number from IP octets — looks plausible
        var parts = ip.Split('.');
        if (parts.Length == 4 &&
            int.TryParse(parts[2], out var c) &&
            int.TryParse(parts[3], out var d))
            return $"90-{c:D3}{d:D4}";
        return "90-0000000";
    }
}

namespace CognitOS.Kernel.Modem;

/// <summary>
/// Simulated RS-232 modem subsystem.
/// Provides Hayes AT command dial sequence to establish a dialup connection.
/// </summary>
internal interface IModem
{
    /// <summary>
    /// Dial a remote host. Writes the full Hayes AT command handshake to <paramref name="output"/>.
    /// Returns true when connected, false when the call fails.
    /// </summary>
    bool Dial(string host, System.IO.TextWriter output);

    /// <summary>Hang up the modem (ATH). Silent.</summary>
    void Hangup();

    /// <summary>Whether the modem currently has an active connection.</summary>
    bool IsConnected { get; }
}

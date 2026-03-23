namespace CognitOS.Kernel.Modem;

using CognitOS.Framework.Kernel;

/// <summary>
/// Simulated RS-232 modem subsystem.
/// Provides Hayes AT command dial sequence to establish a dialup connection.
/// </summary>
internal interface IModem
{
    /// <summary>
    /// Dial a remote host. Schedules the full Hayes AT command handshake via <paramref name="uow"/>.
    /// Returns true when connected, false when the call fails.
    /// </summary>
    bool Dial(IUnitOfWork uow, string host);

    /// <summary>Hang up the modem (ATH). Silent.</summary>
    void Hangup();

    /// <summary>Whether the modem currently has an active connection.</summary>
    bool IsConnected { get; }
}

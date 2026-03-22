namespace CognitOS.Framework.Transport;

/// <summary>
/// Low-level transport source for incoming JSON protocol lines.
/// </summary>
internal interface IInputSource : IDisposable
{
    string? ReadProtocolLine();
}

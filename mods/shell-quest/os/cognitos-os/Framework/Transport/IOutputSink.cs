namespace CognitosOs.Framework.Transport;

/// <summary>
/// Low-level transport sink for JSON protocol lines emitted by the sidecar.
/// </summary>
internal interface IOutputSink : IDisposable
{
    void WriteProtocolLine(string line);
    void Flush();
}

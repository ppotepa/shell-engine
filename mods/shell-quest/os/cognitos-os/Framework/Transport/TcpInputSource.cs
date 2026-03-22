namespace CognitosOs.Framework.Transport;

using System.Net.Sockets;
using System.Text;

internal sealed class TcpInputSource : IInputSource
{
    private readonly StreamReader _reader;

    public TcpInputSource(NetworkStream stream)
    {
        _reader = new StreamReader(stream, new UTF8Encoding(false), detectEncodingFromByteOrderMarks: false, leaveOpen: true);
    }

    public string? ReadProtocolLine() => _reader.ReadLine();

    public void Dispose() { }
}

namespace CognitosOs.Framework.Transport;

using System.Net.Sockets;
using System.Text;

internal sealed class TcpOutputSink : IOutputSink
{
    private readonly StreamWriter _writer;

    public TcpOutputSink(NetworkStream stream)
    {
        _writer = new StreamWriter(stream, new UTF8Encoding(false), leaveOpen: true)
        {
            AutoFlush = true,
        };
    }

    public void WriteProtocolLine(string line)
    {
        _writer.WriteLine(line);
    }

    public void Flush() => _writer.Flush();

    public void Dispose() => _writer.Flush();
}

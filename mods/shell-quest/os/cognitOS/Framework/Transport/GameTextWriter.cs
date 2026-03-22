namespace CognitOS.Framework.Transport;

using System.Text;
using CognitOS.Core;

/// <summary>
/// Bridges accidental Console.WriteLine debugging into the sidecar protocol.
/// Each flushed line becomes a normal `out` event.
/// </summary>
internal sealed class GameTextWriter : TextWriter
{
    private readonly IOutputSink _sink;
    private readonly StringBuilder _buffer = new();

    public GameTextWriter(IOutputSink sink)
    {
        _sink = sink;
    }

    public override Encoding Encoding => Encoding.UTF8;

    public override void Write(char value)
    {
        if (value == '\n')
        {
            FlushBuffer();
            return;
        }

        if (value != '\r')
            _buffer.Append(value);
    }

    public override void Write(string? value)
    {
        if (string.IsNullOrEmpty(value))
            return;

        foreach (var ch in value)
            Write(ch);
    }

    public override void WriteLine(string? value)
    {
        if (!string.IsNullOrEmpty(value))
            _buffer.Append(value);
        FlushBuffer();
    }

    public override void Flush()
    {
        FlushBuffer();
        _sink.Flush();
    }

    private void FlushBuffer()
    {
        var text = _buffer.ToString();
        _buffer.Clear();
        Protocol.Send(_sink, new
        {
            type = "out",
            lines = new[] { text }
        });
    }
}

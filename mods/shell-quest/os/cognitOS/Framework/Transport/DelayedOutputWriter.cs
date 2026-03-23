namespace CognitOS.Framework.Transport;

using System.Text;
using CognitOS.Core;

/// <summary>
/// TextWriter that emits lines with configurable delays for realistic timing simulation.
/// Used by commands that need to simulate network latency, disk spin-up, or other hardware delays.
/// </summary>
internal sealed class DelayedOutputWriter : TextWriter
{
    private readonly IOutputSink _sink;
    private readonly StringBuilder _buffer = new();
    private ulong _accumulatedDelayMs;

    public DelayedOutputWriter(IOutputSink sink)
    {
        _sink = sink;
    }

    public override Encoding Encoding => Encoding.UTF8;

    /// <summary>
    /// Set delay for the next line to be emitted.
    /// Call before WriteLine to schedule line output after specified ms.
    /// </summary>
    public void SetNextLineDelay(ulong delayMs)
    {
        _accumulatedDelayMs += delayMs;
    }

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

        // Skip empty lines from spurious Flush/Dispose calls
        if (text.Length == 0) return;

        Protocol.EmitLine(_sink, text, _accumulatedDelayMs > 0 ? _accumulatedDelayMs : null);
        // Do NOT reset _accumulatedDelayMs — keep it accumulating so each SetNextLineDelay
        // adds to the previous, giving correct absolute per-line timestamps on the engine side.
    }
}

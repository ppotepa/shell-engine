namespace CognitOS.Framework.Transport;

using System.Text;
using CognitOS.Framework.Kernel;

/// <summary>
/// TextWriter that emits lines with configurable delays for realistic timing simulation.
/// Routes through IUnitOfWork.ScheduleOutput so delays are drained by the tick loop.
/// </summary>
internal sealed class DelayedOutputWriter : TextWriter
{
    private readonly IUnitOfWork _uow;
    private readonly StringBuilder _buffer = new();
    private ulong _nextDeltaMs;

    public DelayedOutputWriter(IUnitOfWork uow)
    {
        _uow = uow;
    }

    public override Encoding Encoding => Encoding.UTF8;

    /// <summary>
    /// Set the delay for the next flushed line relative to the previous one.
    /// Accumulates until the next WriteLine/Flush.
    /// </summary>
    public void SetNextLineDelay(ulong delayMs)
    {
        _nextDeltaMs += delayMs;
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
    }

    private void FlushBuffer()
    {
        var text = _buffer.ToString();
        _buffer.Clear();

        if (text.Length == 0) return;

        _uow.ScheduleOutput(text, _nextDeltaMs);
        _nextDeltaMs = 0;
    }
}

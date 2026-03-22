namespace CognitOS.Framework.Transport;

internal sealed class ConsoleOutputSink : IOutputSink
{
    private readonly TextWriter _output;

    public ConsoleOutputSink(TextWriter output)
    {
        _output = output;
    }

    public void WriteProtocolLine(string line)
    {
        _output.WriteLine(line);
    }

    public void Flush() => _output.Flush();

    public void Dispose() => Flush();
}

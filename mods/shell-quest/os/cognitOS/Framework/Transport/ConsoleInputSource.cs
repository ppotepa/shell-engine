namespace CognitOS.Framework.Transport;

internal sealed class ConsoleInputSource : IInputSource
{
    private readonly TextReader _input;

    public ConsoleInputSource(TextReader input)
    {
        _input = input;
    }

    public string? ReadProtocolLine() => _input.ReadLine();

    public void Dispose() { }
}

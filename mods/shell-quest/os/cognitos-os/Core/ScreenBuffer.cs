namespace CognitosOs.Core;

internal sealed class ScreenBuffer
{
    private readonly List<string> _visible = new();
    private int _viewportRows = 40;
    private int _viewportCols = 120;
    private string _promptPrefix = string.Empty;
    private string _inputLine = string.Empty;
    private int _cursorX;
    private int _cursorY;

    public void SetViewport(int cols, int rows)
    {
        if (cols > 0)
        {
            _viewportCols = cols;
        }
        if (rows > 0)
        {
            _viewportRows = rows;
        }
    }

    public void SetPrompt(string prefix)
    {
        _promptPrefix = prefix ?? string.Empty;
        UpdateCursor();
        SendFrame();
    }

    public void SetInputLine(string value)
    {
        _inputLine = value ?? string.Empty;
        UpdateCursor();
        SendFrame();
    }

    public void CommitInputLine()
    {
        _visible.Add(_promptPrefix + _inputLine);
        _inputLine = string.Empty;
        UpdateCursor();
        SendFrame();
    }

    public void ClearViewport()
    {
        _visible.Clear();
        UpdateCursor();
        SendFrame();
    }

    public void Append(params string[] lines)
    {
        foreach (var line in lines)
        {
            _visible.Add(line);
        }
        UpdateCursor();
        SendFrame();
    }

    private void UpdateCursor()
    {
        _cursorY = BuildVisibleFrameLines().Count - 1;
        _cursorX = (_promptPrefix + _inputLine).Length;
    }

    private List<string> BuildVisibleFrameLines()
    {
        var allLines = new List<string>(_visible.Count + 1);
        allLines.AddRange(_visible);
        allLines.Add(_promptPrefix + _inputLine);

        if (allLines.Count > _viewportRows)
        {
            allLines = allLines.Skip(allLines.Count - _viewportRows).ToList();
        }
        return allLines;
    }

    private void SendFrame()
    {
        var frame = BuildVisibleFrameLines();
        Protocol.Send(new
        {
            type = "screen-full",
            cols = _viewportCols,
            rows = _viewportRows,
            lines = frame.ToArray(),
            cursor_x = _cursorX,
            cursor_y = _cursorY
        });
    }
}

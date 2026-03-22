namespace CognitOS.Core;

using CognitOS.Framework.Transport;

internal sealed class ScreenBuffer
{
    private readonly IOutputSink _sink;
    private readonly List<string> _visible = new();
    private int _viewportRows = 40;
    private int _viewportCols = 120;
    private string _promptPrefix = string.Empty;
    private string _inputLine = string.Empty;
    private int _cursorX;
    private int _cursorY;

    public ScreenBuffer(IOutputSink sink)
    {
        _sink = sink;
    }

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
        foreach (var line in _visible)
            allLines.AddRange(WrapLine(line));
        allLines.Add(_promptPrefix + _inputLine);

        if (allLines.Count > _viewportRows)
            allLines = allLines.Skip(allLines.Count - _viewportRows).ToList();
        return allLines;
    }

    /// <summary>
    /// Hard-wraps a single line to <see cref="_viewportCols"/> visible characters.
    /// Markup tags of the form [colour]…[/] are treated as zero-width so they
    /// do not count toward the column limit.
    /// </summary>
    private IEnumerable<string> WrapLine(string line)
    {
        if (VisibleLength(line) <= _viewportCols)
        {
            yield return line;
            yield break;
        }

        var sb = new System.Text.StringBuilder();
        var visible = 0;
        var inTag = false;

        foreach (var ch in line)
        {
            if (ch == '[') { inTag = true;  sb.Append(ch); continue; }
            if (ch == ']') { inTag = false; sb.Append(ch); continue; }
            if (inTag)     { sb.Append(ch); continue; }

            if (visible == _viewportCols)
            {
                yield return sb.ToString();
                sb.Clear();
                visible = 0;
            }
            sb.Append(ch);
            visible++;
        }

        if (sb.Length > 0)
            yield return sb.ToString();
    }

    private static int VisibleLength(string line)
    {
        var count = 0;
        var inTag = false;
        foreach (var ch in line)
        {
            if (ch == '[') { inTag = true;  continue; }
            if (ch == ']') { inTag = false; continue; }
            if (!inTag) count++;
        }
        return count;
    }

    private void SendFrame()
    {
        var frame = BuildVisibleFrameLines();
        Protocol.Send(_sink, new
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

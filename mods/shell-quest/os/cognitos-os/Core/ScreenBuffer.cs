namespace CognitosOs.Core;

internal sealed class ScreenBuffer
{
    private readonly List<string> _visible = new();
    private int _viewportRows = 40;

    public void SetViewport(int rows)
    {
        if (rows > 0)
        {
            _viewportRows = rows;
        }
    }

    public void ClearViewport()
    {
        _visible.Clear();
        Protocol.Send(new { type = "clear" });
    }

    public void Append(params string[] lines)
    {
        var needsRepaint = false;
        foreach (var line in lines)
        {
            _visible.Add(line);
            if (_visible.Count > _viewportRows)
            {
                _visible.RemoveAt(0);
                needsRepaint = true;
            }
        }

        if (needsRepaint)
        {
            Protocol.Send(new { type = "clear" });
            Protocol.Send(new { type = "out", lines = _visible.ToArray() });
            return;
        }

        Protocol.Send(new { type = "out", lines });
    }
}

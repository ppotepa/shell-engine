namespace CognitOS.Core;

internal static class Style
{
    // Muted CRT-ish palette: readable but not neon.
    public const string PromptUser = "#a8c2a2";
    public const string PromptHost = "#9ab3c9";
    public const string PromptPath = "#b7b39a";
    public const string Error = "#c18f8f";
    public const string Warn = "#c3b38f";
    public const string Info = "#aeb6bf";

    // Boot sequence highlights
    public const string Bright = "#e8e8e8";
    public const string BootKeyword = "#c8d8c8";
    public const string BootOk = "#a8d8a8";

    public static string Fg(string colour, string text) => $"[{colour}]{text}[/]";

    /// <summary>
    /// Brighten a hex colour by the given factor (e.g. 1.15 = +15%).
    /// Clamps each channel to 255.
    /// </summary>
    public static string BrightenHex(string hex, double factor)
    {
        var raw = hex.TrimStart('#');
        if (raw.Length != 6) return hex;
        if (!byte.TryParse(raw[..2], System.Globalization.NumberStyles.HexNumber, null, out var r)) return hex;
        if (!byte.TryParse(raw[2..4], System.Globalization.NumberStyles.HexNumber, null, out var g)) return hex;
        if (!byte.TryParse(raw[4..6], System.Globalization.NumberStyles.HexNumber, null, out var b)) return hex;
        r = (byte)Math.Min(255, (int)(r * factor));
        g = (byte)Math.Min(255, (int)(g * factor));
        b = (byte)Math.Min(255, (int)(b * factor));
        return $"#{r:x2}{g:x2}{b:x2}";
    }
}

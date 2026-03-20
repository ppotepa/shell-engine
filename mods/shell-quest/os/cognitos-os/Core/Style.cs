namespace CognitosOs.Core;

internal static class Style
{
    // Muted CRT-ish palette: readable but not neon.
    public const string PromptUser = "#a8c2a2";
    public const string PromptHost = "#9ab3c9";
    public const string PromptPath = "#b7b39a";
    public const string Error = "#c18f8f";
    public const string Warn = "#c3b38f";
    public const string Info = "#aeb6bf";

    public static string Fg(string colour, string text) => $"[{colour}]{text}[/]";
}

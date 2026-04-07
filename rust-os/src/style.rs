/// ANSI-compatible markup helpers.
/// Format: [color]text[/] where color is a hex string like #7a9e7e
/// These are stripped for width calculations but rendered by the terminal layer.
pub const PROMPT_USER: &str = "#8fbc8f";
pub const PROMPT_HOST: &str = "#7a9e7e";
pub const PROMPT_PATH: &str = "#a0c4a0";
pub const ERROR: &str = "#cc6666";
pub const WARN: &str = "#de935f";
pub const INFO: &str = "#81a2be";
pub const BRIGHT: &str = "#c5c8c6";
pub const BOOT_KEYWORD: &str = "#b5bd68";
pub const BOOT_OK: &str = "#8abeb7";
pub const ANOMALY: &str = "#cc99cc";
pub const DIM: &str = "#555753";

pub fn fg(color: &str, text: &str) -> String {
    format!("[{color}]{text}[/]")
}

/// Strip markup tags for plain-text width calculation.
pub fn strip(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '[' => in_tag = true,
            ']' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// Visible character width of a markup string.
pub fn visible_len(s: &str) -> usize {
    strip(s).chars().count()
}

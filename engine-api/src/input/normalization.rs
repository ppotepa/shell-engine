//! Input normalization and alias helpers.

pub fn normalize_input_code(code: &str) -> String {
    if code == " " {
        return " ".to_string();
    }
    let trimmed = code.trim();
    let lowered = trimmed.to_ascii_lowercase();
    match lowered.as_str() {
        "space" => return " ".to_string(),
        "arrowup" | "up" => return "Up".to_string(),
        "arrowdown" | "down" => return "Down".to_string(),
        "arrowleft" | "left" => return "Left".to_string(),
        "arrowright" | "right" => return "Right".to_string(),
        _ => {}
    }
    if trimmed.len() == 1 {
        return lowered;
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_input_code;

    #[test]
    fn normalizes_arrow_aliases_and_space() {
        assert_eq!(normalize_input_code("ArrowUp"), "Up");
        assert_eq!(normalize_input_code("up"), "Up");
        assert_eq!(normalize_input_code("ArrowLeft"), "Left");
        assert_eq!(normalize_input_code("right"), "Right");
        assert_eq!(normalize_input_code("Space"), " ");
        assert_eq!(normalize_input_code(" "), " ");
        assert_eq!(normalize_input_code("A"), "a");
    }
}

use std::env;
use serde_yaml::Value;

/// Detected terminal capabilities.
#[derive(Debug, Clone)]
pub struct TerminalCaps {
    /// Number of colours the terminal supports (8, 16, 256, or 16_777_216).
    pub colours: u32,
    /// Terminal width in columns.
    pub width: u16,
    /// Terminal height in rows.
    pub height: u16,
}

/// Requirements parsed from a mod manifest's `terminal:` block.
#[derive(Debug, Clone, Default)]
pub struct TerminalRequirements {
    pub min_colours: Option<u32>,
    pub min_width:   Option<u16>,
    pub min_height:  Option<u16>,
    pub target_fps:  Option<u16>,
}

pub const DEFAULT_TARGET_FPS: u16 = 60;
pub const MAX_TARGET_FPS: u16 = 240;

/// Violation reported when a terminal requirement is not met.
#[derive(Debug, Clone)]
pub struct TerminalViolation {
    pub requirement: String,
    pub required:    String,
    pub detected:    String,
}

impl TerminalCaps {
    /// Detect the current terminal's capabilities.
    pub fn detect() -> std::io::Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        let colours = detect_colour_count();
        Ok(Self { colours, width, height })
    }

    /// Validate detected capabilities against mod requirements.
    /// Returns a list of violations; empty means all requirements are met.
    pub fn validate(&self, req: &TerminalRequirements) -> Vec<TerminalViolation> {
        let mut violations = Vec::new();

        if let Some(min) = req.min_colours {
            if self.colours < min {
                violations.push(TerminalViolation {
                    requirement: "min_colours".into(),
                    required:    format!("{}", min),
                    detected:    format!("{}", self.colours),
                });
            }
        }

        if let Some(min) = req.min_width {
            if self.width < min {
                violations.push(TerminalViolation {
                    requirement: "min_width".into(),
                    required:    format!("{}", min),
                    detected:    format!("{}", self.width),
                });
            }
        }

        if let Some(min) = req.min_height {
            if self.height < min {
                violations.push(TerminalViolation {
                    requirement: "min_height".into(),
                    required:    format!("{}", min),
                    detected:    format!("{}", self.height),
                });
            }
        }

        violations
    }
}

impl TerminalRequirements {
    /// Parse requirements from the `terminal:` block of a mod manifest.
    /// Returns `None` when the block is absent — no requirements to enforce.
    pub fn from_manifest(manifest: &Value) -> Option<Self> {
        let block = manifest.get("terminal")?;

        Some(Self {
            min_colours: block.get("min_colours").and_then(Value::as_u64).map(|v| v as u32),
            min_width:   block.get("min_width").and_then(Value::as_u64).map(|v| v as u16),
            min_height:  block.get("min_height").and_then(Value::as_u64).map(|v| v as u16),
            target_fps:  block
                .get("target_fps")
                .or_else(|| block.get("target-fps"))
                .and_then(Value::as_u64)
                .map(|v| (v as u16).clamp(1, MAX_TARGET_FPS)),
        })
    }
}

pub fn target_fps_from_manifest(manifest: &Value) -> u16 {
    TerminalRequirements::from_manifest(manifest)
        .and_then(|req| req.target_fps)
        .unwrap_or(DEFAULT_TARGET_FPS)
}

/// Detect colour depth from environment variables.
///
/// Checks (in order of reliability):
///   1. `COLORTERM`  — `truecolor` / `24bit` → 16 777 216
///   2. `COLORTERM`  — `256color`            → 256
///   3. `TERM`       — contains `256color`   → 256
///   4. `TERM`       — contains `16color`    → 16
///   5. fallback                             → 8
fn detect_colour_count() -> u32 {
    if let Ok(ct) = env::var("COLORTERM") {
        match ct.to_lowercase().as_str() {
            "truecolor" | "24bit" => return 16_777_216,
            "256color"            => return 256,
            _                     => {}
        }
    }

    if let Ok(term) = env::var("TERM") {
        let term = term.to_lowercase();
        if term.contains("256color") { return 256; }
        if term.contains("16color")  { return 16;  }
    }

    8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(colours: Option<u32>, width: Option<u16>, height: Option<u16>) -> TerminalRequirements {
        TerminalRequirements {
            min_colours: colours,
            min_width: width,
            min_height: height,
            target_fps: None,
        }
    }

    fn caps(colours: u32, width: u16, height: u16) -> TerminalCaps {
        TerminalCaps { colours, width, height }
    }

    #[test]
    fn no_violations_when_requirements_met() {
        let violations = caps(256, 200, 50).validate(&req(Some(256), Some(120), Some(30)));
        assert!(violations.is_empty());
    }

    #[test]
    fn reports_colour_violation() {
        let violations = caps(8, 200, 50).validate(&req(Some(256), None, None));
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].requirement, "min_colours");
    }

    #[test]
    fn reports_multiple_violations() {
        let violations = caps(8, 80, 20).validate(&req(Some(256), Some(120), Some(30)));
        assert_eq!(violations.len(), 3);
    }

    #[test]
    fn no_requirements_means_no_violations() {
        let violations = caps(8, 40, 10).validate(&req(None, None, None));
        assert!(violations.is_empty());
    }

    #[test]
    fn parses_requirements_from_manifest() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "terminal:\n  min_colours: 256\n  min_width: 120\n  min_height: 30\n  target_fps: 30\n"
        ).unwrap();
        let req = TerminalRequirements::from_manifest(&yaml).unwrap();
        assert_eq!(req.min_colours, Some(256));
        assert_eq!(req.min_width, Some(120));
        assert_eq!(req.min_height, Some(30));
        assert_eq!(req.target_fps, Some(30));
    }

    #[test]
    fn returns_none_when_terminal_block_absent() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>("name: test\n").unwrap();
        assert!(TerminalRequirements::from_manifest(&yaml).is_none());
    }

    #[test]
    fn target_fps_defaults_to_sixty() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>("name: test\n").unwrap();
        assert_eq!(target_fps_from_manifest(&yaml), 60);
    }

    #[test]
    fn target_fps_supports_kebab_case_alias() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            "terminal:\n  target-fps: 30\n"
        ).unwrap();
        assert_eq!(target_fps_from_manifest(&yaml), 30);
    }
}

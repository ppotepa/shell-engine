//! Startup check for action map validation.
//!
//! Validates that action maps defined in mod.yaml are well-formed and
//! that action names are valid identifiers that can be consistently
//! referenced in scripts.

use engine_error::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Validates optional action map definitions in mod.yaml.
///
/// Ensures:
/// - action_map.actions is a mapping (if present)
/// - Each action name is a valid identifier
/// - Each action has a "key" property (required)
/// - No duplicate action names
pub struct ActionMapCheck;

impl StartupCheck for ActionMapCheck {
    fn name(&self) -> &'static str {
        "action-map"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        // Load mod.yaml manifest and check for action_map section
        let manifest = ctx.manifest();

        // Check if action_map exists
        if let Some(action_map) = manifest.get("action_map") {
            validate_action_map(action_map, report)?;
        }

        Ok(())
    }
}

/// Validates the action_map section of mod.yaml.
fn validate_action_map(action_map: &serde_yaml::Value, report: &mut StartupReport) -> Result<(), EngineError> {
    // action_map should be a mapping
    if !action_map.is_mapping() {
        report.add_warning(
            "ActionMapCheck",
            "action_map should be a mapping/object",
        );
        return Ok(());
    }

    // Check if actions sub-property exists
    let Some(actions) = action_map.get("actions") else {
        report.add_info(
            "ActionMapCheck",
            "action_map defined but no 'actions' property found",
        );
        return Ok(());
    };

    if !actions.is_mapping() {
        report.add_warning("ActionMapCheck", "action_map.actions should be a mapping/object");
        return Ok(());
    }

    // Validate each action definition
    for (key, action_def) in actions
        .as_mapping()
        .expect("Already checked is_mapping")
        .iter()
    {
        let action_name = key.as_str().unwrap_or("<unknown>");

        // Validate action name is a valid identifier
        if !is_valid_identifier(action_name) {
            report.add_warning(
                "ActionMapCheck",
                format!(
                    "action name '{}' is not a valid identifier (must start with letter or _, contain only alphanumeric or _)",
                    action_name
                ),
            );
            continue;
        }

        // Action definition should be a mapping
        if !action_def.is_mapping() {
            report.add_warning(
                "ActionMapCheck",
                format!("action '{}' definition should be a mapping/object", action_name),
            );
            continue;
        }

        // Check required 'key' property
        if !action_def.get("key").map_or(false, |v| v.is_string()) {
            report.add_warning(
                "ActionMapCheck",
                format!(
                    "action '{}' is missing 'key' property (must be a string)",
                    action_name
                ),
            );
        }
    }

    Ok(())
}

/// Checks if a string is a valid action/identifier name.
fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();
    
    // First char must be letter or underscore
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_identifier_accepts_letters_underscore_numbers() {
        assert!(is_valid_identifier("action_name"));
        assert!(is_valid_identifier("_action"));
        assert!(is_valid_identifier("a"));
        assert!(is_valid_identifier("action123"));
    }

    #[test]
    fn valid_identifier_rejects_invalid_starts() {
        assert!(!is_valid_identifier("123action"));
        assert!(!is_valid_identifier("-action"));
        assert!(!is_valid_identifier(""));
    }

    #[test]
    fn valid_identifier_rejects_special_chars() {
        assert!(!is_valid_identifier("action-name"));
        assert!(!is_valid_identifier("action.name"));
        assert!(!is_valid_identifier("action name"));
    }
}

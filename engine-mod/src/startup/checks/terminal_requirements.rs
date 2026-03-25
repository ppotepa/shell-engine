//! Checks that the current terminal satisfies any minimum capability requirements declared in `mod.yaml`.

use engine_core::terminal_caps::{TerminalCaps, TerminalRequirements, TerminalViolation};
use engine_error::EngineError;
use engine_runtime::{RuntimeSettings, VirtualPolicy};

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check that fails when the current terminal does not meet the mod's declared requirements.
pub struct TerminalRequirementsCheck;

impl StartupCheck for TerminalRequirementsCheck {
    fn name(&self) -> &'static str {
        "terminal-requirements"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let Some(req) = TerminalRequirements::from_manifest(ctx.manifest()) else {
            report.add_info(self.name(), "no terminal requirements declared");
            return Ok(());
        };

        let caps = TerminalCaps::detect()?;
        let mut violations = caps.validate(&req);
        append_virtual_buffer_violations(ctx, &caps, &mut violations);
        if violations.is_empty() {
            report.add_info(self.name(), "terminal requirements satisfied");
            return Ok(());
        }

        let details = violations
            .iter()
            .map(|v| {
                format!(
                    "{}: requires {}, detected {}",
                    v.requirement, v.required, v.detected
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        Err(EngineError::TerminalRequirementsNotMet(details))
    }
}

fn append_virtual_buffer_violations(
    ctx: &StartupContext,
    caps: &TerminalCaps,
    violations: &mut Vec<TerminalViolation>,
) {
    let runtime = RuntimeSettings::from_manifest(ctx.manifest());
    if !runtime.use_virtual_buffer {
        return;
    }
    if runtime.virtual_policy != VirtualPolicy::Strict {
        return;
    }
    if runtime.virtual_size_max_available {
        return;
    }
    if caps.width < runtime.virtual_width || caps.height < runtime.virtual_height {
        violations.push(TerminalViolation {
            requirement: "virtual_buffer(strict)".to_string(),
            required: format!("{}x{}", runtime.virtual_width, runtime.virtual_height),
            detected: format!("{}x{}", caps.width, caps.height),
        });
    }
}

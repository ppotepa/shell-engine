use crate::terminal_caps::{TerminalCaps, TerminalRequirements};
use crate::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

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
        let violations = caps.validate(&req);
        if violations.is_empty() {
            report.add_info(self.name(), "terminal requirements satisfied");
            return Ok(());
        }

        let details = violations
            .iter()
            .map(|v| format!("{}: requires {}, detected {}", v.requirement, v.required, v.detected))
            .collect::<Vec<_>>()
            .join("; ");
        Err(EngineError::TerminalRequirementsNotMet(details))
    }
}


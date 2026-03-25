//! The [`StartupCheck`] trait — implemented by every pre-run validation step.

use engine_error::EngineError;

use super::context::StartupContext;
use super::report::StartupReport;

/// A single pre-run validation step run by [`StartupRunner`](super::runner::StartupRunner).
pub trait StartupCheck {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError>;
}

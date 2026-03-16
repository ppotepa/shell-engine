use crate::EngineError;

use super::context::StartupContext;
use super::report::StartupReport;

pub trait StartupCheck {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError>;
}


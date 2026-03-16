use crate::EngineError;

use super::check::StartupCheck;
use super::checks::{
    EffectRegistryCheck, FontGlyphCoverageCheck, FontManifestCheck, SceneGraphCheck,
    TerminalRequirementsCheck,
};
use super::context::StartupContext;
use super::report::StartupReport;

pub struct StartupRunner {
    checks: Vec<Box<dyn StartupCheck + Send + Sync>>,
}

impl StartupRunner {
    pub fn with_checks(checks: Vec<Box<dyn StartupCheck + Send + Sync>>) -> Self {
        Self { checks }
    }

    pub fn run(&self, ctx: &StartupContext) -> Result<StartupReport, EngineError> {
        let mut report = StartupReport::default();
        for check in &self.checks {
            check.run(ctx, &mut report)?;
        }
        Ok(report)
    }
}

impl Default for StartupRunner {
    fn default() -> Self {
        Self::with_checks(vec![
            Box::new(TerminalRequirementsCheck),
            Box::new(SceneGraphCheck),
            Box::new(EffectRegistryCheck),
            Box::new(FontManifestCheck),
            Box::new(FontGlyphCoverageCheck),
        ])
    }
}


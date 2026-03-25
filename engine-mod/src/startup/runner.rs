//! [`StartupRunner`] — executes an ordered list of [`StartupCheck`]s and returns a [`StartupReport`].

use engine_error::EngineError;

use super::check::StartupCheck;
use super::checks::{
    EffectRegistryCheck, FontGlyphCoverageCheck, FontManifestCheck, ImageAssetsCheck,
    RhaiScriptsCheck, SceneGraphCheck, TerminalRequirementsCheck,
};
use super::context::StartupContext;
use super::report::StartupReport;

/// Orchestrates the startup pipeline by running each registered [`StartupCheck`] in order.
pub struct StartupRunner {
    checks: Vec<Box<dyn StartupCheck + Send + Sync>>,
}

impl StartupRunner {
    /// Creates a runner with the provided custom set of checks.
    pub fn with_checks(checks: Vec<Box<dyn StartupCheck + Send + Sync>>) -> Self {
        Self { checks }
    }

    /// Runs all checks against `ctx` and returns the accumulated [`StartupReport`], or the first fatal error.
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
            Box::new(RhaiScriptsCheck),
            Box::new(EffectRegistryCheck),
            Box::new(ImageAssetsCheck),
            Box::new(FontManifestCheck),
            Box::new(FontGlyphCoverageCheck),
        ])
    }
}
